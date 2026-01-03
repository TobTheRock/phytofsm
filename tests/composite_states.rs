use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "../src/test/composite_states/composite_states.puml",
    log_level = "debug"
);

use composite_states::{CompositeStates, ICompositeStatesActions, ICompositeStatesEventParams};
use mockall::mock;

mock! {
    CompositeStatesActions {}
    impl ICompositeStatesActions for CompositeStatesActions {
        fn action_in_aaa(&mut self, event: <MockCompositeStatesActions as ICompositeStatesEventParams>::ToAabParams);
        fn action_in_aa(&mut self, event: <MockCompositeStatesActions as ICompositeStatesEventParams>::ToAbParams);
        fn action_in_a(&mut self, event: <MockCompositeStatesActions as ICompositeStatesEventParams>::ToBParams);
    }
}

impl ICompositeStatesEventParams for MockCompositeStatesActions {
    type ToAabParams = ();
    type ToAbParams = ();
    type ToBParams = ();
}

#[test]
fn should_change_between_nested_substates() {
    let _ = stderrlog::new().verbosity(log::Level::Trace).init();
    let mut actions = MockCompositeStatesActions::new();
    // Only way to reach state AAB is through AAA -> entering substates works
    actions.expect_action_in_aaa().returning(|_| ()).times(1);

    let mut fsm = CompositeStates::new(actions);
    fsm.to_aab(());
}

#[test]
fn should_change_between_substates() {
    let mut actions = MockCompositeStatesActions::new();
    // This guaranteses we can exit nested substates, if the parent has a respective transition for
    // the event
    actions.expect_action_in_aa().returning(|_| ()).times(1);

    let mut fsm = CompositeStates::new(actions);
    fsm.to_ab(());
}

#[test]
fn should_change_between_top_level_states() {
    let mut actions = MockCompositeStatesActions::new();
    // This guaranteses we can exit nested substates, if the parent has a respective transition for
    // the event
    actions.expect_action_in_a().returning(|_| ()).times(1);
    let mut fsm = CompositeStates::new(actions);
    fsm.to_b(());
}
