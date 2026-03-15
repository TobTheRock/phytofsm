//! # Four Seasons — Interactive CLI
//!
//! Demonstrates the Four Seasons FSM generated from PlantUML with ASCII art
//! visualization and keyboard controls.
//!
//! ```text
//! cargo run --example four_seasons
//! ```
//!
//! Controls:
//!   - `t`     — advance time (season changes after 3 advances)
//!   - `+`/`=` — raise temperature (warmer substate)
//!   - `-`     — lower temperature (colder substate)
//!   - `q`/Esc — quit

use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;
use std::time::Duration;

use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{self, ClearType};

use phyto_fsm::generate_fsm;
generate_fsm!(file_path = "test/four_seasons/four_seasons.puml");

use plant_fsm::{IPlantFsmActions, IPlantFsmEventParams};

// ---------------------------------------------------------------------------
// Display state shared between FSM actions and the render loop
// ---------------------------------------------------------------------------

struct DisplayState {
    season: &'static str,
    substate: &'static str,
    temperature: i32,
    time_counter: u32,
    message: String,
}

// ---------------------------------------------------------------------------
// FSM actions
// ---------------------------------------------------------------------------

struct PlantActions {
    state: Rc<RefCell<DisplayState>>,
}

impl IPlantFsmEventParams for PlantActions {
    type TemperatureRisesParams = ();
    type TemperatureDropsParams = ();
    type TimeAdvancesParams = ();
}

impl IPlantFsmActions for PlantActions {
    fn start_blooming(&mut self, _: ()) {
        let mut s = self.state.borrow_mut();
        s.season = "Summer";
        s.substate = "Balmy";
        s.time_counter = 0;
        s.message = "Flowers bloom — summer arrives!".into();
    }

    fn ripen_fruit(&mut self, _: ()) {
        let mut s = self.state.borrow_mut();
        s.season = "Autumn";
        s.substate = "Crisp";
        s.time_counter = 0;
        s.message = "Fruits ripen — autumn settles in.".into();
    }

    fn drop_petals(&mut self, _: ()) {
        let mut s = self.state.borrow_mut();
        s.time_counter = 0;
        s.message = "Petals drop — winter approaches...".into();
    }

    fn spontaneous_combustion(&mut self, _: ()) {
        self.state.borrow_mut().message = "SPONTANEOUS COMBUSTION! Too hot!".into();
    }

    fn start_blizzard(&mut self) {
        let mut s = self.state.borrow_mut();
        s.substate = "ArcticBlast";
        s.message = "A fierce blizzard engulfs everything!".into();
    }

    fn start_heat_wave(&mut self) {
        let mut s = self.state.borrow_mut();
        s.substate = "Scorching";
        s.message = "A scorching heat wave begins!".into();
    }

    fn winter_is_coming(&mut self) {
        let mut s = self.state.borrow_mut();
        s.season = "Winter";
        s.substate = "Freezing";
        s.message = "Winter is coming!".into();
    }

    fn end_heat_wave(&mut self) {
        let mut s = self.state.borrow_mut();
        s.substate = "Balmy";
        s.message = "The heat wave subsides.".into();
    }

    fn enough_time_passed(&self, _: &()) -> bool {
        self.state.borrow().time_counter >= 3
    }

    fn has_very_cold_weather(&self) -> bool {
        self.state.borrow().temperature < -15
    }
}

// ---------------------------------------------------------------------------
// ASCII art — one scene per season + substate
// ---------------------------------------------------------------------------

fn art_for(season: &str, substate: &str) -> &'static str {
    match (season, substate) {
        ("Winter", "Freezing") => "\
         *  .  *       .  *
       .    *    .       *

             ||
            /||\\
           / || \\
          /  ||  \\
         /   ||   \\
              ||
              ||
      ~~~~~~~~||~~~~~~~~
      ::::::::::::::::::: FROZEN
      ::::::::::::::::::: GROUND",

        ("Winter", "Mild") => "\
           .    .
         .   .    .

              ||
             /||\\
            / || \\
           /  ||  \\
          /   ||   \\
              ||
              ||
      --------||--------
      .  .  .  .  .  .  .
         Light snow",

        ("Winter", "ArcticBlast") => "\
      * * * BLIZZARD! * * *
        <<<  *  *  *  >>>
      *  *  <<<  >>>  *  *
            //||\\\\
           // || \\\\
          <<  ||  >>
         <<   ||   >>
        <<    ||    >>
              ||
      ========||========
      ####################
      ## ARCTIC  BLAST! ##
      ####################",

        ("Spring", "Brisk") => "\
           ~ ~ ~
            \\|/
         .-'***'-.
        /  * . *  \\
       |  . * . *  |
        \\  * . *  /
         '-.*.*.-'
            ||
            ||
            ||
      ------||------
      ~ ~ ~ ~ ~ ~ ~ ~
        Buds forming",

        ("Spring", "Temperate") => "\
          \\  |  /
         .-'''''-.
        / @  ()  @ \\
       | ()  @  ()  |
       |  @ ()  @   |
        \\ ()  @ () /
         '-......-'
            ||
            ||
            ||
      ------||------
      * @ * @ * @ * @
      Flowers in bloom",

        ("Summer", "Balmy") => "\
         \\  |  /
       --- \\|/ ---
         .-'''''--.
        / # # # # \\
       | # # # # # |
       | # # # # # |
        \\ # # # # /
         '-.....-'
            ||
            ||
      ======||======
      ~~~~~~~~~~~~~~~~
       Warm and green",

        ("Summer", "Scorching") => "\
      ~~ ~~ HEAT ~~ ~~
        \\\\  ||  //
      --- \\\\||// ---
         .-'WWW'-.
        / W W W W \\
       | W  W  W W |
       |  W W W  W |
        \\ W W W W /
         '-.....-'
            ||
            ||
      ======||======
      ^^^^^^^^^^^^^^^^
       SCORCHING HOT!",

        ("Autumn", "Crisp") => "\
            ,
         .-'oOo'-.
        / o O o O \\
       | O o . o O |
        \\ o O o O /
         '-..o..-'
         o   ||  O
        O    || o
       o     ||   O
             ||
      -------||-------
      o O o O o O o O
       Crisp autumn",

        ("Autumn", "Pleasant") => "\
           . * .
        .-'*.*.'-.
       / * . * . * \\
      | . * . * . * |
       \\ * . * . * /
        '--.*.*.--'
            ||
            ||
            ||
            ||
      ------||------
      * . * . * . * .
       Golden autumn",

        _ => "  [unknown state]",
    }
}

// ---------------------------------------------------------------------------
// Render
// ---------------------------------------------------------------------------

fn draw(stdout: &mut io::Stdout, state: &DisplayState) -> io::Result<()> {
    execute!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    for line in art_for(state.season, state.substate).lines() {
        write!(stdout, "  {line}\r\n")?;
    }

    write!(stdout, "\r\n")?;
    write!(
        stdout,
        "  Season: {}  |  Substate: {}  |  Temp: {}°C  |  Time: {}/3\r\n",
        state.season, state.substate, state.temperature, state.time_counter,
    )?;
    write!(stdout, "  > {}\r\n\r\n", state.message)?;
    write!(
        stdout,
        "  [t] Advance Time   [+/-] Temperature   [q] Quit\r\n"
    )?;
    stdout.flush()
}

// ---------------------------------------------------------------------------
// Event loop
// ---------------------------------------------------------------------------

fn run() -> io::Result<()> {
    let mut stdout = io::stdout();

    let state = Rc::new(RefCell::new(DisplayState {
        season: "",
        substate: "",
        temperature: -5,
        time_counter: 0,
        message: String::new(),
    }));

    let actions = PlantActions {
        state: Rc::clone(&state),
    };
    // FSM starts in Winter/Freezing — winter_is_coming fires automatically
    let mut fsm = plant_fsm::start(actions);

    loop {
        draw(&mut stdout, &state.borrow())?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Char('t') => {
                state.borrow_mut().time_counter += 1;
                let old_season = state.borrow().season;

                fsm.time_advances(());

                // Winter -> Spring has no action callback; detect and apply here
                if state.borrow().season == "Winter"
                    && old_season == "Winter"
                    && state.borrow().time_counter >= 3
                {
                    let mut s = state.borrow_mut();
                    s.season = "Spring";
                    s.substate = "Brisk";
                    s.time_counter = 0;
                    s.message = "Spring has arrived!".into();
                }
            }

            KeyCode::Char('+') | KeyCode::Char('=') => {
                state.borrow_mut().temperature += 1;
                let before = state.borrow().substate;

                fsm.temperature_rises(());

                // Actionless substate transitions — action callbacks already
                // handle Balmy->Scorching (start_heat_wave) and Scorching
                // internal (spontaneous_combustion).
                if state.borrow().substate == before {
                    let mut s = state.borrow_mut();
                    match (s.season, before) {
                        ("Winter", "Freezing") => {
                            s.substate = "Mild";
                            s.message = "A mild winter breeze.".into();
                        }
                        ("Spring", "Brisk") => {
                            s.substate = "Temperate";
                            s.message = "Temperate spring weather.".into();
                        }
                        ("Autumn", "Crisp") => {
                            s.substate = "Pleasant";
                            s.message = "A pleasant autumn day.".into();
                        }
                        _ => {}
                    }
                }
            }

            KeyCode::Char('-') => {
                state.borrow_mut().temperature -= 1;
                let before = state.borrow().substate;

                fsm.temperature_drops(());

                // Actionless substate transitions — action callbacks already
                // handle Scorching->Balmy (end_heat_wave) and direct
                // Freezing->ArcticBlast (start_blizzard).
                if state.borrow().substate == before {
                    let mut s = state.borrow_mut();
                    match (s.season, before) {
                        ("Winter", "Mild") => {
                            s.substate = "Freezing";
                            s.message = "Back to freezing cold.".into();
                        }
                        ("Spring", "Temperate") => {
                            s.substate = "Brisk";
                            s.message = "A brisk spring chill.".into();
                        }
                        ("Autumn", "Pleasant") => {
                            s.substate = "Crisp";
                            s.message = "Crisp autumn air returns.".into();
                        }
                        _ => {}
                    }
                }
            }

            KeyCode::Char('q') | KeyCode::Esc => break,
            _ => {}
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let result = run();
    terminal::disable_raw_mode()?;
    execute!(
        io::stdout(),
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )?;
    result
}
