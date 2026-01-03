use std::collections::HashSet;

use indextree::Arena;
use itertools::Itertools;

use super::builder::{StateData, StateId, TransitionData};
use super::types::{Action, Event, StateType};

#[derive(Clone)]
pub struct ParsedFsm {
    name: String,
    enter_state: StateId,
    arena: Arena<StateData>,
}

impl std::fmt::Debug for ParsedFsm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ParsedFsm {{")?;
        writeln!(f, "  name: {:?}", self.name)?;
        writeln!(f, "  states:")?;
        self.fmt_state_tree(f, self.enter_state(), 2)?;
        writeln!(f, "  transitions:")?;
        for t in self.transitions() {
            let action = t
                .action
                .map(|a| format!(" / {}", a.0))
                .unwrap_or_default();
            writeln!(
                f,
                "    {} --[{}{}]--> {}",
                t.source.name(),
                t.event.0,
                action,
                t.destination.name()
            )?;
        }
        write!(f, "}}")
    }
}

impl ParsedFsm {
    fn fmt_state_tree(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        state: State<'_>,
        indent: usize,
    ) -> std::fmt::Result {
        let prefix = " ".repeat(indent * 2);
        let type_marker = match state.state_type() {
            StateType::Enter => "[*] ",
            StateType::Simple => "",
        };
        writeln!(f, "{}{}{}", prefix, type_marker, state.name())?;
        for substate in state.substates() {
            self.fmt_state_tree(f, substate, indent + 1)?;
        }
        Ok(())
    }
}

impl ParsedFsm {
    pub(super) fn new(name: String, enter_state: StateId, arena: Arena<StateData>) -> Self {
        Self {
            name,
            enter_state,
            arena,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn events(&self) -> impl Iterator<Item = &Event> {
        self.transitions().map(|t| t.event).unique()
    }

    pub fn actions(&self) -> impl Iterator<Item = (&Action, &Event)> {
        self.transitions()
            .filter_map(|t| t.action.map(|action| (action, t.event)))
            .unique()
    }

    pub fn enter_state(&self) -> State<'_> {
        State::new(self.enter_state, &self.arena)
    }

    pub fn states(&self) -> impl Iterator<Item = State<'_>> {
        let arena = &self.arena;
        self.arena.iter().map(move |node| {
            let id = arena.get_node_id(node).unwrap();
            State::new(id, arena)
        })
    }

    pub fn transitions(&self) -> impl Iterator<Item = Transition<'_>> {
        let arena = &self.arena;
        self.arena.iter().flat_map(move |node| {
            node.get()
                .transitions
                .iter()
                .map(|t| Transition::from(t, arena))
        })
    }
}

impl PartialEq for ParsedFsm {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.enter_states_eq(other)
            && self.states_eq(other)
            && self.transitions_eq(other)
    }
}

impl Eq for ParsedFsm {}

impl ParsedFsm {
    fn enter_states_eq(&self, other: &Self) -> bool {
        let self_enter = self.enter_state();
        let other_enter = other.enter_state();
        self_enter.name() == other_enter.name()
            && self_enter.state_type() == other_enter.state_type()
    }

    fn states_eq(&self, other: &Self) -> bool {
        let self_states: HashSet<_> = self
            .states()
            .map(|s| (s.name().to_string(), s.state_type()))
            .collect();
        let other_states: HashSet<_> = other
            .states()
            .map(|s| (s.name().to_string(), s.state_type()))
            .collect();
        self_states == other_states
    }

    fn transitions_eq(&self, other: &Self) -> bool {
        let self_transitions: HashSet<_> = self.transitions().map(transition_key).collect();
        let other_transitions: HashSet<_> = other.transitions().map(transition_key).collect();
        self_transitions == other_transitions
    }
}

fn transition_key(t: Transition) -> (String, Event, Option<Action>) {
    (
        t.destination.name().to_string(),
        t.event.clone(),
        t.action.cloned(),
    )
}

#[derive(Debug, Clone)]
pub struct State<'a> {
    id: StateId,
    arena: &'a Arena<StateData>,
}

impl<'a> State<'a> {
    fn new(id: StateId, arena: &'a Arena<StateData>) -> Self {
        Self { id, arena }
    }

    pub fn name(&self) -> &str {
        &self.node_data().name
    }

    pub fn state_type(&self) -> StateType {
        self.node_data().state_type
    }

    pub fn transitions(&self) -> impl Iterator<Item = Transition<'_>> {
        let arena = self.arena;
        self.node_data()
            .transitions
            .iter()
            .map(move |t| Transition::from(t, arena))
    }

    pub fn parent(&self) -> Option<State<'a>> {
        self.node()
            .parent()
            .map(|parent_id| State::new(parent_id, self.arena))
    }

    pub fn substates(&self) -> impl Iterator<Item = State<'a>> {
        self.id
            .children(self.arena)
            .map(move |child_id| State::new(child_id, self.arena))
    }

    fn node(&self) -> &indextree::Node<StateData> {
        &self.arena[self.id]
    }

    fn node_data(&self) -> &StateData {
        self.node().get()
    }
}

impl<'a> PartialEq for State<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
            && self.state_type() == other.state_type()
            && self.parent() == other.parent()
    }
}

#[derive(Debug, Clone)]
pub struct Transition<'a> {
    pub source: State<'a>,
    pub destination: State<'a>,
    pub event: &'a Event,
    pub action: Option<&'a Action>,
}

impl<'a> Transition<'a> {
    fn from(data: &'a TransitionData, arena: &'a Arena<StateData>) -> Transition<'a> {
        Transition {
            source: State::new(data.source, arena),
            destination: State::new(data.destination, arena),
            event: &data.event,
            action: data.action.as_ref(),
        }
    }
}
