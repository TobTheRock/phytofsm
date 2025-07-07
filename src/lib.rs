use heck::ToSnakeCase;
use heck::ToUpperCamelCase;
use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::Ident;

mod error;
mod parser;
#[cfg(test)]
mod reference;
#[cfg(test)]
mod test_data;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Event(String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Action(String);

struct Transition {
    pub from_state: String,
    pub event: Event,
    pub to_state: String,
    pub action: Option<String>,
}

// TODO replace with interface
struct ParsedFsmData {
    pub fsm_name: String,
    pub transitions: Vec<Transition>,
}

impl Event {
    pub fn params_ident(&self) -> Ident {
        format_ident!("{}Params", self.0.to_upper_camel_case())
    }

    pub fn ident(&self) -> Ident {
        Ident::new(&self.0.to_upper_camel_case(), Span::call_site())
    }
}

impl Action {
    pub fn ident(&self) -> Ident {
        Ident::new(&self.0.to_snake_case(), Span::call_site())
    }
}

impl Transition {
    // TODO
    pub fn to_state_ident(&self) -> Ident {
        Ident::new(&self.to_state, Span::call_site())
    }
    pub fn from_state_ident(&self) -> Ident {
        Ident::new(&self.from_state, Span::call_site())
    }
}

impl ParsedFsmData {
    pub fn fsm_ident(&self) -> Ident {
        Ident::new(&self.fsm_name.to_upper_camel_case(), Span::call_site())
    }

    pub fn module_ident(&self) -> Ident {
        Ident::new(&self.fsm_name.to_snake_case(), Span::call_site())
    }

    pub fn event_params_trait_ident(&self) -> Ident {
        format_ident!("I{}EventParams", self.fsm_name.to_upper_camel_case())
    }

    pub fn event_enum_ident(&self) -> Ident {
        format_ident!("{}Event", self.fsm_name.to_upper_camel_case())
    }
    pub fn action_trait_ident(&self) -> Ident {
        format_ident!("I{}Actions", self.fsm_name.to_upper_camel_case())
    }

    // TODO move to parser mod
    pub fn all_events(&self) -> impl Iterator<Item = &Event> {
        self.transitions.iter().map(|t| &t.event).unique()
    }

    // pub fn state_idents(&self) -> Vec<Ident> {
    //     self.transitions
    //         .iter()
    //         .flat_map(|t| [&t.from_state, &t.to_state])
    //         .unique()
    //         .map(|name| Ident::new(&name.to_upper_camel_case(), Span::call_site()))
    //         .collect()
    // }
}

fn fsm_event_params_trait(data: &ParsedFsmData) -> TokenStream2 {
    let trait_ident = data.event_params_trait_ident();
    let associated_types = data.all_events().map(|event| {
        let type_ident = event.params_ident();
        quote! { type #type_ident; }
    });

    quote! {
        pub trait #trait_ident {
            #(#associated_types)*
        }
    }
}

fn fsm_actions_trait(data: &ParsedFsmData) -> TokenStream2 {
    let action_methods = data.transitions.iter().filter_map(|transition| {
        if let Some(action) = &transition.action {
            let action_ident = Ident::new(&action.to_snake_case(), Span::call_site());
            let params_ident = transition.event.params_ident();
            let action_method = quote! {
                fn #action_ident(&mut self, params: Self::#params_ident);
            };
            Some(action_method)
        } else {
            None
        }
    });
    let event_params_trait = data.event_params_trait_ident();
    let trait_ident = data.action_trait_ident();

    quote! {
        pub trait #trait_ident : #event_params_trait{
            #(#action_methods)*
        }
    }
}

fn event_enum(data: &ParsedFsmData) -> TokenStream2 {
    let event_variants = data.all_events().map(|event| {
        let params_ident = event.params_ident();
        let event_ident = event.ident();
        quote! { #event_ident(P::#params_ident),}
    });
    let event_enum_ident = data.event_enum_ident();
    let action_ident = data.action_trait_ident();
    quote! {
        pub enum #event_enum_ident<P: #action_ident> {
            #(#event_variants)*
        }
    }
}

// pub enum PlantFsmEvent<T: IPlantFsmActions> {
//     TemperatureRises(T::TemperatureRisesParams),
//     DaylightIncreases(T::DaylightIncreasesParams),
//     TemperatureDrops(T::TemperatureDropsParams),
//     DaylightDecreases(T::DaylightDecreasesParams),
// }

#[proc_macro]
pub fn generate_fsm(input: TokenStream) -> TokenStream {
    // TODO relative path handling
    // let span = proc_macro::Span::call_site();

    // Use the `Span` to get the `SourceFile`
    // let source_file = span.source_file();
    // Get the path of the `SourceFile`
    // let file_path.path();

    //
    // print!("START");
    // let file_path = syn::parse_macro_input!(input as syn::LitStr).value();
    // dbg!(&file_path);
    // let abs_file_path = fs::canonicalize(file_path).unwrap();
    //
    // // TODO proper error formating
    // dbg!(&abs_file_path);
    // let contents = std::fs::read_to_string(&abs_file_path).expect("File not found");

    // INPUTS: TODO from file name or from  parsed content
    let module_name = "plant_fsm";
    let fsm_name = "PlantFsm";
    // TODO from parser
    let event_names = vec![
        "TemperatureRises",
        "DaylightIncreases",
        "DaylightDecreases",
        "TemperatureDrops",
    ];

    let fsm_data = ParsedFsmData {
        fsm_name: fsm_name.to_string(),
        transitions: vec![
            Transition {
                from_state: "Winter".to_string(),
                event: Event("TemperatureRises".to_string()),
                to_state: "Spring".to_string(),
                action: None,
            },
            Transition {
                from_state: "Spring".to_string(),
                event: Event("DaylightIncreases".to_string()),
                to_state: "Summer".to_string(),
                action: Some("StartBlooming".to_string()),
            },
            Transition {
                from_state: "Summer".to_string(),
                event: Event("DaylightDecreases".to_string()),
                to_state: "Autumn".to_string(),
                action: Some("RipenFruit".to_string()),
            },
            Transition {
                from_state: "Autumn".to_string(),
                event: Event("TemperatureDrops".to_string()),
                to_state: "Winter".to_string(),
                action: Some("DropPetals".to_string()),
            },
        ],
    };

    let event_params_trait = fsm_event_params_trait(&fsm_data);
    let action_trait = fsm_actions_trait(&fsm_data);
    let event_enum = event_enum(&fsm_data);

    let mod_ident = format_ident!("{}", module_name);
    let fsm_code = quote! {
        mod #mod_ident {
            pub type NoEventData = ();

            #event_params_trait
            #action_trait
            #event_enum
        }
    };
    println!("{}", fsm_code);

    // event_params_trait.into()
    fsm_code.into()
}
