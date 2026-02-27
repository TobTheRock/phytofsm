use crate::parser::{ParsedFsmBuilder, StateType, TransitionParameters};

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
    builder.add_transition(TransitionParameters {
        source: "Start",
        target: "End",
        event: "EventA".into(),
        action: Some("DuplicateAction".into()),
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "Start",
        target: "End",
        event: "EventB".into(),
        action: Some("DuplicateAction".into()),
        guard: None,
    });
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_conflicting_transitions_fails() {
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_state("A", StateType::Enter);
    builder.add_transition(TransitionParameters {
        source: "A",
        target: "B",
        event: "EventA".into(),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "A",
        target: "C",
        event: "EventA".into(),
        action: None,
        guard: None,
    });
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_guarded_conflicting_transitions_succeeds() {
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_state("A", StateType::Enter);
    builder.add_transition(TransitionParameters {
        source: "A",
        target: "B",
        event: "EventA".into(),
        action: None,
        guard: Some("GuardOne".into()),
    });
    builder.add_transition(TransitionParameters {
        source: "A",
        target: "C",
        event: "EventA".into(),
        action: None,
        guard: Some("GuardTwo".into()),
    });
    let result = builder.build();
    assert!(result.is_ok());
}

#[test]
fn build_with_partially_guarded_conflicting_transitions_fails() {
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_state("A", StateType::Enter);
    builder.add_transition(TransitionParameters {
        source: "A",
        target: "B",
        event: "EventA".into(),
        action: None,
        guard: Some("GuardOne".into()),
    });
    builder.add_transition(TransitionParameters {
        source: "A",
        target: "C",
        event: "EventA".into(),
        action: None,
        guard: None,
    });
    let result = builder.build();
    assert!(result.is_err());
}
