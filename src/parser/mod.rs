use crate::{
    error::{Error, Result},
    parser::builder::StateId,
};

mod builder;
mod context;
mod fsm;
mod plantuml;
mod types;

pub use builder::ParsedFsmBuilder;
pub use fsm::{ParsedFsm, State};
use log::trace;
pub use types::{Action, Event, StateType};

impl ParsedFsm {
    pub fn try_parse<C>(content: C) -> Result<ParsedFsm>
    where
        C: AsRef<str>,
    {
        let diagram = plantuml::StateDiagram::parse(content.as_ref())?;
        trace!("Parsed PlantUML diagram: {:#?}", diagram);
        diagram.try_into()
    }
}

impl TryFrom<plantuml::StateDiagram<'_>> for ParsedFsm {
    type Error = Error;
    fn try_from(diagram: plantuml::StateDiagram<'_>) -> Result<Self> {
        let name = diagram.name().map(|s| s.to_string()).unwrap_or_default();
        let mut builder = ParsedFsmBuilder::new(name);

        add_fsm_elements(&mut builder, diagram.elements(), None)?;

        builder.build()
    }
}

// TODO order matters here. there might be a mismatch on how plantuml processes this (line by line
// vs element by element), need to verify
fn add_fsm_elements(
    builder: &mut ParsedFsmBuilder,
    elements: &plantuml::StateElements<'_>,
    scope: Option<StateId>,
) -> Result<()> {
    let previous_scope = builder.set_scope(scope);

    for composite in &elements.composite_states {
        let state = builder.add_state(composite.name, StateType::Simple);
        add_fsm_elements(builder, &composite.elements, Some(state))?;
    }

    for enter_state in &elements.enter_states {
        builder.add_state(enter_state, StateType::Enter);
    }
    // Add transitions last, as they can create new states
    for transition in &elements.transitions {
        let ctx = context::TransitionContext::try_from(transition.description)?;
        builder.add_transition(transition.from, transition.to, ctx.event, ctx.action);
    }

    for desc in &elements.state_descriptions {
        if let Ok(ctx) = context::StateContext::try_from(desc.description) {
            if let Some(action) = ctx.enter_action {
                builder.set_state_enter_action(desc.name, action);
            }
            if let Some(action) = ctx.exit_action {
                builder.set_state_exit_action(desc.name, action);
            }
        }
    }

    builder.set_scope(previous_scope);
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{parser::ParsedFsm, test::FsmTestData};
    use pretty_assertions::assert_eq;
    use test_casing::{TestCases, cases, test_casing};

    const FSM_CASES: TestCases<FsmTestData> = cases!(FsmTestData::all());

    #[test_casing(7, FSM_CASES)]
    fn parses_fsm(data: FsmTestData) {
        crate::logging::init();
        let fsm = ParsedFsm::try_parse(data.content).unwrap();
        assert_eq!(data.parsed, fsm);
    }
}
