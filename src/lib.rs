#![doc = include_str!("../Readme.md")]

use proc_macro::TokenStream as TokenStream1;
pub(crate) use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
pub(crate) use quote::{quote, ToTokens};

mod error;
mod utils;

pub(crate) use error::*;

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
    let expr_str = expression.to_string(); // the expression as a string
    let mut message_parts = vec![quote! { "assertion `", #expr_str, "` failed" }]; // the static parts of the message
    let mut dynamic_args = vec![]; // the dynamic format arguments to the message
    let mut setup = TokenStream::new(); // setup code to evaluate before the expression

    if !format.is_empty() {
        message_parts.push(quote! { ": {}" });
        dynamic_args.push(quote! { ::std::format_args!(#format) });
    }

    if let Some((a, b, combiner)) = eval_expr(expr) {
        expression = combiner;
        setup.extend(quote! {
            let __a_var = #a;
            let __b_var = #b;
        });

        let a_str = a.to_token_stream().to_string();
        let b_str = b.to_token_stream().to_string();
        let start_len = a_str.len().max(b_str.len()); // TODO: switch to unicode width
        let a_str = format!("{:>width$}", a_str, width = start_len);
        let b_str = format!("{:>width$}", b_str, width = start_len);
        message_parts.push(quote! { "
 ", #a_str, ": {:?}
 ", #b_str, ": {:?}" });
        dynamic_args.push(quote! { __a_var });
        dynamic_args.push(quote! { __b_var });
    }

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

fn eval_expr(e: syn::Expr) -> Option<(syn::Expr, syn::Expr, TokenStream)> {
    match e {
        syn::Expr::Binary(syn::ExprBinary {
            left, op, right, ..
        }) => {
            let (a_var, b_var) = stretched_span(&left, &right);
            Some((*left, *right, quote! { #a_var #op #b_var }))
        }
        _ => None,
    }
}

fn stretched_span(a: &impl ToTokens, b: &impl ToTokens) -> (TokenStream, TokenStream) {
    let a_span = a.to_token_stream().into_iter().next().unwrap().span();
    let a_var = quote_spanned! {a_span => __a_var};
    let b_span = b.to_token_stream().into_iter().last().unwrap().span();
    let b_var = quote_spanned! {b_span => __b_var};
    (a_var, b_var)
}
