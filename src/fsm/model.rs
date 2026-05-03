use std::collections::HashSet;

use super::types::{Action, Event, StateType};

pub type StateId = indextree::NodeId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransitionParameters<'a> {
    pub source: &'a str,
    /// No target indicates an internal transition
    pub target: Option<&'a str>,
    /// No event indicates a direct transition
    pub event: Option<Event>,
    pub action: Option<Action>,
    pub guard: Option<Action>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransitionData {
    pub source: StateId,
    pub target: Option<StateId>,
    pub event: Option<Event>,
    pub action: Option<Action>,
    pub guard: Option<Action>,
}

#[derive(Debug, Clone)]
pub struct StateData {
    pub name: String,
    pub state_type: StateType,
    pub transitions: Vec<TransitionData>,
    pub enter_action: Option<Action>,
    pub exit_action: Option<Action>,
    pub enter_state: Option<StateId>,
    /// Includes the inherited events from potential parents
    pub deferred_events: Vec<Event>,
}

#[derive(Clone)]
pub struct UmlFsm {
    name: String,
    enter_state: StateId,
    arena: indextree::Arena<StateData>,
}

impl UmlFsm {
    pub fn new(
        name: String,
        enter_state: StateId,
        arena: indextree::Arena<StateData>,
    ) -> Self {
        Self {
            name,
            enter_state,
            arena,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
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

impl PartialEq for UmlFsm {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.enter_states_eq(other)
            && self.states_eq(other)
            && self.transitions_eq(other)
    }
}

impl Eq for UmlFsm {}

impl UmlFsm {
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

impl std::fmt::Display for UmlFsm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "UmlFsm {{")?;
        writeln!(f, "  name: {}", self.name)?;
        writeln!(f, "  states:")?;
        self.fmt_state_tree(f, self.enter_state(), 2)?;
        writeln!(f, "  transitions:")?;
        let mut transitions: Vec<_> = self.transitions().collect();
        transitions.sort();
        for t in transitions {
            writeln!(f, "    {}", t)?;
        }
        write!(f, "}}")
    }
}

impl std::fmt::Debug for UmlFsm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl UmlFsm {
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
        let enter = state
            .enter_action()
            .map(|a| format!(" > {}", a.0))
            .unwrap_or_default();
        let exit = state
            .exit_action()
            .map(|a| format!(" < {}", a.0))
            .unwrap_or_default();
        writeln!(
            f,
            "{}{}{}{}{}",
            prefix,
            type_marker,
            state.name(),
            enter,
            exit
        )?;
        for substate in state.substates() {
            self.fmt_state_tree(f, substate, indent + 1)?;
        }
        Ok(())
    }
}

fn transition_key(
    t: Transition,
) -> (
    Option<String>,
    Option<Event>,
    Option<Action>,
    Option<Action>,
) {
    (
        t.destination.map(|d| d.name().to_string()),
        t.event.cloned(),
        t.action.cloned(),
        t.guard.cloned(),
    )
}

#[derive(Debug, Clone)]
pub struct State<'a> {
    id: StateId,
    arena: &'a indextree::Arena<StateData>,
}

impl<'a> State<'a> {
    fn new(id: StateId, arena: &'a indextree::Arena<StateData>) -> Self {
        Self { id, arena }
    }

    pub fn name(&self) -> &str {
        &self.node_data().name
    }

    pub fn state_type(&self) -> StateType {
        self.node_data().state_type
    }

    pub fn enter_action(&self) -> Option<&Action> {
        self.node_data().enter_action.as_ref()
    }

    pub fn exit_action(&self) -> Option<&Action> {
        self.node_data().exit_action.as_ref()
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

    pub fn enter_state(&self) -> State<'a> {
        let data = self.node_data();
        if let Some(enter_id) = data.enter_state {
            State::new(enter_id, self.arena)
        } else {
            State::new(self.id, self.arena)
        }
    }

    pub fn deferred_events(&self) -> impl Iterator<Item = &Event> {
        self.node_data().deferred_events.iter()
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
    pub destination: Option<State<'a>>,
    pub event: Option<&'a Event>,
    pub action: Option<&'a Action>,
    pub guard: Option<&'a Action>,
}

impl<'a> Transition<'a> {
    fn from(data: &'a TransitionData, arena: &'a indextree::Arena<StateData>) -> Transition<'a> {
        Transition {
            source: State::new(data.source, arena),
            destination: data.target.map(|id| State::new(id, arena)),
            event: data.event.as_ref(),
            action: data.action.as_ref(),
            guard: data.guard.as_ref(),
        }
    }
}

impl std::fmt::Display for Transition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let event_name = self.event.map(|e| e.0.as_str()).unwrap_or("(direct)");
        let guard = self
            .guard
            .map(|g| format!(" [{}]", g.0))
            .unwrap_or_default();
        let action = self
            .action
            .map(|a| format!(" / {}", a.0))
            .unwrap_or_default();
        let dest = self
            .destination
            .as_ref()
            .map(|d| d.name())
            .unwrap_or("(internal)");
        write!(
            f,
            "{} --[{}{}{}]--> {}",
            self.source.name(),
            event_name,
            guard,
            action,
            dest
        )
    }
}

impl PartialEq for Transition<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.source.name() == other.source.name() && self.event == other.event
    }
}

impl Eq for Transition<'_> {}

impl PartialOrd for Transition<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Transition<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.source.name().cmp(other.source.name()).then_with(|| {
            let self_event = self.event.map(|e| e.0.as_str()).unwrap_or("");
            let other_event = other.event.map(|e| e.0.as_str()).unwrap_or("");
            self_event.cmp(other_event)
        })
    }
}
