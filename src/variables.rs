use std::fmt::Display;

use super::*;

struct Variable {
    /// Name of the variable
    name: String,
    /// Debug-printed value
    debug_value: TokenStream,
    /// Setup code to create the variable
    setup: TokenStream,
    /// Flag if the variable might be moved by the condition
    might_move: bool,
}

pub struct Variables {
    /// Pairs of (variable name, debug-printed value) that are used in the assertion and should be printed in the error message
    variables: Vec<Variable>,
    /// Counter for creating unique identifiers
    next_ident_id: usize,
}

impl Variables {
    pub fn new() -> Self {
        Self {
            variables: vec![],
            next_ident_id: 0,
        }
    }

    /// Ensure that there is no conflict between identifiers in the generated code by adding an incrementing number to each identifier
    fn create_ident(&mut self, name: impl Display) -> syn::Ident {
        let name = format!("__one_assert_{}_{}", name, self.next_ident_id);
        self.next_ident_id += 1;
        syn::Ident::new(&name, Span::call_site())
    }

    /// Create a variable from an expression and store it in the setup code.
    ///
    /// It is assumed that the expression moves the value, so it needs to be debug printed in advance
    pub fn add_moving_var(
        &mut self,
        expr: impl Borrow<syn::Expr>,
        identifier: impl Display,
        display: impl Display,
    ) -> TokenStream {
        self.add_var_internal(expr, identifier, display, true)
    }

    /// Create a variable from an expression and store it in the setup code.
    ///
    /// The variable is only used in a borrowed form, so we can debug print only when needed
    pub fn add_borrowed_var(
        &mut self,
        expr: impl Borrow<syn::Expr>,
        identifier: impl Display,
        display: impl Display,
    ) -> TokenStream {
        self.add_var_internal(expr, identifier, display, false)
    }

    fn add_var_internal(
        &mut self,
        expr: impl Borrow<syn::Expr>,
        identifier: impl Display,
        display: impl Display,
        might_move: bool,
    ) -> TokenStream {
        let expr = expr.borrow();
        let mut setup = TokenStream::new();
        let var_access = if matches!(expr, syn::Expr::Path(_)) {
            // could be a variable of a type that doesn't implement Copy, so we can't store it by value.
            // Instead, we just use the variable directly.
            expr.to_token_stream()
        } else {
            // any other expression. Compute the result once and store it
            let var_ident = self.create_ident(&identifier);
            setup.extend(quote! {
                let #var_ident = __OneAssertWrapper(#expr);
            });

            // See note at the end of the file for an explanation on the span manipulation here
            let expr_span = FullSpan::from_spanned(&expr);
            expr_span.apply(quote! { #var_ident }, quote! { .0 })
        };

        let debug_value = if might_move {
            let var_debug_str = self.create_ident(format_args!("{identifier}_str"));
            setup.extend(quote! {
                let #var_debug_str = ::std::format!("{:?}", #var_access);
            });
            var_debug_str.to_token_stream()
        } else {
            var_access.clone()
        };

        // store variable for now instead of printing it immediately, so that all the variables can be aligned
        self.variables.push(Variable {
            name: display.to_string(),
            debug_value,
            setup,
            might_move,
        });

        var_access
    }

    /// Add a `Name: Value` block for all currently stored variables to the format message
    pub fn resolve_variables(
        &mut self,
        setup: &mut TokenStream,
        format_message: &mut FormatMessage,
    ) {
        let Some(max_name_len) = self
            .variables
            .iter()
            .map(|Variable { name, .. }| name.len())
            .max()
        else {
            return; // no variables to print
        };

        for Variable {
            name,
            debug_value,
            setup: var_setup,
            might_move,
        } in self.variables.drain(..)
        {
            setup.extend(var_setup);

            format_message.add_text(format_args!("\n    {name:>max_name_len$}: "));

            if might_move {
                // value was already debug-printed ahead of time => just print the string normally
                format_message.add_placeholder("{}", debug_value);
            } else {
                format_message.add_placeholder("{:?}", debug_value);
            }
        }
    }
}
