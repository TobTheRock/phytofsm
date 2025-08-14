use derive_more::{From, Into};

mod context;
mod nom;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateType {
    Simple,
    Enter,
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub name: String,
    pub state_type: StateType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transition {
    pub source: State,
    pub destination: State,
    // TODO make this optional for direct transitions
    pub event: Event,
    pub action: Option<Action>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FsmRepr {
    pub name: String,
    pub transitions: Vec<Transition>,
}

impl FsmRepr {
    // TODO move to test module
    pub fn simple_four_seasons() -> Self {
        let winter = State {
            name: "Winter".to_string(),
            state_type: StateType::Enter,
        };
        let spring = State {
            name: "Spring".to_string(),
            state_type: StateType::Simple,
        };
        let summer = State {
            name: "Summer".to_string(),
            state_type: StateType::Simple,
        };
        let autumn = State {
            name: "Autumn".to_string(),
            state_type: StateType::Simple,
        };
        Self {
            name: "PlantFsm".to_string(),
            transitions: vec![
                Transition {
                    source: winter.clone(),
                    destination: spring.clone(),
                    event: Event("TemperatureRises".to_string()),
                    action: None,
                },
                Transition {
                    source: spring.clone(),
                    destination: summer.clone(),
                    event: Event("DaylightIncreases".to_string()),
                    action: Some(Action("StartBlooming".to_string())),
                },
                Transition {
                    source: summer.clone(),
                    destination: autumn.clone(),
                    event: Event("DaylightDecreases".to_string()),
                    action: Some(Action("RipenFruit".to_string())),
                },
                Transition {
                    source: autumn.clone(),
                    destination: winter.clone(),
                    event: Event("TemperatureDrops".to_string()),
                    action: Some(Action("DropPetals".to_string())),
                },
            ],
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parser::{FsmRepr, plantuml::PlantUmlFsmParser};

    const DATA: &str = include_str!("../test_data/simple.puml");

    #[test]
    fn parse_simple_fsm() {
        let test_data = FsmRepr::simple_four_seasons();
        // TODO use FsmFile?
        let mut parser = PlantUmlFsmParser::new();
        let fsm = parser.parse(DATA).unwrap();
        assert_eq!(test_data, fsm);
    }
}
