use crate::{
    error::Result,
    parser::{Action, Event, ParsedFsm, ParsedFsmBuilder, StateType, TransitionParameters},
    test::{utils::get_adjacent_file_path, FsmTestData},
};

fn build_internal_transitions_fsm() -> Result<ParsedFsm> {
    let mut builder = ParsedFsmBuilder::new("InternalTransitions");
    builder.add_state("StateA", StateType::Enter);
    builder.add_enter_action("StateA", Action::from("EnterStateA"));
    builder.add_exit_action("StateA", Action::from("ExitStateA"));

    // Internal transition (no target — stays in state, no exit/entry)
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: None,
        event: Event("InternalEvent".into()),
        action: Some(Action("HandleInternalEvent".into())),
        guard: None,
    });

    // Self transition (target = source — triggers exit/entry)
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateA"),
        event: Event("SelfTransitionEvent".into()),
        action: Some(Action("HandleSelfTransitionEvent".into())),
        guard: None,
    });

    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateB"),
        event: Event("GoToB".into()),
        action: None,
        guard: None,
    });

    // Composite StateB
    let state_b = builder.add_state("StateB", StateType::Simple);
    builder.add_enter_action("StateB", Action::from("EnterStateB"));
    builder.add_exit_action("StateB", Action::from("ExitStateB"));

    builder.set_scope(Some(state_b));
    builder.add_state("StateBa", StateType::Enter);

    // Internal transition on StateBa
    builder.add_transition(TransitionParameters {
        source: "StateBa",
        target: None,
        event: Event("InternalEvent".into()),
        action: Some(Action("HandleInternalEvent".into())),
        guard: None,
    });

    // Self transition on StateBa
    builder.add_transition(TransitionParameters {
        source: "StateBa",
        target: Some("StateBa"),
        event: Event("SelfTransitionEvent".into()),
        action: Some(Action("HandleSelfTransitionEvent".into())),
        guard: None,
    });

    builder.build()
}

fn build_guards_fsm() -> Result<ParsedFsm> {
    let mut builder = ParsedFsmBuilder::new("Guards");
    builder.add_state("StateA", StateType::Enter);

    // Root level guarded transitions
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateA"),
        event: Event("ChangeState".into()),
        action: Some(Action("ActionToA".into())),
        guard: Some(Action("AGuard".into())),
    });
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateB"),
        event: Event("ChangeState".into()),
        action: Some(Action("ActionToB".into())),
        guard: Some(Action("BGuard".into())),
    });
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateC"),
        event: Event("ChangeState".into()),
        action: Some(Action("ActionToC".into())),
        guard: Some(Action("CGuard".into())),
    });

    // Composite StateC
    let state_c = builder.add_state("StateC", StateType::Simple);
    builder.set_scope(Some(state_c));
    builder.add_transition(TransitionParameters {
        source: "StateC",
        target: Some("StateCa"),
        event: Event("ChangeState".into()),
        action: Some(Action("ActionToCa".into())),
        guard: Some(Action("CaGuard".into())),
    });
    builder.add_transition(TransitionParameters {
        source: "StateC",
        target: Some("StateCb"),
        event: Event("ChangeState".into()),
        action: Some(Action("ActionToCb".into())),
        guard: Some(Action("CbGuard".into())),
    });

    builder.build()
}

fn build_transitions_fsm() -> Result<ParsedFsm> {
    let mut builder = ParsedFsmBuilder::new("TestFsm");
    builder.add_state("StateA", StateType::Enter);
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateA"),
        event: Event("SelfTransition".into()),
        action: Some(Action("Action1".into())),
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateB"),
        event: Event("GoToB".into()),
        action: Some(Action("Action2".into())),
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateB"),
        event: Event("GoToBDifferently".into()),
        action: Some(Action("Action3".into())),
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateC"),
        event: Event("GoToC".into()),
        action: None,
        guard: None,
    });
    builder.build()
}

impl FsmTestData {
    pub fn guards() -> Self {
        let path = get_adjacent_file_path(file!(), "guards.puml");
        Self {
            name: "guards",
            content: include_str!("./guards.puml"),
            parsed: build_guards_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }

    pub fn internal_transitions() -> Self {
        let path = get_adjacent_file_path(file!(), "internal_transitions.puml");
        Self {
            name: "internal_transitions",
            content: include_str!("./internal_transitions.puml"),
            parsed: build_internal_transitions_fsm()
                .expect("Failed to create expected FSM"),
            path,
        }
    }

    pub fn transitions() -> Self {
        let path = get_adjacent_file_path(file!(), "transitions.puml");
        Self {
            name: "transitions",
            content: include_str!("./transitions.puml"),
            parsed: build_transitions_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }
}
