pub mod reference;

use crate::{
    error::Result,
    parser::{Action, Event, ParsedFsm, ParsedFsmBuilder},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_four_seasons_fsm() -> Result<ParsedFsm> {
    let mut builder = ParsedFsmBuilder::new("PlantFsm");
    builder.add_enter_state("Winter")?;
    builder
        .add_transition("Winter", "Spring", Event("TemperatureRises".into()), None)?
        .add_transition("Spring", "Summer", Event("DaylightIncreases".into()), Some(Action("StartBlooming".into())))?
        .add_transition("Summer", "Autumn", Event("DaylightDecreases".into()), Some(Action("RipenFruit".into())))?
        .add_transition("Autumn", "Winter", Event("TemperatureDrops".into()), Some(Action("DropPetals".into())))?;
    builder.build()
}

impl FsmTestData {
    pub fn four_seasons() -> Self {
        let path = get_adjacent_file_path(file!(), "four_seasons.puml");
        Self {
            name: "four_seasons",
            content: include_str!("./four_seasons.puml"),
            parsed: build_four_seasons_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }
}
