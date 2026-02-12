use crate::parser::{Action, ParsedFsm, ParsedFsmBuilder, State, StateType};

#[test]
fn set_state_enter_and_exit_actions() {
    let mut builder = builder_with_enter();
    builder.set_state_enter_action("Start", Action::from("OnEnter"));
    builder.set_state_exit_action("Start", Action::from("OnExit"));
    let fsm = builder.build().unwrap();

    let start = find_state(&fsm, "Start");
    assert_eq!(start.enter_action(), Some(&Action::from("OnEnter")));
    assert_eq!(start.exit_action(), Some(&Action::from("OnExit")));
}

#[test]
fn set_actions_on_state_created_by_transition() {
    let mut builder = builder_with_enter();
    builder.add_transition("Start", "Other", "GoToOther".into(), None);
    builder.set_state_enter_action("Other", Action::from("OnEnterOther"));
    builder.set_state_exit_action("Other", Action::from("OnExitOther"));
    let fsm = builder.build().unwrap();

    let other = find_state(&fsm, "Other");
    assert_eq!(other.enter_action(), Some(&Action::from("OnEnterOther")));
    assert_eq!(other.exit_action(), Some(&Action::from("OnExitOther")));
}

#[test]
fn set_substate_actions() {
    let mut builder = builder_with_enter();
    let parent = builder.add_state("Parent", StateType::Simple);
    builder.set_scope(Some(parent));
    builder.add_state("Child", StateType::Enter);
    builder.set_state_enter_action("Child", Action::from("OnEnterChild"));
    builder.set_state_exit_action("Child", Action::from("OnExitChild"));
    let fsm = builder.build().unwrap();

    let parent_state = find_state(&fsm, "Parent");
    let child = parent_state.substates().find(|s| s.name() == "Child").unwrap();
    assert_eq!(child.enter_action(), Some(&Action::from("OnEnterChild")));
    assert_eq!(child.exit_action(), Some(&Action::from("OnExitChild")));
}

fn builder_with_enter() -> ParsedFsmBuilder {
    let mut builder = ParsedFsmBuilder::new("TestFSM");
    builder.add_state("Start", StateType::Enter);
    builder
}

fn find_state<'a>(fsm: &'a ParsedFsm, name: &str) -> State<'a> {
    fsm.states().find(|s| s.name() == name).unwrap()
}
