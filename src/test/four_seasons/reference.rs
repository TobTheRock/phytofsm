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
    // guards
    fn enough_time_passed(&self, event: &Self::TimeAdvancesParams) -> bool;
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
    id: PlantFsmStateId,
    // Transition based on an event, depending on the state
    transition: fn(event: PlantFsmEvent<T>, actions: &mut T) -> Option<Self>,
    // state to enter when transitioned to, if there are no substates this is Self
    enter_state: fn() -> Self,
    // enter action, composite states check for internal transitions
    enter: fn(actions: &mut T, from: &Self) -> (),
    // exit action, composite states check for internal transitions
    exit: fn(actions: &mut T, to: &Self) -> (),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum PlantFsmStateId {
    Winter,
    WinterMild,
    WinterFreezing,
    Spring,
    SpringBrisk,
    SpringTemperate,
    Summer,
    SummerBalmy,
    SummerScorching,
    Autumn,
    AutumnCrisp,
    AutumnPleasant,
    _PlantFsmInitialState_,
}

impl From<PlantFsmStateId> for &'static str {
    fn from(name: PlantFsmStateId) -> Self {
        match name {
            PlantFsmStateId::Winter => "Winter",
            PlantFsmStateId::WinterMild => "Winter::Mild",
            PlantFsmStateId::WinterFreezing => "Winter::Freezing",
            PlantFsmStateId::Spring => "Spring",
            PlantFsmStateId::SpringBrisk => "Spring::Brisk",
            PlantFsmStateId::SpringTemperate => "Spring::Temperate",
            PlantFsmStateId::Summer => "Summer",
            PlantFsmStateId::SummerBalmy => "Summer::Balmy",
            PlantFsmStateId::SummerScorching => "Summer::Scorching",
            PlantFsmStateId::Autumn => "Autumn",
            PlantFsmStateId::AutumnCrisp => "Autumn::Crisp",
            PlantFsmStateId::AutumnPleasant => "Autumn::Pleasant",
            PlantFsmStateId::_PlantFsmInitialState_ => "[*]",
        }
    }
}

impl std::fmt::Display for PlantFsmStateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = (*self).into();
        write!(f, "{}", name)
    }
}

impl<T: IPlantFsmActions> Clone for PlantFsmState<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            transition: self.transition,
            enter_state: self.enter_state,
            enter: self.enter,
            exit: self.exit,
        }
    }
}

impl<T: IPlantFsmActions> PartialEq for PlantFsmState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> PlantFsmState<T>
where
    T: IPlantFsmActions,
{
    // Hidden init state to trigger enter actions when starting the FSM
    fn init() -> Self {
        Self {
            id: PlantFsmStateId::_PlantFsmInitialState_,
            transition: |_event, _action| None,
            enter_state: Self::init,
            enter: |_actions, _from| {},
            exit: |_actions, _to| {},
        }
    }

    fn winter() -> Self {
        Self {
            id: PlantFsmStateId::Winter,
            transition: |event, actions| match event {
                PlantFsmEvent::TimeAdvances(params) if actions.enough_time_passed(&params) => {
                    Some(Self::spring())
                }
                _ => None,
            },
            enter_state: Self::winter_freezing,
            enter: |actions, from| {
                if matches!(
                    from.id,
                    PlantFsmStateId::WinterFreezing | PlantFsmStateId::WinterMild
                ) {
                    return;
                }
                actions.winter_is_coming();
            },
            exit: |_actions, _to| {},
        }
    }

    fn winter_freezing() -> Self {
        Self {
            id: PlantFsmStateId::WinterFreezing,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::winter_mild()),
                // Check the parent
                _ => {
                    let parent = Self::winter();
                    (parent.transition)(event, actions)
                }
            },
            enter_state: Self::winter_freezing,
            enter: |actions, from| (Self::winter().enter)(actions, from),
            exit: |actions, to| (Self::winter().exit)(actions, to),
        }
    }

    fn winter_mild() -> Self {
        Self {
            id: PlantFsmStateId::WinterMild,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureDrops(_) => Some(Self::winter_freezing()),
                _ => {
                    let parent = Self::winter();
                    (parent.transition)(event, actions)
                }
            },
            enter_state: Self::winter_mild,
            enter: |actions, from| (Self::winter().enter)(actions, from),
            exit: |actions, to| (Self::winter().exit)(actions, to),
        }
    }

    fn spring() -> Self {
        Self {
            id: PlantFsmStateId::Spring,
            transition: |event, actions| match event {
                PlantFsmEvent::TimeAdvances(params) if actions.enough_time_passed(&params) => {
                    actions.start_blooming(params);
                    Some(Self::summer())
                }
                _ => None,
            },
            enter_state: Self::spring_brisk,
            enter: |_actions, _from| {},
            exit: |_actions, _to| {},
        }
    }

    fn spring_brisk() -> Self {
        Self {
            id: PlantFsmStateId::SpringBrisk,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::spring_temperate()),
                _ => {
                    let parent = Self::spring();
                    (parent.transition)(event, actions)
                }
            },
            enter_state: Self::spring_brisk,
            enter: |actions, from| (Self::spring().enter)(actions, from),
            exit: |actions, to| (Self::spring().exit)(actions, to),
        }
    }

    fn spring_temperate() -> Self {
        Self {
            id: PlantFsmStateId::SpringTemperate,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureDrops(_) => Some(Self::spring_brisk()),
                _ => {
                    let parent = Self::spring();
                    (parent.transition)(event, actions)
                }
            },
            enter_state: Self::spring_temperate,
            enter: |actions, from| (Self::spring().enter)(actions, from),
            exit: |actions, to| (Self::spring().exit)(actions, to),
        }
    }

    fn summer() -> Self {
        Self {
            id: PlantFsmStateId::Summer,
            transition: |event, actions| match event {
                PlantFsmEvent::TimeAdvances(params) if actions.enough_time_passed(&params) => {
                    actions.ripen_fruit(params);
                    Some(Self::autumn())
                }
                _ => None,
            },
            enter_state: Self::summer_balmy,
            enter: |_actions, _from| {},
            exit: |_actions, _to| {},
        }
    }

    fn summer_balmy() -> Self {
        Self {
            id: PlantFsmStateId::SummerBalmy,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::summer_scorching()),
                _ => {
                    let parent = Self::summer();
                    (parent.transition)(event, actions)
                }
            },
            enter_state: Self::summer_balmy,
            enter: |actions, from| (Self::summer().enter)(actions, from),
            exit: |actions, to| (Self::summer().exit)(actions, to),
        }
    }

    fn summer_scorching() -> Self {
        Self {
            id: PlantFsmStateId::SummerScorching,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureDrops(_) => Some(Self::summer_balmy()),
                _ => {
                    let parent = Self::summer();
                    (parent.transition)(event, actions)
                }
            },
            enter_state: Self::summer_scorching,
            enter: |actions, _from| actions.start_heat_wave(),
            exit: |actions, _to| actions.end_heat_wave(),
        }
    }

    fn autumn() -> Self {
        Self {
            id: PlantFsmStateId::Autumn,
            transition: |event, actions| match event {
                PlantFsmEvent::TimeAdvances(params) if actions.enough_time_passed(&params) => {
                    actions.drop_petals(params);
                    Some(Self::winter())
                }
                _ => None,
            },
            enter_state: Self::autumn_crisp,
            enter: |_actions, _from| {},
            exit: |_actions, _to| {},
        }
    }

    fn autumn_crisp() -> Self {
        Self {
            id: PlantFsmStateId::AutumnCrisp,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::autumn_pleasant()),
                _ => {
                    let parent = Self::autumn();
                    (parent.transition)(event, actions)
                }
            },
            enter_state: Self::autumn_crisp,
            enter: |actions, from| (Self::autumn().enter)(actions, from),
            exit: |actions, to| (Self::autumn().exit)(actions, to),
        }
    }

    fn autumn_pleasant() -> Self {
        Self {
            id: PlantFsmStateId::AutumnPleasant,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureDrops(_) => Some(Self::autumn_crisp()),
                _ => {
                    let parent = Self::autumn();
                    (parent.transition)(event, actions)
                }
            },
            enter_state: Self::autumn_pleasant,
            enter: |actions, from| (Self::autumn().enter)(actions, from),
            exit: |actions, to| (Self::autumn().exit)(actions, to),
        }
    }
}

struct PlantFsmImpl<A: IPlantFsmActions> {
    actions: A,
    current_state: PlantFsmState<A>,
}

impl<A> PlantFsmImpl<A>
where
    A: IPlantFsmActions,
{
    fn trigger_event(&mut self, event: PlantFsmEvent<A>) {
        let event_name = format!("{}", event);
        if let Some(transition_state) = (self.current_state.transition)(event, &mut self.actions) {
            let enter_state = (transition_state.enter_state)();

            debug!(
                "PlantFsm: {} -[{}]-> {}, entering {}",
                self.current_state.id, event_name, enter_state.id, transition_state.id
            );

            self.change_state(enter_state);
        }
    }

    fn change_state(&mut self, next_state: PlantFsmState<A>) {
        (self.current_state.exit)(&mut self.actions, &next_state);
        (next_state.enter)(&mut self.actions, &self.current_state);
        self.current_state = next_state;
    }
}

pub struct PlantFsm<A: IPlantFsmActions>(PlantFsmImpl<A>);
// This may already trigger actions, depending on the initial state
pub fn start<A: IPlantFsmActions>(mut actions: A) -> PlantFsm<A> {
    let init = PlantFsmState::init();
    let enter_state = PlantFsmState::winter_freezing();
    (enter_state.enter)(&mut actions, &init);
    PlantFsm(PlantFsmImpl {
        actions,
        current_state: enter_state,
    })
}

impl<A> PlantFsm<A>
where
    A: IPlantFsmActions,
{
    pub fn temperature_rises(
        &mut self,
        params: <A as IPlantFsmEventParams>::TemperatureRisesParams,
    ) {
        self.0
            .trigger_event(PlantFsmEvent::TemperatureRises(params));
    }

    pub fn temperature_drops(
        &mut self,
        params: <A as IPlantFsmEventParams>::TemperatureDropsParams,
    ) {
        self.0
            .trigger_event(PlantFsmEvent::TemperatureDrops(params));
    }

    pub fn time_advances(&mut self, params: <A as IPlantFsmEventParams>::TimeAdvancesParams) {
        self.0.trigger_event(PlantFsmEvent::TimeAdvances(params));
    }
}

#[cfg(test)]
mod test {
    use mockall::{mock, predicate};

    use super::{IPlantFsmActions, IPlantFsmEventParams, NoEventData};

    mock! {
        PlantFsmActions {}
        impl IPlantFsmActions for PlantFsmActions {
            fn start_blooming(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
            fn ripen_fruit(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
            fn drop_petals(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);

            fn start_heat_wave(&mut self);
            fn winter_is_coming(&mut self);

            fn end_heat_wave(&mut self);

            fn enough_time_passed(
                &self,
                event: &<MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams,
            ) -> bool;
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
            .expect_enough_time_passed()
            .returning(|_| true)
            .times(4);

        actions
            .expect_start_blooming()
            .returning(|_| ())
            .with(predicate::eq(time))
            .times(1);
        actions.expect_ripen_fruit().returning(|_| ()).times(1);
        actions.expect_drop_petals().returning(|_| ()).times(1);
        // Called twice: once on init (entering Winter::Freezing), once when returning from Autumn
        actions.expect_winter_is_coming().returning(|| ()).times(2);

        let mut fsm = super::start(actions);
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
        actions
            .expect_enough_time_passed()
            .returning(|_| true)
            .times(2);

        // Called once on init (entering Winter::Freezing)
        actions.expect_winter_is_coming().returning(|| ()).times(1);
        actions.expect_start_blooming().returning(|_| ()).times(1);
        actions.expect_start_heat_wave().returning(|| ()).times(1);
        actions.expect_end_heat_wave().returning(|| ()).times(1);

        let mut fsm = super::start(actions);
        // To Summer
        fsm.time_advances(time);
        fsm.time_advances(time);
        // To/Out Scorching
        fsm.temperature_rises(());
        fsm.temperature_drops(());
    }
}
