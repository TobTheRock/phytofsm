# FSM Code Generator - Architectural Improvements

## 🔥 Critical Priority (Fix Immediately)

### 3. **Remove Hardcoded Unwraps and Debug Code**

**Files**: `src/lib.rs:173-177, 189-191, 216-217`
**Issue**: Production code with panics and debug prints

```rust
// Current: .unwrap(), print!("START"), println!
// Goal: Proper error handling
```

**Action**:

- Replace all `unwrap()` with proper error handling
- Remove debug prints and commented code
- Add comprehensive error types for proc macro failures

## 🚨 High Priority (Next Sprint)

### 4. **Create Proper Error Hierarchy**

**Files**: `src/error.rs:1-12`
**Issue**: Generic error types don't provide context

```rust
// Current: Generic Error enum
// Goal: Context-specific errors
```

**Action**:

```rust
pub enum ParseError { InvalidSyntax(String), MissingEntryState, ... }
pub enum ValidationError { DuplicateStates(String), ... }
pub enum CodegenError { InvalidIdentifier(String), ... }
```

### 5. **Implement Builder Pattern for FSM Construction**

**Files**: `src/parser/mod.rs:67-113, src/fsm.rs:105-122`
**Issue**: Direct conversion is fragile and hard to extend

```rust
// Goal: Flexible construction with validation
pub struct FsmBuilder {
    name: String,
    transitions: Vec<Transition>,
}
impl FsmBuilder {
    pub fn with_transition(mut self, t: Transition) -> Self
    pub fn validate(self) -> Result<ParsedFsm, ValidationError>
}
```

### 6. **Separate Parser Concerns**

**Files**: `src/parser/plantuml.rs:130-150, 115-128`
**Issue**: Parsing logic mixed with domain conversion

```rust
// Current: PlantUML parser does semantic validation
// Goal: Pure syntax → semantic separation
```

**Action**:

- `plantuml.rs` → pure syntax parsing → AST
- `semantic.rs` → AST validation → domain model
- Clear error attribution (syntax vs semantic errors)

## 🛠️ Medium Priority (Future Iterations)

### 7. **Extract Identifier Generation Strategy**

**Files**: `src/fsm.rs:18-28, 30-50`
**Issue**: Naming logic scattered, hard to customize

```rust
    pub fn enter_state(&self) -> &State {
        &self.enter_state
    }

// Goal: Configurable naming strategy
pub trait NamingStrategy {
    fn fsm_name(&self, base: &str) -> Ident;
    fn event_name(&self, base: &str) -> Ident;
}
pub struct RustNamingStrategy;
pub struct TypeScriptNamingStrategy; // Future extension
```

### 8. **Add Configuration Layer**

**Files**: All generation code
**Issue**: Hard-coded generation options

```rust
// Goal: Configurable generation
pub struct CodegenConfig {
    pub naming: Box<dyn NamingStrategy>,
    pub generate_docs: bool,
    pub add_logging: bool,
    pub output_format: OutputFormat,
}
```

### 9. **Modularize Code Generation Templates**

**Files**: `src/lib.rs:12-167`
**Issue**: Large monolithic functions hard to maintain

```rust
// Goal: Template-based generation
pub trait CodeTemplate {
    fn generate(&self, fsm: &Fsm, config: &CodegenConfig) -> TokenStream2;
}

pub struct EventTraitTemplate;
pub struct ActionTraitTemplate;
pub struct StateMachineTemplate;
```

### 10. **Improve Test Organization**

**Files**: `src/test/**/*`
**Issue**: Test data mixed with production code

```rust
// Goal: Proper test structure
tests/
├── fixtures/           // Test data files
├── unit/              // Unit tests
├── integration/       // End-to-end tests
└── property/          // Property-based tests
```

## 🔧 Low Priority (Technical Debt)

### 11. **Add Comprehensive Documentation**

**Files**: All public APIs
**Issue**: Missing rustdoc and examples
**Action**:

- Add module-level documentation
- Document all public types and functions
- Add usage examples in doc comments
- Create architecture decision records (ADRs)

### 12. **Implement Caching for Expensive Operations**

**Files**: `src/fsm.rs:67-98`
**Issue**: Recomputing identifiers and state maps

```rust
// Goal: Lazy evaluation with caching
pub struct Fsm {
    repr: ParsedFsm,
    cached_idents: OnceCell<Idents>,
    cached_state_map: OnceCell<HashMap<State, Vec<Transition>>>,
}
```

### 13. **Add Support for More Input Formats**

**Files**: `src/parser/mod.rs:25-29`
**Issue**: Only PlantUML support, tightly coupled

```rust
// Goal: Parser abstraction
pub trait FsmParser {
    fn parse(&self, input: &str) -> Result<ParsedFsm>;
}

pub struct PlantUmlParser;
pub struct JsonParser;     // Future
pub struct YamlParser;     // Future
```

### 14. **Improve Type Safety**

**Files**: Various string-based identifiers
**Issue**: Stringly-typed code prone to errors

```rust
// Goal: Newtype wrappers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StateName(String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]  
pub struct EventName(String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionName(String);
```

## 📊 Architecture Overview

```
Current Structure Issues:
├── lib.rs              ❌ Everything mixed together
├── parser/mod.rs       ❌ Duplicate Fsm with fsm.rs  
├── fsm.rs             ❌ Domain logic + codegen mixed
└── error.rs           ❌ Generic errors

Target Structure:
├── lib.rs              ✅ Proc macro entry only
├── parser/
│   ├── mod.rs          ✅ File I/O + parser routing
│   ├── ast.rs          ✅ Raw syntax tree
│   ├── semantic.rs     ✅ Validated domain model
│   └── plantuml.rs     ✅ PlantUML syntax only
├── domain/
│   ├── mod.rs          ✅ Core FSM domain logic
│   ├── validation.rs   ✅ Business rule validation
│   └── builder.rs      ✅ Flexible FSM construction
├── codegen/
│   ├── mod.rs          ✅ Generation orchestration
│   ├── templates/      ✅ Individual generators
│   ├── identifiers.rs  ✅ Naming strategies
│   └── config.rs       ✅ Generation options
└── error.rs            ✅ Hierarchical error types
```

## 🎯 Success Metrics

- **Maintainability**: Each module has single responsibility
- **Testability**: All business logic unit testable  
- **Extensibility**: Easy to add new parsers/generators
- **Reliability**: No panics, comprehensive error handling
- **Performance**: Lazy evaluation, efficient caching

## Done

### 1. **Eliminate Duplicate FSM Types**

**Files**: `src/parser/mod.rs:60`, `src/fsm.rs:58`
**Issue**: Two different `Fsm` structs with overlapping responsibilities

```rust
// Current: parser::Fsm AND fsm::Fsm
// Goal: Clear separation of concerns
```

**Action**:

- Rename `parser::Fsm` → `parser::ParsedFsm` (raw parsed data)
- Keep `fsm::Fsm` for domain logic and code generation
- Remove duplicate logic between them

### 2. **Extract Code Generation from Proc Macro Entry Point**

**Files**: `src/lib.rs:12-167, 169-219`
**Issue**: Business logic mixed with proc macro plumbing

```rust
// Current: All generation functions in lib.rs
// Goal: Clean separation
```

**Action**:

- Create `src/codegen/mod.rs` with generation functions
- Move template functions (`fsm_event_params_trait`, etc.) to dedicated modules
- Keep only proc macro infrastructure in `lib.rs`
