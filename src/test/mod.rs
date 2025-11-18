use std::path::PathBuf;

use crate::parser;
mod actions;
mod four_seasons;
mod self_transition;
mod utils;

pub struct FsmTestData {
    pub name: &'static str,
    pub content: &'static str,
    pub parsed: parser::ParsedFsm,
    pub path: PathBuf,
}

impl FsmTestData {
    pub fn all() -> impl Iterator<Item = Self> {
        vec![
            Self::actions(),
            Self::four_seasons(),
            Self::self_transition(),
        ]
        .into_iter()
    }
}
