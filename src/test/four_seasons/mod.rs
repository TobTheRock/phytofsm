pub mod reference;
use std::path::Path;

use crate::{parser, test::FsmTestData};

impl FsmTestData {
    /// Simple FSM with state transistions  with actions and events
    pub fn four_seasons() -> Self {
        let winter = parser::State {
            name: "Winter".to_string(),
            state_type: parser::StateType::Enter,
        };
        let spring = parser::State {
            name: "Spring".to_string(),
            state_type: parser::StateType::Simple,
        };
        let summer = parser::State {
            name: "Summer".to_string(),
            state_type: parser::StateType::Simple,
        };
        let autumn = parser::State {
            name: "Autumn".to_string(),
            state_type: parser::StateType::Simple,
        };
        let expected = parser::ParsedFsm::try_new(
            "PlantFsm".to_string(),
            vec![
                parser::Transition {
                    source: winter.clone(),
                    destination: spring.clone(),
                    event: parser::Event("TemperatureRises".to_string()),
                    action: None,
                },
                parser::Transition {
                    source: spring.clone(),
                    destination: summer.clone(),
                    event: parser::Event("DaylightIncreases".to_string()),
                    action: Some(parser::Action("StartBlooming".to_string())),
                },
                parser::Transition {
                    source: summer.clone(),
                    destination: autumn.clone(),
                    event: parser::Event("DaylightDecreases".to_string()),
                    action: Some(parser::Action("RipenFruit".to_string())),
                },
                parser::Transition {
                    source: autumn.clone(),
                    destination: winter.clone(),
                    event: parser::Event("TemperatureDrops".to_string()),
                    action: Some(parser::Action("DropPetals".to_string())),
                },
            ],
        )
        .expect("Failed to create expected FSM");
        let parent_dir = Path::new(file!())
            .parent()
            .expect("Failed to get parent directory for test data");
        let path = parent_dir
            .join("./four_seasons.puml")
            .canonicalize()
            .expect("Failed to canonicalize path for test data");
        Self {
            content: include_str!("./four_seasons.puml"),
            fsm: expected,
            path,
        }
    }
}
