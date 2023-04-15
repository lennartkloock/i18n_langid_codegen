use proc_macro::TokenStream;
use std::{
    collections::{BTreeSet, HashSet},
    fs,
};
use std::collections::BTreeMap;
use proc_macro2::{Ident, Span};
use quote::quote;
use serde_yaml::Value;
use syn::{LitStr, parse_macro_input};

#[proc_macro]
pub fn i18n(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let dir = fs::read_dir(input.value()).expect("failed to read dir");
    let mut keys = BTreeSet::new();
    for locale in dir {
        let file = locale.expect("failed to get dir entry");
        let content = fs::read_to_string(file.path()).expect("failed to read file content");
        let parsed: serde_yaml::value::Mapping =
            serde_yaml::from_str(&content).expect("failed to parse file content as YAML");
        for key in parsed.keys() {
            keys.insert(Ident::new(
                key.as_str().expect("key was no string"),
                Span::call_site(),
            ));
        }
    }
    eprintln!("Keys: {keys:?}");
    let keys_iter = keys.iter();
    quote!(
        struct I18n {
            #(#keys_iter: ::core::option::Option<::std::string::String>),*
        }
    )
        .into()
}

fn gen_struct(name: &str, mapping: serde_yaml::Mapping) -> proc_macro2::TokenStream {
    let mut structs = vec![];
    let mut keys = BTreeMap::new();
    for (key, value) in mapping {
        match value {
            Value::String(_) => {
                keys.insert(Ident::new(
                    key.as_str().expect("key was no string"),
                    Span::call_site(),
                ), Ident::new("::core::option::Option<::std::string::String>", Span::call_site()));
            }
            Value::Mapping(m) => {
                let struct_name = key.as_str().expect("key was no string");
                structs.push(gen_struct(struct_name, m));
                keys.insert(Ident::new(
                    key.as_str().expect("key was no string"),
                    Span::call_site(),
                ), Ident::new(&format!("::core::option::Option<{}>", struct_name), Span::call_site()));
            }
            _ => panic!("value can only be string or mapping"),
        }
    }
    let keys_iter = keys.iter().map(|(k, v)| quote!(
        #k: #v
    ));
    quote!(
        #(#structs)*

        struct #name {
            #(#keys_iter),*
        }
    )
}
