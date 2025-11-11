use std::path::PathBuf;

use crate::parser;
mod actions;
mod four_seasons;

pub struct FsmTestData {
    pub name: &'static str,
    pub content: &'static str,
    pub parsed: parser::ParsedFsm,
    pub path: PathBuf,
}

impl FsmTestData {
    pub fn all() -> impl Iterator<Item = Self> {
        vec![Self::actions(), Self::four_seasons()].into_iter()
    }
}
