use crate::{
    parser,
    test::{FsmTestData, utils::get_adjacent_file_path},
};

impl FsmTestData {
    pub fn self_transition() -> Self {
        let state_a = parser::State {
            name: "StateA".to_string(),
            state_type: parser::StateType::Enter,
        };
        let state_b = parser::State {
            name: "StateB".to_string(),
            state_type: parser::StateType::Simple,
        };
        let parsed = parser::ParsedFsm::try_new(
            "TestFsm".to_string(),
            vec![
                parser::Transition {
                    source: state_a.clone(),
                    destination: state_a.clone(),
                    event: parser::Event("SelfTransition".to_string()),
                    action: Some(parser::Action("Action1".to_string())),
                },
                parser::Transition {
                    source: state_a.clone(),
                    destination: state_b.clone(),
                    event: parser::Event("GoToB".to_string()),
                    action: Some(parser::Action("Action2".to_string())),
                },
            ],
        )
        .expect("Failed to create expected FSM");
        let path = get_adjacent_file_path(file!(), "self_transition.puml");
        Self {
            name: "self_transition",
            content: include_str!("./self_transition.puml"),
            parsed,
            path,
        }
    }
}
