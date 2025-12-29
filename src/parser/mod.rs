use crate::error::{Error, Result};

mod builder;
mod context;
mod fsm;
mod plantuml;
mod types;

pub use builder::ParsedFsmBuilder;
pub use fsm::{ParsedFsm, State, Transition};
use itertools::Itertools;
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

        for enter_state in diagram.enter_states() {
            builder.add_state(enter_state, StateType::Enter);
        }
        for transition in diagram.transitions() {
            let ctx = context::TransitionContext::try_from(transition.description)?;
            builder.add_transition(transition.from, transition.to, ctx.event, ctx.action);
        }

        add_composite_states(&mut builder, &diagram)?;

        builder.build()
    }
}

fn add_composite_states(
    builder: &mut ParsedFsmBuilder,
    diagram: &plantuml::StateDiagram<'_>,
) -> Result<()> {
    let mut queue = diagram.composite_states().map(|c| (None, c)).collect_vec();
    while let Some((parent, composite)) = queue.pop() {
        builder.set_scope(parent);
        let state_id = builder.add_state(composite.name, StateType::Simple);

        builder.set_scope(Some(state_id));

        for enter_state in &composite.enter_states {
            builder.add_state(enter_state, StateType::Enter);
        }

        for transition in &composite.transitions {
            let ctx = context::TransitionContext::try_from(transition.description)?;
            builder.add_transition(transition.from, transition.to, ctx.event, ctx.action);
        }

        for child in &composite.children {
            queue.push((Some(state_id), child));
        }
    }
    builder.set_scope(None);
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{parser::ParsedFsm, test::FsmTestData};
    use test_casing::{cases, test_casing, TestCases};

    const FSM_CASES: TestCases<FsmTestData> = cases!(FsmTestData::all());

    #[test_casing(4, FSM_CASES)]
    fn parses_fsm(data: FsmTestData) {
        let fsm = ParsedFsm::try_parse(data.content)
            .expect(&format!("Failed to parse FSM for test data: {}", data.name));
        assert_eq!(data.parsed, fsm);
    }
}
