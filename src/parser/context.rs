use regex::Regex;

use crate::{
    error::Error,
    parser::{Action, Event},
};

#[derive(Clone, Debug)]
pub struct TransitionContext {
    pub action: Option<Action>,
    pub event: Event,
}

impl TryFrom<&str> for TransitionContext {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_transaction_description(value)
    }
}

impl TryFrom<&String> for TransitionContext {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

pub fn parse_transaction_description(input: &str) -> Result<TransitionContext, Error> {
    let alpha_numeric = "[a-zA-Z0-9]+";
    let pattern = format!(r"^\s*({alpha_numeric})\s*(?:/\s*({alpha_numeric}))?\s*$");
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

#[cfg(test)]
mod test {
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
}
