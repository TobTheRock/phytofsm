pub mod reference;

use crate::{
    error::Result,
    parser::{Action, Event, ParsedFsm, ParsedFsmBuilder, StateType},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_four_seasons_fsm() -> Result<ParsedFsm> {
    let mut builder = ParsedFsmBuilder::new("PlantFsm");

    // Root level states
    let winter = builder.add_state("Winter", StateType::Enter);
    builder.add_enter_action("Winter", Action::from("WinterIsComing"));
    let spring = builder.add_state("Spring", StateType::Simple);
    let summer = builder.add_state("Summer", StateType::Simple);
    let autumn = builder.add_state("Autumn", StateType::Simple);

    // Root level transitions
    builder.add_transition("Winter", "Spring", Event("TimeAdvances".into()), None);
    builder.add_transition(
        "Spring",
        "Summer",
        Event("TimeAdvances".into()),
        Some(Action("StartBlooming".into())),
    );
    builder.add_transition(
        "Summer",
        "Autumn",
        Event("TimeAdvances".into()),
        Some(Action("RipenFruit".into())),
    );
    builder.add_transition(
        "Autumn",
        "Winter",
        Event("TimeAdvances".into()),
        Some(Action("DropPetals".into())),
    );

    // Winter substates
    builder.set_scope(Some(winter));
    builder.add_state("Freezing", StateType::Enter);
    builder.add_state("Mild", StateType::Simple);
    builder.add_transition("Freezing", "Mild", Event("TemperatureRises".into()), None);
    builder.add_transition("Mild", "Freezing", Event("TemperatureDrops".into()), None);

    // Spring substates
    builder.set_scope(Some(spring));
    builder.add_state("Brisk", StateType::Enter);
    builder.add_state("Temperate", StateType::Simple);
    builder.add_transition("Brisk", "Temperate", Event("TemperatureRises".into()), None);
    builder.add_transition("Temperate", "Brisk", Event("TemperatureDrops".into()), None);

    // Summer substates
    builder.set_scope(Some(summer));
    builder.add_state("Balmy", StateType::Enter);
    builder.add_state("Scorching", StateType::Simple);
    builder.add_enter_action("Scorching", Action::from("StartHeatWave"));
    builder.add_exit_action("Scorching", Action::from("EndHeatWave"));
    builder.add_transition("Balmy", "Scorching", Event("TemperatureRises".into()), None);
    builder.add_transition("Scorching", "Balmy", Event("TemperatureDrops".into()), None);

    // Autumn substates
    builder.set_scope(Some(autumn));
    builder.add_state("Crisp", StateType::Enter);
    builder.add_state("Pleasant", StateType::Simple);
    builder.add_transition("Crisp", "Pleasant", Event("TemperatureRises".into()), None);
    builder.add_transition("Pleasant", "Crisp", Event("TemperatureDrops".into()), None);

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
