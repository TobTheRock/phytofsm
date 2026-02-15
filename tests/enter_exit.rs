use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "test/actions/enter_exit.puml",
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
    type GoToAFromAParams = ();
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
        let mut t = Self {
            actions: MockActions::new(),
            seq: Sequence::new(),
        };
        // ignore: [*] -> A
        t.expect_enter_a();
        t
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

    fn expect_a_to_c1(&mut self) {
        self.expect_exit_a();
        self.expect_enter_c();
        self.expect_enter_c1();
    }

    fn expect_a_to_c2(&mut self) {
        self.expect_exit_a();
        // C2 doesn't have its own enter, so it should call C's enter
        self.expect_enter_c();
    }
}

#[test]
fn enter_action_called_on_initial_state() {
    let mut actions = MockActions::new();
    actions.expect_enter_a().returning(|| ()).times(1);

    let _fsm = EnterExitActions::start(actions);
}

#[test]
fn exit_action_called_when_leaving_state() {
    let mut t = EnterExitTests::new();

    t.expect_exit_a();

    let mut fsm = EnterExitActions::start(t.actions);
    fsm.go_to_b(());
}

#[test]
fn enter_action_called_when_entering_state() {
    let mut t = EnterExitTests::new();

    // A -> B
    t.expect_exit_a();
    // B -> A
    t.expect_enter_a();

    let mut fsm = EnterExitActions::start(t.actions);
    fsm.go_to_b(());
    fsm.go_to_a_from_b(());
}

#[test]
fn parent_enter_before_substate_enter() {
    let mut t = EnterExitTests::new();

    t.expect_a_to_c1();

    let mut fsm = EnterExitActions::start(t.actions);
    fsm.go_to_c1_from_a(());
}

#[test]
fn substate_exit_before_parent_exit() {
    let mut t = EnterExitTests::new();

    t.expect_a_to_c1();

    // C1 -> A
    t.expect_exit_c1();
    t.expect_exit_c();
    t.expect_enter_a();

    let mut fsm = EnterExitActions::start(t.actions);
    fsm.go_to_c1_from_a(());
    fsm.go_to_a_from_c(());
}

#[test]
fn substate_entry_defaults_to_parent_enter() {
    let mut t = EnterExitTests::new();

    t.expect_a_to_c2();

    let mut fsm = EnterExitActions::start(t.actions);
    fsm.go_to_c2_from_a(());
}

#[test]
fn substate_exit_defaults_to_parent_exit() {
    let mut t = EnterExitTests::new();

    t.expect_a_to_c2();
    // C2 -> A
    t.expect_exit_c();
    t.expect_enter_a();

    let mut fsm = EnterExitActions::start(t.actions);
    fsm.go_to_c2_from_a(());
    fsm.go_to_a_from_c(());
}

#[test]
fn internal_substate_transition_only_calls_substate_actions() {
    let mut t = EnterExitTests::new();

    t.expect_a_to_c1();
    // C1 -> C2
    t.expect_exit_c1();
    t.actions.expect_enter_c().never();
    t.actions.expect_exit_c().never();

    let mut fsm = EnterExitActions::start(t.actions);
    fsm.go_to_c1_from_a(());
    fsm.go_to_c2(());
}

#[test]
fn self_transition_calls_exit_and_enter() {
    let mut t = EnterExitTests::new();

    t.expect_exit_a();
    t.expect_enter_a();

    let mut fsm = EnterExitActions::start(t.actions);
    fsm.go_to_a_from_a(());
}

// TODO internal transitions
