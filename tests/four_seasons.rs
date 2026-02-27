///Test that the FSM generated from four_seasons.puml works as expected.
///Covers direct transitions and single event transitions with actions
use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "test/four_seasons/four_seasons.puml",
    log_level = "debug"
);

use plant_fsm::{IPlantFsmActions, IPlantFsmEventParams, NoEventData};

use mockall::{mock, predicate};

mock! {
    PlantFsmActions {}
    impl IPlantFsmActions for PlantFsmActions {
        // Transition actions
        fn start_blooming(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
        fn ripen_fruit(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
        fn drop_petals(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
        // Enter actions
        fn winter_is_coming(&mut self);
        fn start_heat_wave(&mut self);
        // Exit actions
        fn end_heat_wave(&mut self);
        // Guards
        fn enough_time_passed(
            &self,
            event: &<MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams,
        ) -> bool;
    }
}

impl IPlantFsmEventParams for MockPlantFsmActions {
    type TemperatureRisesParams = NoEventData;
    type TemperatureDropsParams = NoEventData;
    type TimeAdvancesParams = std::time::SystemTime;
}

#[test]
/// This test covers:
/// - state transitions associated with only one event with and without actions
fn test_transitions() {
    // let _ = stderrlog::new().verbosity(log::Level::Debug).init();
    let time = std::time::SystemTime::now();
    let mut actions = MockPlantFsmActions::new();

    // Guards
    actions
        .expect_enough_time_passed()
        .returning(|_| true)
        .times(4);

    // Transition actions
    actions
        .expect_start_blooming()
        .returning(|_| ())
        .with(predicate::eq(time))
        .times(1);
    actions.expect_ripen_fruit().returning(|_| ()).times(1);
    actions.expect_drop_petals().returning(|_| ()).times(1);

    // Enter/exit actions
    // winter_is_coming: called on start() and when returning from Autumn
    actions.expect_winter_is_coming().returning(|| ()).times(2);
    // start_heat_wave/end_heat_wave: not called (never enter Scorching)
    actions.expect_start_heat_wave().never();
    actions.expect_end_heat_wave().never();

    let mut fsm = plant_fsm::start(actions);
    fsm.temperature_rises(());
    fsm.time_advances(time);
    fsm.time_advances(time);
    fsm.temperature_drops(());
    fsm.time_advances(time);
    fsm.time_advances(time);
}
