use crate::{
    error::Result,
    parser::{Action, Event, ParsedFsm, ParsedFsmBuilder, StateType, TransitionParameters},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_actions_fsm() -> Result<ParsedFsm> {
    let mut builder = ParsedFsmBuilder::new("TestFsm");
    builder.add_state("StateA", StateType::Enter);
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: "StateB",
        event: Event("GoToB".into()),
        action: Some(Action("Action1".into())),
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "StateB",
        target: "StateA",
        event: Event("GoToA".into()),
        action: Some(Action("Action2".into())),
        guard: None,
    });
    builder.build()
}

fn build_enter_exit_fsm() -> Result<ParsedFsm> {
    let mut builder = ParsedFsmBuilder::new("EnterExitActions");

    // Root level states
    builder.add_state("A", StateType::Enter);
    builder.add_enter_action("A", Action::from("EnterA"));
    builder.add_exit_action("A", Action::from("ExitA"));
    builder.add_state("B", StateType::Simple);

    // Root level transitions
    builder.add_transition(TransitionParameters {
        source: "A",
        target: "A",
        event: Event::from("GoToAFromA"),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "A",
        target: "B",
        event: Event::from("GoToB"),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "B",
        target: "A",
        event: Event::from("GoToAFromB"),
        action: None,
        guard: None,
    });

    // Composite state C
    let state_c = builder.add_state("C", StateType::Simple);
    builder.add_enter_action("C", Action::from("EnterC"));
    builder.add_exit_action("C", Action::from("ExitC"));

    // C's children
    builder.set_scope(Some(state_c));
    builder.add_state("C1", StateType::Enter);
    builder.add_enter_action("C1", Action::from("EnterC1"));
    builder.add_exit_action("C1", Action::from("ExitC1"));
    builder.add_state("C2", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "C1",
        target: "C2",
        event: Event::from("GoToC2"),
        action: None,
        guard: None,
    });

    // Root level transitions involving C
    builder.set_scope(None);
    builder.add_transition(TransitionParameters {
        source: "A",
        target: "C",
        event: Event::from("GoToC"),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "A",
        target: "C1",
        event: Event::from("GoToC1FromA"),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "A",
        target: "C2",
        event: Event::from("GoToC2FromA"),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "C",
        target: "A",
        event: Event::from("GoToAFromC"),
        action: None,
        guard: None,
    });

    builder.build()
}

impl FsmTestData {
    pub fn actions() -> Self {
        let path = get_adjacent_file_path(file!(), "actions.puml");
        Self {
            name: "actions",
            content: include_str!("./actions.puml"),
            parsed: build_actions_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }

    pub fn enter_exit() -> Self {
        let path = get_adjacent_file_path(file!(), "enter_exit.puml");
        Self {
            name: "enter_exit",
            content: include_str!("./enter_exit.puml"),
            parsed: build_enter_exit_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }
}
