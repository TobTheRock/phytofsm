use proc_macro::TokenStream;

mod codegen;
mod error;
mod parser;
#[cfg(test)]
mod test;

use crate::{
    codegen::{FsmCodeGenerator, ident::Idents},
    parser::FsmFile,
};

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
    let parsed_fsm = file.try_parse().expect("Failed to parse FSM file");

    let generator = FsmCodeGenerator::default();
    let fsm_code = parsed_fsm.generate_from(generator);

    // TODO rm
    println!("{}", fsm_code);
    fsm_code.into()
}
