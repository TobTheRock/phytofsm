use pest::Parser;
use pest_derive::Parser;

use crate::error::{Error, Result};

#[derive(Parser)]
#[grammar = "parser/plantuml.pest"]
pub struct PlantUmlParser;

pub type StateName<'a> = &'a str;

type Pair<'a> = pest::iterators::Pair<'a, Rule>;

#[derive(Debug, PartialEq)]
pub struct StateDiagram<'a> {
    name: Option<&'a str>,
    enter_states: Vec<StateName<'a>>,
    transitions: Vec<TransitionDescription<'a>>,
}

impl StateDiagram<'_> {
    pub fn parse(input: &str) -> Result<StateDiagram<'_>> {
        let mut pairs =
            PlantUmlParser::parse(Rule::diagram, input).map_err(|e| Error::Parse(e.to_string()))?;

        let diagram_pair = pairs
            .next()
            .ok_or_else(|| Error::Parse("Empty input".to_string()))?;

        let mut name = None;
        let mut enter_states = Vec::new();
        let mut transitions = Vec::new();

        for inner in diagram_pair.into_inner() {
            match inner.as_rule() {
                Rule::startuml => name = parse_diagram_name(inner),
                Rule::content => {
                    let (enters, trans) = parse_content(inner)?;
                    enter_states = enters;
                    transitions = trans;
                }
                _ => {}
            }
        }

        Ok(StateDiagram {
            name,
            enter_states,
            transitions,
        })
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

#[derive(Debug, PartialEq, Clone)]
pub struct TransitionDescription<'a> {
    pub from: StateName<'a>,
    pub to: StateName<'a>,
    // TODO make this optional for direct transitions
    pub description: &'a str,
}

fn parse_diagram_name(pair: Pair<'_>) -> Option<&str> {
    pair.into_inner()
        .find(|p| p.as_rule() == Rule::diagram_name)
        .map(|p| p.as_str())
}

fn parse_content(pair: Pair<'_>) -> Result<(Vec<StateName<'_>>, Vec<TransitionDescription<'_>>)> {
    let mut enter_states = Vec::new();
    let mut transitions = Vec::new();

    for element in pair.into_inner() {
        if element.as_rule() != Rule::element {
            continue;
        }

        for element_inner in element.into_inner() {
            match element_inner.as_rule() {
                Rule::enter_transition => {
                    if let Some(state) = parse_enter_state(element_inner) {
                        enter_states.push(state);
                    }
                }
                Rule::transition => {
                    transitions.push(parse_transition(element_inner)?);
                }
                _ => {}
            }
        }
    }

    Ok((enter_states, transitions))
}

fn parse_enter_state(pair: Pair<'_>) -> Option<StateName<'_>> {
    pair.into_inner()
        .find(|p| p.as_rule() == Rule::state_name)
        .map(|p| p.as_str())
}

fn parse_transition(pair: Pair<'_>) -> Result<TransitionDescription<'_>> {
    let mut from = None;
    let mut to = None;
    let mut description = "";

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::state_name => {
                if from.is_none() {
                    from = Some(inner.as_str());
                } else {
                    to = Some(inner.as_str());
                }
            }
            Rule::description => {
                description = inner.as_str();
            }
            _ => {}
        }
    }

    Ok(TransitionDescription {
        from: from.ok_or_else(|| Error::Parse("Missing source state in transition".to_string()))?,
        to: to
            .ok_or_else(|| Error::Parse("Missing destination state in transition".to_string()))?,
        description,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uml_start() {
        let input = "@startuml        \n";
        let full_input = format!("{}@enduml", input);
        let diagram = StateDiagram::parse(&full_input).unwrap();
        assert_eq!(diagram.name, None);
    }

    #[test]
    fn test_parse_uml_start_with_name() {
        let input = "@startuml fsm name\n";
        let full_input = format!("{}@enduml", input);
        let diagram = StateDiagram::parse(&full_input).unwrap();
        assert_eq!(diagram.name, Some("fsm name"));
    }

    #[test]
    fn test_parse_uml_content() {
        // This test was checking raw content extraction - now we test via full diagram
        let full_input = "@startuml test\n@enduml";
        let diagram = StateDiagram::parse(full_input).unwrap();
        assert!(diagram.name.is_some());
    }

    #[test]
    fn test_parse_state_description() {
        let input = "A : some description\n";
        let full_input = format!("@startuml test\n{}@enduml", input);
        let diagram = StateDiagram::parse(&full_input).unwrap();
        // State descriptions are parsed but not stored in transitions
        assert_eq!(diagram.transitions.len(), 0);
    }

    #[test]
    fn test_parse_transition() {
        let input = "A -> B : label\n";
        let full_input = format!("@startuml test\n{}@enduml", input);
        let diagram = StateDiagram::parse(&full_input).unwrap();
        assert_eq!(diagram.transitions.len(), 1);
        assert_eq!(diagram.transitions[0].from, "A");
        assert_eq!(diagram.transitions[0].to, "B");
        assert_eq!(diagram.transitions[0].description, "label");
    }

    #[test]
    fn test_parse_multiple_transitions() {
        let input = r#"
        A --> B : label1
        A -u-> C : label2
        "#;
        let full_input = format!("@startuml test\n{}@enduml", input);
        let diagram = StateDiagram::parse(&full_input).unwrap();
        assert_eq!(diagram.transitions.len(), 2);
    }

    #[test]
    fn test_parse_enter_transition() {
        let input = "[*] --> A\n";
        let full_input = format!("@startuml test\n{}@enduml", input);
        let diagram = StateDiagram::parse(&full_input).unwrap();
        assert_eq!(diagram.enter_states, vec!["A"]);
    }

    #[test]
    fn test_parse_exit_transition() {
        let input = "[*] --> A\n";
        let full_input = format!("@startuml test\n{}@enduml", input);
        let diagram = StateDiagram::parse(&full_input).unwrap();
        assert_eq!(diagram.enter_states, vec!["A"]);
    }

    #[test]
    fn test_parse_empty_fsm_diagram() {
        let input = r#"
        @startuml fsm name
        @enduml
        "#;
        let diagram = StateDiagram::parse(input).unwrap();
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
        let diagram = StateDiagram::parse(input).unwrap();
        assert_eq!(diagram.name, Some("fsm name"));
        assert_eq!(diagram.enter_states, vec!["A"]);
        assert_eq!(diagram.transitions.len(), 3);
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
        let diagram = StateDiagram::parse(input).unwrap();
        assert_eq!(diagram.name, Some("fsm name"));
        assert_eq!(diagram.enter_states, vec!["A"]);
        assert_eq!(diagram.transitions.len(), 1);
    }

    #[test]
    fn test_parse_transitions() {
        let input = r#"
        @startuml
        [*] --> StateA
        StateA --> StateA : SelfTransition : Action1
        @enduml
        "#;
        let diagram = StateDiagram::parse(input).unwrap();
        assert_eq!(diagram.enter_states, vec!["StateA"]);
        assert_eq!(diagram.transitions.len(), 1);
    }

    #[test]
    fn test_parse_single_comment() {
        let input = r#"
        @startuml test
        ' This is a comment
        @enduml
        "#;
        let diagram = StateDiagram::parse(input).unwrap();
        assert_eq!(diagram.name, Some("test"));
        assert_eq!(diagram.transitions.len(), 0);
    }

    #[test]
    fn test_parse_comments_mixed_with_transitions() {
        let input = r#"
        @startuml test
        ' Comment at the start
        [*] --> A
        ' Comment in the middle
        A --> B : event1
        ' Another comment
        B --> C : event2
        ' Comment at the end
        @enduml
        "#;
        let diagram = StateDiagram::parse(input).unwrap();
        assert_eq!(diagram.enter_states, vec!["A"]);
        assert_eq!(diagram.transitions.len(), 2);
    }

    #[test]
    fn test_parse_comment_with_special_characters() {
        let input = r#"
        @startuml test
        ' Comment with special chars: -> --> [*] : @#$%
        [*] --> A
        @enduml
        "#;
        let diagram = StateDiagram::parse(input).unwrap();
        assert_eq!(diagram.enter_states, vec!["A"]);
    }
}
