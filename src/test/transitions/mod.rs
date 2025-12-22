use crate::{
    error::Result,
    parser::{Action, Event, ParsedFsm, ParsedFsmBuilder, StateType},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_transitions_fsm() -> Result<ParsedFsm> {
    let mut builder = ParsedFsmBuilder::new("TestFsm");
    builder.add_state("StateA", StateType::Enter);
    builder.add_transition("StateA", "StateA", Event("SelfTransition".into()), Some(Action("Action1".into())));
    builder.add_transition("StateA", "StateB", Event("GoToB".into()), Some(Action("Action2".into())));
    builder.add_transition("StateA", "StateB", Event("GoToBDifferently".into()), Some(Action("Action3".into())));
    builder.add_transition("StateA", "StateC", Event("GoToC".into()), None);
    builder.build()
}

impl FsmTestData {
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
