use indextree::{Arena, NodeId};
use log::{debug, trace};

use crate::error::{Error, Result};

use super::fsm::ParsedFsm;
use super::types::{Action, Event, StateType};

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
    scope: Option<StateId>,
}

impl ParsedFsmBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arena: Arena::new(),
            scope: None,
        }
    }

    pub fn set_scope(&mut self, scope: Option<StateId>) {
        self.scope = scope;
    }

    pub fn add_state(&mut self, name: &str, state_type: StateType) -> StateId {
        debug!("Adding state '{}' of type {:?}", name, state_type);

        if let Some(id) = self.find_state(name) {
            self.update_non_simple_state_type(id, state_type, name);
            return id;
        }

        self.create_state_in_scope(name, state_type)
    }

    pub fn add_transition(&mut self, from: &str, to: &str, event: Event, action: Option<Action>) {
        debug!(
            "Adding transition from '{}' to '{}' on event {:?}",
            from, to, event
        );

        let mut get_or_create = |name: &str| {
            if let Some(id) = self.find_descendant_state(name) {
                id
            } else {
                self.create_state_in_scope(name, StateType::Simple)
            }
        };

        let from_id = get_or_create(from);
        let to_id = get_or_create(to);

        let transition = TransitionData {
            source: from_id,
            destination: to_id,
            event,
            action,
        };

        self.arena[from_id].get_mut().transitions.push(transition);
    }

    pub fn build(self) -> Result<ParsedFsm> {
        trace!(
            "All states: {:?}",
            self.arena
                .iter()
                .map(|node| node.get().name.as_str())
                .collect::<Vec<_>>()
        );

        let enter_state = self.find_root_enter_state()?;
        debug!("Found root enter state: {:?}", enter_state);

        let name = self.name;
        if name.trim().is_empty() {
            return Err(Error::Parse("FSM name cannot be empty".to_string()));
        }

        Ok(ParsedFsm::new(name, enter_state, self.arena))
    }

    fn find_root_enter_state(&self) -> Result<StateId> {
        use itertools::Itertools;
        let enter_states = self
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
        while let Some(nested_enter) = current
            .children(&self.arena)
            .find(|child| self.arena[*child].get().state_type == StateType::Enter)
        {
            current = nested_enter;
        }
        current
    }

    fn find_state(&self, name: &str) -> Option<StateId> {
        self.nodes_in_scope()
            .find(|node| node.get().name == name)
            .and_then(|node| self.arena.get_node_id(node))
    }

    fn find_descendant_state(&self, name: &str) -> Option<StateId> {
        use itertools::Either;

        let scopes = match self.scope {
            Some(scope_id) => Either::Left(std::iter::once(scope_id)),
            None => Either::Right(self.root_node_ids()),
        };

        let mut nodes = scopes
            .flat_map(|id| id.descendants(&self.arena))
            .map(|node_id| &self.arena[node_id]);

        nodes
            .find(|node| node.get().name == name)
            .and_then(|node| self.arena.get_node_id(node))
    }

    fn nodes_in_scope(&self) -> impl Iterator<Item = &indextree::Node<StateData>> + Clone {
        use itertools::Either;
        match self.scope {
            Some(scope_id) => {
                Either::Left(scope_id.children(&self.arena).map(|id| &self.arena[id]))
            }
            None => Either::Right(self.root_nodes()),
        }
    }

    fn root_nodes(&self) -> impl Iterator<Item = &indextree::Node<StateData>> + Clone {
        self.arena.iter().filter(|node| node.parent().is_none())
    }

    fn root_node_ids(&self) -> impl Iterator<Item = StateId> + '_ {
        self.root_nodes()
            .filter_map(|node| self.arena.get_node_id(node))
    }

    fn create_state_in_scope(&mut self, name: &str, state_type: StateType) -> StateId {
        debug!("Creating state '{}' in scope {:?}", name, self.scope);
        let state_data = StateData::new_without_transitions(name, state_type);
        let child_id = self.arena.new_node(state_data);
        if let Some(parent_id) = self.scope {
            parent_id.append(child_id, &mut self.arena);
        }

        child_id
    }

    fn update_non_simple_state_type(&mut self, id: NodeId, state_type: StateType, name: &str) {
        let current_type = self.arena[id].get().state_type;
        // Only states of type Simple can be updated. Simple can be a placeholder until a real type is set
        if state_type != StateType::Simple && current_type == StateType::Simple {
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

#[cfg(test)]
mod test {
    use crate::parser::{Event, ParsedFsmBuilder, StateType};

    #[test]
    fn add_enter_state() {
        let mut builder = ParsedFsmBuilder::new("TestFSM");

        builder.add_state("Start", StateType::Enter);

        let fsm = builder.build().unwrap();
        let enter = fsm.enter_state();
        assert_eq!(enter.name(), "Start");
        assert_eq!(enter.state_type(), StateType::Enter);
    }

    #[test]
    fn add_enter_state_twice_fails() {
        let mut builder = ParsedFsmBuilder::new("TestFSM");
        builder.add_state("Start", StateType::Enter);
        builder.add_state("AnotherStart", StateType::Enter);
        let result = builder.build();
        assert!(result.is_err());
    }

    #[test]
    fn add_enter_state_after_transition() {
        let mut builder = ParsedFsmBuilder::new("TestFSM");

        builder.add_transition("A", "B", "Event".into(), None);
        builder.add_state("Start", StateType::Enter);

        let fsm = builder.build().unwrap();
        let enter = fsm.enter_state();
        assert_eq!(enter.name(), "Start");
        assert_eq!(enter.state_type(), StateType::Enter);
    }

    #[test]
    fn add_transition_after_enter_state() {
        let mut builder = ParsedFsmBuilder::new("TestFSM");

        builder.add_state("Start", StateType::Enter);
        builder.add_transition("A", "B", "Event".into(), None);

        let fsm = builder.build().unwrap();
        let enter = fsm.enter_state();
        assert_eq!(enter.name(), "Start");
        assert_eq!(enter.state_type(), StateType::Enter);
    }

    #[test]
    fn add_state() {
        let mut builder = ParsedFsmBuilder::new("TestFSM");
        builder.add_state("Start", StateType::Enter);
        builder.add_state("State1", StateType::Simple);
        let fsm = builder.build().unwrap();

        assert_eq!(fsm.states().count(), 2);
        let state1 = fsm.states().find(|s| s.name() == "State1").unwrap();
        assert_eq!(state1.state_type(), StateType::Simple);
    }

    #[test]
    fn add_transition() {
        let mut builder = ParsedFsmBuilder::new("TestFSM");
        builder.add_state("A", StateType::Enter);
        builder.add_transition("A", "B", "EventAB".into(), Some("ActionAB".into()));
        let fsm = builder.build().unwrap();

        assert_eq!(fsm.states().count(), 2);
        let transitions: Vec<_> = fsm.transitions().collect();
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].destination.name(), "B");
        assert_eq!(transitions[0].event, &Event::from("EventAB"));
        assert_eq!(transitions[0].action, Some(&"ActionAB".into()));
    }

    #[test]
    fn add_transition_creates_states() {
        let mut builder = ParsedFsmBuilder::new("TestFSM");
        builder.add_state("Start", StateType::Enter);
        builder.add_transition("A", "B", "Event".into(), None);
        let fsm = builder.build().unwrap();

        let names: Vec<_> = fsm.states().map(|s| s.name().to_string()).collect();
        assert!(names.contains(&"Start".to_string()));
        assert!(names.contains(&"A".to_string()));
        assert!(names.contains(&"B".to_string()));
    }

    #[test]
    fn add_state_reuses_existing() {
        let mut builder = ParsedFsmBuilder::new("TestFSM");
        builder.add_state("A", StateType::Enter);
        builder.add_transition("A", "B", "E1".into(), None);
        builder.add_state("B", StateType::Simple); // Should reuse B from transition
        let fsm = builder.build().unwrap();

        assert_eq!(fsm.states().count(), 2);
    }

    #[test]
    fn enter_state_not_overwritten_by_simple() {
        let mut builder = ParsedFsmBuilder::new("TestFSM");
        builder.add_state("Start", StateType::Enter);
        builder.add_state("Start", StateType::Simple); // Should NOT overwrite Enter
        let fsm = builder.build().unwrap();

        let start = find_state(&fsm, "Start");
        assert_eq!(start.state_type(), StateType::Enter);
    }

    #[test]
    fn simple_state_upgraded_to_enter() {
        let mut builder = ParsedFsmBuilder::new("TestFSM");
        builder.add_transition("Start", "B", "E1".into(), None); // Creates Start as Simple
        builder.add_state("Start", StateType::Enter); // Should upgrade to Enter
        let fsm = builder.build().unwrap();

        let start = find_state(&fsm, "Start");
        assert_eq!(start.state_type(), StateType::Enter);
    }

    #[test]
    fn build_without_enter_state_fails() {
        let builder = ParsedFsmBuilder::new("TestFSM");
        let result = builder.build();
        assert!(result.is_err());
    }

    #[test]
    fn build_with_empty_name_fails() {
        let mut builder = ParsedFsmBuilder::new("  ");
        builder.add_state("Start", StateType::Enter);
        let result = builder.build();
        assert!(result.is_err());
    }

    use crate::parser::{ParsedFsm, State};

    #[test]
    fn add_substate() {
        let mut builder = builder_with_enter();
        add_state_with_substate(&mut builder, "Parent", "Child", StateType::Simple);
        let fsm = builder.build().unwrap();

        let parent = find_state(&fsm, "Parent");
        assert_eq!(parent.substates().count(), 1);

        let substate = parent.substates().next().unwrap();
        assert_eq!(substate.name(), "Child");
        assert_eq!(substate.state_type(), StateType::Simple);
        assert_eq!(substate.parent().unwrap(), parent);
    }

    #[test]
    fn add_substate_same_name_different_parents() {
        let mut builder = builder_with_enter();
        add_state_with_substate(&mut builder, "Parent1", "Child", StateType::Simple);
        add_state_with_substate(&mut builder, "Parent2", "Child", StateType::Simple);
        let fsm = builder.build().unwrap();

        assert_n_times_state(&fsm, "Parent1", 1);
        assert_n_times_state(&fsm, "Parent2", 1);
        assert_n_times_state(&fsm, "Child", 2);
    }

    #[test]
    fn add_substate_enter() {
        let mut builder = builder_with_enter();
        add_state_with_substate(&mut builder, "Parent", "InitialChild", StateType::Enter);
        let fsm = builder.build().unwrap();

        let parent = find_state(&fsm, "Parent");
        let child = first_substate(&parent);
        assert_eq!(child.name(), "InitialChild");
        assert_eq!(child.state_type(), StateType::Enter);
    }

    #[test]
    fn add_nested_substates() {
        let mut builder = builder_with_enter();
        let (_, l2) = add_state_with_substate(&mut builder, "Level1", "Level2", StateType::Simple);
        builder.set_scope(Some(l2));
        builder.add_state("Level3", StateType::Simple);
        let fsm = builder.build().unwrap();

        let level1 = find_state(&fsm, "Level1");
        let level2 = first_substate(&level1);
        let level3 = first_substate(&level2);
        assert_eq!(level3.name(), "Level3");
        assert_eq!(level3.parent().unwrap(), level2);
    }

    fn builder_with_enter() -> ParsedFsmBuilder {
        let mut builder = ParsedFsmBuilder::new("TestFSM");
        builder.add_state("Start", StateType::Enter);
        builder
    }

    fn find_state<'a>(fsm: &'a ParsedFsm, name: &str) -> State<'a> {
        fsm.states().find(|s| s.name() == name).unwrap()
    }

    fn assert_n_times_state(fsm: &ParsedFsm, name: &str, n: usize) {
        let count = fsm.states().filter(|s| s.name() == name).count();
        assert_eq!(
            count, n,
            "Expected {} states named '{}' found {}",
            n, name, count
        );
    }

    fn add_state_with_substate(
        builder: &mut ParsedFsmBuilder,
        parent: &str,
        child: &str,
        child_type: StateType,
    ) -> (super::StateId, super::StateId) {
        let parent_id = builder.add_state(parent, StateType::Simple);
        builder.set_scope(Some(parent_id));
        let child_id = builder.add_state(child, child_type);
        builder.set_scope(None);
        (parent_id, child_id)
    }

    fn first_substate<'a>(state: &'a State<'a>) -> State<'a> {
        state.substates().next().unwrap()
    }

    fn find_substate<'a>(parent: &'a State<'a>, name: &str) -> State<'a> {
        parent.substates().find(|s| s.name() == name).unwrap()
    }

    fn assert_transition(state: &State<'_>, dest: &str, event: &str) {
        let t = state
            .transitions()
            .next()
            .expect("expected at least one transition");
        assert_eq!(t.destination.name(), dest, "transition destination");
        assert_eq!(t.event, &Event::from(event), "transition event");
    }

    #[test]
    fn add_substate_transition() {
        let mut builder = builder_with_enter();
        let parent = builder.add_state("Parent", StateType::Simple);
        builder.set_scope(Some(parent));
        builder.add_state("A", StateType::Enter);
        builder.add_state("B", StateType::Simple);
        builder.add_transition("A", "B", "E1".into(), None);
        let fsm = builder.build().unwrap();

        let parent_state = find_state(&fsm, "Parent");
        let a = find_substate(&parent_state, "A");
        assert_transition(&a, "B", "E1");
    }

    #[test]
    fn add_substate_transition_same_name_different_parents() {
        let mut builder = builder_with_enter();
        let p1 = builder.add_state("Parent1", StateType::Simple);
        let p2 = builder.add_state("Parent2", StateType::Simple);

        builder.set_scope(Some(p1));
        builder.add_state("A", StateType::Enter);
        builder.add_state("B", StateType::Simple);
        builder.add_transition("A", "B", "E1".into(), None);

        builder.set_scope(Some(p2));
        builder.add_state("A", StateType::Enter);
        builder.add_state("B", StateType::Simple);
        builder.add_transition("A", "B", "E2".into(), None);

        let fsm = builder.build().unwrap();

        let parent1 = find_state(&fsm, "Parent1");
        let parent2 = find_state(&fsm, "Parent2");
        let p1_a = find_substate(&parent1, "A");
        let p2_a = find_substate(&parent2, "A");
        assert_transition(&p1_a, "B", "E1");
        assert_transition(&p2_a, "B", "E2");
    }

    #[test]
    fn add_substate_transition_creates_substates() {
        let mut builder = builder_with_enter();
        let parent = builder.add_state("Parent", StateType::Simple);
        builder.set_scope(Some(parent));
        builder.add_transition("A", "B", "E1".into(), None);
        let fsm = builder.build().unwrap();

        assert_eq!(find_state(&fsm, "Parent").substates().count(), 2);
    }

    #[test]
    fn add_transition_finds_existing_substate_from_root_scope() {
        crate::logging::init();
        let mut builder = builder_with_enter();

        // Create parent with substate
        let parent = builder.add_state("Parent", StateType::Simple);
        builder.set_scope(Some(parent));
        builder.add_state("Child", StateType::Simple);

        // Back to root - add transition referencing the substate
        builder.set_scope(None);
        builder.add_transition("Child", "Other", "toOther".into(), None);

        let fsm = builder.build().unwrap();

        // Child should only exist once (as substate), not duplicated at root
        assert_n_times_state(&fsm, "Child", 1);

        // The transition should be on Parent's substate
        let parent_state = find_state(&fsm, "Parent");
        let child = find_substate(&parent_state, "Child");
        assert_transition(&child, "Other", "toOther");
    }

    #[test]
    fn enter_state_resolves_to_deepest_nested_enter() {
        let mut builder = ParsedFsmBuilder::new("TestFSM");

        // Root: [*] --> RootEnter (with nested enter states)
        let root = builder.add_state("RootEnter", StateType::Enter);

        // RootEnter contains: [*] --> NestedEnter
        builder.set_scope(Some(root));
        let nested = builder.add_state("NestedEnter", StateType::Enter);
        builder.add_state("NestedSimple", StateType::Simple);

        // NestedEnter contains: [*] --> DeepestEnter
        builder.set_scope(Some(nested));
        builder.add_state("DeepestEnter", StateType::Enter);
        builder.add_state("DeepestSimple", StateType::Simple);

        let fsm = builder.build().unwrap();

        // The enter_state should be the deepest nested enter state
        assert_eq!(fsm.enter_state().name(), "DeepestEnter");
    }
}
