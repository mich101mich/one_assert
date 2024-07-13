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
                    let msg = format!("incomplete expression: {}", e);
                    syn::Error::new_spanned(span_source, msg) // checked in tests/fail/malformed_expr.rs
                } else if let Ok(comma) = input.parse::<syn::Token![,]>() {
                    // syn's error would point at the ',' saying "expected an expression"
                    let msg = format!("Expression before the comma is incomplete: {}", e);
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

#[proc_macro]
pub fn assert(input: TokenStream1) -> TokenStream1 {
    let input = syn::parse_macro_input!(input as Args);
    match assert_internal(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into(),
    }
}

struct State {
    /// Code that sets up the variables for the assertion
    setup: TokenStream,
    /// The message that is displayed if the assertion fails. Must contain one `{}` for each dynamic argument
    format_message: String,
    /// Arguments that are only evaluated if the assertion fails
    dynamic_args: Vec<TokenStream>,
    /// Pairs of (variable name, debug-printed value) that are used in the assertion and should be printed in the error message
    variables: Vec<(String, TokenStream)>,
    /// Whether the expression is in an unsafe block
    possibly_unsafe: TokenStream,
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
            next_ident_id: 0,
        }
    }

    fn fork(&self) -> Self {
        Self {
            setup: TokenStream::new(),                   // initial setup is shared
            format_message: self.format_message.clone(), // format message is printed by fork
            dynamic_args: self.dynamic_args.clone(),     // args are tied to the format message
            variables: self.variables.clone(),           // keep any non-resolved variables
            possibly_unsafe: TokenStream::new(),         // only the outermost block needs unsafe
            next_ident_id: self.next_ident_id,           // identifiers should be unique
        }
    }

    /// Ensure that there is no conflict between identifiers in the generated code by adding an incrementing number to each identifier
    fn create_ident(&mut self, name: &str) -> syn::Ident {
        let name = format!("__one_assert_{}_{}", name, self.next_ident_id);
        self.next_ident_id += 1;
        syn::Ident::new(&name, Span::call_site())
    }

    /// Create a variable from an expression and store it in the setup code
    fn add_var(&mut self, expr: syn::Expr, identifier: &str, display: &str) -> TokenStream {
        let var_debug_str = self.create_ident(&format!("{identifier}_str"));

        let var_access;
        if matches!(expr, syn::Expr::Path(_)) {
            // could be a variable of a type that doesn't implement Copy, so we can't store it by value.
            // Instead, we just use the variable directly.
            var_access = expr.to_token_stream();
            self.setup.extend(quote! {
                let #var_debug_str = ::std::format!("{:?}", #var_access);
            });
        } else {
            let var_ident = self.create_ident(identifier);

            // See note at the end of the file for an explanation on the span manipulation here
            let expr_span = utils::FullSpan::from_spanned(&expr);
            var_access = expr_span.apply(quote! { #var_ident }, quote! { .0 });

            self.setup.extend(quote! {
                let #var_ident = __OneAssertWrapper(#expr);
                let #var_debug_str = ::std::format!("{:?}", #var_access);
            });
        }

        // store variable for now instead of printing it immediately, so that all the variables can be aligned
        self.variables
            .push((display.to_owned(), var_debug_str.to_token_stream()));

        var_access
    }

    /// Add a `Name: Value` block for all currently stored variables to the format message
    fn resolve_variables(&mut self) {
        let max_name_len = self
            .variables
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(0);

        for (name, var_debug_str) in self.variables.drain(..) {
            self.format_message += &format!("\n    {name:>max_name_len$}: {{}}");
            self.dynamic_args.push(var_debug_str.to_token_stream());
        }
    }

    /// Adds a "caused by" message to the format message
    fn add_cause(&mut self, cause: &str) {
        self.format_message += &format!("\n  caused by: {}", cause);
    }
}

fn assert_internal(input: Args) -> Result<TokenStream> {
    let Args { expr, format } = input;

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

    eval_expr(expr, state)
}

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
            let lhs = state.add_var(*left, "lhs", "left");
            let rhs = state.add_var(*right, "rhs", "right");
            assert_condition = quote! { #(#attrs)* #lhs #op #rhs };
        }

        // { ... }
        syn::Expr::Block(syn::ExprBlock { block, .. }) => return eval_block(block, state),

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
            let out_args = args.into_iter().enumerate().map(|(i, arg)| {
                state.add_var(arg, &format!("arg{i}"), &format!("arg {i:>index_len$}"))
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
        syn::Expr::Const(syn::ExprConst { block, .. }) => return eval_block(block, state),
        // the way this is structured means you can technically assert a non-const block while pretending it's a const block,
        // but then again, why do you have a const block in an assert?

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

        // group with invisible delimiters?
        syn::Expr::Group(syn::ExprGroup { expr, .. }) => {
            return eval_expr(*expr, state);
        }

        // if cond { ... } else { ... }
        syn::Expr::If(syn::ExprIf {
            cond,
            then_branch,
            else_branch: Some((else_token, else_branch)),
            attrs,
            if_token,
        }) => {
            let condition_str = printable_expr_string(&cond);
            let condition =
                state.add_var(*cond, "condition", &format!("condition `{condition_str}`"));

            let then_branch = eval_block(then_branch, state.fork())?;
            let else_branches = recurse_else_branches(*else_branch, state.fork())?;

            state.resolve_variables(); // only resolve variables after the recursive calls so that the forks can align the conditions

            let State {
                setup,
                possibly_unsafe,
                ..
            } = state;

            let output = quote! {
                #[allow(unused)]
                #possibly_unsafe {
                    #setup
                    #(#attrs)* #if_token #condition {
                        #then_branch
                    } #else_token #else_branches
                }
            };
            return Ok(output);
        }
        syn::Expr::If(_) => {} // if without else: let the compiler generate the error

        // expr[index]
        syn::Expr::Index(syn::ExprIndex {
            index,
            expr,
            attrs,
            bracket_token,
        }) => {
            if !matches!(*index, syn::Expr::Lit(_)) {
                let index = state.add_var(*index, "index", "index");
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
        syn::Expr::Match(syn::ExprMatch {
            arms,
            expr,
            attrs,
            match_token,
            brace_token,
        }) => {
            let expr_str = printable_expr_string(&expr);
            let match_expr = state.add_var(*expr, "matched", "matched value");

            state.resolve_variables();

            let mut arms_output = TokenStream::new();
            for arm in arms {
                let syn::Arm {
                    pat,
                    guard,
                    body,
                    attrs,
                    fat_arrow_token,
                    ..
                } = arm;

                let guard = guard
                    .map(|(if_token, expr)| quote! { #if_token #expr })
                    .unwrap_or_default();

                let pattern = quote! { #pat #guard };

                let mut arm_state = state.fork();

                arm_state.add_cause(&format!(
                    "match {expr_str} entered arm `{}` where assertion `{}` failed",
                    printable_expr_string(&pattern),
                    printable_expr_string(&body)
                ));

                let assert_eval = eval_expr(*body, arm_state)?;

                arms_output.extend(quote! {
                    #(#attrs)* #pattern #fat_arrow_token {
                        #assert_eval
                    }
                });
            }

            // output: `quote! { #(#attrs)* #match_token #match_expr { #arms_output } }` except we want to use the original braces for span purposes
            let mut inner_tokens = quote! { #(#attrs)* #match_token #match_expr };
            brace_token.surround(&mut inner_tokens, |out| out.extend(arms_output));

            let State {
                setup,
                possibly_unsafe,
                ..
            } = state;

            let output = quote! {
                #[allow(unused)]
                #possibly_unsafe {
                    #setup
                    #inner_tokens
                }
            };
            return Ok(output);
        }

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
            let obj = state.add_var(*receiver, "object", "object");
            let index_len = (args.len().saturating_sub(1)).to_string().len();
            let out_args = args.into_iter().enumerate().map(|(i, arg)| {
                state.add_var(arg, &format!("arg{i}"), &format!("arg {i:>index_len$}"))
            });

            // output: `quote! { #(attrs)* #obj #dot_token #method #turbofish ( #(#out_args),* ) }` except we want to use the original parentheses for span purposes
            assert_condition = quote! { #(#attrs)* #obj #dot_token #method #turbofish };
            paren_token.surround(&mut assert_condition, |out| {
                out.extend(quote! { #(#out_args),* })
            });
        }

        // (expr)
        syn::Expr::Paren(syn::ExprParen { expr, .. }) => {
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

        // op expr
        syn::Expr::Unary(syn::ExprUnary { expr, op, attrs }) => {
            let original = state.add_var(*expr, "original", "original");
            assert_condition = quote! { #(#attrs)* #op #original };
        }

        // unsafe { ... }
        syn::Expr::Unsafe(syn::ExprUnsafe {
            block,
            attrs,
            unsafe_token,
        }) => {
            state.possibly_unsafe = quote! { #(#attrs)* #unsafe_token };
            return eval_block(block, state);
        }

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
        ..
    } = state;

    let output = quote! {
        #[allow(unused)]
        #possibly_unsafe {
            #setup
            if #assert_condition {
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

fn eval_block(mut block: syn::Block, mut state: State) -> Result<TokenStream> {
    state.resolve_variables();

    let original_tokens = block.to_token_stream();

    let Some(syn::Stmt::Expr(expr, None)) = block.stmts.pop() else {
        let State {
            setup,
            possibly_unsafe,
            ..
        } = state;
        return Ok(quote! {
            #[allow(unused)]
            #possibly_unsafe {
                #setup
                if #original_tokens {}
            }
        });
    };

    let condition_str = printable_expr_string(&expr);
    state.add_cause(&format!("block return assertion `{condition_str}` failed"));

    for stmt in block.stmts {
        state.setup.extend(stmt.to_token_stream());
    }

    eval_expr(expr, state)
}

fn recurse_else_branches(branch: syn::Expr, mut state: State) -> Result<TokenStream> {
    match branch {
        // else { ... }
        syn::Expr::Block(syn::ExprBlock { block, .. }) => {
            let body = eval_block(block, state)?;
            Ok(quote! { { #body } })
        }

        // else if cond { ... }
        syn::Expr::If(syn::ExprIf {
            cond,
            else_branch: Some((else_token, else_branch)),
            then_branch,
            attrs,
            if_token,
        }) => {
            let condition_str = printable_expr_string(&cond);
            let condition =
                state.add_var(*cond, "condition", &format!("condition `{condition_str}`"));

            let then_branch = eval_block(then_branch, state.fork())?;
            let else_branches = recurse_else_branches(*else_branch, state.fork())?;

            state.resolve_variables(); // only resolve variables after the recursive calls so that the forks can align the conditions

            let State { setup, .. } = state;

            Ok(quote! {
                {
                    #setup
                    #(#attrs)* #if_token #condition {
                        #then_branch
                    } #else_token #else_branches
                }
            })
        }
        syn::Expr::If(_) => Ok(branch.to_token_stream()), // if without else: let the compiler generate the error

        _ => {
            // docs on syn::ExprIf (in 2.0.71): "The `else` branch expression may only be an `If` or `Block` expression."
            let msg = "parsing error: expected else block or if-else chain";
            Error::err_spanned(branch, msg) // should not be reachable, thus not checked
        }
    }
}

fn printable_expr_string(expr: &impl quote::ToTokens) -> String {
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
