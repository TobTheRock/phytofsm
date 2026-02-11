use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "../src/test/composite_states/substate_to_substate.puml",
    log_level = "debug"
);

use mockall::mock;
use substate_to_substate::{
    ISubstateToSubstateActions, ISubstateToSubstateEventParams, SubstateToSubstate,
};

mock! {
    SubstateToSubstateActions {}
    impl ISubstateToSubstateActions for SubstateToSubstateActions {
        fn action_in_aa(&mut self, event: <MockSubstateToSubstateActions as ISubstateToSubstateEventParams>::ToBaParams);
        fn action_in_ba(&mut self, event: <MockSubstateToSubstateActions as ISubstateToSubstateEventParams>::ToBbParams);
    }
}

impl ISubstateToSubstateEventParams for MockSubstateToSubstateActions {
    type ToBaParams = ();
    type ToBbParams = ();
}

#[test]
fn should_transition_from_substate_to_substate_across_parents() {
    // let _ = stderrlog::new().verbosity(log::Level::Trace).init();
    let mut actions = MockSubstateToSubstateActions::new();
    // Starting in A::AA, transition to B::BA should trigger action_in_aa
    actions.expect_action_in_aa().returning(|_| ()).times(1);

    let mut fsm = SubstateToSubstate::start(actions);
    fsm.to_ba(());
}

#[test]
fn should_transition_within_substate() {
    let mut actions = MockSubstateToSubstateActions::new();
    // First transition AA -> BA, then BA -> BB
    actions.expect_action_in_aa().returning(|_| ()).times(1);
    actions.expect_action_in_ba().returning(|_| ()).times(1);

    let mut fsm = SubstateToSubstate::start(actions);
    fsm.to_ba(());
    fsm.to_bb(());
}
