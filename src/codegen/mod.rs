pub mod generators;
pub mod ident;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::parser;
use generators::*;
use ident::Idents;

type GeneratedCode = TokenStream2;

pub struct GenerationContext<'a> {
    pub fsm: &'a parser::ParsedFsm,
    pub idents: &'a Idents,
}

trait CodeGenerator {
    fn generate(&self, ctx: &GenerationContext) -> TokenStream2;
}
type CodeGeneratorPtr = Box<dyn CodeGenerator>;

pub struct FsmCodeGenerator {
    generators: Vec<CodeGeneratorPtr>,
}

impl FsmCodeGenerator {
    pub fn new(generators: Vec<CodeGeneratorPtr>) -> Self {
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

impl Default for FsmCodeGenerator {
    fn default() -> Self {
        Self::new(vec![
            Box::new(EventParamsTraitGenerator),
            Box::new(ActionTraitGenerator),
            Box::new(EventEnumGenerator),
            Box::new(StateStructGenerator),
            Box::new(StateImplGenerator),
            Box::new(FsmStructGenerator),
            Box::new(FsmImplGenerator),
        ])
    }
}
