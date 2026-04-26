use crate::parser::{Event, ParsedFsmBuilder, StateType};

#[test]
fn add_deferred_event_to_state() {
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_state("Start", StateType::Enter);
    builder.add_deferred_event("Start", Event::from("MyEvent"));
    let fsm = builder.build().unwrap();

    let start = fsm.states().find(|s| s.name() == "Start").unwrap();
    let deferred: Vec<_> = start.deferred_events().collect();
    assert_eq!(deferred, vec![&Event::from("MyEvent")]);
}

#[test]
fn state_without_deferred_events_returns_empty() {
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_state("Start", StateType::Enter);
    let fsm = builder.build().unwrap();

    let start = fsm.states().find(|s| s.name() == "Start").unwrap();
    assert_eq!(start.deferred_events().count(), 0);
}

#[test]
fn multiple_deferred_events_on_same_state() {
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_state("Start", StateType::Enter);
    builder.add_deferred_event("Start", Event::from("EventA"));
    builder.add_deferred_event("Start", Event::from("EventB"));
    let fsm = builder.build().unwrap();

    let start = fsm.states().find(|s| s.name() == "Start").unwrap();
    let deferred: Vec<_> = start.deferred_events().collect();
    assert_eq!(
        deferred,
        vec![&Event::from("EventA"), &Event::from("EventB")]
    );
}
