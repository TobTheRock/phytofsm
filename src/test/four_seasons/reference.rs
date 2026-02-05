use log::debug;

pub trait IPlantFsmEventParams {
    type TemperatureRisesParams;
    type TemperatureDropsParams;
    type TimeAdvancesParams;
}

// TODO
// - action results
pub trait IPlantFsmActions: IPlantFsmEventParams {
    // transition actions
    fn start_blooming(&mut self, event: Self::TimeAdvancesParams);
    fn ripen_fruit(&mut self, event: Self::TimeAdvancesParams);
    fn drop_petals(&mut self, event: Self::TimeAdvancesParams);
    // enter actions
    fn start_heat_wave(&mut self);
    fn winter_is_coming(&mut self);
    // exit actions
    fn end_heat_wave(&mut self);
}

type NoEventData = ();
enum PlantFsmEvent<T: IPlantFsmActions> {
    TemperatureRises(T::TemperatureRisesParams),
    TemperatureDrops(T::TemperatureDropsParams),
    TimeAdvances(T::TimeAdvancesParams),
}

impl<A> std::fmt::Display for PlantFsmEvent<A>
where
    A: IPlantFsmActions,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            PlantFsmEvent::TemperatureRises(_) => "TemperatureRises",
            PlantFsmEvent::TemperatureDrops(_) => "TemperatureDrops",
            PlantFsmEvent::TimeAdvances(_) => "TimeAdvances",
        };

        write!(f, "{}", name)
    }
}

#[derive(Copy)]
struct PlantFsmState<T: IPlantFsmActions> {
    // TODO maybe make this an enum?
    name: &'static str,
    // Transition based on an event, depending on the state
    transition: fn(event: PlantFsmEvent<T>, actions: &mut T) -> Option<Self>,
    // state to enter when transitioned to, if there are no substates this is Self
    enter_state: fn() -> Self,
    // enter action
    enter: fn(actions: &mut T) -> (),
    // exit action
    exit: fn(actions: &mut T) -> (),
}

impl<T: IPlantFsmActions> Clone for PlantFsmState<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            transition: self.transition,
            enter_state: self.enter_state,
            enter: self.enter,
            exit: self.exit,
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
            enter_state: Self::winter_freezing,
            enter: |actions| {
                actions.winter_is_coming();
            },
            exit: |_| {},
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
            enter_state: Self::winter_freezing,
            enter: |actions| (Self::winter().enter)(actions),
            exit: |actions| (Self::winter().exit)(actions),
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
            enter_state: Self::winter_mild,
            enter: |actions| (Self::winter().enter)(actions),
            exit: |actions| (Self::winter().exit)(actions),
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
            enter_state: Self::spring_brisk,
            enter: |_| {},
            exit: |_| {},
        }
    }

    fn spring_brisk() -> Self {
        Self {
            name: "Spring::Brisk",
            transition: |event, action| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::spring_temperate()),
                _ => {
                    let parent = Self::spring();
                    (parent.transition)(event, action)
                }
            },
            enter_state: Self::spring_brisk,
            enter: |actions| (Self::spring().enter)(actions),
            exit: |actions| (Self::spring().exit)(actions),
        }
    }

    fn spring_temperate() -> Self {
        Self {
            name: "Spring::Temperate",
            transition: |event, action| match event {
                PlantFsmEvent::TemperatureDrops(_) => Some(Self::spring_brisk()),
                _ => {
                    let parent = Self::spring();
                    (parent.transition)(event, action)
                }
            },
            enter_state: Self::spring_temperate,
            enter: |actions| (Self::spring().enter)(actions),
            exit: |actions| (Self::spring().exit)(actions),
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
            enter_state: Self::summer_balmy,
            enter: |_| {},
            exit: |_| {},
        }
    }

    fn summer_balmy() -> Self {
        Self {
            name: "Summer::Balmy",
            transition: |event, action| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::summer_scorching()),
                _ => {
                    let parent = Self::summer();
                    (parent.transition)(event, action)
                }
            },
            enter_state: Self::summer_balmy,
            enter: |actions| (Self::summer().enter)(actions),
            exit: |actions| (Self::summer().exit)(actions),
        }
    }

    fn summer_scorching() -> Self {
        Self {
            name: "Summer::Scorching",
            transition: |event, action| match event {
                PlantFsmEvent::TemperatureDrops(_) => Some(Self::summer_balmy()),
                _ => {
                    let parent = Self::summer();
                    (parent.transition)(event, action)
                }
            },
            enter_state: Self::summer_scorching,
            enter: |actions| actions.start_heat_wave(),
            exit: |actions| actions.end_heat_wave(),
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
            enter_state: Self::autumn_crisp,
            enter: |_| {},
            exit: |_| {},
        }
    }

    fn autumn_crisp() -> Self {
        Self {
            name: "Autumn::Crisp",
            transition: |event, action| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::autumn_pleasant()),
                _ => {
                    let parent = Self::autumn();
                    (parent.transition)(event, action)
                }
            },
            enter_state: Self::autumn_crisp,
            enter: |actions| (Self::autumn().enter)(actions),
            exit: |actions| (Self::autumn().exit)(actions),
        }
    }

    fn autumn_pleasant() -> Self {
        Self {
            name: "Autumn::Pleasant",
            transition: |event, action| match event {
                PlantFsmEvent::TemperatureDrops(_) => Some(Self::autumn_crisp()),
                _ => {
                    let parent = Self::autumn();
                    (parent.transition)(event, action)
                }
            },
            enter_state: Self::autumn_pleasant,
            enter: |actions| (Self::autumn().enter)(actions),
            exit: |actions| (Self::autumn().exit)(actions),
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
        let event_name = format!("{}", event);
        if let Some(new_state) = (self.current_state.transition)(event, &mut self.actions) {
            let new_state_name = new_state.name;
            let enter_state = self.enter_new_state(new_state);

            debug!(
                "PlantFsm: {} -[{}]-> {}, entering {}",
                self.current_state.name, event_name, enter_state.name, new_state_name
            );

            self.exit_current_state(enter_state);
        }
    }

    fn enter_new_state(&mut self, new_state: PlantFsmState<A>) -> PlantFsmState<A> {
        let enter_state = (new_state.enter_state)();

        (enter_state.enter)(&mut self.actions);
        enter_state
    }

    fn exit_current_state(&mut self, new_state: PlantFsmState<A>) {
        (self.current_state.exit)(&mut self.actions);
        self.current_state = new_state;
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

    use super::{IPlantFsmActions, IPlantFsmEventParams, NoEventData, PlantFsm};

    mock! {
        PlantFsmActions {}
        impl IPlantFsmActions for PlantFsmActions {
            fn start_blooming(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
            fn ripen_fruit(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
            fn drop_petals(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);

            fn start_heat_wave(&mut self);
            fn winter_is_coming(&mut self);

            fn end_heat_wave(&mut self);
        }
    }

    // TODO add a paremeter which is a reference
    impl IPlantFsmEventParams for MockPlantFsmActions {
        type TemperatureRisesParams = NoEventData;
        type TemperatureDropsParams = NoEventData;
        type TimeAdvancesParams = u32;
    }

    fn setup() -> MockPlantFsmActions {
        let _ = stderrlog::new().verbosity(log::Level::Debug).init();
        MockPlantFsmActions::new()
    }

    #[test]
    fn test_transitions() {
        let time = 42;
        let mut actions = setup();

        actions
            .expect_start_blooming()
            .returning(|_| ())
            .with(predicate::eq(time))
            .times(1);
        actions.expect_ripen_fruit().returning(|_| ()).times(1);
        actions.expect_drop_petals().returning(|_| ()).times(1);
        actions.expect_winter_is_coming().returning(|| ()).times(1);

        let mut fsm = PlantFsm::new(actions);
        fsm.temperature_rises(());
        fsm.time_advances(time);
        fsm.time_advances(time);
        fsm.temperature_drops(());
        fsm.time_advances(time);
        fsm.time_advances(time);
    }

    #[test]
    fn test_substate_enter_exit_actions() {
        let time = 42;
        let mut actions = setup();

        actions.expect_start_blooming().returning(|_| ()).times(1);
        actions.expect_start_heat_wave().returning(|| ()).times(1);
        actions.expect_end_heat_wave().returning(|| ()).times(1);

        let mut fsm = PlantFsm::new(actions);
        // To Summer
        fsm.time_advances(time);
        fsm.time_advances(time);
        // To/Out Scorching
        fsm.temperature_rises(());
        fsm.temperature_drops(());
    }
}
