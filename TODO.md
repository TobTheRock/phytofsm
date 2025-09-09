# FSM Code Generator - Architectural Improvements

## 🔥 Critical Priority (Fix Immediately)

**Action**:

- Replace `expect()` calls in `src/lib.rs:23, 25` with proper error handling  
- Remove debug `println!` in `src/lib.rs:30`
- Clean up test `unwrap()` calls (acceptable in tests but could be improved)
- One remaining `unwrap()` in `src/parser/mod.rs:103` for enter_state extraction

## 🚨 High Priority (Next Sprint)

### 4. **Create Proper Error Hierarchy**

**Files**: `src/error.rs:1-9`
**Issue**: Error types could be more specific, but basic structure exists

```rust
// Current: Basic Error enum with InvalidFile and Parse variants
// Goal: More context-specific errors for better debugging
```

**Action**:

```rust
pub enum ParseError { InvalidSyntax(String), MissingEntryState, InvalidTransition(String) }
pub enum ValidationError { DuplicateStates(String), InvalidStateGraph, NoEntryState }
pub enum CodegenError { InvalidIdentifier(String), TemplateError(String) }
```

**Status**: Partially implemented - basic error structure exists but could be more granular

### 5. **Implement Builder Pattern for FSM Construction**

**Files**: `src/parser/mod.rs:95-117` (TryFrom implementation exists)
**Issue**: Direct conversion exists but could be more flexible

```rust
// Current: TryFrom<StateDiagram> for ParsedFsm works but is rigid
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

**Status**: Basic conversion exists, but builder pattern would improve flexibility

### 6. **Separate Parser Concerns**

**Files**: `src/parser/plantuml.rs` and `src/parser/mod.rs:95-117`
**Issue**: Parsing and domain conversion are mixed but better separated now

```rust
// Current: plantuml.rs does syntax parsing, mod.rs does conversion via TryFrom
// Goal: Could be cleaner with explicit semantic validation step
```

**Action**:

- `plantuml.rs` → pure syntax parsing → AST ✅ (mostly done)
- `semantic.rs` → AST validation → domain model (could be extracted from TryFrom)
- Clear error attribution (syntax vs semantic errors)

**Status**: Better separated than before, but could extract semantic validation

## 🛠️ Medium Priority (Future Iterations)

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

**Files**: `src/codegen/generators.rs` - Already implemented!
**Issue**: ~~Large monolithic functions~~ **RESOLVED**

```rust
// Current: ✅ Well-organized generator traits and implementations
pub trait CodeGenerator {
    fn generate(&self, ctx: &GenerationContext) -> TokenStream2;
}

// Individual generators implemented:
EventParamsTraitGenerator, ActionTraitGenerator, EventEnumGenerator,
StateStructGenerator, StateImplGenerator, FsmStructGenerator, FsmImplGenerator
```

**Status**: ✅ **COMPLETED** - Code generation is well-modularized with trait-based system

### 10. **Improve Test Organization**

**Files**: `src/test/**/*` and `tests/`
**Issue**: Test organization could be improved

```rust
// Current: tests/ folder exists, src/test/ for test data/helpers
// Goal: Better separation of test types
tests/
├── fixtures/           // Test data files  
├── unit/              // Unit tests
├── integration/       // End-to-end tests  
└── property/          // Property-based tests
```

**Status**: Basic test structure exists but could be more organized

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
