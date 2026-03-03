use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "../src/test/transitions/internal_transitions.puml",
    log_level = "debug"
);

use internal_transitions::{IInternalTransitionsActions, IInternalTransitionsEventParams};
use mockall::{Sequence, mock};

mock! {
    Actions {}
    impl IInternalTransitionsActions for Actions {
        fn handle_internal_event(&mut self, params: <MockActions as IInternalTransitionsEventParams>::InternalEventParams);
        fn handle_self_transition_event(&mut self, params: <MockActions as IInternalTransitionsEventParams>::SelfTransitionEventParams);
        fn enter_state_a(&mut self);
        fn enter_state_b(&mut self);
        fn exit_state_a(&mut self);
        fn exit_state_b(&mut self);
    }
}

impl IInternalTransitionsEventParams for MockActions {
    type InternalEventParams = ();
    type SelfTransitionEventParams = ();
    type GoToBParams = ();
}

struct Fixture {
    actions: MockActions,
    seq: Sequence,
}

impl Fixture {
    fn new() -> Self {
        let mut f = Self {
            actions: MockActions::new(),
            seq: Sequence::new(),
        };
        f.expect_enter_state_a();
        f
    }

    fn start(self) -> internal_transitions::InternalTransitions<MockActions> {
        internal_transitions::start(self.actions)
    }

    fn navigate_to_state_b(&mut self) {
        self.expect_exit_state_a();
        self.expect_enter_state_b();
    }

    fn expect_enter_state_a(&mut self) {
        self.actions
            .expect_enter_state_a()
            .returning(|| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }

    fn expect_exit_state_a(&mut self) {
        self.actions
            .expect_exit_state_a()
            .returning(|| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }

    fn expect_enter_state_b(&mut self) {
        self.actions
            .expect_enter_state_b()
            .returning(|| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }

    fn expect_handle_internal_event(&mut self) {
        self.actions
            .expect_handle_internal_event()
            .returning(|_| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }

    fn expect_handle_self_transition_event(&mut self) {
        self.actions
            .expect_handle_self_transition_event()
            .returning(|_| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }
}

// --- StateA (simple state) ---

#[test]
fn internal_transition_calls_action_but_not_exit_enter() {
    let mut f = Fixture::new();
    f.expect_handle_internal_event();
    f.actions.expect_exit_state_a().never();

    let mut fsm = f.start();
    fsm.internal_event(());
}

#[test]
fn self_transition_calls_action_and_exit_enter() {
    let mut f = Fixture::new();
    f.expect_handle_self_transition_event();
    f.expect_exit_state_a();
    f.expect_enter_state_a();

    let mut fsm = f.start();
    fsm.self_transition_event(());
}

// --- StateBa (substate inside composite StateB) ---

#[test]
fn composite_internal_transition_does_not_trigger_exit_enter() {
    let mut f = Fixture::new();
    f.navigate_to_state_b();
    f.expect_handle_internal_event();
    f.actions.expect_exit_state_b().never();

    let mut fsm = f.start();
    fsm.go_to_b(());
    fsm.internal_event(());
}

#[test]
fn composite_self_transition_does_not_trigger_parent_exit_enter() {
    let mut f = Fixture::new();
    f.navigate_to_state_b();
    f.expect_handle_self_transition_event();
    f.actions.expect_exit_state_b().never();

    let mut fsm = f.start();
    fsm.go_to_b(());
    fsm.self_transition_event(());
}
