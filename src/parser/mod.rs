use derive_more::{From, Into};
use itertools::Itertools;

mod plantuml;

pub struct FsmFile {}

// impl FsmFile {
//     pub fn open() -> Result<Self> {
//         todo!()
//     }
//
//     pub fn parse(&self) -> Result<FsmRepr> {
//         todo!()
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into)]
pub struct Event(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into)]
pub struct Action(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateType {
    Plain,
    Enter,
    Exit,
}

// TODO extend with type and parent
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into)]
pub struct State(pub String);

pub struct Transition {
    pub from_state: State,
    pub event: Event,
    pub to_state: State,
    pub action: Option<Action>,
}

pub struct FsmRepr {
    name: String,
    transitions: Vec<Transition>,
}

impl FsmRepr {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn transitions(&self) -> impl Iterator<Item = &Transition> {
        self.transitions.iter()
    }

    pub fn all_events(&self) -> impl Iterator<Item = &Event> {
        self.transitions().map(|t| &t.event).unique()
    }

    pub fn transitions_by_source_state(&self) -> impl Iterator<Item = (&State, Vec<&Transition>)> {
        self.transitions()
            .map(|t| (&t.from_state, t))
            .into_group_map()
            .into_iter()
    }

    // TODO move to test module
    pub fn simple_four_seasons() -> Self {
        Self {
            name: "PlantFsm".to_string(),
            transitions: vec![
                Transition {
                    from_state: State("Winter".to_string()),
                    event: Event("TemperatureRises".to_string()),
                    to_state: State("Spring".to_string()),
                    action: None,
                },
                Transition {
                    from_state: State("Spring".to_string()),
                    event: Event("DaylightIncreases".to_string()),
                    to_state: State("Summer".to_string()),
                    action: Some(Action("StartBlooming".to_string())),
                },
                Transition {
                    from_state: State("Summer".to_string()),
                    event: Event("DaylightDecreases".to_string()),
                    to_state: State("Autumn".to_string()),
                    action: Some(Action("RipenFruit".to_string())),
                },
                Transition {
                    from_state: State("Autumn".to_string()),
                    event: Event("TemperatureDrops".to_string()),
                    to_state: State("Winter".to_string()),
                    action: Some(Action("DropPetals".to_string())),
                },
            ],
        }
    }
}

#[cfg(test)]
mod test {

    // #[test]
    // fn parse_simple_fsm() {
    //     let test_data = TestFsm::simple_fsm();
    // }
}
