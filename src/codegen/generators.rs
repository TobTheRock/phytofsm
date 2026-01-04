use crate::parser;

use super::{CodeGenerator, GenerationContext};

pub struct EventParamsTraitGenerator;
pub struct ActionTraitGenerator;
pub struct EventEnumGenerator;
pub struct EventEnumDisplayImplGenerator;
pub struct StateStructGenerator;
pub struct StateImplGenerator;
pub struct FsmStructGenerator;
pub struct FsmImplGenerator;
pub struct FsmImplGeneratorWithLogging {
    log_level: log::Level,
}
pub struct FsmImplGeneratorCommon;

impl CodeGenerator for EventParamsTraitGenerator {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let trait_ident = &ctx.idents.event_params_trait;
        let associated_types = ctx.fsm.events().map(|event| {
            let type_ident = event.params_ident();
            quote::quote! { type #type_ident; }
        });

        quote::quote! {
            pub trait #trait_ident {
                #(#associated_types)*
            }
        }
    }
}

impl CodeGenerator for ActionTraitGenerator {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let action_methods = ctx.fsm.actions().map(|(action, event)| {
            let action_ident = action.ident();
            let params_ident = event.params_ident();
            quote::quote! {
                fn #action_ident(&mut self, params: Self::#params_ident);
            }
        });

        let event_params_trait = &ctx.idents.event_params_trait;
        let trait_ident = &ctx.idents.action_trait;

        quote::quote! {
            pub trait #trait_ident : #event_params_trait{
                #(#action_methods)*
            }
        }
    }
}

impl CodeGenerator for EventEnumGenerator {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let event_variants = ctx.fsm.events().map(|event| {
            let params_ident = event.params_ident();
            let event_ident = event.ident();
            quote::quote! { #event_ident(P::#params_ident),}
        });

        let event_enum_ident = &ctx.idents.event_enum;
        let action_ident = &ctx.idents.action_trait;
        quote::quote! {
            enum #event_enum_ident<P: #action_ident> {
                #(#event_variants)*
            }
        }
    }
}

impl CodeGenerator for EventEnumDisplayImplGenerator {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let event_enum_ident = &ctx.idents.event_enum;
        let event_variants = ctx.fsm.events().map(|event| {
            let event_ident = event.ident();
            let event_name = &event.0;
            quote::quote! { #event_enum_ident::#event_ident(_) => write!(f, "{}", #event_name), }
        });

        let action_ident = &ctx.idents.action_trait;
        quote::quote! {
            impl<P: #action_ident> std::fmt::Display for #event_enum_ident<P> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#event_variants)*
                    }
                }
            }
        }
    }
}

impl CodeGenerator for StateStructGenerator {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let state_ident = &ctx.idents.state_struct;
        let actions_trait = &ctx.idents.action_trait;
        let event_enum = &ctx.idents.event_enum;

        quote::quote! {
            struct #state_ident<A: #actions_trait> {
                name: &'static str,
                transition: fn(event: #event_enum<A>, actions: &mut A) -> Option<Self>,
                direct_enter: Option<fn() -> Self>,
            }


            impl<A: #actions_trait> Clone for #state_ident<A> {
                fn clone(&self) -> Self {
                    Self {
                        name: self.name,
                        transition: self.transition,
                        direct_enter: self.direct_enter,
                    }
                }
            }
        }
    }
}

impl CodeGenerator for StateImplGenerator {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let state_fns = ctx.fsm.states().map(|state| {
            let state_name = state.name_literal();
            let fn_name = state.function_ident();

            let transitions = state.transitions().map(|t| {
                let event_ident = t.event.ident();
                let event_enum = &ctx.idents.event_enum;
                let next_state = t.destination.function_ident();
                let action = if let Some(a) = t.action {
                    let action_ident = a.ident();
                    quote::quote! { action.#action_ident(params); }
                } else {
                    quote::quote! {}
                };

                quote::quote! {
                    #event_enum::#event_ident(params) => {
                        #action
                        Some(Self::#next_state())
                    }
                }
            });

            let enter_state = state
                .substates()
                .find(|substate| substate.state_type() == parser::StateType::Enter);
            let direct_enter = if let Some(enter_state) = enter_state {
                let enter_fn = enter_state.function_ident();
                quote::quote! {
                    Some(Self::#enter_fn)
                }
            } else {
                quote::quote! {
                    None
                }
            };

            let parent_transition = if let Some(parent) = state.parent() {
                let parent_fn = parent.function_ident();
                quote::quote! {
                        {
                        let parent = Self::#parent_fn();
                        (parent.transition)(event, action)
                    }
                }
            } else {
                quote::quote! {
                    None
                }
            };

            quote::quote! {
                    fn #fn_name() -> Self {
                        Self {
                            name: #state_name,
                            transition: |event, action| match event {
                                #(#transitions,)*
                                _ => #parent_transition,
                            },
                            direct_enter: #direct_enter,
                    }
                }
            }
        });

        let struct_ident = &ctx.idents.state_struct;
        let actions_trait = &ctx.idents.action_trait;
        quote::quote! {
        impl<A: #actions_trait> #struct_ident<A> {
                #(#state_fns)*
            }
        }
    }
}

impl CodeGenerator for FsmStructGenerator {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let fsm = &ctx.idents.fsm;
        let action = &ctx.idents.action_trait;
        let state = &ctx.idents.state_struct;
        quote::quote! {
            pub struct #fsm<A: #action> {
                actions: A,
                current_state: #state<A>,
            }
        }
    }
}

impl CodeGenerator for FsmImplGenerator {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let fsm = &ctx.idents.fsm;
        let action = &ctx.idents.action_trait;
        let event_enum = &ctx.idents.event_enum;
        let state = &ctx.idents.state_struct;

        quote::quote! {
            impl<A> #fsm<A>
            where
                A: #action,
            {
                pub fn trigger_event(&mut self, event: #event_enum<A>) {
                    if let Some(new_state) = (self.current_state.transition)(event, &mut self.actions) {
                        let next_state = self.enter_new_state(new_state);
                        self.current_state = next_state;
                    }
                }

                fn enter_new_state(&self, mut new_state: #state<A>) -> #state<A> {
                    while let Some(direct_enter_fn) = new_state.direct_enter {
                        let direct_state = direct_enter_fn();
                        new_state = direct_state;
                    }

                    new_state
                }
            }
        }
    }
}

impl CodeGenerator for FsmImplGeneratorWithLogging {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let fsm = &ctx.idents.fsm;
        let action = &ctx.idents.action_trait;
        let event_enum = &ctx.idents.event_enum;
        let level = self.log_level_token();
        let state = &ctx.idents.state_struct;

        let log_transition = format! {"{}: {{}} -[{{}}]-> {{}}", ctx.fsm.name()};
        let log_direct_enter = format! {"{}: Directly entering {{}}", ctx.fsm.name()};
        quote::quote! {
            impl<A> #fsm<A>
            where
                A: #action,
            {
                fn trigger_event(&mut self, event: #event_enum<A>) {
                    let event_name = format!("{}", event);
                    if let Some(new_state) = (self.current_state.transition)(event, &mut self.actions) {
                        ::log::log!(#level, #log_transition, self.current_state.name, event_name, new_state.name);
                        let next_state = self.enter_new_state(new_state);
                        self.current_state = next_state;
                    }
                }

                fn enter_new_state(&self, mut new_state: #state<A>) -> #state<A> {
                    while let Some(direct_enter_fn) = new_state.direct_enter {
                        let direct_state = direct_enter_fn();
                        ::log::log!(#level, #log_direct_enter, direct_state.name);
                        new_state = direct_state;
                    }

                    new_state
                }
            }
        }
    }
}

impl CodeGenerator for FsmImplGeneratorCommon {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let fsm = &ctx.idents.fsm;
        let action = &ctx.idents.action_trait;
        let state = &ctx.idents.state_struct;
        let entry_state = ctx.fsm.enter_state().function_ident();
        let event_enum = &ctx.idents.event_enum;
        let event_params_trait = &ctx.idents.event_params_trait;

        let methods = ctx.fsm.events().map(|event| {
            let fn_ident = event.method_ident();
            let event_ident = event.ident();
            let params_ident = event.params_ident();
            quote::quote! {
                pub fn #fn_ident(&mut self, params: <A as #event_params_trait>::#params_ident) {
                    self.trigger_event(#event_enum::#event_ident(params));
                }
            }
        });

        quote::quote! {
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


                #(#methods)*
            }
        }
    }
}

impl FsmImplGeneratorWithLogging {
    pub fn new(log_level: log::Level) -> Self {
        Self { log_level }
    }

    fn log_level_token(&self) -> proc_macro2::TokenStream {
        match self.log_level {
            log::Level::Error => quote::quote! {log::Level::Error},
            log::Level::Warn => quote::quote! {log::Level::Warn},
            log::Level::Info => quote::quote! {log::Level::Info},
            log::Level::Debug => quote::quote! {log::Level::Debug},
            log::Level::Trace => quote::quote! {log::Level::Trace},
        }
    }
}
