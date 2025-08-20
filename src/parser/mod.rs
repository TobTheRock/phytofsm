use crate::error::{Error, Result};

use derive_more::{From, Into};
use itertools::Itertools;

mod context;
mod nom;
mod plantuml;

#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into)]
pub struct Event(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into)]
pub struct Action(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateType {
    Simple,
    Enter,
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
pub struct ParsedFsm {
    name: String,
    transitions: Vec<Transition>,
    enter_state: State,
}

impl ParsedFsm {
    pub fn try_parse<C>(content: C) -> Result<ParsedFsm>
    where
        C: AsRef<str>,
    {
        let diagram = plantuml::StateDiagram::parse(content.as_ref())?;
        diagram.try_into()
    }

    #[cfg(test)]
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
            .map_err(|_| Error::Parse("FSM must have exactly one enter state".to_string()))?;

        Ok(Self {
            name,
            transitions,
            enter_state: enter,
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn transitions(&self) -> impl Iterator<Item = &Transition> {
        self.transitions.iter()
    }

    pub fn events(&self) -> impl Iterator<Item = &Event> {
        self.transitions().map(|t| &t.event).unique()
    }

    pub fn actions(&self) -> impl Iterator<Item = (&Action, &Event)> {
        self.transitions()
            .filter_map(|t| t.action.as_ref().map(|action| (action, &t.event)))
            .unique()
    }

    pub fn enter_state(&self) -> &State {
        &self.enter_state
    }
}

impl TryFrom<plantuml::StateDiagram<'_>> for ParsedFsm {
    type Error = Error;
    fn try_from(diagram: plantuml::StateDiagram<'_>) -> Result<Self> {
        let enter_state = *diagram
            .enter_states()
            .exactly_one()
            .map_err(|_| Error::Parse("FSM must have exactly one enter state".to_string()))?;

        let transitions = diagram
            .transitions()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .map(|t| t.try_into_transition(enter_state))
            .collect::<Result<Vec<Transition>>>()?;
        let name = diagram.name().map(|s| s.to_string()).unwrap_or_default();

        if name.trim().is_empty() {
            return Err(Error::Parse("FSM name cannot be empty".to_string()));
        }

        Ok(ParsedFsm {
            name,
            transitions,
            enter_state: State::from(enter_state, enter_state),
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
    use crate::{file::FsmFile, test::FsmTestData};

    // TODO reenable once the data conversion is refactored
    // #[test]
    // fn parse_simple_fsm() {
    //     let test_data = FsmTestData::four_seasons();
    //     let fsm = FsmFile::try_open(&test_data.path.to_string_lossy())
    //         .expect("Failed to open FSM file")
    //         .try_parse()
    //         .expect("Failed to parse FSM");
    //     assert_eq!(test_data.fsm, fsm);
    // }
}
