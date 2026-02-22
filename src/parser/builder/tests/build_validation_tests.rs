use crate::parser::{ParsedFsmBuilder, StateType};

#[test]
fn build_without_enter_state_fails() {
    let builder = ParsedFsmBuilder::new("TestFSM");
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_empty_name_fails() {
    let mut builder = ParsedFsmBuilder::new("  ");
    builder.add_state("Start", StateType::Enter);
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_duplicate_events_per_action_fails() {
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_state("Start", StateType::Enter);
    builder.add_transition(
        "Start",
        "End",
        "EventA".into(),
        Some("DuplicateAction".into()),
    );
    builder.add_transition(
        "Start",
        "End",
        "EventB".into(),
        Some("DuplicateAction".into()),
    );
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_conflicting_transitions_fails() {
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_state("A", StateType::Enter);
    builder.add_transition("A", "B", "EventA".into(), None);
    builder.add_transition("A", "C", "EventA".into(), None);
    let result = builder.build();
    assert!(result.is_err());
}
