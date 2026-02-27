use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "test/transitions/guards.puml",
    log_level = "debug"
);

use guards::{IGuardsActions, IGuardsEventParams};
use mockall::mock;

mock! {
    GuardsActions {}
    impl IGuardsActions for GuardsActions {
        // Transition actions
        fn action_to_a(&mut self, event: <MockGuardsActions as IGuardsEventParams>::ChangeStateParams);
        fn action_to_b(&mut self, event: <MockGuardsActions as IGuardsEventParams>::ChangeStateParams);
        fn action_to_c(&mut self, event: <MockGuardsActions as IGuardsEventParams>::ChangeStateParams);
        fn action_to_ca(&mut self, event: <MockGuardsActions as IGuardsEventParams>::ChangeStateParams);
        fn action_to_cb(&mut self, event: <MockGuardsActions as IGuardsEventParams>::ChangeStateParams);
        // Guards
        fn a_guard(&self, event: &<MockGuardsActions as IGuardsEventParams>::ChangeStateParams) -> bool;
        fn b_guard(&self, event: &<MockGuardsActions as IGuardsEventParams>::ChangeStateParams) -> bool;
        fn c_guard(&self, event: &<MockGuardsActions as IGuardsEventParams>::ChangeStateParams) -> bool;
        fn ca_guard(&self, event: &<MockGuardsActions as IGuardsEventParams>::ChangeStateParams) -> bool;
        fn cb_guard(&self, event: &<MockGuardsActions as IGuardsEventParams>::ChangeStateParams) -> bool;
    }
}

impl IGuardsEventParams for MockGuardsActions {
    type ChangeStateParams = ();
}

#[test]
fn no_transition_when_all_guards_deny() {
    let mut actions = MockGuardsActions::new();
    actions.expect_a_guard().returning(|_| false);
    actions.expect_b_guard().returning(|_| false);
    actions.expect_c_guard().returning(|_| false);
    actions.expect_action_to_a().never();
    actions.expect_action_to_b().never();
    actions.expect_action_to_c().never();

    let mut fsm = guards::start(actions);
    fsm.change_state(());
}

#[test]
fn guard_allows_specific_transition() {
    let mut actions = MockGuardsActions::new();
    actions.expect_a_guard().returning(|_| false);
    actions.expect_b_guard().returning(|_| true);
    actions.expect_action_to_b().returning(|_| ()).once();

    let mut fsm = guards::start(actions);
    fsm.change_state(());
}

#[test]
fn first_matching_guard_wins() {
    let mut actions = MockGuardsActions::new();
    actions.expect_a_guard().returning(|_| true);
    actions.expect_b_guard().returning(|_| true).never();
    actions.expect_action_to_a().returning(|_| ()).once();
    actions.expect_action_to_b().never();

    let mut fsm = guards::start(actions);
    fsm.change_state(());
}

#[test]
fn guard_on_composite_substate_transition() {
    let mut actions = MockGuardsActions::new();

    actions.expect_a_guard().returning(|_| false);
    actions.expect_b_guard().returning(|_| false);
    actions.expect_c_guard().returning(|_| true);
    actions.expect_ca_guard().returning(|_| true);

    actions.expect_action_to_c().returning(|_| ()).once();
    actions.expect_action_to_ca().returning(|_| ()).once();

    let mut fsm = guards::start(actions);
    fsm.change_state(()); // StateA -> StateC
    fsm.change_state(()); // StateC -> StateCa
}
