use itertools::Itertools;

use crate::error::{Error, Result};

use super::StateData;
use super::scoped_arena::ScopedArena;

pub(super) fn validate_injective_action_mapping(arena: &ScopedArena<StateData>) -> Result<()> {
    let action_events = arena
        .iter()
        .flat_map(|node| node.get().transitions.iter())
        .dedup_by(|a, b| (a.event == b.event) && (a.action == b.action))
        .filter_map(|t| {
            t.action
                .as_ref()
                .map(|action| (action.clone(), t.event.clone()))
        });

    action_events
        .chunk_by(|(action, _)| action.clone())
        .into_iter()
        .try_for_each(|(action, group)| {
            let items = group.collect_vec();
            if items.len() == 1 {
                Ok(())
            } else {
                let events: String = Itertools::intersperse(
                    items.into_iter().map(|(_, event)| String::from(event)),
                    ", ".to_owned(),
                )
                .collect();
                Err(Error::Parse(format!(
                    "Action {} is associated with multiple events: {events}",
                    action.0
                )))
            }
        })
}

pub(super) fn validate_no_conflicting_transitions(arena: &ScopedArena<StateData>) -> Result<()> {
    let transitions = arena
        .iter()
        .flat_map(|node| node.get().transitions.iter())
        .map(|t| (t.source, t.event.clone()));
    transitions
        .chunk_by(|(source, event)| (*source, event.clone()))
        .into_iter()
        .try_for_each(|((source, event), group)| {
            let items = group.collect_vec();
            if items.len() == 1 {
                Ok(())
            } else {
                let state_name = &arena[source].get().name;
                Err(Error::Parse(format!(
                    "State '{}' has multiple transitions for event {:?}",
                    state_name, event
                )))
            }
        })
}
