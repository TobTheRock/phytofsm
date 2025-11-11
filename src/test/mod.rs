use std::path::PathBuf;

use crate::parser;
mod four_seasons;

pub struct FsmTestData {
    pub name: &'static str,
    pub content: &'static str,
    pub parsed: parser::ParsedFsm,
    pub path: PathBuf,
}

impl FsmTestData {
    pub fn all() -> impl Iterator<Item = Self> {
        vec![Self::four_seasons()].into_iter()
    }
}
