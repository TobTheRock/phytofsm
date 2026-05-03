use itertools::Itertools;

use crate::fsm::{Action, Event, UmlFsm};

pub fn events(fsm: &UmlFsm) -> impl Iterator<Item = &Event> {
    fsm.transitions().filter_map(|t| t.event).unique()
}

pub fn actions(fsm: &UmlFsm) -> impl Iterator<Item = (&Action, &Event)> {
    fsm.transitions()
        .filter_map(|t| {
            let event = t.event?;
            t.action.map(|action| (action, event))
        })
        .unique()
}

pub fn guards(fsm: &UmlFsm) -> impl Iterator<Item = (&Action, &Event)> {
    fsm.transitions()
        .filter_map(|t| {
            let event = t.event?;
            t.guard.map(|guard| (guard, event))
        })
        .unique()
}

pub fn direct_transition_actions(fsm: &UmlFsm) -> impl Iterator<Item = &Action> {
    fsm.transitions()
        .filter(|t| t.event.is_none())
        .filter_map(|t| t.action)
        .unique()
}

pub fn direct_transition_guards(fsm: &UmlFsm) -> impl Iterator<Item = &Action> {
    fsm.transitions()
        .filter(|t| t.event.is_none())
        .filter_map(|t| t.guard)
        .unique()
}

pub fn enter_actions(fsm: &UmlFsm) -> impl Iterator<Item = Action> + '_ {
    fsm.states()
        .filter_map(|s| s.enter_action().cloned())
        .unique()
}

pub fn exit_actions(fsm: &UmlFsm) -> impl Iterator<Item = Action> + '_ {
    fsm.states()
        .filter_map(|s| s.exit_action().cloned())
        .unique()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsm::{StateType, TransitionParameters, UmlFsmBuilder};

    #[test]
    fn direct_transitions_not_in_events() {
        let mut builder = UmlFsmBuilder::new("TestFSM");
        builder.add_state("A", StateType::Enter);
        builder.add_transition(TransitionParameters {
            source: "A",
            target: Some("B"),
            event: None,
            action: Some("DoSomething".into()),
            guard: None,
        });
        builder.add_transition(TransitionParameters {
            source: "B",
            target: Some("A"),
            event: Some("GoBack".into()),
            action: None,
            guard: None,
        });
        let fsm = builder.build().unwrap();

        let evts: Vec<_> = events(&fsm).collect();
        assert_eq!(evts.len(), 1);
        assert_eq!(evts[0], &Event::from("GoBack"));
    }

    #[test]
    fn direct_transition_actions_separate_from_event_actions() {
        let mut builder = UmlFsmBuilder::new("TestFSM");
        builder.add_state("A", StateType::Enter);
        builder.add_transition(TransitionParameters {
            source: "A",
            target: Some("B"),
            event: None,
            action: Some("DirectAction".into()),
            guard: None,
        });
        builder.add_transition(TransitionParameters {
            source: "B",
            target: Some("A"),
            event: Some("GoBack".into()),
            action: Some("EventAction".into()),
            guard: None,
        });
        let fsm = builder.build().unwrap();

        let act: Vec<_> = actions(&fsm).collect();
        assert_eq!(act.len(), 1);
        assert_eq!(act[0].0, &Action::from("EventAction"));

        let direct: Vec<_> = direct_transition_actions(&fsm).collect();
        assert_eq!(direct.len(), 1);
        assert_eq!(direct[0], &Action::from("DirectAction"));
    }

    #[test]
    fn direct_transition_guards_separate_from_event_guards() {
        let mut builder = UmlFsmBuilder::new("TestFSM");
        builder.add_state("A", StateType::Enter);
        builder.add_transition(TransitionParameters {
            source: "A",
            target: Some("B"),
            event: None,
            action: None,
            guard: Some("DirectGuard".into()),
        });
        builder.add_transition(TransitionParameters {
            source: "A",
            target: Some("C"),
            event: Some("GoToC".into()),
            action: None,
            guard: Some("EventGuard".into()),
        });
        let fsm = builder.build().unwrap();

        let g: Vec<_> = guards(&fsm).collect();
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].0, &Action::from("EventGuard"));

        let direct: Vec<_> = direct_transition_guards(&fsm).collect();
        assert_eq!(direct.len(), 1);
        assert_eq!(direct[0], &Action::from("DirectGuard"));
    }
}
