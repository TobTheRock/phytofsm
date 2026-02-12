use crate::parser::{ParsedFsmBuilder, StateType};

#[test]
fn build_without_enter_state_fails() {
    let builder = ParsedFsmBuilder::new("TestFSM");
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_empty_name_fails() {
    let mut builder = ParsedFsmBuilder::new("  ");
    builder.add_state("Start", StateType::Enter);
    let result = builder.build();
    assert!(result.is_err());
}
