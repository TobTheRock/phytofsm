use crate::fsm::types::{Action, Event};

use super::StateId;
use super::state::{State, StateData};

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
pub struct Transition<'a> {
    pub source: State<'a>,
    pub destination: Option<State<'a>>,
    pub event: Option<&'a Event>,
    pub action: Option<&'a Action>,
    pub guard: Option<&'a Action>,
}

impl<'a> Transition<'a> {
    pub fn from(
        data: &'a TransitionData,
        arena: &'a indextree::Arena<StateData>,
    ) -> Transition<'a> {
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
