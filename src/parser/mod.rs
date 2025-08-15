use derive_more::{From, Into};

mod context;
mod nom;
mod plantuml;

pub struct FsmFile {}

// impl FsmFile {
//     pub fn open() -> Result<Self> {
//         todo!()
//     }
//
//     pub fn parse(&self) -> Result<FsmRepr> {
//         todo!()
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into)]
pub struct Event(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into)]
pub struct Action(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateType {
    Simple,
    Enter,
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub name: String,
    pub state_type: StateType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transition {
    pub source: State,
    pub destination: State,
    // TODO make this optional for direct transitions
    pub event: Event,
    pub action: Option<Action>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fsm {
    pub name: String,
    pub transitions: Vec<Transition>,
}

#[cfg(test)]
mod test {
    use crate::parser::{Fsm, plantuml::PlantUmlFsmParser};
    use crate::test::FsmTestData;

    #[test]
    fn parse_simple_fsm() {
        let test_data = FsmTestData::four_seasons();
        // TODO use FsmFile?
        let mut parser = PlantUmlFsmParser::new();
        let fsm = parser.parse(test_data.content).unwrap();
        assert_eq!(test_data.fsm, fsm);
    }
}
