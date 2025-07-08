use phyto_fsm::generate_fsm;
generate_fsm!("../src/test_data/simple_fsm.puml");
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

fn setup() -> MockPlantFsmActions {
    let _ = stderrlog::new().verbosity(log::Level::Debug).init();
    MockPlantFsmActions::new()
}

#[test]
fn simple_fsm() {
    let lumen = 42;
    let mut actions = setup();

    actions
        .expect_start_blooming()
        .returning(|_| ())
        .with(predicate::eq(lumen))
        .times(1);
    actions.expect_ripen_fruit().returning(|_| ()).times(2);
    actions.expect_drop_petals().returning(|_| ()).times(1);

    let mut fsm = PlantFsm::new(actions);
    fsm.trigger_event(PlantFsmEvent::TemperatureRises(()));
    fsm.trigger_event(PlantFsmEvent::DaylightIncreases(lumen));
    fsm.trigger_event(PlantFsmEvent::DaylightDecreases(()));
    fsm.trigger_event(PlantFsmEvent::TemperatureDrops(()));
}
