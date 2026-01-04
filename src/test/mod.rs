use std::path::PathBuf;

use crate::parser;
mod actions;
mod composite_states;
mod four_seasons;
mod transitions;
mod utils;

#[derive(Debug)]
pub struct FsmTestData {
    pub name: &'static str,
    pub content: &'static str,
    pub parsed: parser::ParsedFsm,
    pub path: PathBuf,
}

impl FsmTestData {
    pub fn all() -> impl Iterator<Item = Self> {
        // TODO lazy iter
        vec![
            Self::actions(),
            Self::composite_states(),
            Self::four_seasons(),
            Self::same_name_substates(),
            Self::transitions(),
        ]
        .into_iter()
    }
}
