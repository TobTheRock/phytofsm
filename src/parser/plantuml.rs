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
    composite_states: Vec<CompositeState<'a>>,
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
        let mut composite_states = Vec::new();

        for inner in diagram_pair.into_inner() {
            match inner.as_rule() {
                Rule::startuml => name = parse_diagram_name(inner),
                Rule::content => {
                    let (enters, trans, composites) = parse_content(inner)?;
                    enter_states = enters;
                    transitions = trans;
                    composite_states = composites;
                }
                _ => {}
            }
        }

        Ok(StateDiagram {
            name,
            enter_states,
            transitions,
            composite_states,
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

    pub fn composite_states(&self) -> impl Iterator<Item = &CompositeState<'_>> {
        self.composite_states.iter()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TransitionDescription<'a> {
    pub from: StateName<'a>,
    pub to: StateName<'a>,
    // TODO make this optional for direct transitions
    pub description: &'a str,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct CompositeState<'a> {
    pub name: StateName<'a>,
    pub children: Vec<CompositeState<'a>>,
    pub enter_states: Vec<StateName<'a>>,
    pub transitions: Vec<TransitionDescription<'a>>,
}

fn parse_diagram_name(pair: Pair<'_>) -> Option<&str> {
    pair.into_inner()
        .find(|p| p.as_rule() == Rule::diagram_name)
        .map(|p| p.as_str())
}

fn parse_content(
    pair: Pair<'_>,
) -> Result<(
    Vec<StateName<'_>>,
    Vec<TransitionDescription<'_>>,
    Vec<CompositeState<'_>>,
)> {
    let mut enter_states = Vec::new();
    let mut transitions = Vec::new();
    let mut composite_states = Vec::new();

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
                Rule::composite_state => {
                    composite_states.push(parse_composite_state(element_inner)?);
                }
                _ => {}
            }
        }
    }

    Ok((enter_states, transitions, composite_states))
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

fn parse_composite_state(pair: Pair<'_>) -> Result<CompositeState<'_>> {
    let mut name = None;
    let mut children = Vec::new();
    let mut enter_states = Vec::new();
    let mut transitions = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::state_name => {
                name = Some(inner.as_str());
            }
            Rule::content => {
                (enter_states, transitions, children) = parse_content(inner)?;
            }
            _ => {}
        }
    }

    Ok(CompositeState {
        name: name.ok_or_else(|| Error::Parse("Missing name in composite state".to_string()))?,
        children,
        enter_states,
        transitions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    impl CompositeState<'_> {
        fn assert_name(&self, expected: &str) -> &Self {
            assert_eq!(self.name, expected);
            self
        }

        fn assert_children(&self, expected: usize) -> &Self {
            assert_eq!(
                self.children.len(),
                expected,
                "children count for '{}'",
                self.name
            );
            self
        }

        fn assert_enters(&self, expected: &[&str]) -> &Self {
            assert_eq!(
                self.enter_states, expected,
                "enter_states for '{}'",
                self.name
            );
            self
        }

        fn assert_transition(&self, idx: usize, from: &str, to: &str) -> &Self {
            let t = &self.transitions[idx];
            assert_eq!(
                (t.from, t.to),
                (from, to),
                "transition[{}] for '{}'",
                idx,
                self.name
            );
            self
        }

        fn child(&self, idx: usize) -> &CompositeState<'_> {
            &self.children[idx]
        }
    }

    #[test]
    fn test_parse_uml_content() {
        let full_input = "@startuml fsm name\n@enduml";
        let diagram = StateDiagram::parse(full_input).unwrap();
        assert_eq!(diagram.name, Some("fsm name"));
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

    #[test]
    fn test_parse_composite_states() {
        let input = r#"
        @startuml CompositeFSM
        state State1 {
            state state1A {
                state state1B {
                }
            }
        }
        @enduml
        "#;
        let fsm = StateDiagram::parse(input).expect("Failed to parse FSM");
        assert_eq!(fsm.name, Some("CompositeFSM"));
        assert_eq!(fsm.composite_states.len(), 1);

        fsm.composite_states[0]
            .assert_name("State1")
            .assert_children(1)
            .child(0)
            .assert_name("state1A")
            .assert_children(1)
            .child(0)
            .assert_name("state1B")
            .assert_children(0);
    }

    #[test]
    fn test_parse_composite_states_with_transitions() {
        let input = r#"
        @startuml CompositeFSM
        state State1 {
            state state1A {
                state state1B {
                    [*] --> State1C
                }
                [*] --> State1B
                State1C -> State1N : toState1B
            }
        }
        @enduml
        "#;
        let fsm = StateDiagram::parse(input).expect("Failed to parse FSM");
        assert_eq!(fsm.name, Some("CompositeFSM"));
        assert_eq!(fsm.composite_states.len(), 1);

        let state1a = fsm.composite_states[0]
            .assert_name("State1")
            .assert_children(1)
            .child(0);

        state1a
            .assert_name("state1A")
            .assert_children(1)
            .assert_enters(&["State1B"])
            .assert_transition(0, "State1C", "State1N");

        state1a
            .child(0)
            .assert_name("state1B")
            .assert_children(0)
            .assert_enters(&["State1C"]);
    }
    #[test]
    fn test_parse_multiple_composite_states() {
        let input = r#"
        @startuml CompositeFSM
        state State1 {
            state state1A {
                state state1B {
                }
            }
        }
        state State2 {
            state state2A {
                state state2B {
                }
            }
        }
        @enduml
        "#;
        let fsm = StateDiagram::parse(input).expect("Failed to parse FSM");
        assert_eq!(fsm.name, Some("CompositeFSM"));
        assert_eq!(fsm.composite_states.len(), 2);

        fsm.composite_states[0]
            .assert_name("State1")
            .assert_children(1)
            .child(0)
            .assert_name("state1A")
            .assert_children(1)
            .child(0)
            .assert_name("state1B")
            .assert_children(0);

        fsm.composite_states[1]
            .assert_name("State2")
            .assert_children(1)
            .child(0)
            .assert_name("state2A")
            .assert_children(1)
            .child(0)
            .assert_name("state2B")
            .assert_children(0);
    }

    #[test]
    fn test_parse_multiple_composite_states_with_transitions() {
        let input = r#"
        @startuml CompositeFSM
        state State1 {
            state state1A {
                state state1B {
                    [*] --> State1C
                }
                [*] --> State1B
                State1C -> State1N : toState1B
            }
        }
        state State2 {
            state state2A {
                state state2B {
                    [*] --> State2C
                }
                [*] --> State2B
                State2C -> State2N : toState2B
            }
        }
        @enduml
        "#;
        let fsm = StateDiagram::parse(input).expect("Failed to parse FSM");
        assert_eq!(fsm.name, Some("CompositeFSM"));
        assert_eq!(fsm.composite_states.len(), 2);

        // State1 hierarchy
        let state1a = fsm.composite_states[0]
            .assert_name("State1")
            .assert_children(1)
            .child(0);

        state1a
            .assert_name("state1A")
            .assert_children(1)
            .assert_enters(&["State1B"])
            .assert_transition(0, "State1C", "State1N");

        state1a
            .child(0)
            .assert_name("state1B")
            .assert_children(0)
            .assert_enters(&["State1C"]);

        // State2 hierarchy
        let state2a = fsm.composite_states[1]
            .assert_name("State2")
            .assert_children(1)
            .child(0);

        state2a
            .assert_name("state2A")
            .assert_children(1)
            .assert_enters(&["State2B"])
            .assert_transition(0, "State2C", "State2N");

        state2a
            .child(0)
            .assert_name("state2B")
            .assert_children(0)
            .assert_enters(&["State2C"]);
    }
}
