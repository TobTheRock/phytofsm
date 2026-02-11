pub mod generators;
pub mod ident;

use crate::parser;

type GeneratedCode = proc_macro2::TokenStream;

#[derive(Default, Debug, Copy, Clone)]
pub struct Options {
    pub log_level: Option<log::Level>,
}

pub struct FsmCodeGenerator {
    generators: Vec<CodeGeneratorPtr>,
}

impl FsmCodeGenerator {
    pub fn new(options: &Options) -> Self {
        let generators: Vec<CodeGeneratorPtr> = vec![
            Box::new(generators::EventParamsTraitGenerator),
            Box::new(generators::ActionTraitGenerator),
            Box::new(generators::EventEnumGenerator),
            Box::new(generators::EventEnumDisplayImplGenerator),
            Box::new(generators::StateIdEnumGenerator),
            Box::new(generators::StateStructGenerator),
            Box::new(generators::StateImplGenerator),
            Box::new(generators::FsmStructGenerator),
            Box::new(generators::FsmImplGeneratorCommon),
            if let Some(log_level) = options.log_level {
                Box::new(generators::FsmImplGeneratorWithLogging::new(log_level))
            } else {
                Box::new(generators::FsmImplGenerator)
            },
        ];

        Self { generators }
    }

    pub fn generate(&self, fsm: parser::ParsedFsm) -> GeneratedCode {
        let idents = ident::Idents::new(fsm.name());
        let ctx = GenerationContext {
            fsm: &fsm,
            idents: &idents,
        };

        let components: Vec<proc_macro2::TokenStream> = self
            .generators
            .iter()
            .map(|generator| generator.generate(&ctx))
            .collect();

        let module_name = &idents.module;
        quote::quote! {
            mod #module_name {
                pub type NoEventData = ();
                #(#components)*
            }
        }
    }
}

pub(crate) struct GenerationContext<'a> {
    pub fsm: &'a parser::ParsedFsm,
    pub idents: &'a ident::Idents,
}

pub(crate) trait CodeGenerator {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream;
}
type CodeGeneratorPtr = Box<dyn CodeGenerator>;

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
