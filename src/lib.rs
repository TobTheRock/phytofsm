use heck::ToSnakeCase;
use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use quote::{format_ident, quote};
use syn::Ident;

mod error;
mod parser;
#[cfg(test)]
mod reference;

impl parser::Event {
    pub fn params_ident(&self) -> Ident {
        format_ident!("{}Params", self.0.to_upper_camel_case())
    }

    pub fn ident(&self) -> Ident {
        Ident::new(&self.0.to_upper_camel_case(), Span::call_site())
    }
}

impl parser::Action {
    pub fn ident(&self) -> Ident {
        Ident::new(&self.0.to_snake_case(), Span::call_site())
    }
}

impl parser::State {
    pub fn function_ident(&self) -> Ident {
        Ident::new(&self.0.to_snake_case(), Span::call_site())
    }
}

impl ToTokens for parser::State {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.0.to_tokens(tokens);
    }
}

// impl Transition {
//     // TODO
//     pub fn to_state_ident(&self) -> Ident {
//         Ident::new(&self.to_state, Span::call_site())
//     }
//     pub fn from_state_ident(&self) -> Ident {
//         Ident::new(&self.from_state, Span::call_site())
//     }
// }

impl parser::FsmRepr {
    pub fn fsm_ident(&self) -> Ident {
        Ident::new(&self.name.to_upper_camel_case(), Span::call_site())
    }

    pub fn module_ident(&self) -> Ident {
        Ident::new(&self.name.to_snake_case(), Span::call_site())
    }

    pub fn event_params_trait_ident(&self) -> Ident {
        format_ident!("I{}EventParams", self.name.to_upper_camel_case())
    }

    pub fn event_enum_ident(&self) -> Ident {
        format_ident!("{}Event", self.name.to_upper_camel_case())
    }
    pub fn action_trait_ident(&self) -> Ident {
        format_ident!("I{}Actions", self.name.to_upper_camel_case())
    }

    fn state_struct_ident(&self) -> Ident {
        format_ident!("{}State", self.name.to_upper_camel_case())
    }

    // TODO move below functions to parser mod

    // pub fn state_idents(&self) -> Vec<Ident> {
    //     self.transitions
    //         .iter()
    //         .flat_map(|t| [&t.from_state, &t.to_state])
    //         .unique()
    //         .map(|name| Ident::new(&name.to_upper_camel_case(), Span::call_site()))
    //         .collect()
    // }
}

fn fsm_event_params_trait(fsm: &parser::FsmRepr) -> TokenStream2 {
    let trait_ident = fsm.event_params_trait_ident();
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

fn fsm_actions_trait(fsm: &parser::FsmRepr) -> TokenStream2 {
    let action_methods = fsm.transitions.iter().filter_map(|transition| {
        if let Some(action) = &transition.action {
            let action_ident = action.ident();
            let params_ident = transition.event.params_ident();
            let action_method = quote! {
                fn #action_ident(&mut self, params: Self::#params_ident);
            };
            Some(action_method)
        } else {
            None
        }
    });
    let event_params_trait = fsm.event_params_trait_ident();
    let trait_ident = fsm.action_trait_ident();

    quote! {
        pub trait #trait_ident : #event_params_trait{
            #(#action_methods)*
        }
    }
}

fn event_enum(fsm: &parser::FsmRepr) -> TokenStream2 {
    let event_variants = fsm.all_events().map(|event| {
        let params_ident = event.params_ident();
        let event_ident = event.ident();
        quote! { #event_ident(P::#params_ident),}
    });
    let event_enum_ident = fsm.event_enum_ident();
    let action_ident = fsm.action_trait_ident();
    quote! {
        pub enum #event_enum_ident<P: #action_ident> {
            #(#event_variants)*
        }
    }
}

fn fsm_state_struct(fsm: &parser::FsmRepr) -> TokenStream2 {
    let state_ident = fsm.state_struct_ident();
    let actions_trait = fsm.action_trait_ident();
    let event_enum = fsm.event_enum_ident();

    quote! {
        struct #state_ident<A: #actions_trait> {
            pub name: &'static str,
            pub transition: fn(event: #event_enum<A>, actions: &mut A) -> Option<#state_ident<A>>,
        }
    }
}

fn fsm_state_impl(fsm: &parser::FsmRepr) -> TokenStream2 {
    let lookup_states = fsm.transitions_by_source_state();

    let state_fns = lookup_states.map(|(state, transitions)| {
        let fn_name = state.function_ident();
        let transitions = transitions.iter().map(|t| {
            let event_ident = t.event.ident();

            let event_enum = fsm.event_enum_ident();
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

    let struct_ident = fsm.state_struct_ident();
    let actions_trait = fsm.action_trait_ident();
    quote! {
        impl<A: #actions_trait> #struct_ident<A> {
            #(#state_fns)*
        }
    }
}

fn fsm_struct(data: &parser::FsmRepr) -> TokenStream2 {
    let fsm = data.fsm_ident();
    let action = data.action_trait_ident();
    let state = data.state_struct_ident();
    quote! {
        pub struct #fsm<A: #action> {
            actions: A,
            current_state: #state<A>,
        }
    }
}

fn fsm_impl(data: &parser::FsmRepr) -> TokenStream2 {
    let fsm = data.fsm_ident();
    let action = data.action_trait_ident();
    let state = data.state_struct_ident();
    let event_enum = data.event_enum_ident();

    // TODO find entry state!!!
    let entry_state = format_ident!("winter");

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
    // TODO relative path handling
    let span = proc_macro::Span::call_site(); // Use the `Span` to get the `SourceFile` let source_file = span.local_file(); Get the path of the `SourceFile` let file_path.path();
    // print!("START");
    // let file_path = syn::parse_macro_input!(input as syn::LitStr).value();
    // dbg!(&file_path);
    // let abs_file_path = fs::canonicalize(file_path).unwrap();
    //
    // // TODO proper error formating
    // dbg!(&abs_file_path);
    // let contents = std::fs::read_to_string(&abs_file_path).expect("File not found");

    // INPUTS: TODO from file name or from  parsed content

    let fsm = parser::FsmRepr::simple_four_seasons();

    let module = fsm.module_ident();
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
