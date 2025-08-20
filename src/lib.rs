use proc_macro::TokenStream;
use quote::quote;

mod codegen;
mod error;
mod file;
mod parser;
#[cfg(test)]
mod test;

use crate::codegen::FsmCodeGenerator;

#[proc_macro]
pub fn generate_fsm(input: TokenStream) -> TokenStream {
    match generate_fsm_inner(input) {
        Ok(tokens) => tokens,
        Err(error) => {
            let error_msg = error.to_string();
            quote! {
                compile_error!(#error_msg);
            }
            .into()
        }
    }
}

fn generate_fsm_inner(input: TokenStream) -> error::Result<TokenStream> {
    let path = input.to_string();
    let file_path = file::FilePath::resolve(&path, proc_macro::Span::call_site());
    let file = file::FsmFile::try_open(file_path)?;
    let parsed_fsm = parser::ParsedFsm::try_parse(file.content())?;
    let generator = FsmCodeGenerator::default();
    let fsm_code = generator.generate(parsed_fsm);

    Ok(fsm_code.into())
}
