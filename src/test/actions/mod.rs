use crate::{
    error::Result,
    parser::{Action, Event, ParsedFsm, ParsedFsmBuilder, StateType},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_actions_fsm() -> Result<ParsedFsm> {
    let mut builder = ParsedFsmBuilder::new("TestFsm");
    builder.add_state("StateA", StateType::Enter);
    builder.add_transition(
        "StateA",
        "StateB",
        Event("GoToB".into()),
        Some(Action("Action1".into())),
    );
    builder.add_transition(
        "StateB",
        "StateA",
        Event("GoToA".into()),
        Some(Action("Action2".into())),
    );
    builder.build()
}

fn build_enter_exit_fsm() -> Result<ParsedFsm> {
    let mut builder = ParsedFsmBuilder::new("EnterExitActions");

    // Root level states
    builder.add_state("A", StateType::Enter);
    builder.set_state_enter_action("A", Action::from("EnterA"));
    builder.set_state_exit_action("A", Action::from("ExitA"));
    builder.add_state("B", StateType::Simple);

    // Root level transitions
    builder.add_transition("A", "B", Event::from("GoToB"), None);
    builder.add_transition("B", "A", Event::from("GoToAFromB"), None);

    // Composite state C
    let state_c = builder.add_state("C", StateType::Simple);
    builder.set_state_enter_action("C", Action::from("EnterC"));
    builder.set_state_exit_action("C", Action::from("ExitC"));

    // C's children
    builder.set_scope(Some(state_c));
    builder.add_state("C1", StateType::Enter);
    builder.set_state_enter_action("C1", Action::from("EnterC1"));
    builder.set_state_exit_action("C1", Action::from("ExitC1"));
    builder.add_state("C2", StateType::Simple);
    builder.add_transition("C1", "C2", Event::from("GoToC2"), None);

    // Root level transitions involving C
    builder.set_scope(None);
    builder.add_transition("A", "C", Event::from("GoToC"), None);
    builder.add_transition("A", "C1", Event::from("GoToC1FromA"), None);
    builder.add_transition("A", "C2", Event::from("GoToC2FromA"), None);
    builder.add_transition("C", "A", Event::from("GoToAFromC"), None);

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
