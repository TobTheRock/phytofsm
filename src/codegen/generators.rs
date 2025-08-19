use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use super::{CodeGenerator, GenerationContext};

pub struct EventParamsTraitGenerator;
pub struct ActionTraitGenerator;
pub struct EventEnumGenerator;
pub struct StateStructGenerator;
pub struct StateImplGenerator;
pub struct FsmStructGenerator;
pub struct FsmImplGenerator;

impl CodeGenerator for EventParamsTraitGenerator {
    fn generate(&self, ctx: &GenerationContext) -> TokenStream2 {
        let trait_ident = &ctx.idents.event_params_trait;
        let associated_types = ctx.fsm.events().map(|event| {
            let type_ident = event.params_ident();
            quote! { type #type_ident; }
        });

        quote! {
            pub trait #trait_ident {
                #(#associated_types)*
            }
        }
    }
}

impl CodeGenerator for ActionTraitGenerator {
    fn generate(&self, ctx: &GenerationContext) -> TokenStream2 {
        let action_methods = ctx.fsm.actions().map(|(action, event)| {
            let action_ident = action.ident();
            let params_ident = event.params_ident();
            quote! {
                fn #action_ident(&mut self, params: Self::#params_ident);
            }
        });

        let event_params_trait = &ctx.idents.event_params_trait;
        let trait_ident = &ctx.idents.action_trait;

        quote! {
            pub trait #trait_ident : #event_params_trait{
                #(#action_methods)*
            }
        }
    }
}

impl CodeGenerator for EventEnumGenerator {
    fn generate(&self, ctx: &GenerationContext) -> TokenStream2 {
        let event_variants = ctx.fsm.events().map(|event| {
            let params_ident = event.params_ident();
            let event_ident = event.ident();
            quote! { #event_ident(P::#params_ident),}
        });

        let event_enum_ident = &ctx.idents.event_enum;
        let action_ident = &ctx.idents.action_trait;
        quote! {
            pub enum #event_enum_ident<P: #action_ident> {
                #(#event_variants)*
            }
        }
    }
}

impl CodeGenerator for StateStructGenerator {
    fn generate(&self, ctx: &GenerationContext) -> TokenStream2 {
        let state_ident = &ctx.idents.state_struct;
        let actions_trait = &ctx.idents.action_trait;
        let event_enum = &ctx.idents.event_enum;

        quote! {
            struct #state_ident<A: #actions_trait> {
                pub name: &'static str,
                pub transition: fn(event: #event_enum<A>, actions: &mut A) -> Option<#state_ident<A>>,
            }
        }
    }
}

impl CodeGenerator for StateImplGenerator {
    fn generate(&self, ctx: &GenerationContext) -> TokenStream2 {
        let lookup_states = ctx.fsm.transitions().map(|t| (&t.source, t)).fold(
            std::collections::HashMap::<&crate::parser::State, Vec<&crate::parser::Transition>>::new(),
            |mut acc, (state, transition)| {
                acc.entry(state).or_default().push(transition);
                acc
            },
        );

        let state_fns = lookup_states.iter().map(|(state, transitions)| {
            let fn_name = state.function_ident();
            let transitions = transitions.iter().map(|t| {
                let event_ident = t.event.ident();

                let event_enum = &ctx.idents.event_enum;
                let next_state = t.destination.function_ident();
                let action = if let Some(a) = &t.action {
                    let action_ident = a.ident();
                    quote! { action.#action_ident(params); }
                } else {
                    quote! {}
                };

                quote! {
                    #event_enum::#event_ident(params) => {
                        #action
                        Some(Self::#next_state())
                    }
                }
            });

            quote! {
                fn #fn_name() -> Self {
                    Self {
                        name: #state,
                        transition: |event, action| match event {
                            #(#transitions,)*
                            _ => None,
                        }
                    }
                }
            }
        });

        let struct_ident = &ctx.idents.state_struct;
        let actions_trait = &ctx.idents.action_trait;
        quote! {
            impl<A: #actions_trait> #struct_ident<A> {
                #(#state_fns)*
            }
        }
    }
}

impl CodeGenerator for FsmStructGenerator {
    fn generate(&self, ctx: &GenerationContext) -> TokenStream2 {
        let fsm = &ctx.idents.fsm;
        let action = &ctx.idents.action_trait;
        let state = &ctx.idents.state_struct;
        quote! {
            pub struct #fsm<A: #action> {
                actions: A,
                current_state: #state<A>,
            }
        }
    }
}

impl CodeGenerator for FsmImplGenerator {
    fn generate(&self, ctx: &GenerationContext) -> TokenStream2 {
        let entry_state = ctx.fsm.enter_state().function_ident();

        let fsm = &ctx.idents.fsm;
        let action = &ctx.idents.action_trait;
        let state = &ctx.idents.state_struct;
        let event_enum = &ctx.idents.event_enum;

        // TODO tracing/logging
        quote! {
            impl<A> #fsm<A>
            where
                A: #action,
            {
                pub fn new(actions: A) -> Self {
                    Self {
                        actions,
                        current_state: #state::#entry_state(),
                    }
                }

                pub fn trigger_event(&mut self, event: #event_enum<A>) {
                    if let Some(new_state) = (self.current_state.transition)(event, &mut self.actions) {
                        self.current_state = new_state;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        codegen::{FsmCodeGenerator, ident::Idents},
        parser::*,
    };

    fn create_test_data() -> (ParsedFsm, Idents) {
        let winter = State {
            name: "Winter".to_string(),
            state_type: StateType::Enter,
        };
        let spring = State {
            name: "Spring".to_string(),
            state_type: StateType::Simple,
        };

        let transitions = vec![
            Transition {
                source: winter.clone(),
                destination: spring.clone(),
                event: Event("TemperatureRises".to_string()),
                action: None,
            },
            Transition {
                source: spring.clone(),
                destination: winter.clone(),
                event: Event("TemperatureDrops".to_string()),
                action: Some(Action("PrepareForWinter".to_string())),
            },
        ];

        let fsm = ParsedFsm::try_new("TestFsm".to_string(), transitions).unwrap();
        let idents = Idents::new(fsm.name());
        (fsm, idents)
    }

    fn write_generator_test_file(
        filename: &str,
        generator_code: proc_macro2::TokenStream,
    ) -> String {
        let complete_code = format!("{}\n\nfn main() {{}}\n", generator_code);
        std::fs::create_dir_all("target/test_files/codegen/pass").unwrap();
        let file_path = format!("target/test_files/codegen/pass/{}", filename);
        std::fs::write(&file_path, complete_code).unwrap();
        file_path
    }

    fn create_event_params_trait_test() -> String {
        let (fsm, idents) = create_test_data();
        let ctx = GenerationContext {
            fsm: &fsm,
            idents: &idents,
        };
        let generator = EventParamsTraitGenerator;
        let result = generator.generate(&ctx);

        write_generator_test_file("event_params_trait.rs", result)
    }

    fn create_traits_combined_test() -> String {
        let (fsm, idents) = create_test_data();
        let ctx = GenerationContext {
            fsm: &fsm,
            idents: &idents,
        };

        let event_params = EventParamsTraitGenerator.generate(&ctx);
        let action_trait = ActionTraitGenerator.generate(&ctx);

        let combined_code = quote! {
            #event_params
            #action_trait
        };

        write_generator_test_file("traits_combined.rs", combined_code)
    }

    fn create_traits_and_enum_test() -> String {
        let (fsm, idents) = create_test_data();
        let ctx = GenerationContext {
            fsm: &fsm,
            idents: &idents,
        };

        let event_params = EventParamsTraitGenerator.generate(&ctx);
        let action_trait = ActionTraitGenerator.generate(&ctx);
        let event_enum = EventEnumGenerator.generate(&ctx);

        let combined_code = quote! {
            #event_params
            #action_trait
            #event_enum
        };

        write_generator_test_file("traits_and_enum.rs", combined_code)
    }

    fn create_core_types_test() -> String {
        let (fsm, idents) = create_test_data();
        let ctx = GenerationContext {
            fsm: &fsm,
            idents: &idents,
        };

        let event_params = EventParamsTraitGenerator.generate(&ctx);
        let action_trait = ActionTraitGenerator.generate(&ctx);
        let event_enum = EventEnumGenerator.generate(&ctx);
        let state_struct = StateStructGenerator.generate(&ctx);

        let combined_code = quote! {
            #event_params
            #action_trait
            #event_enum
            #state_struct
        };

        write_generator_test_file("core_types.rs", combined_code)
    }

    // TODO restructure, this should go in the toplevel
    fn create_complete_fsm_test() -> String {
        let (fsm, idents) = create_test_data();
        let generator = FsmCodeGenerator::default();
        let module_code = generator.generate(fsm);

        let complete_code = format!("{}\n\nfn main() {{}}\n", module_code);

        std::fs::create_dir_all("target/test_files/codegen/pass").unwrap();
        let file_path = "target/test_files/codegen/pass/complete_fsm.rs".to_string();
        std::fs::write(&file_path, complete_code).unwrap();
        file_path
    }

    #[test]
    fn test_all_generators_compile() {
        let test_files = vec![
            create_event_params_trait_test(),
            create_traits_combined_test(),
            create_traits_and_enum_test(),
            create_core_types_test(),
            create_complete_fsm_test(),
        ];

        let t = trybuild::TestCases::new();
        for test_file in test_files {
            t.pass(&test_file);
        }
    }
}
