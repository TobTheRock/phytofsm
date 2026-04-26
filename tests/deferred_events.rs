/// Test that the FSM generated from deferred.puml handles deferred events correctly.
///
/// Covers:
/// - Events deferred in one state fire after transitioning to a state that handles them
/// - Events re-deferred when transitioning to another deferring state
/// - Events re-evaluated after direct transitions
use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "test/deferred_events/deferred.puml",
    log_level = "debug"
);

use deferred_events::{IDeferredEventsActions, IDeferredEventsEventParams, NoEventData};
use mockall::mock;

mock! {
    DeferredEventsActions {}
    impl IDeferredEventsActions for DeferredEventsActions {
        fn enter_a(&mut self);
        fn enter_b(&mut self);
        fn enter_c(&mut self);
        fn enter_d(&mut self);
    }
}

impl IDeferredEventsEventParams for MockDeferredEventsActions {
    type GoToAParams = NoEventData;
    type GoToBParams = NoEventData;
    type GoToCParams = NoEventData;
    type GoToDParams = NoEventData;
    type GoToBFromCParams = NoEventData;
}

#[test]
fn deferred_event_fires_after_leaving_deferring_state() {
    let mut actions = MockDeferredEventsActions::new();

    actions.expect_enter_a().returning(|| ()).times(2); // once on start, once when deferred GoToA fires
    actions.expect_enter_b().returning(|| ()).times(1); // GoToB transitions to StateB
    actions.expect_enter_c().never();
    actions.expect_enter_d().never();

    let mut fsm = deferred_events::start(actions);

    // In StateA: GoToA is deferred
    fsm.go_to_a(());

    // GoToB transitions StateA -> StateB
    // Deferred GoToA fires in StateB: StateB -> StateA (enter_a called again)
    fsm.go_to_b(());
}

#[test]
fn event_is_redeferred() {
    let mut actions = MockDeferredEventsActions::new();

    actions.expect_enter_a().returning(|| ()).times(2); // once on start, once when deferred GoToA finally fires
    actions.expect_enter_b().returning(|| ()).times(1); // GoToBFromC transitions StateC -> StateB
    actions.expect_enter_c().returning(|| ()).times(1); // GoToC transitions StateA -> StateC
    actions.expect_enter_d().never();

    let mut fsm = deferred_events::start(actions);

    // GoToA is deferred in StateA
    fsm.go_to_a(());

    // GoToC: StateA -> StateC. Deferred GoToA is re-evaluated: also deferred in StateC (re-deferred)
    fsm.go_to_c(());

    // GoToBFromC: StateC -> StateB. Deferred GoToA fires: StateB -> StateA
    fsm.go_to_b_from_c(());
}

#[test]
fn deferred_event_fires_after_direct_transition() {
    let mut actions = MockDeferredEventsActions::new();

    actions.expect_enter_a().returning(|| ()).times(2); // once on start, once when deferred GoToA fires
    actions.expect_enter_b().returning(|| ()).times(1); // direct transition StateD -> StateB
    actions.expect_enter_c().never();
    actions.expect_enter_d().returning(|| ()).times(1); // GoToD transitions StateA -> StateD

    let mut fsm = deferred_events::start(actions);

    // GoToA is deferred in StateA
    fsm.go_to_a(());

    // GoToD: StateA -> StateD, direct transition StateD -> StateB
    // Deferred GoToA fires in StateB: StateB -> StateA
    fsm.go_to_d(());
}
