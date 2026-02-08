use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "../src/test/actions/enter_exit.puml",
    log_level = "debug"
);

use enter_exit_actions::{
    EnterExitActions, IEnterExitActionsActions, IEnterExitActionsEventParams,
};
use mockall::{Sequence, mock};

mock! {
    Actions {}
    impl IEnterExitActionsActions for Actions {
        fn enter_a(&mut self);
        fn exit_a(&mut self);
        fn enter_c(&mut self);
        fn exit_c(&mut self);
        fn enter_c1(&mut self);
        fn exit_c1(&mut self);
    }
}

impl IEnterExitActionsEventParams for MockActions {
    type GoToBParams = ();
    type GoToAFromBParams = ();
    type GoToCParams = ();
    type GoToC1FromAParams = ();
    type GoToC2FromAParams = ();
    type GoToAFromCParams = ();
    type GoToC2Params = ();
}

struct EnterExitTests {
    actions: MockActions,
    seq: Sequence,
}

impl EnterExitTests {
    fn new() -> Self {
        Self {
            actions: MockActions::new(),
            seq: Sequence::new(),
        }
    }
    fn expect_enter_a(&mut self) {
        self.actions
            .expect_enter_a()
            .returning(|| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }

    fn expect_exit_a(&mut self) {
        self.actions
            .expect_exit_a()
            .returning(|| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }

    fn expect_enter_c(&mut self) {
        self.actions
            .expect_enter_c()
            .returning(|| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }

    fn expect_exit_c(&mut self) {
        self.actions
            .expect_exit_c()
            .returning(|| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }

    fn expect_enter_c1(&mut self) {
        self.actions
            .expect_enter_c1()
            .returning(|| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }

    fn expect_exit_c1(&mut self) {
        self.actions
            .expect_exit_c1()
            .returning(|| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }
}

#[test]
fn enter_action_called_on_initial_state() {
    let mut actions = MockActions::new();
    actions.expect_enter_a().returning(|| ()).times(1);

    let _fsm = EnterExitActions::new(actions);
}

#[test]
fn exit_action_called_when_leaving_state() {
    let mut t = EnterExitTests::new();

    t.expect_enter_a();
    t.expect_exit_a();

    let mut fsm = EnterExitActions::new(t.actions);
    fsm.go_to_b(());
}

#[test]
fn enter_action_called_when_entering_state() {
    let mut t = EnterExitTests::new();

    t.expect_enter_a();
    t.expect_exit_a();
    t.expect_enter_a();

    let mut fsm = EnterExitActions::new(t.actions);
    fsm.go_to_b(());
    fsm.go_to_a_from_b(());
}

#[test]
fn substate_entry_overwrites_parent_enter() {
    let mut t = EnterExitTests::new();

    t.expect_enter_a();
    t.expect_exit_a();
    t.expect_enter_c1();
    t.actions.expect_enter_c().times(0);

    let mut fsm = EnterExitActions::new(t.actions);
    fsm.go_to_c1_from_a(());
}

#[test]
fn substate_exit_overwrites_parent_exit() {
    let mut t = EnterExitTests::new();

    t.expect_enter_a();
    t.expect_exit_a();
    t.expect_enter_c1();
    t.expect_exit_c1();
    t.expect_enter_a();

    let mut fsm = EnterExitActions::new(t.actions);
    fsm.go_to_c1_from_a(());
    fsm.go_to_a_from_c(());
}

#[test]
fn substate_entry_defaults_to_parent_enter() {
    let mut t = EnterExitTests::new();

    t.expect_enter_a();
    t.expect_exit_a();
    t.expect_enter_c();

    let mut fsm = EnterExitActions::new(t.actions);
    fsm.go_to_c2_from_a(());
}

#[test]
fn substate_exit_defaults_to_parent_exit() {
    let mut t = EnterExitTests::new();

    t.expect_enter_a();
    t.expect_exit_a();
    t.expect_enter_c();
    t.expect_exit_c();
    t.expect_enter_a();

    let mut fsm = EnterExitActions::new(t.actions);
    fsm.go_to_c2_from_a(());
    fsm.go_to_a_from_c(());
}

// TODO for in the reference: We must check if the new state we are going to is an internal
// substate! Before calling enter/exit actions. Skip for now
#[ignore]
#[test]
fn internal_substate_transition_only_calls_substate_actions() {
    let mut t = EnterExitTests::new();

    t.expect_enter_a();
    t.expect_enter_c1();
    t.expect_exit_a();
    t.expect_exit_c1();
    t.actions.expect_enter_c().times(0);
    t.actions.expect_exit_c().times(0);

    let mut fsm = EnterExitActions::new(t.actions);
    fsm.go_to_c1_from_a(());
    fsm.go_to_c2(());
}
