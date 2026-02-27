use crate::parser::{ParsedFsm, ParsedFsmBuilder, State, StateType, TransitionParameters};

#[test]
fn add_state_creates_simple_state() {
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_state("Start", StateType::Enter);
    builder.add_state("State1", StateType::Simple);
    let fsm = builder.build().unwrap();

    assert_eq!(fsm.states().count(), 2);
    let state1 = find_state(&fsm, "State1");
    assert_eq!(state1.state_type(), StateType::Simple);
}

#[test]
fn add_state_reuses_existing() {
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_state("A", StateType::Enter);
    builder.add_transition(TransitionParameters {
        source: "A",
        target: "B",
        event: "E1".into(),
        action: None,
        guard: None,
    });
    builder.add_state("B", StateType::Simple);
    let fsm = builder.build().unwrap();

    assert_eq!(fsm.states().count(), 2);
}

#[test]
fn enter_state_not_overwritten_by_simple() {
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_state("Start", StateType::Enter);
    builder.add_state("Start", StateType::Simple);
    let fsm = builder.build().unwrap();

    let start = find_state(&fsm, "Start");
    assert_eq!(start.state_type(), StateType::Enter);
}

#[test]
fn simple_state_upgraded_to_enter() {
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters {
        source: "Start",
        target: "B",
        event: "E1".into(),
        action: None,
        guard: None,
    });
    builder.add_state("Start", StateType::Enter);
    let fsm = builder.build().unwrap();

    let start = find_state(&fsm, "Start");
    assert_eq!(start.state_type(), StateType::Enter);
}

fn find_state<'a>(fsm: &'a ParsedFsm, name: &str) -> State<'a> {
    fsm.states().find(|s| s.name() == name).unwrap()
}
