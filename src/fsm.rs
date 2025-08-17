use crate::{codegen::ident::Idents, error, parser};

use heck::{ToSnakeCase, ToUpperCamelCase};
use itertools::Itertools;
use quote::format_ident;

// TODO move to seperate file?

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
        let idents = Idents::new(repr.name());
        let entry = repr.enter_state().clone();
        Ok(Fsm {
            repr,
            entry,
            idents,
        })
    }
}
