use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

mod error;
mod fsm;
mod parser;
// #[cfg(test)]
mod test;
use crate::{parser::FsmFile, test::FsmTestData};

fn fsm_event_params_trait(fsm: &fsm::Fsm) -> TokenStream2 {
    let trait_ident = &fsm.idents().event_params_trait;
    let associated_types = fsm.all_events().map(|event| {
        let type_ident = event.params_ident();
        quote! { type #type_ident; }
    });

    quote! {
        pub trait #trait_ident {
            #(#associated_types)*
        }
    }
}

fn fsm_actions_trait(fsm: &fsm::Fsm) -> TokenStream2 {
    let action_methods = fsm.actions().map(|(action, event)| {
        let action_ident = action.ident();
        let params_ident = event.params_ident();
        quote! {
            fn #action_ident(&mut self, params: Self::#params_ident);
        }
    });

    let idents = fsm.idents();
    let event_params_trait = &idents.event_params_trait;
    let trait_ident = &idents.action_trait;

    quote! {
        pub trait #trait_ident : #event_params_trait{
            #(#action_methods)*
        }
    }
}

fn event_enum(fsm: &fsm::Fsm) -> TokenStream2 {
    let event_variants = fsm.all_events().map(|event| {
        let params_ident = event.params_ident();
        let event_ident = event.ident();
        quote! { #event_ident(P::#params_ident),}
    });

    let idents = fsm.idents();
    let event_enum_ident = &idents.event_enum;
    let action_ident = &idents.action_trait;
    quote! {
        pub enum #event_enum_ident<P: #action_ident> {
            #(#event_variants)*
        }
    }
}

fn fsm_state_struct(fsm: &fsm::Fsm) -> TokenStream2 {
    let idents = fsm.idents();
    let state_ident = &idents.state_struct;
    let actions_trait = &idents.action_trait;
    let event_enum = &idents.event_enum;

    quote! {
        struct #state_ident<A: #actions_trait> {
            pub name: &'static str,
            pub transition: fn(event: #event_enum<A>, actions: &mut A) -> Option<#state_ident<A>>,
        }
    }
}

fn fsm_state_impl(fsm: &fsm::Fsm) -> TokenStream2 {
    let lookup_states = fsm.transitions_by_source_state();
    let idents = fsm.idents();

    let state_fns = lookup_states.map(|(state, transitions)| {
        let fn_name = state.function_ident();
        let transitions = transitions.iter().map(|t| {
            let event_ident = t.event.ident();

            let event_enum = &idents.event_enum;
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

    let struct_ident = &idents.state_struct;
    let actions_trait = &idents.action_trait;
    quote! {
        impl<A: #actions_trait> #struct_ident<A> {
            #(#state_fns)*
        }
    }
}

fn fsm_struct(fsm: &fsm::Fsm) -> TokenStream2 {
    let idents = fsm.idents();
    let fsm = &idents.fsm;
    let action = &idents.action_trait;
    let state = &idents.state_struct;
    quote! {
        pub struct #fsm<A: #action> {
            actions: A,
            current_state: #state<A>,
        }
    }
}

fn fsm_impl(fsm: &fsm::Fsm) -> TokenStream2 {
    let idents = fsm.idents();
    let entry_state = fsm.entry().function_ident();

    let fsm = &idents.fsm;
    let action = &idents.action_trait;
    let state = &idents.state_struct;
    let event_enum = &idents.event_enum;

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

#[proc_macro]
pub fn generate_fsm(input: TokenStream) -> TokenStream {
    // TODO relative path handling with error handling. Also handle abs paths
    let span = proc_macro::Span::call_site(); // Use the `Span` to get the `SourceFile` let source_file = span.local_file(); Get the path of the `SourceFile` let file_path.path();
    let caller_file = span.local_file().unwrap();
    let caller_dir = caller_file.parent().unwrap();
    let path = caller_dir.join(&input.to_string().trim_matches('"'));

    print!("START");
    // let file_path = syn::parse_macro_input!(input as syn::LitStr).value();
    // dbg!(&file_path);
    // let abs_file_path = std::fs::canonicalize(file_path).unwrap();

    // TODO proper error formating
    // dbg!(&abs_file_path);q
    // let contents = std::fs::read_to_string(&abs_file_path).expect("File not found");

    // INPUTS: TODO from file name or from  parsed content

    // TODO rm
    let file = FsmFile::try_open(path.to_str().unwrap()).expect("Failed to open FSM file");
    let parsed = file.try_parse().expect("Failed to parse FSM file");
    let fsm = fsm::Fsm::try_from(parsed).expect("Failed to create FSM from representation");

    let module = &fsm.idents().module;

    let event_params_trait = fsm_event_params_trait(&fsm);
    let action_trait = fsm_actions_trait(&fsm);
    let event_enum = event_enum(&fsm);
    let state_struct = fsm_state_struct(&fsm);
    let state_impl = fsm_state_impl(&fsm);
    let fsm_struct = fsm_struct(&fsm);
    let fsm_impl = fsm_impl(&fsm);

    let fsm_code = quote! {
        mod #module {
            pub type NoEventData = ();

            #event_params_trait
            #action_trait
            #event_enum
            #state_struct
            #state_impl
            #fsm_struct
            #fsm_impl
        }
    };
    // TODO rm
    println!("{}", fsm_code);
    fsm_code.into()
}
