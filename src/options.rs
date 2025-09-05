use syn::{
    LitStr,
    parse::{Parse, ParseStream},
};

pub struct Options {
    pub file_path: String,
    pub log_level: Option<log::Level>,
}

impl Options {
    fn try_from_file_path(lit: &LitStr) -> syn::Result<Self> {
        let file_path = lit.value();
        if file_path.trim().is_empty() {
            return Err(syn::Error::new(lit.span(), "File path cannot be empty"));
        }
        Ok(Self {
            file_path,
            log_level: None,
        })
    }
}

impl Parse for Options {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(syn::Error::new(input.span(), "Expected macro input"));
        }

        if input.peek(syn::LitStr) {
            let file_path: LitStr = input.parse()?;
            return Options::try_from_file_path(&file_path);
        }

        todo!()
    }
}

#[cfg(test)]
mod test {
    use syn::parse::Parser;

    use super::*;

    fn try_parse_file_path(input: &str) -> syn::Result<Options> {
        let token_stream = quote::quote!(#input);
        Options::parse.parse2(token_stream)
    }

    #[test]
    fn parse_file_path_only() {
        let options = try_parse_file_path("path/to/fsm.puml").unwrap();
        assert_eq!(options.file_path, "path/to/fsm.puml");
        assert_eq!(options.log_level, None);
    }

    #[test]
    fn error_from_empty_file_path() {
        let result = try_parse_file_path("");
        assert!(result.is_err());

        let result = try_parse_file_path("    ");
        assert!(result.is_err());
    }
}
