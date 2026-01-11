///Test that the FSM generated from four_seasons.puml works as expected.
///Covers direct transitions and single event transitions with actions
use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "../src/test/four_seasons/four_seasons.puml",
    log_level = "debug"
);
use plant_fsm::{IPlantFsmActions, IPlantFsmEventParams, NoEventData, PlantFsm};

use mockall::{mock, predicate};

mock! {
    PlantFsmActions {}
    impl IPlantFsmActions for PlantFsmActions {
        fn start_blooming(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
        fn ripen_fruit(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
        fn drop_petals(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
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

    actions
        .expect_start_blooming()
        .returning(|_| ())
        .with(predicate::eq(time))
        .times(1);
    actions.expect_ripen_fruit().returning(|_| ()).times(1);
    actions.expect_drop_petals().returning(|_| ()).times(1);

    let mut fsm = PlantFsm::new(actions);
    fsm.temperature_rises(());
    fsm.time_advances(time);
    fsm.time_advances(time);
    fsm.temperature_drops(());
    fsm.time_advances(time);
    fsm.time_advances(time);
}
