#![doc = include_str!("../Readme.md")]

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};

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
    let expr_str = printable_expr_string(&expr); // the expression as a string

    if expr_str == "true" {
        return Ok(assert_true_flavor());
    } else if expr_str == "false" {
        return Ok(quote! {
            ::std::panic!("Surprisingly, `false` did not evaluate to true");
        });
    }

    let mut message_parts = vec![quote! { "assertion `", #expr_str, "` failed" }]; // the static parts of the message
    let mut dynamic_args = vec![]; // the dynamic format arguments to the message
    let mut setup = TokenStream::new(); // setup code to evaluate before the expression
    let mut ident_deduplicator = 0; // to avoid identical names on recursive calls to eval_expr

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
        &mut ident_deduplicator,
    )?;

    let output = quote! {{
        #setup
        if #expression {
            // using an empty if instead of `!(#expression)` to avoid messing with the spans in `expression`
        } else {
            ::std::panic!(::std::concat!(#(#message_parts),*), #(#dynamic_args),*);
        }
    }};

    Ok(output)
}

fn eval_expr(
    e: syn::Expr,
    setup: &mut TokenStream,
    expression: &mut TokenStream,
    message_parts: &mut Vec<TokenStream>,
    dynamic_args: &mut Vec<TokenStream>,
    ident_deduplicator: &mut usize,
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
            let (lhs, rhs) = stretched_span(&left, &right, ident_deduplicator);
            setup.extend(quote! {
                let #lhs = #left;
                let #rhs = #right;
            });
            *expression = quote! { #lhs #op #rhs };
            message_parts.push(quote! { "\n  left: {:?}\n right: {:?}" });
            dynamic_args.push(lhs.to_token_stream());
            dynamic_args.push(rhs.to_token_stream());
        }

        // { ... }
        syn::Expr::Block(syn::ExprBlock { block, .. }) => {
            return eval_block(
                block,
                setup,
                expression,
                message_parts,
                dynamic_args,
                ident_deduplicator,
            )
        }

        // break
        syn::Expr::Break(_) => {
            // we need to generate our own error, because break returns `!` so it compiles, but the assertion makes no sense
            let msg = "Expected a boolean expression, found a break statement";
            return Error::err_spanned(e, msg);
        }

        // function(args...)
        syn::Expr::Call(_) => todo!("split args"),

        // expr as ty
        syn::Expr::Cast(_) => (), // let the compiler generate the error.
        // Might work if expr is `true as bool`, which would actually be a workaround for the `assert!(true)` case

        // |args| { ... }
        syn::Expr::Closure(_) => (), // let the compiler generate the error

        // const { ... }
        syn::Expr::Const(syn::ExprConst { block, .. }) => {
            return eval_block(
                block,
                setup,
                expression,
                message_parts,
                dynamic_args,
                ident_deduplicator,
            )
        }

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
            return eval_expr(
                *expr,
                setup,
                expression,
                message_parts,
                dynamic_args,
                ident_deduplicator,
            );
        }

        // if cond { ... } else { ... }
        syn::Expr::If(_) => (), // might work if both branches return a boolean
        // There is a better failure message that could be printed by checking the condition, printing
        // its result (with left/right etc if false) and then using the eval_block to print the block,
        // but that is hugely complicated considering that the else branch should not be touched in the
        // true case and vice versa, and would basically require completely replacing the entire assert
        // structure for an if that is possibly nested somewhere deep in other blocks. If a user wants
        // fancy output from their if, they should just have separate asserts in the if and else blocks.

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
        syn::Expr::Lit(_) => (), // might work if the literal is a boolean
        // The base case for `assert!(true)` and `assert!(false)` was already caught in the initial
        // setup. This is the case where a recursive call contained a play `true` or `false`, so we
        // shall accept them without printing weird messages

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
            return eval_expr(
                *expr,
                setup,
                expression,
                message_parts,
                dynamic_args,
                ident_deduplicator,
            );
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

fn eval_block(
    mut block: syn::Block,
    setup: &mut TokenStream,
    expression: &mut TokenStream,
    message_parts: &mut Vec<TokenStream>,
    dynamic_args: &mut Vec<TokenStream>,
    ident_deduplicator: &mut usize,
) -> std::result::Result<(), Error> {
    let Some(last_stmt) = block.stmts.pop() else {
        return Ok(()); // let the compiler generate the error
    };
    let syn::Stmt::Expr(expr, None) = last_stmt else {
        return Ok(()); // let the compiler generate the error
    };

    let condition_string = printable_expr_string(&expr);
    let message = format!(
        "\n caused by: block return assertion `{}` failed",
        condition_string
    );
    message_parts.push(quote! { #message });

    for stmt in block.stmts {
        setup.extend(stmt.to_token_stream());
    }

    eval_expr(
        expr,
        setup,
        expression,
        message_parts,
        dynamic_args,
        ident_deduplicator,
    )
}

fn printable_expr_string(expr: &syn::Expr) -> String {
    expr.to_token_stream()
        .to_string()
        .replace('{', "{{")
        .replace('}', "}}")
        .replace(" ;", ";") // workaround for syn? on older rust compilers? inserting a random space
}

fn make_ident(name: &str, span: Span, ident_deduplicator: &mut usize) -> syn::Ident {
    let name = format!("__assert_{}_{}", name, *ident_deduplicator);
    *ident_deduplicator += 1;
    syn::Ident::new(&name, span)
}

fn stretched_span(
    lhs: &impl ToTokens,
    rhs: &impl ToTokens,
    ident_deduplicator: &mut usize,
) -> (syn::Ident, syn::Ident) {
    let lhs_span = lhs.to_token_stream().into_iter().next().unwrap().span();
    let lhs_var = make_ident("lhs", lhs_span, ident_deduplicator);
    let rhs_span = rhs.to_token_stream().into_iter().last().unwrap().span();
    let rhs_var = make_ident("rhs", rhs_span, ident_deduplicator);
    (lhs_var, rhs_var)
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
