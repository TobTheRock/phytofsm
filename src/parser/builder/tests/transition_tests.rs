use crate::parser::{Event, ParsedFsmBuilder, StateType, TransitionParameters};

#[test]
fn add_transition() {
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_state("A", StateType::Enter);
    builder.add_transition(TransitionParameters {
        source: "A",
        target: "B",
        event: "EventAB".into(),
        action: Some("ActionAB".into()),
        guard: None,
    });
    let fsm = builder.build().unwrap();

    assert_eq!(fsm.states().count(), 2);
    let transitions: Vec<_> = fsm.transitions().collect();
    assert_eq!(transitions.len(), 1);
    assert_eq!(transitions[0].destination.name(), "B");
    assert_eq!(transitions[0].event, &Event::from("EventAB"));
    assert_eq!(transitions[0].action, Some(&"ActionAB".into()));
}

#[test]
fn add_transition_creates_states() {
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_state("Start", StateType::Enter);
    builder.add_transition(TransitionParameters {
        source: "A",
        target: "B",
        event: "Event".into(),
        action: None,
        guard: None,
    });
    let fsm = builder.build().unwrap();

    let names: Vec<_> = fsm.states().map(|s| s.name().to_string()).collect();
    assert!(names.contains(&"Start".to_string()));
    assert!(names.contains(&"A".to_string()));
    assert!(names.contains(&"B".to_string()));
}

#[test]
fn add_transition_finds_existing_substate_from_root_scope() {
    crate::logging::init();
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_state("Start", StateType::Enter);

    let parent = builder.add_state("Parent", StateType::Simple);
    builder.set_scope(Some(parent));
    builder.add_state("Child", StateType::Simple);

    builder.set_scope(None);
    builder.add_transition(TransitionParameters {
        source: "Child",
        target: "Other",
        event: "toOther".into(),
        action: None,
        guard: None,
    });

    let fsm = builder.build().unwrap();

    // Child should only exist once (as substate)
    let count = fsm.states().filter(|s| s.name() == "Child").count();
    assert_eq!(count, 1);

    // The transition should be on Parent's substate
    let parent_state = fsm.states().find(|s| s.name() == "Parent").unwrap();
    let child = parent_state
        .substates()
        .find(|s| s.name() == "Child")
        .unwrap();
    let t = child.transitions().next().unwrap();
    assert_eq!(t.destination.name(), "Other");
}
