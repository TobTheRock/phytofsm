use regex::Regex;

use crate::{
    error::{Error, Result},
    parser::{Action, Event},
};

#[derive(Clone, Debug)]
pub struct TransitionContext {
    pub action: Option<Action>,
    pub event: Event,
}

#[derive(Clone, Debug)]
pub struct StateContext {
    pub enter_action: Option<Action>,
    pub exit_action: Option<Action>,
}

impl TryFrom<&str> for TransitionContext {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self> {
        parse_transaction_description(value)
    }
}

impl TryFrom<&String> for TransitionContext {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self> {
        Self::try_from(value.as_str())
    }
}

const ALPHA_NUMERIC: &str = "[a-zA-Z0-9]+";

fn parse_transaction_description(input: &str) -> Result<TransitionContext> {
    let pattern = format!(r"^\s*({ALPHA_NUMERIC})\s*(?:/\s*({ALPHA_NUMERIC}))?\s*$");
    let re = Regex::new(&pattern).expect("Invalid regex pattern");

    let captures = re
        .captures(input)
        .ok_or_else(|| Error::Parse(format!("Invalid transition description: '{}'", input)))?;

    let event = captures
        .get(1)
        .ok_or_else(|| Error::Parse("Event name is required".to_string()))?
        .as_str()
        .to_owned()
        .into();

    let action = captures.get(2).map(|m| m.as_str().to_owned().into());

    Ok(TransitionContext { action, event })
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
    let enter_pattern = r"^\s*>\s*([a-zA-Z0-9]+)\s*$";
    let exit_pattern = r"^\s*<\s*([a-zA-Z0-9]+)\s*$";
    let enter_re = Regex::new(enter_pattern).expect("Invalid regex pattern");
    let exit_re = Regex::new(exit_pattern).expect("Invalid regex pattern");
    if let Some(captures) = enter_re.captures(input) {
        let enter_action = captures.get(1).map(|m| m.as_str().to_owned().into());
        return Ok(StateContext {
            enter_action,
            exit_action: None,
        });
    }
    if let Some(captures) = exit_re.captures(input) {
        let exit_action = captures.get(1).map(|m| m.as_str().to_owned().into());
        return Ok(StateContext {
            enter_action: None,
            exit_action,
        });
    }
    Err(Error::Parse(format!(
        "Invalid state description: '{}'",
        input
    )))
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
        let desc = StateContext::try_from("> doSomeThing").unwrap();
        assert_eq!(desc.enter_action, Some("doSomeThing".to_owned().into()));
        assert_eq!(desc.exit_action, None);
    }

    #[test]
    fn parse_exit_action() {
        let desc = StateContext::try_from("< doSomeThing").unwrap();
        assert_eq!(desc.exit_action, Some("doSomeThing".to_owned().into()));
        assert_eq!(desc.enter_action, None);
    }
}
