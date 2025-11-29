use std::collections::HashMap;

use indextree::{Arena, NodeId};

use crate::error::{Error, Result};

use super::fsm::ParsedFsm;
use super::types::{Action, Event, StateType};

pub(super) type StateId = NodeId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct TransitionData {
    pub destination: StateId,
    pub event: Event,
    pub action: Option<Action>,
}

#[derive(Debug, Clone)]
pub(super) struct StateData {
    pub name: String,
    pub state_type: StateType,
    pub transitions: Vec<TransitionData>,
}

impl StateData {
    fn new_without_transitions(name: &str, state_type: StateType) -> Self {
        StateData {
            name: name.to_string(),
            state_type,
            transitions: vec![],
        }
    }
}

#[derive(Debug)]
pub struct ParsedFsmBuilder {
    name: String,
    arena: Arena<StateData>,
    state_map: HashMap<String, StateId>,
    enter_state: Option<StateId>,
}

impl ParsedFsmBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arena: Arena::new(),
            state_map: HashMap::new(),
            enter_state: None,
        }
    }

    pub fn add_state(&mut self, name: &str) -> StateId {
        self.get_or_create_state(name, StateType::Simple)
    }

    pub fn add_enter_state(&mut self, name: &str) -> Result<StateId> {
        if self.enter_state.is_some() {
            return Err(Error::Parse(
                "FSM must have exactly one enter state".to_string(),
            ));
        }

        let id = self.get_or_create_state(name, StateType::Enter);
        self.enter_state = Some(id);
        Ok(id)
    }

    pub fn add_transition(
        &mut self,
        from: &str,
        to: &str,
        event: Event,
        action: Option<Action>,
    ) -> Result<&mut Self> {
        let from_id = self.get_or_create_state(from, StateType::Simple);
        let to_id = self.get_or_create_state(to, StateType::Simple);

        let transition = TransitionData {
            destination: to_id,
            event,
            action,
        };

        self.arena[from_id].get_mut().transitions.push(transition);
        Ok(self)
    }

    pub fn build(self) -> Result<ParsedFsm> {
        let name = self.name;
        if name.trim().is_empty() {
            return Err(Error::Parse("FSM name cannot be empty".to_string()));
        }

        let enter_state = self.enter_state.ok_or(Error::Parse(
            "FSM must have exactly one enter state".to_string(),
        ))?;

        Ok(ParsedFsm::new(name, enter_state, self.arena))
    }

    fn get_or_create_state(&mut self, name: &str, state_type: StateType) -> StateId {
        if let Some(&id) = self.state_map.get(name) {
            self.arena[id].get_mut().state_type = state_type;
            return id;
        }

        let state_data = StateData::new_without_transitions(name, state_type);
        let id = self.arena.new_node(state_data);
        self.state_map.insert(name.to_string(), id);
        id
    }
}
