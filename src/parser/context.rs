use nom::{
    IResult, Parser,
    character::complete::{alphanumeric1, char},
    combinator::opt,
    sequence::preceded,
};

use crate::{
    error::Error,
    parser::{Action, Event, nom::ws},
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
            .map(|(_, desc)| desc)
            .map_err(|e| Error::ParseError(e.to_string()))
    }
}
impl TryFrom<&String> for TransitionContext {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

pub fn parse_transaction_description(input: &str) -> IResult<&str, TransitionContext> {
    let event = ws(alphanumeric1);
    let action = preceded(char('/'), ws(alphanumeric1));
    let (input, (event, action)) = (event, opt(action)).parse(input)?;

    Ok((
        input,
        TransitionContext {
            action: action.map(|a| a.to_owned().into()),
            event: event.to_owned().into(),
        },
    ))
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
