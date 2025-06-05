use crate::error::Result;
use std::iter::Iterator;

mod plantuml;

pub struct FsmFile {}

impl FsmFile {
    pub fn open() -> Result<Self> {
        todo!()
    }

    pub fn parse(&self) -> Result<FsmRepr> {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FsmRepr {
    pub name: String,
    pub states: Vec<State>,
    pub transitions: Vec<State>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateType {
    Simple,
    Enter,
    Exit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub name: String,
    pub state_type: StateType,
    pub descriptions: Vec<String>,
    pub parent: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub description: Option<String>,
}

#[cfg(test)]
mod test {
    use crate::test_data::TestFsm;

    #[test]
    fn parse_simple_fsm() {
        let test_data = TestFsm::simple_fsm();
    }
}
