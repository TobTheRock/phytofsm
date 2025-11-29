use log::debug;

pub trait IPlantFsmEventParams {
    type TemperatureRisesParams;
    type TemperatureDropsParams;
    type TimeAdvancesParams;
}

// TODO
// - action results
pub trait IPlantFsmActions: IPlantFsmEventParams {
    fn start_blooming(&mut self, event: Self::TimeAdvancesParams);
    fn ripen_fruit(&mut self, event: Self::TimeAdvancesParams);
    fn drop_petals(&mut self, event: Self::TimeAdvancesParams);
}

type NoEventData = ();
enum PlantFsmEvent<T: IPlantFsmActions> {
    TemperatureRises(T::TemperatureRisesParams),
    TemperatureDrops(T::TemperatureDropsParams),
    TimeAdvances(T::TimeAdvancesParams),
}

#[derive(Copy)]
struct PlantFsmState<T: IPlantFsmActions> {
    // TODO maybe make this an enum?
    name: &'static str,
    // Transition based on an event, depending on the state
    transition: fn(event: PlantFsmEvent<T>, actions: &mut T) -> Option<Self>,
    // next state to enter directly (substate or direct transition)
    direct_enter: Option<fn() -> Self>,
}

impl<T: IPlantFsmActions> Clone for PlantFsmState<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            transition: self.transition,
            direct_enter: self.direct_enter,
        }
    }
}

impl<T> PlantFsmState<T>
where
    T: IPlantFsmActions,
{
    fn winter() -> Self {
        Self {
            name: "Winter",
            transition: |event, _| match event {
                PlantFsmEvent::TimeAdvances(_) => Some(Self::spring()),
                _ => None,
            },
            direct_enter: Some(Self::winter_freezing),
        }
    }

    fn winter_freezing() -> Self {
        Self {
            name: "Winter::Freezing",
            transition: |event, action| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::winter_mild()),
                // Check the parent
                _ => {
                    let parent = Self::winter();
                    (parent.transition)(event, action)
                }
            },
            direct_enter: None,
        }
    }

    fn winter_mild() -> Self {
        Self {
            name: "Winter::Mild",
            transition: |event, action| match event {
                PlantFsmEvent::TemperatureDrops(_) => Some(Self::winter_freezing()),
                _ => {
                    let parent = Self::winter();
                    (parent.transition)(event, action)
                }
            },
            direct_enter: None,
        }
    }

    fn spring() -> Self {
        Self {
            name: "Spring",
            transition: |event, action| match event {
                PlantFsmEvent::TimeAdvances(params) => {
                    action.start_blooming(params);
                    Some(Self::summer())
                }
                _ => None,
            },
            direct_enter: Some(Self::spring_chilly),
        }
    }

    fn spring_chilly() -> Self {
        Self {
            name: "Spring::Chilly",
            transition: |event, action| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::spring_warm()),
                _ => {
                    let parent = Self::spring();
                    (parent.transition)(event, action)
                }
            },
            direct_enter: None,
        }
    }

    fn spring_warm() -> Self {
        Self {
            name: "Spring",
            transition: |event, action| match event {
                PlantFsmEvent::TemperatureDrops(_) => Some(Self::spring_chilly()),
                _ => {
                    let parent = Self::spring();
                    (parent.transition)(event, action)
                }
            },
            direct_enter: None,
        }
    }

    fn summer() -> Self {
        Self {
            name: "Summer",
            transition: |event, action| match event {
                PlantFsmEvent::TimeAdvances(params) => {
                    action.ripen_fruit(params);
                    Some(Self::autumn())
                }
                _ => None,
            },
            direct_enter: Some(Self::summer_warm),
        }
    }

    fn summer_warm() -> Self {
        Self {
            name: "Summer::Warm",
            transition: |event, action| match event {
                PlantFsmEvent::TemperatureDrops(_) => Some(Self::summer_cool()),
                _ => {
                    let parent = Self::summer();
                    (parent.transition)(event, action)
                }
            },
            direct_enter: None,
        }
    }

    fn summer_cool() -> Self {
        Self {
            name: "Summer::Cool",
            transition: |event, action| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::summer_warm()),
                _ => {
                    let parent = Self::summer();
                    (parent.transition)(event, action)
                }
            },
            direct_enter: None,
        }
    }

    fn autumn() -> Self {
        Self {
            name: "Autumn",
            transition: |event, action| match event {
                PlantFsmEvent::TimeAdvances(params) => {
                    action.drop_petals(params);
                    Some(Self::winter())
                }
                _ => None,
            },
            direct_enter: Some(Self::autumn_chilly),
        }
    }

    fn autumn_chilly() -> Self {
        Self {
            name: "Autumn::Chilly",
            transition: |event, action| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::autumn_cool()),
                _ => {
                    let parent = Self::autumn();
                    (parent.transition)(event, action)
                }
            },
            direct_enter: None,
        }
    }

    fn autumn_cool() -> Self {
        Self {
            name: "Autumn::Cool",
            transition: |event, action| match event {
                PlantFsmEvent::TemperatureDrops(_) => Some(Self::autumn_chilly()),
                _ => {
                    let parent = Self::autumn();
                    (parent.transition)(event, action)
                }
            },
            direct_enter: None,
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
        if let Some(mut new_state) = (self.current_state.transition)(event, &mut self.actions) {
            debug!(
                "PlantFsm: Transitioning from {} to {}",
                self.current_state.name, new_state.name
            );

            // TODO exit function of current state here

            let new_state = self.enter_new_state(new_state);

            self.current_state = new_state;
        }
    }

    fn enter_new_state(&self, mut new_state: PlantFsmState<A>) -> PlantFsmState<A> {
        // TODO loop over on enter functions here..
        while let Some(direct_enter_fn) = new_state.direct_enter {
            let direct_state = direct_enter_fn();
            debug!("PlantFsm: Directly entering {}", direct_state.name);
            new_state = direct_state;
        }

        new_state
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

    pub fn time_advances(&mut self, params: <A as IPlantFsmEventParams>::TimeAdvancesParams) {
        self.trigger_event(PlantFsmEvent::TimeAdvances(params));
    }
}

#[cfg(test)]
mod test {
    use mockall::{mock, predicate};

    use super::{IPlantFsmActions, IPlantFsmEventParams, NoEventData, PlantFsm, PlantFsmEvent};

    mock! {
        PlantFsmActions {}
        impl IPlantFsmActions for PlantFsmActions {
            fn start_blooming(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
            fn ripen_fruit(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
            fn drop_petals(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
        }
    }

    // TODO add a paremeter which is a reference
    impl IPlantFsmEventParams for MockPlantFsmActions {
        type TemperatureRisesParams = NoEventData;
        type TemperatureDropsParams = NoEventData;
        type TimeAdvancesParams = std::time::SystemTime;
    }

    fn setup() -> MockPlantFsmActions {
        let _ = stderrlog::new().verbosity(log::Level::Debug).init();
        MockPlantFsmActions::new()
    }

    #[test]
    fn test_transitions() {
        let time = std::time::SystemTime::now();
        let mut actions = setup();

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
}
