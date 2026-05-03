mod deferred;
mod extract;
mod generators;
mod ident;

use crate::fsm;

type GeneratedCode = proc_macro2::TokenStream;

#[derive(Default, Debug, Copy, Clone)]
pub struct Options {
    pub log_level: Option<log::Level>,
}

pub struct FsmCodeGenerator {
    options: Options,
}

impl FsmCodeGenerator {
    pub fn new(options: &Options) -> Self {
        Self { options: *options }
    }

    pub fn generate(&self, fsm: fsm::UmlFsm) -> GeneratedCode {
        let idents = ident::Idents::new(fsm.name());
        let deferred = deferred::DeferredEventsCodegen::new(&fsm, &idents);
        let ctx = GenerationContext {
            fsm: &fsm,
            deferred: &deferred,
            idents: &idents,
            options: &self.options,
        };

        let event_params_trait = generators::generate_event_params_trait(&ctx);
        let action_trait = generators::generate_action_trait(&ctx);
        let event_enum = generators::generate_event_enum(&ctx);
        let event_enum_display = generators::generate_event_enum_display(&ctx);
        let state_id_enum = generators::generate_state_id_enum(&ctx);
        let state_struct = generators::generate_state_struct(&ctx);
        let state_impl = generators::generate_state_impl(&ctx);
        let fsm = generators::generate_fsm(&ctx);

        let module_name = &idents.module;
        quote::quote! {
            mod #module_name {
                pub type NoEventData = ();
                #event_params_trait
                #action_trait
                #event_enum
                #event_enum_display
                #state_id_enum
                #state_struct
                #state_impl
                #fsm
            }
        }
    }
}

pub struct GenerationContext<'a> {
    pub fsm: &'a fsm::UmlFsm,
    pub deferred: &'a deferred::DeferredEventsCodegen,
    pub idents: &'a ident::Idents,
    pub options: &'a Options,
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{
        codegen::{FsmCodeGenerator, Options},
        test::FsmTestData,
    };

    fn create_codegen_test(
        test_data: FsmTestData,
        options: &Options,
        test_name: &str,
    ) -> std::path::PathBuf {
        let generator = FsmCodeGenerator::new(options);

        let module_code = generator.generate(test_data.parsed);
        let complete_code = format!("#![allow(warnings)] {module_code}\n\nfn main() {{}}\n");

        let base_name = format!("target/tests/data/codegen/{test_name}");
        let base_path = Path::new(&base_name);
        std::fs::create_dir_all(base_path).unwrap();
        let file_path = base_path.join(format!("{}.rs", test_data.name));
        std::fs::write(&file_path, complete_code).unwrap();

        file_path
    }

    fn test_all_generators_with_options(options: &Options, test_name: &str) {
        let test_data = FsmTestData::all();
        let test_files = test_data.map(|data| create_codegen_test(data, options, test_name));
        let t = trybuild::TestCases::new();
        for test_file in test_files {
            t.pass(&test_file);
        }
    }

    #[test]
    fn all_generators_default_options() {
        let options = Options::default();
        test_all_generators_with_options(&options, "default_options");
    }

    #[test]
    fn all_generators_logging() {
        let options = Options {
            log_level: Some(log::Level::Info),
        };
        test_all_generators_with_options(&options, "logging_options");
    }
}
