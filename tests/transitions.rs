use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "../src/test/transitions/transitions.puml",
    log_level = "debug"
);

use mockall::mock;
use test_fsm::{ITestFsmActions, ITestFsmEventParams};

mock! {
    TestFsmActions {}
    impl ITestFsmActions for TestFsmActions {
        fn action1(&mut self, event: <MockTestFsmActions as ITestFsmEventParams>::SelfTransitionParams);
        fn action2(&mut self, event: <MockTestFsmActions as ITestFsmEventParams>::SelfTransitionParams);
        fn action3(&mut self, event: <MockTestFsmActions as ITestFsmEventParams>::SelfTransitionParams);
    }
}

impl ITestFsmEventParams for MockTestFsmActions {
    type SelfTransitionParams = ();
    type GoToBParams = ();
    type GoToBDifferentlyParams = ();
    type GoToCParams = ();
}

#[test]
fn self_transition_action_called() {
    let mut actions = MockTestFsmActions::new();
    actions.expect_action1().returning(|_| ()).times(1);
    actions.expect_action2().returning(|_| ()).times(1);
    let mut fsm = test_fsm::start(actions);

    fsm.self_transition(());
    fsm.go_to_b(());
}

#[test]
fn final_state() {
    let mut actions = MockTestFsmActions::new();
    actions.expect_action1().times(0);
    actions.expect_action2().returning(|_| ()).times(1);
    let mut fsm = test_fsm::start(actions);

    fsm.go_to_b(());
    fsm.self_transition(());
}

#[test]
fn alternative_transition() {
    let mut actions = MockTestFsmActions::new();
    actions.expect_action3().returning(|_| ()).times(1);
    actions.expect_action2().times(0);
    let mut fsm = test_fsm::start(actions);

    fsm.go_to_b_differently(());
}
