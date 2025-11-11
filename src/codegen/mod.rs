pub mod generators;
pub mod ident;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::parser;
use generators::*;
use ident::Idents;

type GeneratedCode = TokenStream2;

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
            Box::new(EventParamsTraitGenerator),
            Box::new(ActionTraitGenerator),
            Box::new(EventEnumGenerator),
            Box::new(EventEnumDisplayImplGenerator),
            Box::new(StateStructGenerator),
            Box::new(StateImplGenerator),
            Box::new(FsmStructGenerator),
            if let Some(log_level) = options.log_level {
                Box::new(FsmImplGeneratorWithLogging::new(log_level))
            } else {
                Box::new(FsmImplGenerator)
            },
        ];

        Self { generators }
    }

    pub fn generate(&self, fsm: parser::ParsedFsm) -> GeneratedCode {
        let idents = Idents::new(fsm.name());
        let ctx = GenerationContext {
            fsm: &fsm,
            idents: &idents,
        };

        let components: Vec<TokenStream2> = self
            .generators
            .iter()
            .map(|generator| generator.generate(&ctx))
            .collect();

        let module_name = &idents.module;
        quote! {
            mod #module_name {
                pub type NoEventData = ();
                #(#components)*
            }
        }
    }
}

pub(crate) struct GenerationContext<'a> {
    pub fsm: &'a parser::ParsedFsm,
    pub idents: &'a Idents,
}

pub(crate) trait CodeGenerator {
    fn generate(&self, ctx: &GenerationContext) -> TokenStream2;
}
type CodeGeneratorPtr = Box<dyn CodeGenerator>;

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::{
        codegen::{FsmCodeGenerator, Options},
        test::FsmTestData,
    };

    fn create_codegen_test(test_data: FsmTestData, options: &Options) -> PathBuf {
        let generator = FsmCodeGenerator::new(&options);

        let module_code = generator.generate(test_data.parsed);
        let complete_code = format!("{}\n\nfn main() {{}}\n", module_code);

        std::fs::create_dir_all("target/test_files/codegen/pass").unwrap();
        let base_path = Path::new("target/test_files/codegen/pass/");
        let file_path = base_path.join(format!("{}.rs", test_data.name));
        std::fs::write(&file_path, complete_code).unwrap();

        file_path
    }

    fn test_all_generators_with_options(options: &Options) {
        let test_data = FsmTestData::all();
        let test_files = test_data.map(|data| create_codegen_test(data, options));
        let t = trybuild::TestCases::new();
        for test_file in test_files {
            t.pass(&test_file);
        }
    }

    #[test]
    fn all_generators_default_options() {
        let options = Options::default();
        test_all_generators_with_options(&options);
    }

    #[test]
    fn all_generators_logging() {
        let options = Options {
            log_level: Some(log::Level::Info),
        };
        test_all_generators_with_options(&options);
    }
}
