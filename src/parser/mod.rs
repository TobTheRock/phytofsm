use crate::error::{Error, Result};

mod builder;
mod context;
mod fsm;
mod plantuml;
mod types;

pub use builder::ParsedFsmBuilder;
pub use fsm::{ParsedFsm, State, Transition};
pub use types::{Action, Event, StateType};

impl ParsedFsm {
    pub fn try_parse<C>(content: C) -> Result<ParsedFsm>
    where
        C: AsRef<str>,
    {
        let diagram = plantuml::StateDiagram::parse(content.as_ref())?;
        diagram.try_into()
    }
}

impl TryFrom<plantuml::StateDiagram<'_>> for ParsedFsm {
    type Error = Error;
    fn try_from(diagram: plantuml::StateDiagram<'_>) -> Result<Self> {
        let name = diagram.name().map(|s| s.to_string()).unwrap_or_default();
        let mut builder = ParsedFsmBuilder::new(name);
        // TODO add states from state descriptions
        // TODO composite states
        for enter_state in diagram.enter_states() {
            builder.add_enter_state(enter_state)?;
        }
        for transition in diagram.transitions() {
            let ctx = context::TransitionContext::try_from(transition.description)?;
            builder.add_transition(transition.from, transition.to, ctx.event, ctx.action)?;
        }

        builder.build()
    }
}

#[cfg(test)]
mod test {
    use crate::{parser::ParsedFsm, test::FsmTestData};

    #[test]
    fn parses_fsm() {
        // TODO use a parameterized test framework
        let test_data = FsmTestData::all();
        for data in test_data {
            let fsm = ParsedFsm::try_parse(data.content)
                .expect(&format!("Failed to parse FSM for test data: {}", data.name));
            assert_eq!(
                data.parsed, fsm,
                "Parsed FSM does not match expected for test data: {}",
                data.name
            );
        }
    }
}
