use proc_macro::TokenStream;
use quote::quote;

mod codegen;
mod error;
mod file;
mod options;
mod parser;
#[cfg(test)]
mod test;

use crate::codegen::FsmCodeGenerator;

#[proc_macro]
/// Parse the given FSM definition file and generate the corresponding Rust code.
///
/// The input to this macro is the path a file containing the FSM definition.
/// Currently, only PlantUML state machine diagrams are supported.
/// This will generate an FSM implementation and traits for events and actions, which the use has
/// to implement.
///
///  # Syntax
/// ```ignore
/// // With default parameters:
/// #generate_fsm!("path/to/fsm_definition.puml")
/// // With parameters:
/// #generate_fsm!(file_path = "path/to/fsm_definition.puml", log_level = "debug")
///
/// # Parameters
///
/// | Parameter | Description |  Default 
/// |-----------|-------------|----------
/// | **file_path** | Path to the FSM definition file. This parameter is required. | None
/// | **log_level** | Optional log level for state transitions. Possible values: `error`, `warn`, `info`, `debug`, `trace`. If not set, no logging is performed. | None
///
///
/// ```
/// # Generated Code
///
/// | Generated Item | Naming Pattern | Description |
/// |---------------|----------------|-------------|
/// | **FSM Struct** | `{DiagramName}` | Main state machine struct (UpperCamelCase) |
/// | **Event Parameters Trait** | `I{DiagramName}EventParams` | Trait defining event parameter types |
/// | **Actions Trait** | `I{DiagramName}Actions` | Trait defining action methods |
/// | **Event Enum** | `{DiagramName}Event` | Enum containing all possible events |
/// | **State Struct** | `{DiagramName}State` | Internal state representation |
/// | **Module** | `{diagram_name}` | Generated module name (snake_case) |
///
/// # Example
///
/// ```rust,ignore
/// use phyto_fsm::generate_fsm;
/// generate_fsm!("path/to/fsm_definition.puml");
///
/// use my_fsm::*; // Import generated module
///
/// struct MyActions;
/// impl IMyFsmActions for MyActions {
///     fn some_action(&mut self, params: SomeEventParams) {
///         // Implement action logic here
///     }
///     // Implement other actions...
/// }
///
/// impl IMyFsmEventParams for MyActions {
///     type SomeEventParams = NoEventData;
///     type OtherEventParams = String;
///     // Define other event parameter types...
/// }
///
/// let actions = MyActions;
/// let mut fsm = MyFsm::new(actions);
/// fsm.trigger_event(MyFsmEvent::SomeEvent(()));
/// fsm.trigger_event(MyFsmEvent::OtherEvent("data".to_string()));
/// ```
pub fn generate_fsm(input: TokenStream) -> TokenStream {
    match generate_fsm_inner(input) {
        Ok(tokens) => tokens,
        Err(error) => {
            let error_msg = format!("[phyto-fsm] {}", error);
            quote! {
                compile_error!(#error_msg);
            }
            .into()
        }
    }
}

fn generate_fsm_inner(input: TokenStream) -> error::Result<TokenStream> {
    let options: options::Options =
        syn::parse(input).map_err(|e| error::Error::InvalidInput(e.to_string()))?;
    let file_path = file::FilePath::resolve(&options.file_path, proc_macro::Span::call_site());
    let file = file::FsmFile::try_open(file_path)?;
    let parsed_fsm = parser::ParsedFsm::try_parse(file.content())?;
    let generator = FsmCodeGenerator::new(&options.codegen);
    let fsm_code = generator.generate(parsed_fsm);

    Ok(fsm_code.into())
}
