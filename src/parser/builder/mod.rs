use indextree::NodeId;
use itertools::Itertools;
use log::{debug, trace};

use crate::error::{Error, Result};

use super::fsm::ParsedFsm;
use super::types::{Action, Event, StateType};

mod scoped_arena;
mod validation;
use scoped_arena::ScopedArena;

#[cfg(test)]
mod tests;

pub(super) type StateId = NodeId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct TransitionData {
    pub source: StateId,
    pub destination: StateId,
    pub event: Event,
    pub action: Option<Action>,
}

#[derive(Debug, Clone)]
pub(super) struct StateData {
    pub name: String,
    pub state_type: StateType,
    pub transitions: Vec<TransitionData>,
    pub enter_action: Option<Action>,
    pub exit_action: Option<Action>,
    pub enter_state: Option<StateId>,
}

impl StateData {
    fn new(name: &str, state_type: StateType) -> Self {
        StateData {
            name: name.to_string(),
            state_type,
            transitions: vec![],
            enter_action: None,
            exit_action: None,
            enter_state: None,
        }
    }
}

#[derive(Debug)]
pub struct ParsedFsmBuilder {
    name: String,
    arena: ScopedArena<StateData>,
}

impl ParsedFsmBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arena: ScopedArena::new(),
        }
    }

    pub fn set_scope(&mut self, scope: Option<StateId>) -> Option<StateId> {
        self.arena.set_scope(scope)
    }

    pub fn add_state(&mut self, name: &str, state_type: StateType) -> StateId {
        debug!("Adding state '{}' of type {:?}", name, state_type);

        if let Some(id) = self.find_state_in_scope(name) {
            self.update_non_simple_state_type(id, state_type, name);
            return id;
        }

        self.create_state(name, state_type)
    }

    pub fn add_transition(&mut self, from: &str, to: &str, event: Event, action: Option<Action>) {
        debug!(
            "Adding transition from '{}' to '{}' on event {:?}",
            from, to, event
        );

        let from_id = self.find_or_create_state(from);
        let to_id = self.find_or_create_state(to);

        let transition = TransitionData {
            source: from_id,
            destination: to_id,
            event,
            action,
        };

        self.arena[from_id].get_mut().transitions.push(transition);
    }

    pub fn add_enter_action(&mut self, state_name: &str, action: Action) {
        if let Some(id) = self.find_descendant_state(state_name) {
            self.arena[id].get_mut().enter_action = Some(action);
        }
    }

    pub fn add_exit_action(&mut self, state_name: &str, action: Action) {
        if let Some(id) = self.find_descendant_state(state_name) {
            self.arena[id].get_mut().exit_action = Some(action);
        }
    }

    pub fn build(mut self) -> Result<ParsedFsm> {
        trace!(
            "All states: {:?}",
            self.arena
                .iter()
                .map(|node| node.get().name.as_str())
                .collect::<Vec<_>>()
        );

        validation::validate_injective_action_mapping(&self.arena)?;
        validation::validate_no_conflicting_transitions(&self.arena)?;
        self.link_enter_states();

        let enter_state = self.find_root_enter_state()?;
        debug!("Found root enter state: {:?}", enter_state);

        let name = self.name;
        if name.trim().is_empty() {
            return Err(Error::Parse("FSM name cannot be empty".to_string()));
        }

        Ok(ParsedFsm::new(name, enter_state, self.arena.into_inner()))
    }

    fn find_or_create_state(&mut self, name: &str) -> StateId {
        self.find_descendant_state(name)
            .unwrap_or_else(|| self.create_state(name, StateType::Simple))
    }

    fn create_state(&mut self, name: &str, state_type: StateType) -> StateId {
        debug!(
            "Creating state '{}' in scope {:?}",
            name,
            self.arena.scope()
        );
        let state_data = StateData::new(name, state_type);
        self.arena.new_node_in_scope(state_data)
    }

    fn link_enter_states(&mut self) {
        let node_ids: Vec<_> = self
            .arena
            .iter()
            .filter_map(|node| self.arena.get_node_id(node))
            .collect();

        for id in node_ids {
            let deepest_enter = self.find_deepest_enter_state(id);
            self.arena[id].get_mut().enter_state = Some(deepest_enter);
        }
    }

    fn find_root_enter_state(&self) -> Result<StateId> {
        let enter_states = self
            .arena
            .root_nodes()
            .filter(|node| node.get().state_type == StateType::Enter);
        let enter_state_names = || enter_states.clone().map(|node| node.get().name.as_str());

        trace!("Root enter states: {:?}", enter_state_names().collect_vec());

        let root_enter = enter_states
            .clone()
            .filter_map(|node| self.arena.get_node_id(node))
            .exactly_one()
            .map_err(|_| {
                let names: String = Itertools::intersperse(enter_state_names(), ", ").collect();
                Error::Parse(format!(
                    "FSM must have exactly one enter state, found {names}"
                ))
            })?;

        Ok(self.find_deepest_enter_state(root_enter))
    }

    fn find_deepest_enter_state(&self, state_id: StateId) -> StateId {
        let mut current = state_id;
        while let Some(nested_enter) = self
            .arena
            .children(current)
            .find(|child| self.arena[*child].get().state_type == StateType::Enter)
        {
            current = nested_enter;
        }
        current
    }

    fn find_state_in_scope(&self, name: &str) -> Option<StateId> {
        self.arena
            .nodes_in_scope()
            .find(|node| node.get().name == name)
            .and_then(|node| self.arena.get_node_id(node))
    }

    fn find_descendant_state(&self, name: &str) -> Option<StateId> {
        self.arena
            .descendants_from_scope()
            .find(|node| node.get().name == name)
            .and_then(|node| self.arena.get_node_id(node))
    }

    fn update_non_simple_state_type(&mut self, id: NodeId, state_type: StateType, name: &str) {
        let current_type = self.arena[id].get().state_type;
        if state_type != StateType::Simple && current_type == StateType::Simple {
            log::debug!(
                "Updating Type of state '{}' from {:?} to {:?}",
                name,
                current_type,
                state_type
            );
            self.arena[id].get_mut().state_type = state_type;
        } else if state_type != StateType::Simple && current_type != state_type {
            log::warn!(
                "State '{}' already has type {:?}, ignoring {:?}",
                name,
                current_type,
                state_type
            );
        }
    }
}
