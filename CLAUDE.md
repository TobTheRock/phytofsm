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
- `target/test/data` code of the generated FSMs from the puml test files

# Coding guidelines

- Always follow DRY, SOLID and clean code principles
- Follow the reference impl, but NEVER alter it without prior discussion
- When implementing new features always do this first:
  - provide a test puml and module (might be provided by your human)
    - if necessary adapt the FSM builder if it is needed for the test module
  - write an integration test
