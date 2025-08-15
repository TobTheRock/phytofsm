use nom::{
    Err, Parser,
    branch::alt,
    bytes::{complete::tag, take_till1, take_until},
    character::complete::{alphanumeric1, line_ending, not_line_ending, space0, space1},
    combinator::{opt, recognize},
    error::{ParseError, context},
    multi::{many0, separated_list1},
    sequence::{delimited, preceded, separated_pair, terminated},
};
use nom_language::error::{VerboseError, convert_error};

use crate::{
    error::{Error, Result},
    parser::{
        FsmRepr, State, StateType, Transition,
        context::TransitionContext,
        nom::{NomResult, multi_ws, ws},
    },
};

fn format_verbose_parse_error(input: &str, error: Err<VerboseError<&str>>) -> String {
    match error {
        Err::Error(e) | Err::Failure(e) => convert_error(input, e),
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
            .map_err(|e| Error::ParseError(format_verbose_parse_error(input, e)))?;
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
        if (diagram.enter_states.len() != 1) {
            return Err(Error::ParseError(
                "FSM must have exactly one enter state".to_string(),
            ));
        }
        let enter_state = &diagram.enter_states[0];

        let transitions = diagram
            .transitions
            .into_iter()
            .map(|t| t.try_into_transition(&enter_state))
            .collect::<Result<Vec<Transition>>>()?;
        Ok(FsmRepr {
            name: diagram.name.unwrap_or_default(),
            transitions,
        })
    }
}

#[derive(Debug, PartialEq)]
enum FsmContent {
    StateDeclaration(String),
    StateDescription(StateDescription),
    EnterTransition(StateName),
    Transition(TransitionDescription),
    Unknown,
}

#[derive(Debug, PartialEq)]
struct FsmDiagram {
    name: Option<String>,
    enter_states: Vec<StateName>,
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

#[derive(Debug, PartialEq, Clone)]
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

fn parse_fsm_diagram(input: &str) -> NomResult<'_, FsmDiagram> {
    let (input, name) = context(
        "parsing PlantUML start tag (@startuml)",
        multi_ws(parse_plantuml_start),
    )
    .parse(input)?;
    let (remaining, content) = context(
        "parsing PlantUML content (between @startuml and @enduml)",
        multi_ws(parse_plantuml_content),
    )
    .parse(input)?;

    let (_, parsed_content) = context(
        "parsing all PlantUML content lines",
        many0(multi_ws(parse_fsm_content_line)),
    )
    .parse(content)?;

    let enter_states = parsed_content
        .iter()
        .filter_map(|item| {
            if let FsmContent::EnterTransition(state) = item {
                Some(state.clone())
            } else {
                None
            }
        })
        .collect();
    let transitions = parsed_content
        .iter()
        .filter_map(|item| {
            if let FsmContent::Transition(transition) = item {
                Some(transition.clone())
            } else {
                None
            }
        })
        .collect();

    let diagram = FsmDiagram {
        name: name.map(|s| s.to_string()),
        enter_states,
        transitions,
    };
    Ok((remaining, diagram))
}

fn parse_plantuml_content(input: &str) -> NomResult<'_, &str> {
    terminated(take_until("@enduml"), (tag("@enduml"), space0)).parse(input)
}

fn parse_plantuml_start(input: &str) -> NomResult<'_, Option<&str>> {
    let start_tag = tag("@startuml");
    let fsm_name = recognize(separated_list1(space1, alphanumeric1));
    delimited(start_tag, ws(opt(fsm_name)), line_ending).parse(input)
}

fn parse_state_declaration(input: &str) -> NomResult<'_, String> {
    let (input, name) =
        terminated(preceded(ws(tag("state")), ws(alphanumeric1)), line_ending).parse(input)?;
    Ok((input, name.to_string()))
}

fn parse_unknown_line(input: &str) -> NomResult<'_, ()> {
    let (input, _) = terminated(not_line_ending, line_ending).parse(input)?;
    Ok((input, ()))
}

fn parse_fsm_content_line(input: &str) -> NomResult<'_, FsmContent> {
    alt((
        |input| {
            parse_enter_transition(input)
                .map(|(rest, state)| (rest, FsmContent::EnterTransition(state.to_string())))
        },
        |input| parse_transition(input).map(|(rest, trans)| (rest, FsmContent::Transition(trans))),
        |input| {
            parse_state_declaration(input)
                .map(|(rest, name)| (rest, FsmContent::StateDeclaration(name)))
        },
        |input| {
            parse_state_description(input)
                .map(|(rest, desc)| (rest, FsmContent::StateDescription(desc)))
        },
        |input| parse_unknown_line(input).map(|(rest, _)| (rest, FsmContent::Unknown)),
    ))
    .parse(input)
}

fn parse_state_description(input: &str) -> NomResult<'_, StateDescription> {
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

fn parse_enter_transition(input: &str) -> NomResult<'_, &str> {
    let enter_tag = (space0, tag("[*]"), ws(parse_arrow));
    delimited(enter_tag, alphanumeric1, (space0, line_ending)).parse(input)
}

fn parse_exit_transition(input: &str) -> NomResult<'_, &str> {
    let exit_tag = (space0, tag("[*]"), ws(parse_arrow));
    delimited(exit_tag, alphanumeric1, (space0, line_ending)).parse(input)
}

fn parse_transition(input: &str) -> NomResult<'_, TransitionDescription> {
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

fn parse_arrow(input: &str) -> NomResult<'_, &str> {
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
        let (_, diagram) = parse_fsm_diagram(input).unwrap();
        assert_eq!(diagram.name, Some("fsm name".to_string()));
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
        assert_eq!(output.enter_states, ["A"]);
        assert_eq!(output.transitions.len(), 3);
    }
    #[test]
    fn test_parse_fsm_diagram_with_state_descriptions() {
        let input = r#"
        @startuml fsm name
        state A
        [*] --> A
        A --> B : label1
        state B: some desc
        @enduml
        "#;
        let (_, output) = parse_fsm_diagram(input).unwrap();
        assert_eq!(output.name, Some("fsm name".to_string()));
        assert_eq!(output.enter_states, ["A"]);
        assert_eq!(output.transitions.len(), 1);
    }
}
