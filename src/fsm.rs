use crate::{error, parser};

use heck::{ToSnakeCase, ToUpperCamelCase};
use itertools::Itertools;
use quote::format_ident;

// TODO move to seperate file?
pub struct Idents {
    pub fsm: proc_macro2::Ident,
    pub module: proc_macro2::Ident,
    pub event_params_trait: proc_macro2::Ident,
    pub event_enum: proc_macro2::Ident,
    pub action_trait: proc_macro2::Ident,
    pub state_struct: proc_macro2::Ident,
}

impl Idents {}
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

impl parser::Event {
    pub fn params_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}Params", self.0.to_upper_camel_case())
    }

    pub fn ident(&self) -> proc_macro2::Ident {
        format_ident!("{}", self.0.to_upper_camel_case())
    }
}

impl parser::Action {
    pub fn ident(&self) -> proc_macro2::Ident {
        format_ident!("{}", self.0.to_snake_case())
    }
}

impl parser::State {
    pub fn function_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}", self.name.to_snake_case())
    }
}

impl quote::ToTokens for parser::State {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.name.to_tokens(tokens);
    }
}

pub struct Fsm {
    repr: parser::ParsedFsm,
    // TODO add this to parser
    entry: parser::State,
    idents: Idents,
}

// ToDO move to parser
impl Fsm {
    pub fn events(&self) -> impl Iterator<Item = &parser::Event> {
        self.repr.events()
    }

    pub fn actions(&self) -> impl Iterator<Item = (&parser::Action, &parser::Event)> {
        self.repr.actions()
    }

    pub fn entry(&self) -> &parser::State {
        &self.entry
    }

    pub fn transitions_by_source_state(
        &self,
    ) -> impl Iterator<Item = (&parser::State, Vec<&parser::Transition>)> {
        self.repr
            .transitions()
            .map(|t| (&t.source, t))
            .into_group_map()
            .into_iter()
    }

    pub fn idents(&self) -> &Idents {
        &self.idents
    }
}

impl TryFrom<parser::ParsedFsm> for Fsm {
    type Error = error::Error;
    fn try_from(repr: parser::ParsedFsm) -> Result<Self, Self::Error> {
        let idents = new(repr.name());
        let entry = repr.enter_state().clone();
        Ok(Fsm {
            repr,
            entry,
            idents,
        })
    }
}
