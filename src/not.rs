use super::*;

#[allow(clippy::match_same_arms)] // every arm needs its own reasoning and consideration
pub(crate) fn eval_not_expr(
    mut e: syn::Expr,
    mut setup: TokenStream,
    mut format_message: FormatMessage,
    not_token: syn::Token![!],
    not_attrs: Vec<syn::Attribute>,
) -> Result<TokenStream> {
    // inline any parentheses and invisible groups as they are not necessary
    enum ParensOrGroups {
        Paren(syn::token::Paren),
        Group(syn::token::Group),
    }
    let mut parens = vec![];
    loop {
        match e {
            syn::Expr::Paren(syn::ExprParen {
                expr, paren_token, ..
            }) => {
                parens.push(ParensOrGroups::Paren(paren_token));
                e = *expr;
            }
            syn::Expr::Group(syn::ExprGroup {
                expr, group_token, ..
            }) => {
                parens.push(ParensOrGroups::Group(group_token));
                e = *expr;
            }
            _ => break,
        }
    }

    let mut assert_condition = e.to_token_stream();

    format_message.add_cause(format_args!(
        "negated expression `{}` evaluated to true",
        printable_expr_string(&e)
    ));

    let mut variables = Variables::new();

    match e {
        // !([a, b, c, d])
        syn::Expr::Array(_) => {} // let the compiler generate the error

        // !(a = b)
        syn::Expr::Assign(syn::ExprAssign { eq_token, .. }) => {
            let msg = "Expected a boolean expression, found an assignment. Did you intend to compare with `==`?";
            return Error::err_spanned(eq_token, msg); // checked in tests/fail/expr/assign.rs
        }

        // !(async { ... })
        syn::Expr::Async(_) => {
            let msg = "Expected a boolean expression, found an async block. Did you intend to await a future?";
            return Error::err_spanned(e, msg); // checked in tests/fail/expr/async.rs
        }

        // !(future.await)
        syn::Expr::Await(_) => {} // might work if the future resolves to a boolean and the assert is in an async context

        // !(left <op> right)
        syn::Expr::Binary(syn::ExprBinary {
            left,
            op,
            right,
            attrs,
        }) => {
            match op {
                // logic operators => preserve fail-fast behavior where expressions like `!vec.empty() && vec[0].ok()` work
                syn::BinOp::And(_) => return Ok(resolve_nand(setup, format_message, left, right)),
                syn::BinOp::Or(_) => return Ok(resolve_nor(setup, format_message, left, right)),

                // comparison operators, handle as expected
                syn::BinOp::Eq(_)
                | syn::BinOp::Lt(_)
                | syn::BinOp::Le(_)
                | syn::BinOp::Ne(_)
                | syn::BinOp::Ge(_)
                | syn::BinOp::Gt(_) => {
                    let lhs = variables.add_borrowed_var(left, "lhs", "left");
                    let rhs = variables.add_borrowed_var(right, "rhs", "right");
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
                    let lhs = variables.add_moving_var(left, "lhs", "left");
                    let rhs = variables.add_moving_var(right, "rhs", "right");
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

        // !({ ... })
        syn::Expr::Block(_) => {} // We could check the condition at the end of the block, but really, this shouldn't be done in the assert

        // !(break)
        syn::Expr::Break(_) => {
            // we need to generate our own error, because break returns `!` so it compiles, but the assertion makes no sense
            let msg = "Expected a boolean expression, found a break statement";
            return Error::err_spanned(e, msg); // checked in tests/fail/expr/break.rs
        }

        // !(function(args...))
        syn::Expr::Call(syn::ExprCall {
            args,
            func,
            paren_token,
            attrs,
        }) if !args.is_empty() => {
            let index_len = (args.len() - 1).to_string().len();
            let out_args = args.iter().enumerate().map(|(i, arg)| {
                variables.add_moving_var(
                    arg,
                    format_args!("arg{i}"),
                    format_args!("arg {i:>index_len$}"),
                )
            });

            // output: `quote! { #(#attrs)* #func ( #(#out_args),* ) }` except we want to use the original parentheses for span purposes
            assert_condition = quote! { #(#attrs)* #func };
            paren_token.surround(&mut assert_condition, |out| {
                out.extend(quote! { #(#out_args),* })
            });
        }
        // !(function()) // no args
        syn::Expr::Call(_) => {} // just a plain function call that returns a boolean or not. Nothing more to add here

        // !(expr as ty)
        syn::Expr::Cast(_) => {} // let the compiler generate the error.
        // Might work if expr is `true as bool`, which would actually be a workaround for the `assert!(true)` case

        // !(|args| { ... })
        syn::Expr::Closure(_) => {} // let the compiler generate the error

        // !(const { ... })
        syn::Expr::Const(_) => {} // same as Expr::Block

        // !(continue)
        syn::Expr::Continue(_) => {
            // we need to generate our own error, because continue returns `!` so it compiles, but the assertion makes no sense
            let msg = "Expected a boolean expression, found a continue statement";
            return Error::err_spanned(e, msg); // checked in tests/fail/expr/continue.rs
        }

        // !(obj.field)
        syn::Expr::Field(_) => {} // might work if the field is a boolean
        // It would be possible to print the object that the field is accessed on, but that won't provide much value.
        // The only part of the object that is interesting is the field, and that is already evaluated as the assertion.

        // !(for pat in { ... })
        syn::Expr::ForLoop(_) => {
            // we generate our own error, because the compiler just says "expected bool, found ()"
            let msg = "Expected a boolean expression, found a for loop";
            return Error::err_spanned(e, msg); // checked in tests/fail/expr/forloop.rs
        }

        // !(group with invisible delimiters)
        syn::Expr::Group(_) => unreachable!(), // inlined at the start of the function

        // !(if cond { ... } else { ... })
        syn::Expr::If(_) => {} // Not doing the same analysis as in resolve_if, because that would be too convoluted of a panic message.

        // !(expr[index])
        syn::Expr::Index(syn::ExprIndex {
            index,
            expr,
            attrs,
            bracket_token,
        }) => {
            if !matches!(*index, syn::Expr::Lit(_)) {
                let index = variables.add_moving_var(index, "index", "index");
                // output: `quote! { #(#attrs)* #expr [#index] }` except we want to use the original brackets for span purposes
                assert_condition = quote! { #(#attrs)* #expr };
                bracket_token.surround(&mut assert_condition, |out| index.to_tokens(out));
            }
            // not printing literals, because their value is already known.

            // not printing the indexed object, because the output could be huge.
            // If we knew the object was a form of array, then we could would slice the range around the index,
            // but it could also be a HashMap or a custom type, so we can't do that.
        }

        // !(_)
        syn::Expr::Infer(_) => {} // let the compiler generate the error

        // !(let pat = expr)
        syn::Expr::Let(_) => {
            // we have to generate our own error, because the produced code is `if #expression`, which would become `if let ...` 😂
            let msg = "Expected a boolean expression, found a let statement";
            return Error::err_spanned(e, msg); // checked in tests/fail/expr/let.rs
        }

        // !(lit)
        syn::Expr::Lit(_) => {} // might work if the literal is a boolean
        // The base case for `assert!(true)` and `assert!(false)` was already caught in the initial
        // setup. This is the case where a recursive call contained a plain `true` or `false`, so we
        // shall accept them without printing weird messages

        // !(loop { ... })
        syn::Expr::Loop(_) => {} // might work if the loop breaks with a boolean
        // If somebody has too much free time on their hands they can go ahead and write some recursive
        // block parsing code to find all the `break` statements so that the error message can say
        // which one was triggered. This would be really useful info for the user, but it's a lot of effort
        // for something that probably nobody will ever see.
        // Side note: Finding a `break` would actually help with the case where there are no breaks, because
        // then the loop would just never return (`!`), so the compiler doesn't complain but the assertion
        // makes no sense.

        // !(some_macro!(...))
        syn::Expr::Macro(_) => {} // not touching this

        // !(match expr { ... })
        syn::Expr::Match(_) => {} // we could check which variant matched etc. etc., but that is excessive for an assert

        // !(receiver.method(args...))
        syn::Expr::MethodCall(syn::ExprMethodCall {
            receiver,
            method,
            turbofish,
            args,
            attrs,
            dot_token,
            paren_token,
        }) => {
            let obj = variables.add_moving_var(receiver, "object", "self");
            let index_len = (args.len().saturating_sub(1)).to_string().len();
            let out_args = args.iter().enumerate().map(|(i, arg)| {
                variables.add_moving_var(
                    arg,
                    format_args!("arg{i}"),
                    format_args!("arg {i:>index_len$}"),
                )
            });

            // output: `quote! { #(attrs)* #obj #dot_token #method #turbofish ( #(#out_args),* ) }` except we want to use the original parentheses for span purposes
            assert_condition = quote! { #(#attrs)* #obj #dot_token #method #turbofish };
            paren_token.surround(&mut assert_condition, |out| {
                out.extend(quote! { #(#out_args),* })
            });
        }

        // !(expr)
        syn::Expr::Paren(_) => unreachable!(), // inlined at the start of the function

        // !(some::path::<of>::stuff)
        syn::Expr::Path(_) => {} // might be a constant of type bool, otherwise let the compiler generate the error

        // !(a..b)
        syn::Expr::Range(_) => {} // let the compiler generate the error

        // !(&expr)
        syn::Expr::Reference(_) => {} // let the compiler generate the error

        // !([x; n])
        syn::Expr::Repeat(_) => {} // let the compiler generate the error

        // !(return expr)
        syn::Expr::Return(_) => {
            // we need to generate our own error, because return returns `!` so it compiles, but the assertion makes no sense
            let msg = "Expected a boolean expression, found a return statement";
            return Error::err_spanned(e, msg); // checked in tests/fail/expr/return.rs
        }

        // !(MyStruct { field: value })
        syn::Expr::Struct(_) => {
            // we generate our own error, because the compiler will suggest adding parentheses around the struct literal
            let msg = "Expected a boolean expression, found a struct literal";
            return Error::err_spanned(e, msg);
        }

        // !(expr?)
        syn::Expr::Try(_) => {} // might work if expr is a Result<bool> or similar, otherwise let the compiler generate the error

        // !((a, b, c))
        syn::Expr::Tuple(_) => {} // let the compiler generate the error

        // !(!expr)
        syn::Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Not(_),
            ..
        }) => {
            // double negation.
            // normally we would just remove both `!` since they cancel out. However, the user might have a type
            // that overloads the `!` operator to return bool, so `!!custom_type` is actually valid code for coerce-to-bool.
            // Therefore we have to keep both `!` operators.
            // If there is demand we might need to analyze the inner expression as well, but for now we just keep it as-is.
        }
        // !(op expr)
        syn::Expr::Unary(_) => {} // just leave it as-is

        // !(unsafe { ... })
        syn::Expr::Unsafe(_) => {} // Same as Expr::Block

        // !(something)
        syn::Expr::Verbatim(_) => {} // even syn doesn't know what this is, so we can't do anything with it

        // !(while cond { ... })
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

    variables.resolve_variables(&mut setup, &mut format_message);

    for paren in parens.into_iter().rev() {
        let inner = std::mem::take(&mut assert_condition);
        match paren {
            ParensOrGroups::Paren(paren_token) => {
                paren_token.surround(&mut assert_condition, |out| out.extend(inner))
            }
            ParensOrGroups::Group(group_token) => {
                group_token.surround(&mut assert_condition, |out| out.extend(inner))
            }
        }
    }

    Ok(quote! { #[allow(unreachable_code)] {
        #setup
        if #(#not_attrs)* #not_token #assert_condition {
            // assertion passed
        } else {
            ::std::panic!(#format_message);
        }
    }})
}

fn resolve_nand(
    setup: TokenStream,
    mut format_message: FormatMessage,
    left: impl Borrow<syn::Expr>,
    right: impl Borrow<syn::Expr>,
) -> TokenStream {
    let left = left.borrow();
    let right = right.borrow();

    format_message.add_cause("both sides of `&&` evaluated to true");

    // `!(a && b)` logic: if first is false, entire expression is true. Otherwise evaluate second
    quote! { #[allow(unreachable_code)] {
        #setup
        if #left {
            if #right {
                // both sides true => !(left && right) is false
                ::std::panic!(#format_message);
            } else {
                // !(true && false) == !(false) == true => assertion passes
            }
        } else {
            // !(false && _) == !(false) == true => assertion passes
        }
    }}
}

fn resolve_nor(
    setup: TokenStream,
    format_message: FormatMessage,
    left: impl Borrow<syn::Expr>,
    right: impl Borrow<syn::Expr>,
) -> TokenStream {
    let left = left.borrow();
    let right = right.borrow();

    let mut message_if_left_true = format_message.clone();
    message_if_left_true.add_cause("left side of `||` evaluated to true");

    let mut message_if_right_true = format_message;
    message_if_right_true
        .add_cause("left side of `||` evaluated to false, but right side evaluated to true");

    // `||` logic: if first is true, inner expression is true, causing the assertion to fail
    quote! { #[allow(unreachable_code)] {
        #setup
        if #left {
            // !(true || _) == !(true) == false => assertion fails
            ::std::panic!(#message_if_left_true);
        } else {
            if #right {
                // !(false || true) == !(true) == false => assertion fails
                ::std::panic!(#message_if_right_true);
            } else {
                // !(false || false) == !(false) == true => assertion passes
            }
        }
    }}
}
