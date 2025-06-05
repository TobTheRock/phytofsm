use crate::parser::{FsmRepr, State, StateType, Transition};

pub struct TestFsm {
    plant_uml_code: &'static str,
    fsm: FsmRepr,
}

impl TestFsm {
    pub fn all() -> impl IntoIterator<Item = Self> {
        [Self::simple_fsm()]
    }

    pub fn simple_fsm() -> Self {
        let fsm = FsmRepr {
            states: vec![State {
                name: String::from("Winter"),
                state_type: StateType::Enter,
                descriptions: vec![],
                parent: None,
            }],
            name: "Plant Seasonal Lifecycle".to_string(),
            transitions: todo!(),
        };
        Self {
            plant_uml_code: include_str!("./simple.puml"),
            fsm,
        }
    }
}
