use pest::Parser;
use pest_derive::Parser;

use crate::{
    error::{Error, Result},
    parser::{Action, Event},
};

#[derive(Parser)]
#[grammar = "parser/uml.pest"]
struct UmlParser;

#[derive(Clone, Debug, PartialEq)]
pub struct TransitionLabel {
    pub action: Option<Action>,
    pub event: Event,
    pub guard: Option<Action>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StateDescription {
    Entry(Action),
    Exit(Action),
    InternalTransition(TransitionLabel),
}

impl TryFrom<&str> for TransitionLabel {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self> {
        parse_transition_description(value)
    }
}

impl TryFrom<&String> for TransitionLabel {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self> {
        Self::try_from(value.as_str())
    }
}

fn parse_transition_description(input: &str) -> Result<TransitionLabel> {
    let pairs = UmlParser::parse(Rule::transition_description, input)
        .map_err(|e| Error::Parse(format!("Invalid transition description: {}", e)))?;

    let mut event = None;
    let mut action = None;
    let mut guard = None;

    for pair in pairs {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::event_name => {
                    event = Some(inner.as_str().to_owned().into());
                }
                Rule::guard_name => {
                    guard = Some(inner.as_str().to_owned().into());
                }
                Rule::action_name => {
                    action = Some(inner.as_str().to_owned().into());
                }
                _ => {}
            }
        }
    }

    Ok(TransitionLabel {
        event: event.ok_or_else(|| Error::Parse("Event name is required".to_string()))?,
        action,
        guard,
    })
}

impl TryFrom<&str> for StateDescription {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self> {
        parse_state_description(value)
    }
}

impl TryFrom<&String> for StateDescription {
    type Error = Error;
    fn try_from(value: &String) -> Result<Self> {
        Self::try_from(value.as_str())
    }
}

fn parse_state_description(input: &str) -> Result<StateDescription> {
    let pairs = UmlParser::parse(Rule::state_description, input)
        .map_err(|e| Error::Parse(format!("Invalid state description: {}", e)))?;

    let flat: Vec<_> = pairs.flatten().collect();

    try_parse_state_action(&flat)
        .or_else(|| try_parse_internal_transition(&flat))
        .ok_or_else(|| Error::Parse(format!("Unrecognised state description: {}", input)))?
}

fn try_parse_state_action(
    pairs: &[pest::iterators::Pair<Rule>],
) -> Option<Result<StateDescription>> {
    let action_pair = pairs
        .iter()
        .find(|p| p.as_rule() == Rule::entry_action || p.as_rule() == Rule::exit_action)?;

    let rule = action_pair.as_rule();
    let action = action_pair
        .clone()
        .into_inner()
        .find(|p| p.as_rule() == Rule::action_name)
        .map(|p| p.as_str().to_owned().into());

    let Some(action) = action else {
        return Some(Err(Error::Parse("Action name is required".to_string())));
    };

    Some(match rule {
        Rule::entry_action => Ok(StateDescription::Entry(action)),
        Rule::exit_action => Ok(StateDescription::Exit(action)),
        _ => unreachable!(),
    })
}

fn try_parse_internal_transition(
    pairs: &[pest::iterators::Pair<Rule>],
) -> Option<Result<StateDescription>> {
    let event_pair = pairs.iter().find(|p| p.as_rule() == Rule::event_name)?;

    let event = event_pair.as_str().to_owned().into();
    let guard = pairs
        .iter()
        .find(|p| p.as_rule() == Rule::guard_name)
        .map(|p| p.as_str().to_owned().into());
    let action = pairs
        .iter()
        .find(|p| p.as_rule() == Rule::action_name)
        .map(|p| p.as_str().to_owned().into());

    Some(Ok(StateDescription::InternalTransition(TransitionLabel {
        event,
        action,
        guard,
    })))
}

#[cfg(test)]
mod test {
    use super::{StateDescription, TransitionLabel};

    #[test]
    fn parse_event_only() {
        let desc = TransitionLabel::try_from("   someEvent   ").unwrap();
        assert_eq!(desc.event, "someEvent".to_owned().into());
        assert_eq!(desc.action, None);
    }

    #[test]
    fn parse_event_and_action() {
        let desc = TransitionLabel::try_from("   someEvent    / someAction").unwrap();
        assert_eq!(desc.event, "someEvent".to_owned().into());
        assert_eq!(desc.action, Some("someAction".to_owned().into()));
    }

    #[test]
    fn parse_invalid_input() {
        let result = TransitionLabel::try_from("");
        assert!(result.is_err());

        // must have at least an event
        let result = TransitionLabel::try_from("   / someAction");
        assert!(result.is_err());
    }

    #[test]
    fn parse_enter_action() {
        let desc = StateDescription::try_from("entry / DoSomeThing").unwrap();
        assert_eq!(
            desc,
            StateDescription::Entry("DoSomeThing".to_owned().into())
        );
    }

    #[test]
    fn parse_enter_action_with_extra_whitespace() {
        let desc = StateDescription::try_from("   entry   /   DoSomeThing   ").unwrap();
        assert_eq!(
            desc,
            StateDescription::Entry("DoSomeThing".to_owned().into())
        );
    }

    #[test]
    fn parse_exit_action() {
        let desc = StateDescription::try_from("exit / DoSomeThing").unwrap();
        assert_eq!(
            desc,
            StateDescription::Exit("DoSomeThing".to_owned().into())
        );
    }

    #[test]
    fn parse_exit_action_with_extra_whitespace() {
        let desc = StateDescription::try_from("   exit   /   DoSomeThing   ").unwrap();
        assert_eq!(
            desc,
            StateDescription::Exit("DoSomeThing".to_owned().into())
        );
    }

    #[test]
    fn parse_random_text_returns_err() {
        let result = StateDescription::try_from("some random text");
        assert!(result.is_err());
    }

    #[test]
    fn parse_empty_string_returns_err() {
        let result = StateDescription::try_from("");
        assert!(result.is_err());
    }

    #[test]
    fn parse_internal_transition() {
        let desc = StateDescription::try_from("SomeEvent [AGuard] / DoSomething").unwrap();
        assert_eq!(
            desc,
            StateDescription::InternalTransition(TransitionLabel {
                event: "SomeEvent".to_owned().into(),
                guard: Some("AGuard".to_owned().into()),
                action: Some("DoSomething".to_owned().into()),
            })
        );
    }

    #[test]
    fn parse_event_with_guard() {
        let desc = TransitionLabel::try_from("ChangeState [AGuard]").unwrap();
        assert_eq!(desc.event, "ChangeState".to_owned().into());
        assert_eq!(desc.guard, Some("AGuard".to_owned().into()));
        assert_eq!(desc.action, None);
    }

    #[test]
    fn parse_event_with_guard_and_action() {
        let desc = TransitionLabel::try_from("ChangeState [AGuard] / DoSomething").unwrap();
        assert_eq!(desc.event, "ChangeState".to_owned().into());
        assert_eq!(desc.guard, Some("AGuard".to_owned().into()));
        assert_eq!(desc.action, Some("DoSomething".to_owned().into()));
    }

    #[test]
    fn parse_event_with_guard_whitespace() {
        let desc =
            TransitionLabel::try_from("  ChangeState  [  AGuard  ]  /  DoSomething  ").unwrap();
        assert_eq!(desc.event, "ChangeState".to_owned().into());
        assert_eq!(desc.guard, Some("AGuard".to_owned().into()));
        assert_eq!(desc.action, Some("DoSomething".to_owned().into()));
    }
}
