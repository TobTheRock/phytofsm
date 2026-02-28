use pest::Parser;
use pest_derive::Parser;

use crate::{
    error::{Error, Result},
    parser::{Action, Event},
};

#[derive(Parser)]
#[grammar = "parser/uml.pest"]
struct UmlParser;

#[derive(Clone, Debug)]
pub struct TransitionContext {
    pub action: Option<Action>,
    pub event: Event,
    pub guard: Option<Action>,
}

#[derive(Clone, Debug)]
pub struct StateContext {
    pub enter_action: Option<Action>,
    pub exit_action: Option<Action>,
}

impl TryFrom<&str> for TransitionContext {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self> {
        parse_transition_description(value)
    }
}

impl TryFrom<&String> for TransitionContext {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self> {
        Self::try_from(value.as_str())
    }
}

fn parse_transition_description(input: &str) -> Result<TransitionContext> {
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

    Ok(TransitionContext {
        event: event.ok_or_else(|| Error::Parse("Event name is required".to_string()))?,
        action,
        guard,
    })
}

impl TryFrom<&str> for StateContext {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self> {
        parse_state_description(value)
    }
}

impl TryFrom<&String> for StateContext {
    type Error = Error;
    fn try_from(value: &String) -> Result<Self> {
        Self::try_from(value.as_str())
    }
}

fn parse_state_description(input: &str) -> Result<StateContext> {
    let empty_context = StateContext {
        enter_action: None,
        exit_action: None,
    };

    let Ok(pairs) = UmlParser::parse(Rule::state_description, input) else {
        return Ok(empty_context);
    };

    let Some(action_pair) = pairs
        .flatten()
        .find(|p| p.as_rule() == Rule::entry_action || p.as_rule() == Rule::exit_action)
    else {
        return Ok(empty_context);
    };

    let rule = action_pair.as_rule();
    let action_name = action_pair
        .into_inner()
        .find(|p| p.as_rule() == Rule::action_name)
        .map(|p| p.as_str().to_owned().into());

    match rule {
        Rule::entry_action => Ok(StateContext {
            enter_action: action_name,
            exit_action: None,
        }),
        Rule::exit_action => Ok(StateContext {
            enter_action: None,
            exit_action: action_name,
        }),
        _ => Ok(empty_context),
    }
}

#[cfg(test)]
mod test {
    use crate::parser::context::StateContext;

    use super::TransitionContext;

    #[test]
    fn parse_event_only() {
        let desc = TransitionContext::try_from("   someEvent   ").unwrap();
        assert_eq!(desc.event, "someEvent".to_owned().into());
        assert_eq!(desc.action, None);
    }

    #[test]
    fn parse_event_and_action() {
        let desc = TransitionContext::try_from("   someEvent    / someAction").unwrap();
        assert_eq!(desc.event, "someEvent".to_owned().into());
        assert_eq!(desc.action, Some("someAction".to_owned().into()));
    }

    #[test]
    fn parse_invalid_input() {
        let result = TransitionContext::try_from("");
        assert!(result.is_err());

        // must have at least an event
        let result = TransitionContext::try_from("   / someAction");
        assert!(result.is_err());
    }

    #[test]
    fn parse_enter_action() {
        let desc = StateContext::try_from("entry / DoSomeThing").unwrap();
        assert_eq!(desc.enter_action, Some("DoSomeThing".to_owned().into()));
        assert_eq!(desc.exit_action, None);
    }

    #[test]
    fn parse_enter_action_with_extra_whitespace() {
        let desc = StateContext::try_from("   entry   /   DoSomeThing   ").unwrap();
        assert_eq!(desc.enter_action, Some("DoSomeThing".to_owned().into()));
        assert_eq!(desc.exit_action, None);
    }

    #[test]
    fn parse_exit_action() {
        let desc = StateContext::try_from("exit / DoSomeThing").unwrap();
        assert_eq!(desc.exit_action, Some("DoSomeThing".to_owned().into()));
        assert_eq!(desc.enter_action, None);
    }

    #[test]
    fn parse_exit_action_with_extra_whitespace() {
        let desc = StateContext::try_from("   exit   /   DoSomeThing   ").unwrap();
        assert_eq!(desc.exit_action, Some("DoSomeThing".to_owned().into()));
        assert_eq!(desc.enter_action, None);
    }

    #[test]
    fn parse_random_text_returns_empty_context() {
        // Random text is ignored, returns empty context
        let desc = StateContext::try_from("some random text").unwrap();
        assert_eq!(desc.enter_action, None);
        assert_eq!(desc.exit_action, None);
    }

    #[test]
    fn parse_empty_string_returns_empty_context() {
        let desc = StateContext::try_from("").unwrap();
        assert_eq!(desc.enter_action, None);
        assert_eq!(desc.exit_action, None);
    }

    #[test]
    fn parse_event_with_guard() {
        let desc = TransitionContext::try_from("ChangeState [AGuard]").unwrap();
        assert_eq!(desc.event, "ChangeState".to_owned().into());
        assert_eq!(desc.guard, Some("AGuard".to_owned().into()));
        assert_eq!(desc.action, None);
    }

    #[test]
    fn parse_event_with_guard_and_action() {
        let desc = TransitionContext::try_from("ChangeState [AGuard] / DoSomething").unwrap();
        assert_eq!(desc.event, "ChangeState".to_owned().into());
        assert_eq!(desc.guard, Some("AGuard".to_owned().into()));
        assert_eq!(desc.action, Some("DoSomething".to_owned().into()));
    }

    #[test]
    fn parse_event_with_guard_whitespace() {
        let desc =
            TransitionContext::try_from("  ChangeState  [  AGuard  ]  /  DoSomething  ").unwrap();
        assert_eq!(desc.event, "ChangeState".to_owned().into());
        assert_eq!(desc.guard, Some("AGuard".to_owned().into()));
        assert_eq!(desc.action, Some("DoSomething".to_owned().into()));
    }
}
