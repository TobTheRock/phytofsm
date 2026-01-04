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
}
