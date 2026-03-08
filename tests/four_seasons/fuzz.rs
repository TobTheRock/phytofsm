use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt;
use std::rc::Rc;

use proptest::prelude::*;

proptest! {
    #[test]
    fn reference_matches_generated(events in prop::collection::vec(fuzz_event_strategy(), 0..100)) {
        let ref_guards = build_guard_queue(&events);
        let gen_guards = ref_guards.clone();

        let ref_recorder = ReferenceRecorder::new(ref_guards);
        let gen_recorder = GeneratedRecorder::new(gen_guards);

        let ref_log = ref_recorder.shared_log();
        let gen_log = gen_recorder.shared_log();

        let mut ref_fsm = super::reference::start(ref_recorder);
        let mut gen_fsm = super::plant_fsm::start(gen_recorder);

        replay_on_reference(&mut ref_fsm, &events);
        replay_on_generated(&mut gen_fsm, &events);

        prop_assert_eq!(ref_log, gen_log);
    }
}

#[derive(Clone)]
struct SharedLog(Rc<RefCell<Vec<String>>>);

impl SharedLog {
    fn new() -> Self {
        Self(Rc::new(RefCell::new(Vec::new())))
    }

    fn record(&self, action: &str) {
        self.0.borrow_mut().push(action.to_string());
    }
}

impl PartialEq for SharedLog {
    fn eq(&self, other: &Self) -> bool {
        *self.0.borrow() == *other.0.borrow()
    }
}

impl Eq for SharedLog {}

impl fmt::Debug for SharedLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.borrow().fmt(f)
    }
}

struct RecorderInner {
    log: SharedLog,
    guards: RefCell<VecDeque<bool>>,
}

impl RecorderInner {
    fn new(guards: VecDeque<bool>) -> Self {
        Self {
            log: SharedLog::new(),
            guards: RefCell::new(guards),
        }
    }

    fn record(&self, action: &str) {
        self.log.record(action);
    }

    fn next_guard(&self) -> bool {
        self.guards.borrow_mut().pop_front().unwrap_or(false)
    }

    fn shared_log(&self) -> SharedLog {
        self.log.clone()
    }
}

struct ReferenceRecorder(RecorderInner);
struct GeneratedRecorder(RecorderInner);

impl ReferenceRecorder {
    fn new(guards: VecDeque<bool>) -> Self {
        Self(RecorderInner::new(guards))
    }

    fn shared_log(&self) -> SharedLog {
        self.0.shared_log()
    }
}

impl GeneratedRecorder {
    fn new(guards: VecDeque<bool>) -> Self {
        Self(RecorderInner::new(guards))
    }

    fn shared_log(&self) -> SharedLog {
        self.0.shared_log()
    }
}

macro_rules! impl_recorder {
    ($event_params:path, $actions:path, $recorder:ident) => {
        impl $event_params for $recorder {
            type TemperatureRisesParams = ();
            type TemperatureDropsParams = ();
            type TimeAdvancesParams = ();
        }

        impl $actions for $recorder {
            fn start_blooming(&mut self, _event: ()) {
                self.0.record("start_blooming");
            }
            fn ripen_fruit(&mut self, _event: ()) {
                self.0.record("ripen_fruit");
            }
            fn drop_petals(&mut self, _event: ()) {
                self.0.record("drop_petals");
            }
            fn spontaneous_combustion(&mut self, _event: ()) {
                self.0.record("spontaneous_combustion");
            }
            fn start_blizzard(&mut self) {
                self.0.record("start_blizzard");
            }
            fn start_heat_wave(&mut self) {
                self.0.record("start_heat_wave");
            }
            fn winter_is_coming(&mut self) {
                self.0.record("winter_is_coming");
            }
            fn end_heat_wave(&mut self) {
                self.0.record("end_heat_wave");
            }
            fn enough_time_passed(&self, _event: &()) -> bool {
                self.0.next_guard()
            }
            fn has_very_cold_weather(&self) -> bool {
                self.0.next_guard()
            }
        }
    };
}

impl_recorder!(
    super::reference::IPlantFsmEventParams,
    super::reference::IPlantFsmActions,
    ReferenceRecorder
);
impl_recorder!(
    super::plant_fsm::IPlantFsmEventParams,
    super::plant_fsm::IPlantFsmActions,
    GeneratedRecorder
);

#[derive(Debug, Clone)]
enum FuzzEvent {
    TemperatureRises,
    TemperatureDrops,
    TimeAdvances(bool),
}

fn fuzz_event_strategy() -> impl Strategy<Value = FuzzEvent> {
    prop_oneof![
        Just(FuzzEvent::TemperatureRises),
        Just(FuzzEvent::TemperatureDrops),
        any::<bool>().prop_map(FuzzEvent::TimeAdvances),
    ]
}

fn build_guard_queue(events: &[FuzzEvent]) -> VecDeque<bool> {
    events
        .iter()
        .filter_map(|e| match e {
            FuzzEvent::TimeAdvances(guard) => Some(*guard),
            _ => None,
        })
        .collect()
}

fn replay_on_reference(
    fsm: &mut super::reference::PlantFsm<ReferenceRecorder>,
    events: &[FuzzEvent],
) {
    for event in events {
        match event {
            FuzzEvent::TemperatureRises => fsm.temperature_rises(()),
            FuzzEvent::TemperatureDrops => fsm.temperature_drops(()),
            FuzzEvent::TimeAdvances(_) => fsm.time_advances(()),
        }
    }
}

fn replay_on_generated(
    fsm: &mut super::plant_fsm::PlantFsm<GeneratedRecorder>,
    events: &[FuzzEvent],
) {
    for event in events {
        match event {
            FuzzEvent::TemperatureRises => fsm.temperature_rises(()),
            FuzzEvent::TemperatureDrops => fsm.temperature_drops(()),
            FuzzEvent::TimeAdvances(_) => fsm.time_advances(()),
        }
    }
}
