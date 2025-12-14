# Project Context

When working with this codebase, prioritize readability over cleverness, always follow clean code guidelines
Ask clarifying questions before making architectural changes. Write tests before starting new implementations (TDD).

# About this project

This library provides a proc macro, which helps generating finite state machines by parsing Plantuml syntax.
Whenever a feature is added a reference implementation is first drafted and updated

# Key directories

- `src/parser/`: parser abstraction and implementation
- `src/codegen/`: code generation abstraction and implementation
- `src/test`: test data (puml, data abstraction), cutting at the parser abstraction, so both the parser and codegen can be tested
- `tests`: integration tests using the same puml files as the module tests
