use std::path::PathBuf;

use crate::parser;
mod four_seasons;

pub struct FsmTestData {
    pub content: &'static str,
    pub path: PathBuf,
    pub fsm: parser::ParsedFsm,
}
