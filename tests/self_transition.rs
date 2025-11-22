use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "../src/test/self_transition/self_transition.puml",
    log_level = "debug"
);

use mockall::mock;
use test_fsm::{ITestFsmActions, ITestFsmEventParams, TestFsm};

mock! {
    TestFsmActions {}
    impl ITestFsmActions for TestFsmActions {
        fn action1(&mut self, event: <MockTestFsmActions as ITestFsmEventParams>::SelfTransitionParams);
        fn action2(&mut self, event: <MockTestFsmActions as ITestFsmEventParams>::SelfTransitionParams);
    }
}

impl ITestFsmEventParams for MockTestFsmActions {
    type SelfTransitionParams = ();
    type GoToBParams = ();
}

#[test]
fn self_transition_action_called() {
    let mut actions = MockTestFsmActions::new();
    actions.expect_action1().returning(|_| ()).times(1);
    actions.expect_action2().returning(|_| ()).times(1);
    let mut fsm = TestFsm::new(actions);

    fsm.self_transition(());
    fsm.go_to_b(());
}

#[test]
fn final_state() {
    let mut actions = MockTestFsmActions::new();
    actions.expect_action1().times(0);
    actions.expect_action2().returning(|_| ()).times(1);
    let mut fsm = TestFsm::new(actions);

    fsm.go_to_b(());
    fsm.self_transition(());
}
