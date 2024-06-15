#![doc = include_str!("../Readme.md")]

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;

mod error;
mod utils;

use error::*;

/// Parsed arguments for the `assert` macro
struct Args {
    /// condition to evaluate
    expr: syn::Expr,
    /// optional message to display if the condition is false
    format: TokenStream,
}

impl syn::parse::Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            let msg = "missing condition to check";
            return Err(syn::Error::new(Span::call_site(), msg)); // checked in tests/fail/missing_params.rs
        }
        let span_source: TokenStream = input.fork().parse().unwrap(); // unwrap: parsing a TokenStream can't fail
        let expr = match input.parse() {
            Ok(expr) => expr,
            Err(e) => {
                let err = if input.is_empty() {
                    // syn's error would use call_site instead of pointing at the broken expression
                    let msg = "incomplete expression";
                    syn::Error::new_spanned(span_source, msg) // checked in tests/fail/malformed_expr.rs
                } else if input.peek(syn::Token![,]) {
                    // syn's error would point at the ',' saying "expected an expression"
                    let msg = "incomplete expression";
                    syn::Error::new_spanned(span_source, msg) // checked in tests/fail/malformed_expr.rs
                } else {
                    e
                };
                return Err(err);
            }
        };

        let format;
        if input.is_empty() {
            format = TokenStream::new();
        } else if let Err(e) = input.parse::<syn::Token![,]>() {
            let msg = "condition has to be followed by a comma, if a message is provided";
            return Err(syn::Error::new(e.span(), msg));
        } else {
            format = input.parse()?;
        }

        Ok(Args { expr, format })
    }
}

#[proc_macro]
pub fn assert(input: TokenStream1) -> TokenStream1 {
    let input = syn::parse_macro_input!(input as Args);
    match assert_internal(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into(),
    }
}

fn assert_internal(input: Args) -> Result<TokenStream> {
    let Args { expr, format } = input;

    let mut expression = expr.to_token_stream(); // the expression to evaluate
    let expr_str = expression.to_string().replace('{', "{{").replace('}', "}}"); // the expression as a string

    if expr_str == "true" {
        return Ok(assert_true_flavor());
    }

    let mut message_parts = vec![quote! { "assertion `", #expr_str, "` failed" }]; // the static parts of the message
    let mut dynamic_args = vec![]; // the dynamic format arguments to the message
    let mut setup = TokenStream::new(); // setup code to evaluate before the expression

    if !format.is_empty() {
        message_parts.push(quote! { ": {}" });
        dynamic_args.push(quote! { ::std::format_args!(#format) });
    }

    eval_expr(
        expr,
        &mut setup,
        &mut expression,
        &mut message_parts,
        &mut dynamic_args,
    )?;

    let output = quote! {
        #setup
        if #expression {
            // using an empty if instead of `!(#expression)` to avoid messing with the spans in `expression`
        } else {
            ::std::panic!(::std::concat!(#(#message_parts),*), #(#dynamic_args),*);
        }
    };

    Ok(output)
}

fn eval_expr(
    e: syn::Expr,
    setup: &mut TokenStream,
    expression: &mut TokenStream,
    message_parts: &mut Vec<TokenStream>,
    dynamic_args: &mut Vec<TokenStream>,
) -> Result<()> {
    match e {
        // [a, b, c, d]
        syn::Expr::Array(_) => (), // let the compiler generate the error

        // a = b
        syn::Expr::Assign(syn::ExprAssign { eq_token, .. }) => {
            let msg = "Expected a boolean expression, found an assignment. Did you intend to compare with `==`?";
            return Error::err_spanned(eq_token, msg);
        }

        // async { ... }
        syn::Expr::Async(_) => {
            let msg = "Expected a boolean expression, found an async block. Did you intend to await a future?";
            return Error::err_spanned(e, msg);
        }

        // future.await
        syn::Expr::Await(_) => (), // might work if the future resolves to a boolean and the assert is in an async context

        // left op right
        syn::Expr::Binary(syn::ExprBinary {
            left, op, right, ..
        }) => {
            let (lhs, rhs) = stretched_span(&left, &right);
            setup.extend(quote! {
                let #lhs = #left;
                let #rhs = #right;
            });
            *expression = quote! { #lhs #op #rhs };
            message_parts.push(quote! { "\n  left: {:?}\n right: {:?}" });
            dynamic_args.push(lhs);
            dynamic_args.push(rhs);
        }

        // { ... }
        syn::Expr::Block(_) => (), // might work if the last statement is a boolean

        // break
        syn::Expr::Break(_) => {
            // we need to generate our own error, because break returns `!` so it compiles, but the assertion makes no sense
            let msg = "Expected a boolean expression, found a break statement";
            return Error::err_spanned(e, msg);
        }

        // function(args...)
        syn::Expr::Call(_) => todo!("split args"),

        // expr as ty
        syn::Expr::Cast(syn::ExprCast { expr, ty, .. }) => {
            if ty.to_token_stream().to_string() == "bool" {
                let span = expr.span();
                let var = quote_spanned! {span => __assert_casted};
                setup.extend(quote! {
                    let #var = #expr;
                });
                *expression = quote! { #var as #ty };
                message_parts.push(quote! { "\n input: {:?}" });
                dynamic_args.push(var);
                todo!("print pre-cast value")
            } else {
                let msg = "Expected a boolean expression, found a cast. Did you mean to use a comparison?";
                return Error::err_spanned(ty, msg);
            }
        }

        // |args| { ... }
        syn::Expr::Closure(_) => (), // let the compiler generate the error

        // const { ... }
        syn::Expr::Const(_) => (), // might work if the constant is a boolean

        // continue
        syn::Expr::Continue(_) => {
            // we need to generate our own error, because continue returns `!` so it compiles, but the assertion makes no sense
            let msg = "Expected a boolean expression, found a continue statement";
            return Error::err_spanned(e, msg);
        }

        // obj.field
        syn::Expr::Field(_) => (), // might work if the field is a boolean
        // It would be possible to print the object that the field is accessed on, but that won't provide much value

        // for pat in { ... }
        syn::Expr::ForLoop(_) => {
            // we generate our own error, because the compiler just says "expected bool, found ()"
            let msg = "Expected a boolean expression, found a for loop";
            return Error::err_spanned(e, msg);
        }

        // group with invisible delimiters?
        syn::Expr::Group(syn::ExprGroup { expr, .. }) => {
            return eval_expr(*expr, setup, expression, message_parts, dynamic_args);
        }

        // if cond { ... } else { ... }
        syn::Expr::If(_) => todo!("print condition"),

        // a[b]
        syn::Expr::Index(_) => todo!("print index"),

        // _
        syn::Expr::Infer(_) => (), // let the compiler generate the error

        // let pat = expr
        syn::Expr::Let(_) => {
            // we have to generate our own error, because the produced code is `if #expression`, which would become `if let ...` 😂
            let msg = "Expected a boolean expression, found a let statement";
            return Error::err_spanned(e, msg);
        }

        // lit
        syn::Expr::Lit(syn::ExprLit { lit, .. }) => {
            if let syn::Lit::Bool(val) = lit {
                // The sane behavior would be to give an error here, because `assert!(true)` is a no-op
                // and `assert!(false)` is a panic. But that would be boring.
                message_parts.clear();
                dynamic_args.clear();
                let msg = if val.value {
                    return Ok(()); // should already be handled by the assert_true_flavor
                } else {
                    "Surprisingly, `false` did not evaluate to true"
                };
                message_parts.push(quote! { #msg });
                *expression = quote! { false }; // yep, we're going to panic either way
            } else {
                // let the compiler generate the error
            }
        }

        // loop { ... }
        syn::Expr::Loop(_) => (), // might work if the loop breaks with a boolean

        // some_macro!(...)
        syn::Expr::Macro(_) => (), // not touching this

        // match expr { ... }
        syn::Expr::Match(_) => todo!("print match"),

        // obj.method(args...)
        syn::Expr::MethodCall(_) => todo!("print object and args"),

        // (expr)
        syn::Expr::Paren(syn::ExprParen { expr, .. }) => {
            return eval_expr(*expr, setup, expression, message_parts, dynamic_args);
        }

        // some::path::<of>::stuff
        syn::Expr::Path(_) => (), // might be a variable of type bool, otherwise let the compiler generate the error

        // a..b
        syn::Expr::Range(_) => (), // let the compiler generate the error

        syn::Expr::Reference(_) => todo!(),
        syn::Expr::Repeat(_) => todo!(),
        syn::Expr::Return(_) => todo!(),
        syn::Expr::Struct(_) => todo!(),
        syn::Expr::Try(_) => todo!(),
        syn::Expr::TryBlock(_) => todo!(),
        syn::Expr::Tuple(_) => todo!(),
        syn::Expr::Unary(_) => todo!(),
        syn::Expr::Unsafe(_) => todo!(),
        syn::Expr::Verbatim(_) => todo!(),
        syn::Expr::While(_) => todo!(),
        syn::Expr::Yield(_) => todo!(),
        _ => todo!(),
    }
    Ok(())
}

fn stretched_span(a: &impl ToTokens, b: &impl ToTokens) -> (TokenStream, TokenStream) {
    let a_span = a.to_token_stream().into_iter().next().unwrap().span();
    let a_var = quote_spanned! {a_span => __assert_lhs};
    let b_span = b.to_token_stream().into_iter().last().unwrap().span();
    let b_var = quote_spanned! {b_span => __assert_rhs};
    (a_var, b_var)
}

fn assert_true_flavor() -> TokenStream {
    quote! {
        let line = ::std::line!();
        if line % 100 == 69 {
            ::std::panic!("You actually used `assert!(true)`? Nice.");
        } else if line % 100 == 0 {
            ::std::panic!("Congratulations! You are the {}th person to use `assert!(true)`! You win a free panic!", line);
        } else if line % 10 == 0 {
            // Have the assertion randomly pass
        } else {
            const MESSAGES: &[&'static ::std::primitive::str] = &[
                "Ha! Did you think `assert!(true)` would do nothing? Fool!",
                "assertion `true` failed:\n  left: tr\n right: ue",
                "assertion `true` failed: `true` did not evaluate to true",
                "assertion `true` failed: `true` did not evaluate to true...? Huh? What? 🤔",
                "Undefined reference to `true`. Did you mean `false`?",
                "assertion `true` failed: `true` did not evaluate to true. What a surprise!",
                "The sky is green", // suggestion from GitHub Copilot
            ];
            let msg = MESSAGES[line as usize % MESSAGES.len()];
            ::std::panic!("{}", msg);
        }
    }
}
