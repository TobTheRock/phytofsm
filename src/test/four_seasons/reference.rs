use log::debug;

pub trait IPlantFsmEventParams {
    type TemperatureRisesParams;
    type DaylightIncreasesParams;
    type TemperatureDropsParams;
    type DaylightDecreasesParams;
}

// TODO
// - action results
pub trait IPlantFsmActions: IPlantFsmEventParams {
    fn start_blooming(&mut self, event: Self::DaylightIncreasesParams);
    fn ripen_fruit(&mut self, event: Self::DaylightDecreasesParams);
    fn drop_petals(&mut self, event: Self::TemperatureDropsParams);
}

type NoEventData = ();
enum PlantFsmEvent<T: IPlantFsmActions> {
    TemperatureRises(T::TemperatureRisesParams),
    DaylightIncreases(T::DaylightIncreasesParams),
    TemperatureDrops(T::TemperatureDropsParams),
    DaylightDecreases(T::DaylightDecreasesParams),
}

// TODO
// - enter /exit actions
// - getter for parent state
struct PlantFsmState<T: IPlantFsmActions> {
    name: &'static str,
    transition: fn(event: PlantFsmEvent<T>, actions: &mut T) -> Option<PlantFsmState<T>>,
}

impl<T> PlantFsmState<T>
where
    T: IPlantFsmActions,
{
    fn winter() -> Self {
        Self {
            name: "Winter",
            transition: |event, _| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::spring()),
                _ => None,
            },
        }
    }

    fn spring() -> Self {
        Self {
            name: "Spring",
            transition: |event, action| match event {
                PlantFsmEvent::DaylightIncreases(params) => {
                    action.start_blooming(params);
                    Some(Self::summer())
                }
                _ => None,
            },
        }
    }

    fn summer() -> Self {
        Self {
            name: "Summer",
            transition: |event, action| match event {
                PlantFsmEvent::DaylightDecreases(params) => {
                    action.ripen_fruit(params);
                    Some(Self::autumn())
                }
                _ => None,
            },
        }
    }

    fn autumn() -> Self {
        Self {
            name: "Autumn",
            transition: |event, action| match event {
                PlantFsmEvent::TemperatureDrops(params) => {
                    action.drop_petals(params);
                    Some(Self::winter())
                }
                _ => None,
            },
        }
    }
}

pub struct PlantFsm<T: IPlantFsmActions> {
    // TODO ownership, can this maybe a ref?
    actions: T,
    current_state: PlantFsmState<T>,
}

impl<A> PlantFsm<A>
where
    A: IPlantFsmActions,
{
    pub fn new(actions: A) -> Self {
        Self {
            actions,
            current_state: PlantFsmState::winter(),
        }
    }

    fn trigger_event(&mut self, event: PlantFsmEvent<A>) {
        if let Some(new_state) = (self.current_state.transition)(event, &mut self.actions) {
            debug!(
                "PlantFsm: Transitioning from {} to {}",
                self.current_state.name, new_state.name
            );
            self.current_state = new_state;
        }
    }

    pub fn daylight_increases(
        &mut self,
        params: <A as IPlantFsmEventParams>::DaylightIncreasesParams,
    ) {
        self.trigger_event(PlantFsmEvent::DaylightIncreases(params));
    }

    pub fn daylight_decreases(
        &mut self,
        params: <A as IPlantFsmEventParams>::DaylightDecreasesParams,
    ) {
        self.trigger_event(PlantFsmEvent::DaylightDecreases(params));
    }

    pub fn temperature_rises(
        &mut self,
        params: <A as IPlantFsmEventParams>::TemperatureRisesParams,
    ) {
        self.trigger_event(PlantFsmEvent::TemperatureRises(params));
    }

    pub fn temperature_drops(
        &mut self,
        params: <A as IPlantFsmEventParams>::TemperatureDropsParams,
    ) {
        self.trigger_event(PlantFsmEvent::TemperatureDrops(params));
    }
}

#[cfg(test)]
mod test {

    use mockall::{mock, predicate};

    use super::{IPlantFsmActions, IPlantFsmEventParams, NoEventData, PlantFsm, PlantFsmEvent};

    mock! {
        PlantFsmActions {}
        impl IPlantFsmActions for PlantFsmActions {
            fn start_blooming(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::DaylightIncreasesParams);
            fn ripen_fruit(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::DaylightDecreasesParams);
            fn drop_petals(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TemperatureDropsParams);
        }
    }

    // TODO add a paremeter which is a reference
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
    fn test_transitions() {
        let lumen = 42;
        let mut actions = setup();

        actions
            .expect_start_blooming()
            .returning(|_| ())
            .with(predicate::eq(lumen))
            .times(1);
        actions.expect_ripen_fruit().returning(|_| ()).times(1);
        actions.expect_drop_petals().returning(|_| ()).times(1);

        let mut fsm = PlantFsm::new(actions);
        fsm.temperature_rises(());
        fsm.daylight_increases(lumen);
        fsm.daylight_decreases(());
        fsm.temperature_drops(());
    }

    #[test]
    fn test_no_transitions() {
        let mut actions = setup();
        actions.expect_start_blooming().returning(|_| ()).times(0);

        let mut fsm = PlantFsm::new(actions);
        fsm.trigger_event(PlantFsmEvent::TemperatureDrops(()));
    }
}
