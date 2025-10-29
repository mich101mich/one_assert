#![deny(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::bare_urls
)]

//! # One Assert
//!
//! ### TL;DR
//! Why have separate macros for `assert_eq` and `assert_ne` (and `assert_gt` etc. with other crates) when you
//! can just get the same output with `assert!(a == b)` (or `assert!(a != b)`, `assert!(a > b)`, ...)?
//! This crate provides a single `assert!` macro that analyzes the expression to provide more detailed output on failure.
//!
//! ### Introduction
//!
//! Rust's standard library provides the [`assert`](https://doc.rust-lang.org/std/macro.assert.html),
//! [`assert_eq`](https://doc.rust-lang.org/std/macro.assert_eq.html) and
//! [`assert_ne`](https://doc.rust-lang.org/std/macro.assert_ne.html) macros.
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
//! As you can see, `assert_eq` is able to provide detailed info on what the individual values were.\
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
//!      left: 1
//!     right: 2"
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
//!      left: 1
//!     right: 2"
//! );
//! ```
//!
//! ### Examples
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
//!      left: 1
//!     right: 2"
//! );
//!
//! let msg = catch_panic!({ one_assert::assert!(x != 1, "x ({}) should not be 1", x); });
//! assert_eq!(msg, "assertion `x != 1` failed: x (1) should not be 1
//!      left: 1
//!     right: 1"
//! );
//!
//! let s = "Hello World";
//! let msg = catch_panic!({ one_assert::assert!(s.starts_with("hello")); });
//! assert_eq!(msg, r#"assertion `s.starts_with("hello")` failed
//!      self: "Hello World"
//!     arg 0: "hello""#
//! );
//! ```
//!
//! ### Limitations
//! - **Several Components need to implement [`Debug`]**
//!   - The macro will take whatever part of the expression is considered useful and debug print it.
//!     This means that those parts need to implement [`Debug`].
//!   - What is printed as part of any given expression type is subject to change, so it is recommended
//!     to only use this in code where pretty much everything implements `Debug`.
//! - **`Debug` printing happens even if the assertion passes**
//!   - Because this macro prints more than just the two sides of an `==` or `!=` comparison, it has to
//!     deal with the fact that some values are moved during the evaluation of the expression. This means
//!     that the values have to be printed in advance.
//!   - Consequence: **Don't use this macro in performance-critical code**.
//!   - Note however, that the expression and each part of it is only **evaluated** once.
//!     - (Though it is also worth noting that fail-fast operators like `&&` might normally only evaluate
//!       the left side and stop, but with this macro it will always evaluate both sides)
//!
//! ### Changelog
//! See [Changelog.md](https://github.com/mich101mich/one_assert/blob/master/Changelog.md)

use std::{borrow::Borrow, fmt::Write};

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
                    let msg = format!("incomplete expression: {e}");
                    syn::Error::new_spanned(span_source, msg) // checked in tests/fail/malformed_expr.rs
                } else if let Ok(comma) = input.parse::<syn::Token![,]>() {
                    // syn's error would point at the ',' saying "expected an expression"
                    let msg = format!("Expression before the comma is incomplete: {e}");
                    syn::Error::new_spanned(comma, msg) // checked in tests/fail/malformed_expr.rs
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
            return Err(syn::Error::new(e.span(), msg)); // checked in tests/fail/malformed_parameters.rs
        } else {
            format = input.parse()?;
        }

        Ok(Args { expr, format })
    }
}

/// The main macro that is used to check a condition and panic if it is false.
///
/// # Syntax
/// ```text
/// assert!(condition: expression);
/// assert!(condition: expression, message: format_string, args...: format_args);
/// ```
/// Parameters:
/// - `condition`: The condition that should be checked. If it evaluates to `false`, the assertion fails.
///   Can be any expression that evaluates to `bool`.
/// - `message`: An optional message that is displayed if the assertion fails. This message can contain `{}`
///   placeholders for dynamic arguments. See [`format_args`] for more information.
/// - `args`: Arguments that are only evaluated if the assertion fails. These arguments are passed to
///   `format_args` to replace the `{}` placeholders in the message.
///
/// # Examples
/// See the crate-level documentation for examples.
#[proc_macro]
pub fn assert(input: TokenStream1) -> TokenStream1 {
    let input = syn::parse_macro_input!(input as Args);
    match assert_internal(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into(),
    }
}

struct Variable {
    /// Name of the variable
    name: String,
    /// Debug-printed value
    debug_value: TokenStream,
    /// Flag if the variable might be moved by the condition
    might_move: bool,
}

struct State {
    /// Code that sets up the variables for the assertion
    setup: TokenStream,
    /// The message that is displayed if the assertion fails. Must contain one `{}` for each dynamic argument
    format_message: String,
    /// Arguments that are only evaluated if the assertion fails
    dynamic_args: Vec<TokenStream>,
    /// Pairs of (variable name, debug-printed value) that are used in the assertion and should be printed in the error message
    variables: Vec<Variable>,
    /// Contains `unsafe` if the assertion should be wrapped in an unsafe block
    possibly_unsafe: TokenStream,
    /// Optional `!`-token if the condition is negated
    not_token: Option<syn::token::Not>,
    /// Counter for creating unique identifiers
    next_ident_id: usize,
}

impl State {
    fn new() -> Self {
        Self {
            setup: TokenStream::new(),
            format_message: String::new(),
            dynamic_args: vec![],
            variables: vec![],
            possibly_unsafe: TokenStream::new(),
            not_token: None,
            next_ident_id: 0,
        }
    }

    /// Ensure that there is no conflict between identifiers in the generated code by adding an incrementing number to each identifier
    fn create_ident(&mut self, name: &str) -> syn::Ident {
        let name = format!("__one_assert_{}_{}", name, self.next_ident_id);
        self.next_ident_id += 1;
        syn::Ident::new(&name, Span::call_site())
    }

    /// Create a variable from an expression and store it in the setup code.
    ///
    /// It is assumed that the expression moves the value, so it needs to be debug printed in advance
    fn add_moving_var(
        &mut self,
        expr: impl Borrow<syn::Expr>,
        identifier: &str,
        display: &str,
    ) -> TokenStream {
        self.add_var_internal(expr, identifier, display, true)
    }

    /// Create a variable from an expression and store it in the setup code.
    ///
    /// The variable is only used in a borrowed form, so we can debug print only when needed
    fn add_borrowed_var(
        &mut self,
        expr: impl Borrow<syn::Expr>,
        identifier: &str,
        display: &str,
    ) -> TokenStream {
        self.add_var_internal(expr, identifier, display, false)
    }

    fn add_var_internal(
        &mut self,
        expr: impl Borrow<syn::Expr>,
        identifier: &str,
        display: &str,
        might_move: bool,
    ) -> TokenStream {
        let expr = expr.borrow();
        let var_access = if matches!(expr, syn::Expr::Path(_)) {
            // could be a variable of a type that doesn't implement Copy, so we can't store it by value.
            // Instead, we just use the variable directly.
            expr.to_token_stream()
        } else {
            // any other expression. Compute the result once and store it
            let var_ident = self.create_ident(identifier);
            self.setup.extend(quote! {
                let #var_ident = __OneAssertWrapper(#expr);
            });

            // See note at the end of the file for an explanation on the span manipulation here
            let expr_span = utils::FullSpan::from_spanned(&expr);
            expr_span.apply(quote! { #var_ident }, quote! { .0 })
        };

        let debug_value = if might_move {
            let var_debug_str = self.create_ident(&format!("{identifier}_str"));
            self.setup.extend(quote! {
                let #var_debug_str = ::std::format!("{:?}", #var_access);
            });
            var_debug_str.to_token_stream()
        } else {
            var_access.clone()
        };

        // store variable for now instead of printing it immediately, so that all the variables can be aligned
        self.variables.push(Variable {
            name: display.to_owned(),
            debug_value,
            might_move,
        });

        var_access
    }

    /// Add a `Name: Value` block for all currently stored variables to the format message
    fn resolve_variables(&mut self) {
        let max_name_len = self
            .variables
            .iter()
            .map(|Variable { name, .. }| name.len())
            .max()
            .unwrap_or(0);

        for Variable {
            name,
            debug_value,
            might_move,
        } in self.variables.drain(..)
        {
            write!(self.format_message, "\n    {name:>max_name_len$}: ").unwrap();
            self.format_message += if might_move { "{}" } else { "{:?}" };
            self.dynamic_args.push(debug_value);
        }
    }

    /// Adds a "caused by" message to the format message
    fn add_cause(&mut self, cause: &str) {
        write!(self.format_message, "\n  caused by: {cause}").unwrap();
    }
}

fn assert_internal(input: Args) -> Result<TokenStream> {
    let Args { mut expr, format } = input;

    let expr_str = printable_expr_string(&expr);

    if expr_str == "true" {
        return Ok(assert_true_flavor());
    } else if expr_str == "false" {
        return Ok(quote! {
            ::std::panic!("surprisingly, `false` did not evaluate to true")
        });
    }

    let mut state = State::new();
    // A wrapper type to create multi-token variables for span manipulation
    state.setup = quote! { struct __OneAssertWrapper<T>(T); };
    state.format_message = format!("assertion `{expr_str}` failed");

    if !format.is_empty() {
        state.format_message += ": {}";
        state
            .dynamic_args
            .push(quote! { ::std::format_args!(#format) });
    }

    // inline any parentheses and invisible groups as they are not necessary
    while let syn::Expr::Paren(syn::ExprParen { expr: inner, .. })
    | syn::Expr::Group(syn::ExprGroup { expr: inner, .. }) = expr
    {
        expr = *inner;
    }

    let output = eval_expr(expr, state)?;
    // println!();
    // println!();
    // println!("{}", output);
    // println!();
    // println!();
    Ok(output)
}

#[allow(clippy::match_same_arms)] // every arm needs its own reasoning and consideration
fn eval_expr(e: syn::Expr, mut state: State) -> Result<TokenStream> {
    let mut assert_condition = e.to_token_stream();
    match e {
        // [a, b, c, d]
        syn::Expr::Array(_) => {} // let the compiler generate the error

        // a = b
        syn::Expr::Assign(syn::ExprAssign { eq_token, .. }) => {
            let msg = "Expected a boolean expression, found an assignment. Did you intend to compare with `==`?";
            return Error::err_spanned(eq_token, msg); // checked in tests/fail/expr/assign.rs
        }

        // async { ... }
        syn::Expr::Async(_) => {
            let msg = "Expected a boolean expression, found an async block. Did you intend to await a future?";
            return Error::err_spanned(e, msg); // checked in tests/fail/expr/async.rs
        }

        // future.await
        syn::Expr::Await(_) => {} // might work if the future resolves to a boolean and the assert is in an async context

        // left <op> right
        syn::Expr::Binary(syn::ExprBinary {
            left,
            op,
            right,
            attrs,
        }) => {
            match op {
                // logic operators => preserve fail-fast behavior where expressions like `!vec.empty() && vec[0].ok()` work
                syn::BinOp::And(_) => {
                    // `&&` logic: if first is true, evaluate second. Otherwise skip second
                    state.resolve_variables();

                    let State {
                        setup,
                        format_message,
                        dynamic_args,
                        possibly_unsafe,
                        not_token,
                        ..
                    } = state;

                    if let Some(not_token) = not_token {
                        assert_condition = quote! { #not_token ( #assert_condition ) };
                    }

                    let output = quote! {
                        #[allow(unused)]
                        #possibly_unsafe {
                            #setup
                            if #assert_condition {
                                // using an empty if instead of `!(#expression)` to avoid messing with the spans in `expression`.
                                // And to produce a better error: "expected bool, found <type>"
                                // instead of: "no unary operator '!' implemented for <type>"
                            } else {
                                ::std::panic!(#format_message, #(#dynamic_args),*);
                            }
                        }
                    };
                    return Ok(output);
                }
                syn::BinOp::Or(_) => todo!(),

                // comparison operators, handle as expected
                syn::BinOp::Eq(_)
                | syn::BinOp::Lt(_)
                | syn::BinOp::Le(_)
                | syn::BinOp::Ne(_)
                | syn::BinOp::Ge(_)
                | syn::BinOp::Gt(_) => {
                    let lhs = state.add_borrowed_var(left, "lhs", "left");
                    let rhs = state.add_borrowed_var(right, "rhs", "right");
                    assert_condition = quote! { #(#attrs)* #lhs #op #rhs };
                }

                // operators that might return a bool, but might move the inputs
                syn::BinOp::Add(_)
                | syn::BinOp::Sub(_)
                | syn::BinOp::Mul(_)
                | syn::BinOp::Div(_)
                | syn::BinOp::Rem(_)
                | syn::BinOp::BitXor(_)
                | syn::BinOp::BitAnd(_)
                | syn::BinOp::BitOr(_)
                | syn::BinOp::Shl(_)
                | syn::BinOp::Shr(_) => {
                    let lhs = state.add_moving_var(left, "lhs", "left");
                    let rhs = state.add_moving_var(right, "rhs", "right");
                    assert_condition = quote! { #(#attrs)* #lhs #op #rhs };
                }

                // operators that don't return anything
                syn::BinOp::AddAssign(_)
                | syn::BinOp::SubAssign(_)
                | syn::BinOp::MulAssign(_)
                | syn::BinOp::DivAssign(_)
                | syn::BinOp::RemAssign(_)
                | syn::BinOp::BitXorAssign(_)
                | syn::BinOp::BitAndAssign(_)
                | syn::BinOp::BitOrAssign(_)
                | syn::BinOp::ShlAssign(_)
                | syn::BinOp::ShrAssign(_) => {
                    // we generate our own error, because the compiler just says "expected bool, found ()"
                    let msg = "Expected a boolean expression, found an assignment";
                    return Error::err_spanned(op, msg); // checked in tests/fail/expr/binary.rs
                }

                // unknown operator, keep as-is
                _ => {}
            }
        }

        // { ... }
        syn::Expr::Block(_) => {} // We could check the condition at the end of the block, but really, this shouldn't be done in the assert

        // break
        syn::Expr::Break(_) => {
            // we need to generate our own error, because break returns `!` so it compiles, but the assertion makes no sense
            let msg = "Expected a boolean expression, found a break statement";
            return Error::err_spanned(e, msg); // checked in tests/fail/expr/break.rs
        }

        // function(args...)
        syn::Expr::Call(syn::ExprCall {
            args,
            func,
            paren_token,
            attrs,
        }) if !args.is_empty() => {
            let index_len = (args.len() - 1).to_string().len();
            let out_args = args.iter().enumerate().map(|(i, arg)| {
                state.add_moving_var(arg, &format!("arg{i}"), &format!("arg {i:>index_len$}"))
            });

            // output: `quote! { #(#attrs)* #func ( #(#out_args),* ) }` except we want to use the original parentheses for span purposes
            assert_condition = quote! { #(#attrs)* #func };
            paren_token.surround(&mut assert_condition, |out| {
                out.extend(quote! { #(#out_args),* })
            });
        }
        // function() // no args
        syn::Expr::Call(_) => {} // just a plain function call that returns a boolean or not. Nothing more to add here

        // expr as ty
        syn::Expr::Cast(_) => {} // let the compiler generate the error.
        // Might work if expr is `true as bool`, which would actually be a workaround for the `assert!(true)` case

        // |args| { ... }
        syn::Expr::Closure(_) => {} // let the compiler generate the error

        // const { ... }
        syn::Expr::Const(_) => {} // same as Expr::Block

        // continue
        syn::Expr::Continue(_) => {
            // we need to generate our own error, because continue returns `!` so it compiles, but the assertion makes no sense
            let msg = "Expected a boolean expression, found a continue statement";
            return Error::err_spanned(e, msg); // checked in tests/fail/expr/continue.rs
        }

        // obj.field
        syn::Expr::Field(_) => {} // might work if the field is a boolean
        // It would be possible to print the object that the field is accessed on, but that won't provide much value.
        // The only part of the object that is interesting is the field, and that is already evaluated as the assertion.

        // for pat in { ... }
        syn::Expr::ForLoop(_) => {
            // we generate our own error, because the compiler just says "expected bool, found ()"
            let msg = "Expected a boolean expression, found a for loop";
            return Error::err_spanned(e, msg); // checked in tests/fail/expr/forloop.rs
        }

        // group with invisible delimiters
        syn::Expr::Group(syn::ExprGroup { expr, .. }) => {
            // usually generated by resolving another macro. To us, it is just an expression
            return eval_expr(*expr, state);
        }

        // if cond { ... } else { ... }
        syn::Expr::If(_) => {} // we could analyze the condition and/or blocks, but that is a bit excessive.
        // If you want better output, put the assert in the if and not the other way around.

        // expr[index]
        syn::Expr::Index(syn::ExprIndex {
            index,
            expr,
            attrs,
            bracket_token,
        }) => {
            if !matches!(*index, syn::Expr::Lit(_)) {
                let index = state.add_moving_var(index, "index", "index");
                // output: `quote! { #(#attrs)* #expr [#index] }` except we want to use the original brackets for span purposes
                assert_condition = quote! { #(#attrs)* #expr };
                bracket_token.surround(&mut assert_condition, |out| index.to_tokens(out));
            }
            // not printing literals, because their value is already known.

            // not printing the indexed object, because the output could be huge.
            // If we knew the object was a form of array, then we could would slice the range around the index,
            // but it could also be a HashMap or a custom type, so we can't do that.
        }

        // _
        syn::Expr::Infer(_) => {} // let the compiler generate the error

        // let pat = expr
        syn::Expr::Let(_) => {
            // we have to generate our own error, because the produced code is `if #expression`, which would become `if let ...` 😂
            let msg = "Expected a boolean expression, found a let statement";
            return Error::err_spanned(e, msg); // checked in tests/fail/expr/let.rs
        }

        // lit
        syn::Expr::Lit(_) => {} // might work if the literal is a boolean
        // The base case for `assert!(true)` and `assert!(false)` was already caught in the initial
        // setup. This is the case where a recursive call contained a plain `true` or `false`, so we
        // shall accept them without printing weird messages

        // loop { ... }
        syn::Expr::Loop(_) => {} // might work if the loop breaks with a boolean
        // If somebody has too much free time on their hands they can go ahead and write some recursive
        // block parsing code to find all the `break` statements so that the error message can say
        // which one was triggered. This would be really useful info for the user, but it's a lot of effort
        // for something that probably nobody will ever see.
        // Side note: Finding a `break` would actually help with the case where there are no breaks, because
        // then the loop would just never return (`!`), so the compiler doesn't complain but the assertion
        // makes no sense.

        // some_macro!(...)
        syn::Expr::Macro(_) => {} // not touching this

        // match expr { ... }
        syn::Expr::Match(_) => {} // we could check which variant matched etc. etc., but that is excessive for an assert

        // receiver.method(args...)
        syn::Expr::MethodCall(syn::ExprMethodCall {
            receiver,
            method,
            turbofish,
            args,
            attrs,
            dot_token,
            paren_token,
        }) => {
            let obj = state.add_moving_var(receiver, "object", "self");
            let index_len = (args.len().saturating_sub(1)).to_string().len();
            let out_args = args.iter().enumerate().map(|(i, arg)| {
                state.add_moving_var(arg, &format!("arg{i}"), &format!("arg {i:>index_len$}"))
            });

            // output: `quote! { #(attrs)* #obj #dot_token #method #turbofish ( #(#out_args),* ) }` except we want to use the original parentheses for span purposes
            assert_condition = quote! { #(#attrs)* #obj #dot_token #method #turbofish };
            paren_token.surround(&mut assert_condition, |out| {
                out.extend(quote! { #(#out_args),* })
            });
        }

        // (expr)
        syn::Expr::Paren(syn::ExprParen { expr, .. }) => {
            // extra parenthesis are redundant and will be stripped
            return eval_expr(*expr, state);
        }

        // some::path::<of>::stuff
        syn::Expr::Path(_) => {} // might be a constant of type bool, otherwise let the compiler generate the error

        // a..b
        syn::Expr::Range(_) => {} // let the compiler generate the error

        // &expr
        syn::Expr::Reference(_) => {} // let the compiler generate the error

        // [x; n]
        syn::Expr::Repeat(_) => {} // let the compiler generate the error

        // return expr
        syn::Expr::Return(_) => {
            // we need to generate our own error, because return returns `!` so it compiles, but the assertion makes no sense
            let msg = "Expected a boolean expression, found a return statement";
            return Error::err_spanned(e, msg); // checked in tests/fail/expr/return.rs
        }

        // MyStruct { field: value }
        syn::Expr::Struct(_) => {
            // we generate our own error, because the compiler will suggest adding parentheses around the struct literal
            let msg = "Expected a boolean expression, found a struct literal";
            return Error::err_spanned(e, msg);
        }

        // expr?
        syn::Expr::Try(_) => {} // might work if expr is a Result<bool> or similar, otherwise let the compiler generate the error

        // (a, b, c)
        syn::Expr::Tuple(_) => {} // let the compiler generate the error

        // !expr
        syn::Expr::Unary(syn::ExprUnary {
            expr,
            op: syn::UnOp::Not(not_token),
            attrs,
        }) => {
            if !attrs.is_empty() {
                return Error::err_spanned(
                    &attrs[0],
                    "Attributes are not supported in this position",
                );
            }

            let expr_str = printable_expr_string(&expr);

            // praying that people didn't override the `Not` operator for their types
            if state.not_token.is_none() {
                state.not_token = Some(not_token);
                state.add_cause(&format!("expression {expr_str} evaluated to true"));
            } else {
                // double negation cancels out
                state.not_token = None;
                state.add_cause(&format!("expression {expr_str} evaluated to false"));
            }

            return eval_expr(*expr, state);
        }
        // op expr
        syn::Expr::Unary(_) => {} // just leave it as-is

        // unsafe { ... }
        syn::Expr::Unsafe(_) => {} // Same as Expr::Block

        // something
        syn::Expr::Verbatim(_) => {} // even syn doesn't know what this is, so we can't do anything with it

        // while cond { ... }
        syn::Expr::While(_) => {
            // we generate our own error, because the compiler just says "expected bool, found ()"
            let msg = "Expected a boolean expression, found a while loop";
            return Error::err_spanned(e, msg);
        }

        _ => {} // we don't know what this is, so we can't do anything with it
                // this includes unstable syntax that is already contained in syn, like
                // syn::Expr::TryBlock
                // syn::Expr::Yield
    }

    state.resolve_variables();

    let State {
        setup,
        format_message,
        dynamic_args,
        possibly_unsafe,
        not_token,
        ..
    } = state;

    if let Some(not_token) = not_token {
        assert_condition = quote! { #not_token ( #assert_condition ) };
    }

    let output = quote! {
        #[allow(unused)]
        #possibly_unsafe {
            #setup
            if #assert_condition {
                // using an empty if instead of `!(#expression)` to avoid messing with the spans in `expression`.
                // And to produce a better error: "expected bool, found <type>"
                // instead of: "no unary operator '!' implemented for <type>"
            } else {
                ::std::panic!(#format_message, #(#dynamic_args),*);
            }
        }
    };
    Ok(output)
}

fn printable_expr_string(expr: &impl ToTokens) -> String {
    expr.to_token_stream()
        .to_string()
        .replace('{', "{{")
        .replace('}', "}}")
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
            ];
            let msg = MESSAGES[line as usize % MESSAGES.len()];
            ::std::panic!("{}", msg);
        }
    }
}

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
