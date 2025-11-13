use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "../src/test/actions/actions.puml",
    log_level = "debug"
);

use mockall::{mock, predicate};
use test_fsm::{ITestFsmActions, ITestFsmEventParams, TestFsm, TestFsmEvent};

mock! {
    TestFsmActions {}
    impl ITestFsmActions for TestFsmActions {
        fn action1(&mut self, event: <MockTestFsmActions as ITestFsmEventParams>::GoToBParams);
        fn action2(&mut self, event: <MockTestFsmActions as ITestFsmEventParams>::GoToAParams);
    }
}

impl ITestFsmEventParams for MockTestFsmActions {
    type GoToBParams = ();
    type GoToAParams = i32;
}

#[test]
fn actions_are_called_when_transitioning() {
    let param_for_action2 = 7;
    let mut actions = MockTestFsmActions::new();

    actions.expect_action1().returning(|_| ()).times(1);
    actions
        .expect_action2()
        .with(predicate::eq(param_for_action2))
        .returning(|_| ())
        .times(1);
    let mut fsm = TestFsm::new(actions);
    fsm.trigger_event(TestFsmEvent::GoToB(()));
    fsm.trigger_event(TestFsmEvent::GoToA(param_for_action2));
}

#[test]
fn no_action_called_when_no_transition() {
    let mut actions = MockTestFsmActions::new();
    actions.expect_action1().times(0);
    actions.expect_action2().times(0);
    let mut fsm = TestFsm::new(actions);
    // Trigger event that does not cause a transition from the initial state
    fsm.trigger_event(TestFsmEvent::GoToA(5));
}
