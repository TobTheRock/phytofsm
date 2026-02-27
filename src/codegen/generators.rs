use super::{CodeGenerator, GenerationContext};

pub struct EventParamsTraitGenerator;
pub struct ActionTraitGenerator;
pub struct EventEnumGenerator;
pub struct EventEnumDisplayImplGenerator;
pub struct StateIdEnumGenerator;
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

        let enter_methods = ctx.fsm.enter_actions().map(|action| {
            let action_ident = action.ident();
            quote::quote! {
                fn #action_ident(&mut self);
            }
        });

        let exit_methods = ctx.fsm.exit_actions().map(|action| {
            let action_ident = action.ident();
            quote::quote! {
                fn #action_ident(&mut self);
            }
        });

        let event_params_trait = &ctx.idents.event_params_trait;
        let trait_ident = &ctx.idents.action_trait;

        quote::quote! {
            pub trait #trait_ident : #event_params_trait{
                #(#action_methods)*
                #(#enter_methods)*
                #(#exit_methods)*
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
            quote::quote! { #event_enum_ident::#event_ident(_) => #event_name, }
        });

        let action_ident = &ctx.idents.action_trait;
        quote::quote! {
            impl<P: #action_ident> std::fmt::Display for #event_enum_ident<P> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let name = match self {
                        #(#event_variants)*
                    };
                    write!(f, "{}", name)
                }
            }
        }
    }
}

impl CodeGenerator for StateIdEnumGenerator {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let state_id_enum = &ctx.idents.state_id_enum;
        let init_state_id_variant = &ctx.idents.init_state_id_variant;

        let variants = ctx.fsm.states().map(|state| {
            let variant_ident = state.state_id_variant_ident();
            quote::quote! { #variant_ident, }
        });

        let from_match_arms = ctx.fsm.states().map(|state| {
            let variant_ident = state.state_id_variant_ident();
            let name_literal = state.name_literal();
            quote::quote! { #state_id_enum::#variant_ident => #name_literal, }
        });

        quote::quote! {
            #[derive(Copy, Clone, PartialEq, Eq, Debug)]
            enum #state_id_enum {
                #(#variants)*
                #init_state_id_variant,
            }

            impl From<#state_id_enum> for &'static str {
                fn from(id: #state_id_enum) -> Self {
                    match id {
                        #(#from_match_arms)*
                        #state_id_enum::#init_state_id_variant => "[*]",
                    }
                }
            }

            impl std::fmt::Display for #state_id_enum {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let name: &'static str = (*self).into();
                    write!(f, "{}", name)
                }
            }
        }
    }
}

impl CodeGenerator for StateStructGenerator {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let state_ident = &ctx.idents.state_struct;
        let state_id_enum = &ctx.idents.state_id_enum;
        let actions_trait = &ctx.idents.action_trait;
        let event_enum = &ctx.idents.event_enum;

        quote::quote! {
            #[derive(Copy)]
            struct #state_ident<A: #actions_trait> {
                id: #state_id_enum,
                transition: fn(event: #event_enum<A>, actions: &mut A) -> Option<Self>,
                enter_state: fn() -> Self,
                enter: fn(&mut A, from: &Self),
                exit: fn(&mut A, to: &Self),
            }

            impl<A: #actions_trait> Clone for #state_ident<A> {
                fn clone(&self) -> Self {
                    Self {
                        id: self.id,
                        transition: self.transition,
                        enter_state: self.enter_state,
                        enter: self.enter,
                        exit: self.exit,
                    }
                }
            }

            impl<A: #actions_trait> PartialEq for #state_ident<A> {
                fn eq(&self, other: &Self) -> bool {
                    self.id == other.id
                }
            }
        }
    }
}

impl StateImplGenerator {
    fn generate_enter_action(
        state: &crate::parser::State<'_>,
        state_id_enum: &proc_macro2::Ident,
    ) -> proc_macro2::TokenStream {
        let enter_action = if let Some(action) = state.enter_action() {
            let action_ident = action.ident();
            quote::quote! {
                actions.#action_ident();
            }
        } else {
            quote::quote! {}
        };
        let internal_guard = Self::generate_internal_transition_guard(state, state_id_enum, true);
        let parent_enter = if let Some(parent) = state.parent() {
            let parent_fn = parent.function_ident();
            quote::quote! {
            (Self::#parent_fn().enter)(actions, from);
            }
        } else {
            quote::quote! {}
        };

        quote::quote! {
            |actions, from|
            {
            #internal_guard
            #parent_enter
            #enter_action
            }
        }
    }

    fn generate_exit_action(
        state: &crate::parser::State<'_>,
        state_id_enum: &proc_macro2::Ident,
    ) -> proc_macro2::TokenStream {
        let exit_action = if let Some(action) = state.exit_action() {
            let action_ident = action.ident();
            quote::quote! {
                actions.#action_ident();
            }
        } else {
            quote::quote! {}
        };
        let internal_guard = Self::generate_internal_transition_guard(state, state_id_enum, false);
        let parent_exit = if let Some(parent) = state.parent() {
            let parent_fn = parent.function_ident();
            quote::quote! {
            (Self::#parent_fn().exit)(actions, to);
            }
        } else {
            quote::quote! {}
        };

        quote::quote! {
            |actions, to|
            {
            #internal_guard
            #exit_action
            #parent_exit
            }
        }
    }

    fn all_substate_ids(
        state: &crate::parser::State<'_>,
        state_id_enum: &proc_macro2::Ident,
    ) -> Vec<proc_macro2::TokenStream> {
        state
            .substates()
            .map(|s| {
                let variant = s.state_id_variant_ident();
                quote::quote! { #state_id_enum::#variant }
            })
            .collect()
    }

    fn generate_internal_transition_guard(
        state: &crate::parser::State<'_>,
        state_id_enum: &proc_macro2::Ident,
        is_enter: bool,
    ) -> proc_macro2::TokenStream {
        let substate_ids = Self::all_substate_ids(state, state_id_enum);
        if substate_ids.is_empty() {
            quote::quote! {}
        } else {
            let check = if is_enter {
                quote::quote! {from}
            } else {
                quote::quote! {to}
            };
            quote::quote! {
                if matches!(#check.id, #(#substate_ids)|*) {
                    return;
                }
            }
        }
    }
}

impl CodeGenerator for StateImplGenerator {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let state_id_enum = &ctx.idents.state_id_enum;
        let init_state_id_variant = &ctx.idents.init_state_id_variant;

        let state_fns = ctx.fsm.states().map(|state| {
            let state_id_variant = state.state_id_variant_ident();
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

            let enter_state = state.enter_state();
            let enter_fn = enter_state.function_ident();
            let enter_action = Self::generate_enter_action(&state, state_id_enum);
            let exit_action = Self::generate_exit_action(&state, state_id_enum);

            quote::quote! {
                fn #fn_name() -> Self {
                    Self {
                        id: #state_id_enum::#state_id_variant,
                        transition: |event, action| match event {
                            #(#transitions,)*
                            _ => #parent_transition,
                        },
                        enter_state: Self::#enter_fn,
                        enter: #enter_action,
                        exit: #exit_action,
                    }
                }
            }
        });

        let struct_ident = &ctx.idents.state_struct;
        let actions_trait = &ctx.idents.action_trait;
        quote::quote! {
            impl<A: #actions_trait> #struct_ident<A> {
                fn init() -> Self {
                    Self {
                        id: #state_id_enum::#init_state_id_variant,
                        transition: |_event, _action| None,
                        enter_state: Self::init,
                        enter: |_actions, _from| {},
                        exit: |_actions, _to| {},
                    }
                }

                #(#state_fns)*
            }
        }
    }
}

impl CodeGenerator for FsmStructGenerator {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let fsm = &ctx.idents.fsm;
        let fsm_inner = &ctx.idents.fsm_inner;
        let action = &ctx.idents.action_trait;
        let state = &ctx.idents.state_struct;
        quote::quote! {
            struct #fsm_inner<A: #action> {
                actions: A,
                current_state: #state<A>,
            }
            pub struct #fsm<A: #action>(#fsm_inner<A>);
        }
    }
}

impl CodeGenerator for FsmImplGenerator {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let fsm_inner = &ctx.idents.fsm_inner;
        let action = &ctx.idents.action_trait;
        let event_enum = &ctx.idents.event_enum;

        quote::quote! {
            impl<A> #fsm_inner<A>
            where
                A: #action,
            {
                fn trigger_event(&mut self, event: #event_enum<A>) {
                    if let Some(transition_state) = (self.current_state.transition)(event, &mut self.actions) {
                        self.change_state(transition_state);
                    }
                }
            }
        }
    }
}

impl CodeGenerator for FsmImplGeneratorWithLogging {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let fsm_inner = &ctx.idents.fsm_inner;
        let action = &ctx.idents.action_trait;
        let event_enum = &ctx.idents.event_enum;
        let level = self.log_level_token();

        let log_transition = format! {"{}: {{}} -[{{}}]-> {{}}, entering {{}}", ctx.fsm.name()};
        quote::quote! {
            impl<A> #fsm_inner<A>
            where
                A: #action,
            {
                fn trigger_event(&mut self, event: #event_enum<A>) {
                    let event_name = format!("{}", event);
                    if let Some(transition_state) = (self.current_state.transition)(event, &mut self.actions) {
                        let enter_state = (transition_state.enter_state)();
                        ::log::log!(#level, #log_transition,
                            self.current_state.id,
                            event_name,
                            transition_state.id,
                            enter_state.id
                        );
                        self.change_state(transition_state);
                    }
                }
            }
        }
    }
}

impl CodeGenerator for FsmImplGeneratorCommon {
    fn generate(&self, ctx: &GenerationContext) -> proc_macro2::TokenStream {
        let fsm = &ctx.idents.fsm;
        let fsm_inner = &ctx.idents.fsm_inner;
        let action = &ctx.idents.action_trait;
        let state = &ctx.idents.state_struct;
        let enter_state = ctx.fsm.enter_state().function_ident();
        let event_enum = &ctx.idents.event_enum;
        let event_params_trait = &ctx.idents.event_params_trait;

        let methods = ctx.fsm.events().map(|event| {
            let fn_ident = event.method_ident();
            let event_ident = event.ident();
            let params_ident = event.params_ident();
            quote::quote! {
                pub fn #fn_ident(&mut self, params: <A as #event_params_trait>::#params_ident) {
                    self.0.trigger_event(#event_enum::#event_ident(params));
                }
            }
        });

        quote::quote! {
            impl<A> #fsm_inner<A>
            where
                A: #action,
            {
                fn change_state(&mut self, transition_state: #state<A>) {
                    let enter_state = (transition_state.enter_state)();
                    (self.current_state.exit)(&mut self.actions, &enter_state);
                    (enter_state.enter)(&mut self.actions, &self.current_state);
                    self.current_state = enter_state;
                }
            }

            impl<A> #fsm<A>
            where
                A: #action,
            {
                #(#methods)*
            }

            pub fn start<A: #action>(mut actions: A) -> #fsm<A> {
                let init = #state::init();
                let enter_state = #state::#enter_state();
                (enter_state.enter)(&mut actions, &init);
                #fsm(#fsm_inner { actions, current_state: enter_state })
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
