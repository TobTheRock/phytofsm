use std::marker::PhantomData;

// TODO
// - split IFs?
// - action results
trait IPlantFsmActions {
    type TemperaturePeaksEventData;
    fn start_blooming(&mut self, event: Self::TemperaturePeaksEventData);
}

// Generated
type EmptyEventData = ();
enum PlantFsmEvent<T: IPlantFsmActions> {
    TemperatureRises(EmptyEventData),
    TemperaturePeaks(T::TemperaturePeaksEventData),
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
                PlantFsmEvent::TemperaturePeaks(data) => {
                    action.start_blooming(data);
                    Some(Self::summer())
                }
                _ => None,
            },
        }
    }

    fn summer() -> Self {
        Self {
            name: "Summer",
            transition: |event, action| todo!(),
        }
    }

    fn autumn() -> Self {
        todo!()
    }
}

pub struct PlantFsm<T: IPlantFsmActions> {
    // TODO ownership, can this maybe a ref?
    actions: T,
    current_state: PlantFsmState<T>,
}

// TODO traces for transitions
impl<T> PlantFsm<T>
where
    T: IPlantFsmActions,
{
    pub fn new(actions: T) -> Self {
        Self {
            actions,
            current_state: PlantFsmState::winter(),
        }
    }

    pub fn trigger_event(&mut self, event: PlantFsmEvent<T>) {
        if let Some(new_state) = (self.current_state.transition)(event, &mut self.actions) {
            self.current_state = new_state;
        }
    }
}

#[cfg(test)]
mod test {
    use super::{EmptyEventData, IPlantFsmActions, PlantFsm, PlantFsmEvent};

    struct LogActions;
    enum Error {
        Bad,
        VeryBad,
    }
    impl IPlantFsmActions for LogActions {
        type TemperaturePeaksEventData = u32;

        fn start_blooming(&mut self, event: Self::TemperaturePeaksEventData) {
            println!("Blooming at temperature {}", event);
        }
    }

    #[test]
    fn test_enum() {
        let lg = LogActions {};
        let mut fsm = PlantFsm::new(lg);

        fsm.trigger_event(PlantFsmEvent::TemperatureRises(()));
        fsm.trigger_event(PlantFsmEvent::TemperaturePeaks(42));
    }
}
