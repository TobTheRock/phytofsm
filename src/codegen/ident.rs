use heck::{ToSnakeCase, ToUpperCamelCase};

use crate::parser;

pub struct Idents {
    pub fsm: proc_macro2::Ident,
    pub module: proc_macro2::Ident,
    pub event_params_trait: proc_macro2::Ident,
    pub event_enum: proc_macro2::Ident,
    pub action_trait: proc_macro2::Ident,
    pub state_struct: proc_macro2::Ident,
}

impl Idents {
    pub fn new(name: &str) -> Idents {
        let name = name.to_string();
        Idents {
            fsm: quote::format_ident!("{}", name.to_upper_camel_case()),
            module: quote::format_ident!("{}", name.to_snake_case()),
            event_params_trait: quote::format_ident!("I{}EventParams", name.to_upper_camel_case()),
            event_enum: quote::format_ident!("{}Event", name.to_upper_camel_case()),
            action_trait: quote::format_ident!("I{}Actions", name.to_upper_camel_case()),
            state_struct: quote::format_ident!("{}State", name.to_upper_camel_case()),
        }
    }
}

impl parser::Event {
    pub fn params_ident(&self) -> proc_macro2::Ident {
        quote::format_ident!("{}Params", self.0.to_upper_camel_case())
    }

    pub fn ident(&self) -> proc_macro2::Ident {
        quote::format_ident!("{}", self.0.to_upper_camel_case())
    }

    pub fn method_ident(&self) -> proc_macro2::Ident {
        quote::format_ident!("{}", self.0.to_snake_case())
    }
}

impl parser::Action {
    pub fn ident(&self) -> proc_macro2::Ident {
        quote::format_ident!("{}", self.0.to_snake_case())
    }
}

impl parser::State<'_> {
    pub fn function_ident(&self) -> proc_macro2::Ident {
        quote::format_ident!("{}", self.qualified_name("_").to_snake_case())
    }

    pub fn name_literal(&self) -> proc_macro2::Literal {
        proc_macro2::Literal::string(&self.qualified_name("::"))
    }

    fn qualified_name(&self, separator: impl Into<String>) -> String {
        use itertools::Itertools;
        let names: Vec<_> = std::iter::successors(Some(self.clone()), |next| next.parent())
            .map(|s| s.name().to_string())
            .collect();
        Itertools::intersperse(names.into_iter().rev(), separator.into()).collect()
    }
}

impl quote::ToTokens for parser::State<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.name().to_tokens(tokens);
    }
}
