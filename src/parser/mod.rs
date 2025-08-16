use crate::{
    error::{Error, Result},
    parser::plantuml::PlantUmlFsmParser,
};

use derive_more::{From, Into};
use itertools::Itertools;

mod context;
mod nom;
mod plantuml;

pub struct FsmFile {
    content: String,
}

// TODO more abstractions: AbsPath -> FsmFile -> ParsedFsm -> generators
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
    enter: State,
}

impl Fsm {
    pub fn try_new(name: String, transitions: Vec<Transition>) -> Result<Self> {
        let enter = transitions
            .iter()
            .filter_map(|t| {
                if t.source.state_type == StateType::Enter {
                    Some(t.source.clone())
                } else {
                    None
                }
            })
            .exactly_one()
            .map_err(|_| Error::ParseError("FSM must have exactly one enter state".to_string()))?;

        Ok(Self {
            name,
            transitions,
            enter,
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn transitions(&self) -> impl Iterator<Item = &Transition> {
        self.transitions.iter()
    }

    pub fn entry(&self) -> &State {
        &self.enter
    }
}

impl TryFrom<plantuml::StateDiagram<'_>> for Fsm {
    type Error = Error;
    fn try_from(diagram: plantuml::StateDiagram<'_>) -> Result<Self> {
        if (diagram.enter_states.len() != 1) {
            return Err(Error::ParseError(
                "FSM must have exactly one enter state".to_string(),
            ));
        }
        let enter_state = diagram.enter_states[0];

        let transitions = diagram
            .transitions
            .into_iter()
            .map(|t| t.try_into_transition(enter_state))
            .collect::<Result<Vec<Transition>>>()?;
        Ok(Fsm {
            name: diagram.name.map(|s| s.to_string()).unwrap_or_default(),
            transitions,
            enter: State::from(enter_state, enter_state),
        })
    }
}

impl plantuml::TransitionDescription<'_> {
    fn try_into_transition(self, enter_state: plantuml::StateName<'_>) -> Result<Transition> {
        let description = context::TransitionContext::try_from(self.description)?;
        let source = State::from(self.from, enter_state);
        let desination = State::from(self.to, enter_state);

        Ok(Transition {
            source,
            destination: desination,
            event: description.event,
            action: description.action,
        })
    }
}

impl State {
    fn from(name: plantuml::StateName<'_>, enter_state: plantuml::StateName<'_>) -> Self {
        let state_type = if name == enter_state {
            StateType::Enter
        } else {
            StateType::Simple
        };

        Self {
            name: name.to_string(),
            state_type,
        }
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
