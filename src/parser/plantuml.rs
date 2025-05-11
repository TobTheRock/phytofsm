use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{
        complete::{tag, take_while},
        take_till, take_till1, take_until,
    },
    character::complete::{alphanumeric1, line_ending, multispace0, multispace1, space0},
    combinator::{map, opt},
    error::ParseError,
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
};

#[derive(Debug, PartialEq)]
struct State {
    name: String,
    // TODO nested  states -> create a state scop
}

#[derive(Debug, PartialEq)]
struct StateDescription {
    name: String,
    description: String,
}

#[derive(Debug, PartialEq)]
struct Transition {
    from: String,
    to: String,
    description: Option<String>,
}

#[derive(Debug, PartialEq)]
struct StateDiagram {
    name: String,
    states: Vec<State>,
    transitions: Vec<Transition>,
    initial_state: Option<String>,
    exit_state: Option<String>,
}

fn parse_puml_state_diagram(input: &str) -> IResult<&str, StateDiagram> {
    let (input, name) = parse_plantuml_start(input)?;
    // let (input, states) = separated_list0(multispace1, parse_state)(input)?;
    // let (input, _) = multispace0(input)?;
    // let (input, transitions) = separated_list0(multispace1, parse_transition)(input)?;
    let (input, _) = parse_plantuml_end(input)?;

    todo!();
    Ok((
        input,
        StateDiagram {
            name: name.to_string(),
            states: vec![],
            transitions: vec![],
            initial_state: None,
            exit_state: None,
        },
    ))
}

fn parse_plantuml_start(input: &str) -> IResult<&str, &str> {
    let start_tag = multi_ws(tag("@startuml"));
    delimited(
        start_tag,
        take_till1(|c| c == '\n' || c == '\r'),
        line_ending,
    )
    .parse(input)
}

fn parse_plantuml_end(input: &str) -> IResult<&str, &str> {
    multi_ws(tag("@enduml")).parse(input)
}

fn parse_state(input: &str) -> IResult<&str, State> {
    let (input, name) = delimited(
        ws(tag("state")),
        take_while(|c: char| c.is_alphanumeric()),
        line_ending,
    )
    .parse(input)?;

    let state = State {
        name: name.to_string(),
    };
    Ok((input, state))
}

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

fn parse_transition(input: &str) -> IResult<&str, Transition> {
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

    let transition = Transition {
        from: from.to_string(),
        to: to.to_string(),
        description: label.map(|s| s.to_string()),
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

fn ws<'a, O, E: ParseError<&'a str>, F>(inner: F) -> impl Parser<&'a str, Output = O, Error = E>
where
    F: Parser<&'a str, Output = O, Error = E>,
{
    delimited(space0, inner, space0)
}

fn multi_ws<'a, O, E: ParseError<&'a str>, F>(
    inner: F,
) -> impl Parser<&'a str, Output = O, Error = E>
where
    F: Parser<&'a str, Output = O, Error = E>,
{
    delimited(multispace0, inner, multispace0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uml_start() {
        let input = "@startuml fsm name\n";
        let (leftover, output) = parse_plantuml_start(input).unwrap();
        assert_eq!(output, "fsm name");
        assert_eq!(leftover, "");
    }

    #[test]
    fn test_parse_uml_end() {
        let input = "@enduml  \n";
        let (leftover, output) = parse_plantuml_end(input).unwrap();
        assert_eq!(output, "@enduml");
        assert_eq!(leftover, "");
    }

    #[test]
    fn test_parse_state() {
        let input = "state A\n";
        let (leftover, output) = parse_state(input).unwrap();
        assert_eq!(output.name, "A");
        assert_eq!(leftover, "");
    }

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
        assert_eq!(output.description, Some("label".to_string()));
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
}
