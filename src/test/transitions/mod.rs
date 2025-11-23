use crate::{
    parser,
    test::{FsmTestData, utils::get_adjacent_file_path},
};

impl FsmTestData {
    /// Covers:
    /// - self transitions
    /// - transitions to same state via different events
    /// - multiple transitions from one state
    /// - final states without outgoing transitions
    pub fn transitions() -> Self {
        let state_a = parser::State {
            name: "StateA".to_string(),
            state_type: parser::StateType::Enter,
        };
        let state_b = parser::State {
            name: "StateB".to_string(),
            state_type: parser::StateType::Simple,
        };
        let state_c = parser::State {
            name: "StateC".to_string(),
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
                parser::Transition {
                    source: state_a.clone(),
                    destination: state_b.clone(),
                    event: parser::Event("GoToBDifferently".to_string()),
                    action: Some(parser::Action("Action3".to_string())),
                },
                parser::Transition {
                    source: state_a.clone(),
                    destination: state_c.clone(),
                    event: parser::Event("GoToC".to_string()),
                    action: None,
                },
            ],
        )
        .expect("Failed to create expected FSM");
        let path = get_adjacent_file_path(file!(), "transitions.puml");
        Self {
            name: "transitions",
            content: include_str!("./transitions.puml"),
            parsed,
            path,
        }
    }
}
