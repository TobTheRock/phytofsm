use nom::{
    Err, Parser,
    branch::alt,
    bytes::{complete::tag, take_till1, take_until},
    character::complete::{alphanumeric1, line_ending, not_line_ending, space0, space1},
    combinator::{opt, recognize},
    error::context,
    multi::{many0, separated_list1},
    sequence::{delimited, preceded, separated_pair, terminated},
};
use nom_language::error::{VerboseError, convert_error};

use crate::{
    error::{Error, Result},
    parser::nom::{NomResult, multi_ws, ws},
};

fn format_verbose_parse_error(input: &str, error: Err<VerboseError<&str>>) -> String {
    match error {
        Err::Error(e) | Err::Failure(e) => convert_error(input, e),
        Err::Incomplete(_) => "Incomplete input - more data needed".to_string(),
    }
}

pub type StateName<'a> = &'a str;

#[derive(Debug, PartialEq)]
enum StateDiagramElement<'a> {
    StateDeclaration(&'a str),
    StateDescription(StateDescription<'a>),
    EnterTransition(StateName<'a>),
    Transition(TransitionDescription<'a>),
    Unknown,
}

#[derive(Debug, PartialEq)]
pub struct StateDiagram<'a> {
    name: Option<&'a str>,
    enter_states: Vec<StateName<'a>>,
    //TODO support exit state
    // exit_state: Option<StateName>,
    transitions: Vec<TransitionDescription<'a>>,
    // TODO support nested states
    // state_contexts: Vec<StateComposite>,
}

impl StateDiagram<'_> {
    pub fn parse(input: &str) -> Result<StateDiagram<'_>> {
        let (_, fsm_diagram) = parse_fsm_diagram(input)
            .map_err(|e| Error::Parse(format_verbose_parse_error(input, e)))?;
        Ok(fsm_diagram)
    }

    pub fn enter_states(&self) -> impl Iterator<Item = &StateName<'_>> {
        self.enter_states.iter()
    }
    pub fn transitions(&self) -> impl Iterator<Item = &TransitionDescription<'_>> {
        self.transitions.iter()
    }
    pub fn name(&self) -> Option<&str> {
        self.name
    }
}

// needed later
#[derive(Debug, PartialEq)]
struct StateDescription<'a> {
    name: StateName<'a>,
    description: &'a str,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TransitionDescription<'a> {
    pub from: StateName<'a>,
    pub to: StateName<'a>,
    // TODO make this optional for direct transitions
    pub description: &'a str,
}

fn parse_fsm_diagram(input: &str) -> NomResult<'_, StateDiagram<'_>> {
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

    let (_, elements) = context(
        "parsing all PlantUML content lines",
        many0(multi_ws(parse_state_diagram_elements)),
    )
    .parse(content)?;

    let (enter_states, transitions) = elements.into_iter().fold(
        (vec![], vec![]),
        |(mut enter_states, mut transitions), element| {
            match element {
                StateDiagramElement::EnterTransition(state) => {
                    enter_states.push(state);
                }
                StateDiagramElement::Transition(transition) => {
                    transitions.push(transition);
                }
                _ => {}
            }
            (enter_states, transitions)
        },
    );

    let diagram = StateDiagram {
        name,
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

fn parse_state_declaration(input: &str) -> NomResult<'_, &str> {
    let (input, name) =
        terminated(preceded(ws(tag("state")), ws(alphanumeric1)), line_ending).parse(input)?;
    Ok((input, name))
}

fn parse_unknown_line(input: &str) -> NomResult<'_, ()> {
    let (input, _) = terminated(not_line_ending, line_ending).parse(input)?;
    Ok((input, ()))
}

fn parse_state_diagram_elements(input: &str) -> NomResult<'_, StateDiagramElement<'_>> {
    alt((
        |input| {
            parse_enter_transition(input)
                .map(|(rest, state)| (rest, StateDiagramElement::EnterTransition(state)))
        },
        |input| {
            parse_transition(input)
                .map(|(rest, trans)| (rest, StateDiagramElement::Transition(trans)))
        },
        |input| {
            parse_state_declaration(input)
                .map(|(rest, name)| (rest, StateDiagramElement::StateDeclaration(name)))
        },
        |input| {
            parse_state_description(input)
                .map(|(rest, desc)| (rest, StateDiagramElement::StateDescription(desc)))
        },
        |input| parse_unknown_line(input).map(|(rest, _)| (rest, StateDiagramElement::Unknown)),
    ))
    .parse(input)
}

fn parse_state_description(input: &str) -> NomResult<'_, StateDescription<'_>> {
    let state_name_parser = ws(alphanumeric1);
    let description_parser = delimited(space0, take_till1(|c| c == '\n' || c == '\r'), line_ending);
    let (input, (name, description)) =
        separated_pair(state_name_parser, tag(":"), description_parser).parse(input)?;

    let state_desc = StateDescription { name, description };
    Ok((input, state_desc))
}

fn parse_enter_transition(input: &str) -> NomResult<'_, &str> {
    let enter_tag = (space0, tag("[*]"), ws(parse_arrow));
    delimited(enter_tag, alphanumeric1, (space0, line_ending)).parse(input)
}

// TODO reenable when exit states are supported
#[cfg(test)]
fn parse_exit_transition(input: &str) -> NomResult<'_, &str> {
    let exit_tag = (space0, tag("[*]"), ws(parse_arrow));
    delimited(exit_tag, alphanumeric1, (space0, line_ending)).parse(input)
}

fn parse_transition(input: &str) -> NomResult<'_, TransitionDescription<'_>> {
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
        from,
        to,
        description: label.unwrap_or(""),
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
    use crate::test::FsmTestData;

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
        assert_eq!(output.description, "label");
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
        assert_eq!(diagram.name, Some("fsm name"));
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
        assert_eq!(output.name, Some("fsm name"));
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
        assert_eq!(output.name, Some("fsm name"));
        assert_eq!(output.enter_states, ["A"]);
        assert_eq!(output.transitions.len(), 1);
    }

    #[test]
    fn test_parse_test_data() {
        let test_data = FsmTestData::four_seasons();
        let input = test_data.content;
        let (_, output) = parse_fsm_diagram(input).unwrap();

        let expected = test_data.parsed;
        assert_eq!(output.name, Some(expected.name()));
        // TODO check entry state
        assert_eq!(output.transitions.len(), expected.transitions().count());
    }
}
