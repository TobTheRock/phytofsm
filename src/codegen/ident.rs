use heck::{ToSnakeCase, ToUpperCamelCase};
use quote::format_ident;

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
            fsm: format_ident!("{}", name.to_upper_camel_case()),
            module: format_ident!("{}", name.to_snake_case()),
            event_params_trait: format_ident!("I{}EventParams", name.to_upper_camel_case()),
            event_enum: format_ident!("{}Event", name.to_upper_camel_case()),
            action_trait: format_ident!("I{}Actions", name.to_upper_camel_case()),
            state_struct: format_ident!("{}State", name.to_upper_camel_case()),
        }
    }
}

impl parser::Event {
    pub fn params_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}Params", self.0.to_upper_camel_case())
    }

    pub fn ident(&self) -> proc_macro2::Ident {
        format_ident!("{}", self.0.to_upper_camel_case())
    }

    pub fn method_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}", self.0.to_snake_case())
    }
}

impl parser::Action {
    pub fn ident(&self) -> proc_macro2::Ident {
        format_ident!("{}", self.0.to_snake_case())
    }
}

impl parser::State<'_> {
    pub fn function_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}", self.name().to_snake_case())
    }
}

impl quote::ToTokens for parser::State<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.name().to_tokens(tokens);
    }
}
