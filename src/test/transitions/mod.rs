use crate::{
    error::Result,
    parser::{Action, Event, ParsedFsm, ParsedFsmBuilder, StateType, TransitionParameters},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_guards_fsm() -> Result<ParsedFsm> {
    let mut builder = ParsedFsmBuilder::new("Guards");
    builder.add_state("StateA", StateType::Enter);

    // Root level guarded transitions
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: "StateA",
        event: Event("ChangeState".into()),
        action: Some(Action("ActionToA".into())),
        guard: Some(Action("AGuard".into())),
    });
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: "StateB",
        event: Event("ChangeState".into()),
        action: Some(Action("ActionToB".into())),
        guard: Some(Action("BGuard".into())),
    });
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: "StateC",
        event: Event("ChangeState".into()),
        action: Some(Action("ActionToC".into())),
        guard: Some(Action("CGuard".into())),
    });

    // Composite StateC
    let state_c = builder.add_state("StateC", StateType::Simple);
    builder.set_scope(Some(state_c));
    builder.add_transition(TransitionParameters {
        source: "StateC",
        target: "StateCa",
        event: Event("ChangeState".into()),
        action: Some(Action("ActionToCa".into())),
        guard: Some(Action("CaGuard".into())),
    });
    builder.add_transition(TransitionParameters {
        source: "StateC",
        target: "StateCb",
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
        target: "StateA",
        event: Event("SelfTransition".into()),
        action: Some(Action("Action1".into())),
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: "StateB",
        event: Event("GoToB".into()),
        action: Some(Action("Action2".into())),
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: "StateB",
        event: Event("GoToBDifferently".into()),
        action: Some(Action("Action3".into())),
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: "StateC",
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
