use super::*;

#[derive(Clone)]
pub struct FormatMessage {
    /// The format message string. Contains `{}` or `{:?}` placeholders for dynamic arguments.
    message: String,
    /// Dynamic arguments to be used with the format message
    dynamic_args: Vec<TokenStream>,
}

impl FormatMessage {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            dynamic_args: Vec::new(),
        }
    }

    pub fn add_text(&mut self, text: impl Display) {
        write!(self.message, "{text}").unwrap();
    }

    pub fn add_placeholder(&mut self, placeholder: impl Display, arg: TokenStream) {
        write!(self.message, "{placeholder}").unwrap();
        self.dynamic_args.push(arg);
    }

    pub fn add_cause(&mut self, cause: impl Display) {
        write!(self.message, "\n    caused by: {cause}").unwrap();
    }
}

impl ToTokens for FormatMessage {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FormatMessage {
            message,
            dynamic_args,
        } = self;
        tokens.extend(quote! { #message, #(#dynamic_args),* });
    }
}
