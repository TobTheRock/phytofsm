use crate::{
    error::Result,
    fsm::{Action, Event, UmlFsm, UmlFsmBuilder, StateType, TransitionParameters},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_deferred_events_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("DeferredEvents");

    builder.add_state("StateA", StateType::Enter);
    builder.add_enter_action("StateA", Action::from("enterA"));
    builder.add_state("StateB", StateType::Simple);
    builder.add_enter_action("StateB", Action::from("enterB"));
    builder.add_state("StateC", StateType::Simple);
    builder.add_enter_action("StateC", Action::from("enterC"));
    builder.add_state("StateD", StateType::Simple);
    builder.add_enter_action("StateD", Action::from("enterD"));

    // StateA defers GoToA
    builder.add_deferred_event("StateA", Event::from("GoToA"));

    // StateA -> StateB on GoToB
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateB"),
        event: Some(Event("GoToB".into())),
        action: None,
        guard: None,
    });

    // StateA -> StateC on GoToC
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateC"),
        event: Some(Event("GoToC".into())),
        action: None,
        guard: None,
    });

    // StateA -> StateD on GoToD
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateD"),
        event: Some(Event("GoToD".into())),
        action: None,
        guard: None,
    });

    // StateB -> StateA on GoToA
    builder.add_transition(TransitionParameters {
        source: "StateB",
        target: Some("StateA"),
        event: Some(Event("GoToA".into())),
        action: None,
        guard: None,
    });

    // StateC defers GoToA
    builder.add_deferred_event("StateC", Event::from("GoToA"));

    // StateC -> StateB on GoToBFromC
    builder.add_transition(TransitionParameters {
        source: "StateC",
        target: Some("StateB"),
        event: Some(Event("GoToBFromC".into())),
        action: None,
        guard: None,
    });

    // StateD defers GoToA
    builder.add_deferred_event("StateD", Event::from("GoToA"));

    // StateD -> StateB (direct transition, no event)
    builder.add_transition(TransitionParameters {
        source: "StateD",
        target: Some("StateB"),
        event: None,
        action: None,
        guard: None,
    });

    builder.build()
}

impl FsmTestData {
    pub fn deferred_events() -> Self {
        let path = get_adjacent_file_path(file!(), "deferred.puml");
        Self {
            name: "deferred_events",
            content: include_str!("./deferred.puml"),
            parsed: build_deferred_events_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }
}
