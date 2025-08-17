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

pub struct FsmCodeGenerator {
    generators: Vec<Box<dyn CodeGenerator>>,
}

impl FsmCodeGenerator {
    pub fn new() -> Self {
        Self {
            generators: vec![
                Box::new(EventParamsTraitGenerator),
                Box::new(ActionTraitGenerator),
                Box::new(EventEnumGenerator),
                Box::new(StateStructGenerator),
                Box::new(StateImplGenerator),
                Box::new(FsmStructGenerator),
                Box::new(FsmImplGenerator),
            ],
        }
    }
}

impl parser::ParsedFsm {
    pub fn generate_from(self, generator: FsmCodeGenerator) -> GeneratedCode {
        let idents = Idents::new(self.name());
        let ctx = GenerationContext {
            fsm: &self,
            idents: &idents,
        };

        let components: Vec<TokenStream2> = generator
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
        Self::new()
    }
}
