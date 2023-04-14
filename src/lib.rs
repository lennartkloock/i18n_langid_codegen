extern crate proc_macro;
use proc_macro::TokenStream;
use std::fs;
use std::path::{Path, PathBuf};
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn i18n(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Path);
    let dir = fs::read_dir(input).expect("failed to read dir");
    let mut keys = vec![];
    for locale in dir {
        let file_path = locale.expect("failed to get dir entry");
        let content = fs::read_to_string(file_path).expect("failed to read file content");
        let parsed: serde_yaml::value::Mapping = serde_yaml::from_str(&content).expect("failed to parse file content as YAML");
        for key in parsed.keys() {
            keys.push(key.as_str().expect("key was no string"));
        }
    }
    quote!(
        struct I18n {
            #(#keys: String),*
        }
    ).into()
}
