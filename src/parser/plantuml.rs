use std::collections::HashSet;

use nom::{
    Err, IResult, Parser,
    branch::alt,
    bytes::{complete::tag, take_till1, take_until},
    character::complete::{alphanumeric0, alphanumeric1, line_ending, multispace0, space0},
    combinator::{cut, opt},
    error::context,
    multi::many0,
    sequence::{delimited, preceded, separated_pair, terminated},
};
use syn::TraitItem;

use crate::{
    error::{Error, Result},
    parser::{
        FsmRepr, State, StateType, Transition,
        context::TransitionContext,
        nom::{multi_ws, ws},
    },
};

fn format_parse_error<E>(_input: &str, error: Err<E>) -> String
where
    E: std::fmt::Debug,
{
    match error {
        Err::Error(e) => format!("Parse error: {:?}", e),
        Err::Failure(e) => format!("Parse failure: {:?}", e),
        Err::Incomplete(_) => "Incomplete input - more data needed".to_string(),
    }
}

pub struct PlantUmlFsmParser {
    // name: String,
    // states: HashSet<StateDescription>,
    // transitions: Vec<Transition>,
}

impl PlantUmlFsmParser {
    // TODO pass overwrites? e.g name
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, input: &str) -> Result<FsmRepr> {
        let (_, fsm_diagram) = parse_fsm_diagram(input)
            .map_err(|e| Error::ParseError(format_parse_error(input, e)))?;
        fsm_diagram.try_into()
    }
}

type StateName = String;

impl State {
    fn from(name: &StateName, enter_state: &StateName) -> Self {
        let state_type = if name == enter_state {
            StateType::Enter
        } else {
            StateType::Simple
        };

        Self {
            name: name.to_string(),
            state_type,
        }
    }
}

impl TryFrom<FsmDiagram> for FsmRepr {
    type Error = Error;
    fn try_from(diagram: FsmDiagram) -> Result<Self> {
        let mut transitions = diagram
            .transitions
            .into_iter()
            .map(|t| t.try_into_transition(&diagram.enter_state))
            .collect::<Result<Vec<Transition>>>()?;
        Ok(FsmRepr {
            name: diagram.name.unwrap_or_default(),
            transitions,
        })
    }
}

#[derive(Debug, PartialEq)]
struct FsmDiagram {
    name: Option<String>,
    enter_state: StateName,
    //TODO support exit state
    // exit_state: Option<StateName>,
    transitions: Vec<TransitionDescription>,
    // TODO support nested states
    // state_contexts: Vec<StateComposite>,
}

#[derive(Debug, PartialEq)]
struct StateComposite {
    name: StateName,
    enter_state: Option<StateName>,
    transitions: Vec<TransitionDescription>,
}

// needed later
#[derive(Debug, PartialEq)]
struct StateDescription {
    name: StateName,
    description: String,
}

#[derive(Debug, PartialEq)]
struct TransitionDescription {
    from: StateName,
    to: StateName,
    // TODO make this optional for direct transitions
    description: String,
}

impl TransitionDescription {
    fn try_into_transition(self, enter_state: &StateName) -> Result<Transition> {
        let description = TransitionContext::try_from(&self.description)?;
        let source = State::from(&self.from, enter_state);
        let desination = State::from(&self.to, enter_state);

        Ok(Transition {
            source,
            destination: desination,
            event: description.event,
            action: description.action,
        })
    }
}

fn parse_fsm_diagram(input: &str) -> IResult<&str, FsmDiagram> {
    let (input, name) =
        context("PlantUML start tag", multi_ws(parse_plantuml_start)).parse(input)?;
    let (remaining, content) =
        context("PlantUML content", multi_ws(parse_plantuml_content)).parse(input)?;

    let (content_remaining, enter_state) = context(
        "initial state transition ([*] --> State)",
        parse_enter_transition,
    )
    .parse(content)?;
    let single_transition = context("state transition", parse_transition);
    let (_, transitions) = context("all state transitions", many0(multi_ws(single_transition)))
        .parse(content_remaining)?;

    let diagram = FsmDiagram {
        name: name.map(|s| s.to_string()),
        enter_state: enter_state.to_string(),
        transitions,
    };
    Ok((remaining, diagram))
}

fn parse_plantuml_content(input: &str) -> IResult<&str, &str> {
    terminated(take_until("@enduml"), (tag("@enduml"), space0)).parse(input)
}

fn parse_plantuml_start(input: &str) -> IResult<&str, Option<&str>> {
    let start_tag = tag("@startuml");
    delimited(
        start_tag,
        ws(opt(take_till1(|c| c == '\n' || c == '\r'))),
        line_ending,
    )
    .parse(input)
}

// TODO this will be needed later for nested states
// fn parse_state(input: &str) -> IResult<&str, State> {
//     let (input, name) = delimited(
//         ws(tag("state")),
//         take_while(|c: char| c.is_alphanumeric()), line_ending,) .parse(input)?; let state = State { name: name.to_string(), };
//     Ok((input, state))
// }

fn parse_state_description(input: &str) -> IResult<&str, StateDescription> {
    let state_name_parser = ws(alphanumeric1);
    let description_parser = delimited(space0, take_till1(|c| c == '\n' || c == '\r'), line_ending);
    let (input, (name, description)) =
        separated_pair(state_name_parser, tag(":"), description_parser).parse(input)?;

    let state_desc = StateDescription {
        name: name.to_string(),
        description: description.to_string(),
    };
    Ok((input, state_desc))
}

fn parse_enter_transition(input: &str) -> IResult<&str, &str> {
    let enter_tag = (space0, tag("[*]"), ws(parse_arrow));
    delimited(enter_tag, alphanumeric1, (space0, line_ending)).parse(input)
}

fn parse_exit_transition(input: &str) -> IResult<&str, &str> {
    let exit_tag = (space0, tag("[*]"), ws(parse_arrow));
    delimited(exit_tag, alphanumeric1, (space0, line_ending)).parse(input)
}

fn parse_transition(input: &str) -> IResult<&str, TransitionDescription> {
    let label_parser = preceded(ws(tag(":")), preceded(space0, take_until("\n")));
    let (input, (from, _, to, label)) = terminated(
        (
            ws(alphanumeric1),
            parse_arrow,
            ws(alphanumeric1),
            opt(label_parser),
        ),
        line_ending,
    )
    .parse(input)?;

    let transition = TransitionDescription {
        from: from.to_string(),
        to: to.to_string(),
        description: label.map_or("".to_string(), |s| s.to_string()),
    };
    Ok((input, transition))
}

fn parse_arrow(input: &str) -> IResult<&str, &str> {
    alt((
        tag("->"),
        tag("-->"),
        tag("-u->"),
        tag("-d->"),
        tag("-l->"),
        tag("-r->"),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uml_start() {
        let input = "@startuml        \n";
        let (leftover, output) = parse_plantuml_start(input).unwrap();
        assert_eq!(output, None);
        assert_eq!(leftover, "");
    }

    #[test]
    fn test_parse_uml_start_with_name() {
        let input = "@startuml fsm name\n";
        let (leftover, output) = parse_plantuml_start(input).unwrap();
        assert_eq!(output, Some("fsm name"));
        assert_eq!(leftover, "");
    }

    #[test]
    fn test_parse_uml_content() {
        let input = "blabla @enduml  ";
        let (leftover, output) = parse_plantuml_content(input).unwrap();
        assert_eq!(output, "blabla ");
        assert_eq!(leftover, "");
    }

    // #[test]
    // fn test_parse_state() {
    //     let input = "state A\n";
    //     let (leftover, output) = parse_state(input).unwrap();
    //     assert_eq!(output.name, "A");
    //     assert_eq!(leftover, "");
    // }

    #[test]
    fn test_parse_state_description() {
        let input = "A : some description\n";
        let (leftover, output) = parse_state_description(input).unwrap();
        assert_eq!(output.name, "A");
        assert_eq!(output.description, "some description");
        assert_eq!(leftover, "");
    }

    #[test]
    fn test_parse_transition() {
        let input = "A -> B : label\n";
        let (leftover, output) = parse_transition(input).unwrap();
        assert_eq!(output.from, "A");
        assert_eq!(output.to, "B");
        assert_eq!(output.description, "label".to_string());
        assert_eq!(leftover, "");
    }

    #[test]
    fn test_parse_multiple_transitions() {
        let input = r#"
        A --> B : label1
        A -u-> C : label2
        "#;
        let (_, output) = many0(multi_ws(parse_transition)).parse(input).unwrap();
        assert_eq!(output.len(), 2);
    }

    #[test]
    fn test_parse_enter_transition() {
        let input = "[*] --> A\n";
        let (leftover, output) = parse_enter_transition(input).unwrap();
        assert_eq!(output, "A");
        assert_eq!(leftover, "");
    }

    #[test]
    fn test_parse_exit_transition() {
        let input = "[*] --> A\n";
        let (leftover, output) = parse_exit_transition(input).unwrap();
        assert_eq!(output, "A");
        assert_eq!(leftover, "");
    }

    #[test]
    fn test_parse_empty_fsm_diagram() {
        let input = r#"
        @startuml fsm name
        @enduml
        "#;
        let result = parse_fsm_diagram(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_fsm_diagram() {
        let input = r#"
        @startuml fsm name
        [*] --> A
        A --> B : label1
        A -u-> C : label2
        B --> D : label3
        @enduml
        "#;
        let (_, output) = parse_fsm_diagram(input).unwrap();
        assert_eq!(output.name, Some("fsm name".to_string()));
        assert_eq!(output.enter_state, "A".to_string());
        assert_eq!(output.transitions.len(), 3);
    }
}
