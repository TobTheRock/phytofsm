use proc_macro::TokenStream;
use proc_macro2::Span;

mod codegen;
mod error;
mod file;
mod parser;
#[cfg(test)]
mod test;

use crate::codegen::{FsmCodeGenerator, ident::Idents};

#[proc_macro]
pub fn generate_fsm(input: TokenStream) -> TokenStream {
    // let file_path = syn::parse_macro_input!(input as syn::LitStr).value();
    // dbg!(&file_path);
    // let abs_file_path = std::fs::canonicalize(file_path).unwrap();

    // TODO proper error formating
    // dbg!(&abs_file_path);q
    // let contents = std::fs::read_to_string(&abs_file_path).expect("File not found");
    let path = input.to_string();
    let file_path = file::FilePath::resolve(&path, proc_macro::Span::call_site());
    let file = file::FsmFile::try_open(file_path).expect("Failed to open FSM file");
    let parsed_fsm = file.try_parse().expect("Failed to parse FSM file");

    let generator = FsmCodeGenerator::default();
    let fsm_code = parsed_fsm.generate_from(generator);

    // TODO rm
    println!("{}", fsm_code);
    fsm_code.into()
}
