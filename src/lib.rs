//! # One Assert
//!
//! ### Introduction
//!
//! Rust's standard library provides the [`assert`](https://doc.rust-lang.org/std/macro.assert.html),
//! [`assert_eq`](https://doc.rust-lang.org/std/macro.assert_eq.html) and [`assert_ne`](https://doc.rust-lang.org/std/macro.assert_ne.html).
//! There are however some inconveniences with these, like how there are no specialization for other inequalities, like
//! `assert_ge` for `>=` etc, or how the names only differ in one or two letters (`assert_eq`, `assert_ne`,
//! `assert_ge`, `assert_gt`, ...) and are thus easy to mix up at a glance.
//!
//! The main reason for not adding more macros is that they can be represented just fine with `assert!(a >= b)`,
//! so there is no need for a separate macro for every use case.
//!
//! But that begs the question: Why do we have `assert_eq` and `assert_ne` in the first place?
//!
//! The practical reason: `assert_eq!(a, b)` provides better output than `assert!(a == b)`:
//! ```
//! # macro_rules! catch_panic {
//! #     ($block: block) => {{
//! #         let error = std::panic::catch_unwind(move || $block).unwrap_err();
//! #         error
//! #             .downcast_ref::<&'static str>()
//! #             .map(|s| s.to_string())
//! #             .unwrap_or_else(|| *error.downcast::<String>().unwrap())
//! #     }};
//! # }
//! let x = 1;
//! let msg = catch_panic!({ assert!(x == 2); });
//! assert_eq!(msg, "assertion failed: x == 2");
//!
//! let msg = catch_panic!({ assert_eq!(x, 2); });
//! assert_eq!(msg, "assertion `left == right` failed
//!   left: 1
//!  right: 2"
//! );
//! ```
//! As you can see, `assert_eq` is able to provide detailed info on what the individual values were.  
//! But: That doesn't have to be the case. Rust has hygienic and procedural macros, so we can just **make `assert!(a == b)` work the same as `assert_eq!(a, b)`**:
//! ```
//! # macro_rules! catch_panic {
//! #     ($block: block) => {{
//! #         let error = std::panic::catch_unwind(move || $block).unwrap_err();
//! #         error
//! #             .downcast_ref::<&'static str>()
//! #             .map(|s| s.to_string())
//! #             .unwrap_or_else(|| *error.downcast::<String>().unwrap())
//! #     }};
//! # }
//! let x = 1;
//! let msg = catch_panic!({ one_assert::assert!(x == 2); });
//! assert_eq!(msg, "assertion `x == 2` failed
//!   left: 1
//!  right: 2"
//! );
//! ```
//! And now we can expand this to as many operators as we want:
//! ```
//! # macro_rules! catch_panic {
//! #     ($block: block) => {{
//! #         let error = std::panic::catch_unwind(move || $block).unwrap_err();
//! #         error
//! #             .downcast_ref::<&'static str>()
//! #             .map(|s| s.to_string())
//! #             .unwrap_or_else(|| *error.downcast::<String>().unwrap())
//! #     }};
//! # }
//! let x = 1;
//! let msg = catch_panic!({ one_assert::assert!(x > 2); });
//! assert_eq!(msg, "assertion `x > 2` failed
//!   left: 1
//!  right: 2"
//! );
//! ```
//!
//! ### Features
//!
//! - copy AddsToBool example
//!
//! ### Limitations
//! - everything has to be debug
//! - everything has to be debug-printed REGARDLESS of assertion success or failure
//!   - reason: Actual expression might move the values, so we can't just store them and print them later
//!   - result: Don't use this in performance critical code

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

/// Ensure that there is no conflict between identifiers in the generated code by adding an incrementing number to each identifier
struct UniqueIdentCreator(usize);
impl UniqueIdentCreator {
    fn create(&mut self, name: &str) -> syn::Ident {
        let name = format!("__one_assert_{}_{}", name, self.0);
        self.0 += 1;
        syn::Ident::new(&name, Span::call_site())
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

    let mut format_message = format!("assertion `{expr_str}` failed"); // the static parts of the message
    let mut dynamic_args = vec![]; // the dynamic format arguments to the message
    let mut setup = TokenStream::new(); // setup code to evaluate before the expression
    let mut ident_id = UniqueIdentCreator(0); // to avoid identical names on recursive calls to eval_expr

    if !format.is_empty() {
        format_message += ": {}";
        dynamic_args.push(quote! { ::std::format_args!(#format) });
    }

    eval_expr(
        expr,
        &mut setup,
        &mut expression,
        &mut format_message,
        &mut dynamic_args,
        &mut ident_id,
    )?;

    let output = quote! {
        #[allow(unused)]
        {
            /// A wrapper type to create multi-token variables for span manipulation
            struct __OneAssertWrapper<T>(T);

            #setup
            if #expression {
                // using an empty if instead of `!(#expression)` to avoid messing with the spans in `expression`.
                // And to produce a better error: "expected bool, found <type>" instead of
                // "no unary operator '!' implemented for <type>"
            } else {
                ::std::panic!(#format_message, #(#dynamic_args),*);
            }
        }
    };

    Ok(output)
}

fn eval_expr(
    e: syn::Expr,
    setup: &mut TokenStream,
    expression: &mut TokenStream,
    format_message: &mut String,
    dynamic_args: &mut Vec<TokenStream>,
    ident_id: &mut UniqueIdentCreator,
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

        // left <op> right
        syn::Expr::Binary(syn::ExprBinary {
            left, op, right, ..
        }) => {
            let (lhs, lhs_str) = var_from_expr(*left, "lhs", setup, ident_id);
            let (rhs, rhs_str) = var_from_expr(*right, "rhs", setup, ident_id);
            *expression = quote! { #lhs #op #rhs };
            *format_message += "\n  left: {}\n right: {}";
            dynamic_args.push(lhs_str);
            dynamic_args.push(rhs_str);
        }

        // { ... }
        syn::Expr::Block(syn::ExprBlock { block, .. }) => {
            return eval_block(
                block,
                setup,
                expression,
                format_message,
                dynamic_args,
                ident_id,
            )
        }

        // break
        syn::Expr::Break(_) => {
            // we need to generate our own error, because break returns `!` so it compiles, but the assertion makes no sense
            let msg = "Expected a boolean expression, found a break statement";
            return Error::err_spanned(e, msg);
        }

        // function(args...)
        syn::Expr::Call(syn::ExprCall { args, func, .. }) => {
            if args.is_empty() {
                return Ok(());
            }
            let index_len = (args.len() - 1).to_string().len();
            let mut out_args = vec![];
            for (i, arg) in args.into_iter().enumerate() {
                *format_message += &format!("\n arg {i:>index_len$}: {{}}");
                let (arg, arg_str) = var_from_expr(arg, &format!("arg{}", i), setup, ident_id);
                dynamic_args.push(arg_str);
                out_args.push(arg);
            }
            *expression = quote! { #func(#(#out_args),*) };
        }

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
                format_message,
                dynamic_args,
                ident_id,
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
                format_message,
                dynamic_args,
                ident_id,
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

        // obj[index]
        syn::Expr::Index(syn::ExprIndex { index, expr, .. }) => {
            // TODO: print the object
            if matches!(*index, syn::Expr::Lit(_)) {
                return Ok(()); // no reason to print a literal
            }
            let (index, index_str) = var_from_expr(*index, "index", setup, ident_id);
            *expression = quote! { #expr[#index] };
            *format_message += "\n index: {}";
            dynamic_args.push(index_str);
        }

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
        // setup. This is the case where a recursive call contained a plain `true` or `false`, so we
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
                format_message,
                dynamic_args,
                ident_id,
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
    format_message: &mut String,
    dynamic_args: &mut Vec<TokenStream>,
    ident_id: &mut UniqueIdentCreator,
) -> std::result::Result<(), Error> {
    let Some(last_stmt) = block.stmts.pop() else {
        return Ok(()); // let the compiler generate the error
    };
    let syn::Stmt::Expr(expr, None) = last_stmt else {
        return Ok(()); // let the compiler generate the error
    };

    let condition_string = printable_expr_string(&expr);
    *format_message += &format!(
        "\n caused by: block return assertion `{}` failed",
        condition_string
    );

    for stmt in block.stmts {
        setup.extend(stmt.to_token_stream());
    }

    eval_expr(
        expr,
        setup,
        expression,
        format_message,
        dynamic_args,
        ident_id,
    )
}

fn printable_expr_string(expr: &syn::Expr) -> String {
    let s = expr.to_token_stream().to_string();
    let mut iter = s.chars().peekable();
    let mut result = String::with_capacity(s.len() * 11 / 10);
    // the following code is just a series of replacements in a string, but I don't like that
    // String::replace creates a new string for every replacement, so I'm doing a manual
    // multi-replacement here
    while let Some(c) = iter.next() {
        match c {
            '{' => result.push_str("{{"), // would otherwise be interpreted as a placeholder
            '}' => result.push_str("}}"),
            ' ' => {
                if let Some(next) = iter.next_if(|c| matches!(c, ';' | '[')) {
                    // character that syn padded with a space where normal code wouldn't
                    result.push(next);
                } else {
                    result.push(' ');
                }
            }
            _ => result.push(c),
        }
    }
    result
}

fn var_from_expr(
    expr: syn::Expr,
    name: &str,
    setup: &mut TokenStream,
    ident_id: &mut UniqueIdentCreator,
) -> (TokenStream, TokenStream) {
    let var_ident = ident_id.create(name);
    let var_str_ident = ident_id.create(&format!("{}_str", name));

    let expr_span = utils::FullSpan::from_spanned(&expr);
    let var_access = expr_span.apply(quote! { #var_ident }, quote! { .0 });

    setup.extend(quote! {
        let #var_ident = __OneAssertWrapper(#expr);
        let #var_str_ident = ::std::format!("{:?}", #var_access);
    });
    (var_access, var_str_ident.to_token_stream())

    // # Span manipulation workaround:
    // Spans cannot be manipulated on stable rust right now (see <https://github.com/rust-lang/rust/issues/54725>).
    // This also applies to getting the full span of an expression, which requires joining the spans of the individual
    // tokens. On stable, .span() will just return the first token, meaning that if you have an expression like
    // `1 + 2` and a compiler error should be printed on the entire expression, it will instead only underline
    // the first token, the `1` in this case.
    // To work around this, the common approach (see syn::Error::new_spanned) is to bind the first and last token
    // of your code to the first and last individual span of the input, so that when the rust compiler wants to
    // underline the "entire" span, it will join the spans for us and underline the entire expression.
    // This requires that the code that should be underlined has more than one token, so that more than one span
    // can be bound to it. This function should create variable names, which are only one token long, so we need
    // to artificially create a multi-token variable. This is the point of the __OneAssertWrapper struct. It simply
    // contains the value of the variable, and any access will be written as `var.0` instead of `var`, giving us
    // the multi-token variable we need.
    //
    // ## Simplified but full example
    //
    // ### Without the span manipulation
    // Input: `assert!(1 + 2);`
    //
    // Output:
    // ```
    // let var = 1 + 2;
    // if var {} else { panic!("assertion failed"); }
    // ```
    //
    // This code would produce a compiler error like this:
    // ```
    // error: mismatched types
    //  1 | assert!(1 + 2);
    //              ^ expected bool, found {integer}
    // ```
    // which is not very helpful, because the error message only points at the first token of the expression.
    //
    // ### With the span manipulation
    // Input: `assert!(1 + 2);`
    //
    // Output:
    // ```
    // let var = __OneAssertWrapper(1 + 2);
    // if var.0 {} else { panic!("assertion failed"); }
    // ```
    // Note that the token-span assignment of the usage of `var.0` is as follows:
    // - `var` is assigned the span of the `1` from the input
    // - `.0` is assigned the span of the `2` from the input
    //
    // Produced error:
    // ```
    // error: mismatched types
    //  1 | assert!(1 + 2);
    //              ^^^^^ expected bool, found {integer}
    // ```
    // As you can see, the compiler wants to underline the full `var.0`, meaning it will end up underlining
    // everything between the original `1` and `2` tokens, which is exactly what we want.
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
