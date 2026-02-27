use crate::{
    error::Result,
    parser::{Action, Event, ParsedFsm, ParsedFsmBuilder, StateType, TransitionParameters},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_composite_states_fsm() -> Result<ParsedFsm> {
    let mut builder = ParsedFsmBuilder::new("Composite States");

    // Root level
    let state_a = builder.add_state("StateA", StateType::Enter);
    builder.add_state("StateB", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: "StateB",
        event: Event("toB".into()),
        action: Some(Action("actionInA".into())),
        guard: None,
    });

    // StateA children
    builder.set_scope(Some(state_a));
    let state_aa = builder.add_state("StateAA", StateType::Enter);
    builder.add_state("StateAB", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "StateAA",
        target: "StateAB",
        event: Event("toAB".into()),
        action: Some(Action("actionInAA".into())),
        guard: None,
    });
    // StateAA children
    builder.set_scope(Some(state_aa));
    builder.add_state("StateAAA", StateType::Enter);
    builder.add_state("StateAAB", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "StateAAA",
        target: "StateAAB",
        event: Event("toAAB".into()),
        action: Some(Action("actionInAAA".into())),
        guard: None,
    });

    builder.build()
}

fn build_substate_to_substate_fsm() -> Result<ParsedFsm> {
    let mut builder = ParsedFsmBuilder::new("Substate To Substate");

    // Root level
    let state_a = builder.add_state("A", StateType::Enter);
    let state_b = builder.add_state("B", StateType::Simple);

    // A's children
    builder.set_scope(Some(state_a));
    builder.add_state("AA", StateType::Enter);

    // B's children
    builder.set_scope(Some(state_b));
    builder.add_state("BA", StateType::Simple);
    builder.add_state("BB", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "BA",
        target: "BB",
        event: Event("toBB".into()),
        action: Some(Action("actionInBA".into())),
        guard: None,
    });

    // Substate to substate transition (defined at root level but references substates)
    builder.set_scope(None);
    builder.add_transition(TransitionParameters {
        source: "AA",
        target: "BA",
        event: Event("toBA".into()),
        action: Some(Action("actionInAA".into())),
        guard: None,
    });

    builder.build()
}

fn build_same_name_substates_fsm() -> Result<ParsedFsm> {
    let mut builder = ParsedFsmBuilder::new("Same Name Substates");

    // Root level
    let parent_a = builder.add_state("ParentA", StateType::Enter);
    let parent_b = builder.add_state("ParentB", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "ParentA",
        target: "ParentB",
        event: Event("toB".into()),
        action: None,
        guard: None,
    });

    // ParentA children
    builder.set_scope(Some(parent_a));
    builder.add_state("Inner", StateType::Enter);
    builder.add_state("Other", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "Inner",
        target: "Other",
        event: Event("toOther".into()),
        action: None,
        guard: None,
    });

    // ParentB children
    builder.set_scope(Some(parent_b));
    builder.add_state("Inner", StateType::Enter);
    builder.add_state("Other", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "Inner",
        target: "Other",
        event: Event("toOther".into()),
        action: None,
        guard: None,
    });

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

    pub fn same_name_substates() -> Self {
        let path = get_adjacent_file_path(file!(), "same_name_substates.puml");
        Self {
            name: "same_name_substates",
            content: include_str!("./same_name_substates.puml"),
            parsed: build_same_name_substates_fsm().expect("Failed to create FSM for testing"),
            path,
        }
    }

    pub fn substate_to_substate() -> Self {
        let path = get_adjacent_file_path(file!(), "substate_to_substate.puml");
        Self {
            name: "substate_to_substate",
            content: include_str!("./substate_to_substate.puml"),
            parsed: build_substate_to_substate_fsm().expect("Failed to create FSM for testing"),
            path,
        }
    }
}
