use crate::{
    error::{Error, Result},
    parser::plantuml::PlantUmlFsmParser,
};

use derive_more::{From, Into};

mod context;
mod nom;
mod plantuml;

pub struct FsmFile {
    content: String,
}

impl FsmFile {
    pub fn try_open(file_path: &str) -> Result<Self> {
        let error = |e: std::io::Error| Error::InvalidFile(file_path.to_string(), e.to_string());
        let abs_path = std::fs::canonicalize(file_path).map_err(error)?;
        let content = std::fs::read_to_string(&abs_path).map_err(error)?;

        Ok(Self { content })
    }

    pub fn try_parse(&self) -> Result<Fsm> {
        // TODO maybe remove the PlantUmlParser
        let mut parser = PlantUmlFsmParser::new();
        parser.parse(&self.content)
    }
}

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
    name: String,
    transitions: Vec<Transition>,
}

impl Fsm {
    pub fn new(name: String, transitions: Vec<Transition>) -> Self {
        Self { name, transitions }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn transitions(&self) -> impl Iterator<Item = &Transition> {
        self.transitions.iter()
    }
}

#[cfg(test)]
mod test {
    use crate::{parser::FsmFile, test::FsmTestData};

    #[test]
    fn parse_simple_fsm() {
        let test_data = FsmTestData::four_seasons();
        let fsm = FsmFile::try_open(&test_data.path.to_string_lossy())
            .expect("Failed to open FSM file")
            .try_parse()
            .expect("Failed to parse FSM");
        assert_eq!(test_data.fsm, fsm);
    }
}
