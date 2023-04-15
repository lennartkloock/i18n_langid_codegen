use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use serde_yaml::Value;
use std::{
    fs,
    fs::DirEntry,
    path::{Path, PathBuf},
};
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn i18n(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let default = read_dir(input.value())
        .find(|d| d.file_name().to_string_lossy().contains("default"))
        .expect("failed to find default language, there needs to be a default lang file");
    let content = fs::read_to_string(default.path()).expect("failed to read file content");
    let default_mapping: serde_yaml::Mapping =
        serde_yaml::from_str(&content).expect("failed to parse file content as YAML");
    let structs = gen_struct(&format_ident!("{}", "I18n"), default_mapping);
    let default_lang = format_ident!("{}", file_prefix(default.path()));
    let default_impl = quote!(
        impl ::std::default::Default for I18n {
            fn default() -> Self {
                Self::#default_lang()
            }
        }
    );

    let mut fns = vec![];
    for locale in read_dir(input.value()) {
        let content = fs::read_to_string(locale.path()).expect("failed to read file content");
        let parsed: serde_yaml::Mapping =
            serde_yaml::from_str(&content).expect("failed to parse file content as YAML");
        fns.push(gen_fn(&file_prefix(locale.path()), &parsed));
    }

    quote!(
        #structs

        impl I18n {
            #(#fns)*
        }

        #default_impl
    )
    .into()
}

fn read_dir<P: AsRef<Path>>(path: P) -> impl Iterator<Item = DirEntry> {
    fs::read_dir(path)
        .expect("failed to read dir")
        .map(|res| res.expect("failed to get dir entry"))
}

fn file_prefix(path: PathBuf) -> String {
    let str = path
        .file_name()
        .expect("failed to get file name")
        .to_str()
        .expect("failed to convert os string to string");
    let mut split = str.split('.');
    match split.next() {
        None => str,
        Some(s) => s,
    }
    .to_string()
}

fn gen_struct(name: &Ident, mapping: serde_yaml::Mapping) -> proc_macro2::TokenStream {
    let mut structs = vec![];
    let mut keys = vec![];
    for (key, value) in mapping {
        match value {
            Value::String(_) => {
                let key = format_ident!("{}", key.as_str().expect("key was no string"));
                keys.push(quote!(
                    #key: ::std::string::String
                ));
            }
            Value::Mapping(m) => {
                let key = key.as_str().expect("key was not a string");
                let key_ident = format_ident!("{}", key);
                let struct_name = format_ident!("{}", capitalize(key));
                structs.push(gen_struct(&struct_name, m));
                keys.push(quote!(
                    #key_ident: #struct_name
                ));
            }
            _ => panic!("value can only be string or mapping"),
        }
    }
    quote!(
        #(#structs)*
        struct #name {
            #(pub #keys),*
        }
    )
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn gen_fn(lang: &str, mapping: &serde_yaml::Mapping) -> proc_macro2::TokenStream {
    let construct = gen_construct(&format_ident!("{}", "Self"), mapping);
    let lang_ident = format_ident!("{}", lang);
    quote!(
        pub fn #lang_ident() -> Self {
            #construct
        }
    )
}

fn gen_construct(ident: &Ident, mapping: &serde_yaml::Mapping) -> proc_macro2::TokenStream {
    let mut values = vec![];
    for (key, value) in mapping {
        let key = key.as_str().expect("failed to convert key to string");
        let key_ident = format_ident!("{}", key);
        match value {
            Value::String(s) => {
                values.push(quote!(
                    #key_ident: #s
                ));
            }
            Value::Mapping(m) => {
                let construct = gen_construct(&format_ident!("{}", capitalize(key)), m);
                values.push(quote!(
                    #key_ident: #construct
                ));
            }
            _ => panic!(),
        }
    }
    quote!(
        #ident {
            #(#values),*
        }
    )
}
