use crate::{
    error::Result,
    parser::{Action, Event, ParsedFsm, ParsedFsmBuilder, StateType},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_composite_states_fsm() -> Result<ParsedFsm> {
    let mut builder = ParsedFsmBuilder::new("Composite States");

    // Root level
    let state_a = builder.add_state("StateA", StateType::Enter);
    builder.add_state("StateB", StateType::Simple);
    builder.add_transition(
        "StateA",
        "StateB",
        Event("toB".into()),
        Some(Action("actionInA".into())),
    );

    // StateA children
    builder.set_scope(Some(state_a));
    let state_aa = builder.add_state("StateAA", StateType::Enter);
    builder.add_state("StateAB", StateType::Simple);
    builder.add_transition(
        "StateAA",
        "StateAB",
        Event("goToAB".into()),
        Some(Action("actionInAA".into())),
    );
    // StateAA children
    builder.set_scope(Some(state_aa));
    builder.add_state("StateAAA", StateType::Enter);
    builder.add_state("StateAAB", StateType::Simple);
    builder.add_transition(
        "StateAAA",
        "StateAAB",
        Event("toAAB".into()),
        Some(Action("actionInAAA".into())),
    );

    builder.build()
}

impl FsmTestData {
    pub fn composite_states() -> Self {
        let path = get_adjacent_file_path(file!(), "composite_states.puml");
        Self {
            name: "composite_states",
            content: include_str!("./composite_states.puml"),
            parsed: build_composite_states_fsm().expect("Failed to create FSM for testing"),
            path,
        }
    }
}
