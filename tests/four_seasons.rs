use itertools::Itertools;
///Test that the FSM generated from four_seasons.puml works as expected.
///Covers direct transitions and single event transitions with actions
use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "../src/test/four_seasons/four_seasons.puml",
    log_level = "debug"
);
use plant_fsm::{IPlantFsmActions, IPlantFsmEventParams, NoEventData, PlantFsm, PlantFsmEvent};

use mockall::{mock, predicate};

mock! {
    PlantFsmActions {}
    impl IPlantFsmActions for PlantFsmActions {
        fn start_blooming(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::DaylightIncreasesParams);
        fn ripen_fruit(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::DaylightDecreasesParams);
        fn drop_petals(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TemperatureDropsParams);
    }
}

impl IPlantFsmEventParams for MockPlantFsmActions {
    type TemperatureRisesParams = NoEventData;
    type DaylightIncreasesParams = i32;
    type DaylightDecreasesParams = NoEventData;
    type TemperatureDropsParams = NoEventData;
}

#[test]
// TODO parametrize somehow so we test also the reference
// TODO maybe use fuzz tests
/// This test covers:
/// - state transitions associated with only one event with and without actions
/// - logging of state transitions
fn simple_four_seasons() {
    mock_logger::init();
    let lumen = 42;
    let mut actions = MockPlantFsmActions::new();

    actions
        .expect_start_blooming()
        .returning(|_| ())
        .with(predicate::eq(lumen))
        .times(1);
    actions.expect_ripen_fruit().returning(|_| ()).times(1);
    actions.expect_drop_petals().returning(|_| ()).times(1);

    let mut fsm = PlantFsm::new(actions);
    // Trigger by reference
    fsm.trigger_event(PlantFsmEvent::TemperatureRises(()));
    fsm.trigger_event(PlantFsmEvent::DaylightIncreases(lumen));
    fsm.trigger_event(PlantFsmEvent::DaylightDecreases(()));
    fsm.trigger_event(PlantFsmEvent::TemperatureDrops(()));

    mock_logger::MockLogger::entries(|entries| {
        let debug_logs = entries
            .iter()
            .filter(|e| e.level == log::Level::Debug)
            .collect_vec();

        let n_transitions = 4;
        assert_eq!(debug_logs.len(), n_transitions);
    })
}
