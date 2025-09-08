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
