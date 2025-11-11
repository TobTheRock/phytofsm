use std::path::Path;

use crate::{parser, test::FsmTestData};

impl FsmTestData {
    pub fn actions() -> Self {
        let state_a = parser::State {
            name: "StateA".to_string(),
            state_type: parser::StateType::Enter,
        };
        let state_b = parser::State {
            name: "StateB".to_string(),
            state_type: parser::StateType::Simple,
        };
        let parsed = parser::ParsedFsm::try_new(
            "Actions".to_string(),
            vec![
                parser::Transition {
                    source: state_a.clone(),
                    destination: state_b.clone(),
                    event: parser::Event("GoToB".to_string()),
                    action: Some(parser::Action("Action1".to_string())),
                },
                parser::Transition {
                    source: state_b.clone(),
                    destination: state_a.clone(),
                    event: parser::Event("GoToA".to_string()),
                    action: Some(parser::Action("Action2".to_string())),
                },
            ],
        )
        .expect("Failed to create expected FSM");
        let parent_dir = Path::new(file!())
            .parent()
            .expect("Failed to get parent directory for test data");
        let path = parent_dir
            .join("./actions.puml")
            .canonicalize()
            .expect("Failed to canonicalize path for test data");
        Self {
            name: "actions",
            content: include_str!("./actions.puml"),
            parsed,
            path,
        }
    }
}
