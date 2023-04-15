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
use unic_langid::LanguageIdentifier;

#[proc_macro]
pub fn i18n(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let default = read_dir(input.value())
        .find(|d| d.file_name().to_string_lossy().ends_with(".default.yml"))
        .expect("failed to find default language, there needs to be a file called '<langid>.default.yml'");
    let content = fs::read_to_string(default.path()).expect("failed to read file content");
    let default_mapping: serde_yaml::Mapping =
        serde_yaml::from_str(&content).expect("failed to parse file content as YAML");
    let structs = gen_struct(None, &default_mapping);
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
        let id = file_prefix(locale.path())
            .parse()
            .expect("failed to parse prefix in file name as langid");
        fns.push(gen_fn(&id, &default_mapping, &parsed));
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
        .filter(|d| {
            d.path()
                .extension()
                .map(|e| e.to_string_lossy() == "yml")
                .unwrap_or(false)
        })
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

fn gen_struct(name: Option<&Ident>, mapping: &serde_yaml::Mapping) -> proc_macro2::TokenStream {
    let mut structs = vec![];
    let mut keys = vec![];
    if name.is_none() {
        keys.push(quote!(lang_id: ::unic_langid::LanguageIdentifier));
    }
    for (key, value) in mapping {
        match value {
            Value::String(_) => {
                let key = format_ident!("{}", key.as_str().expect("key was no string"));
                keys.push(quote!(
                    #key: &'static str
                ));
            }
            Value::Mapping(m) => {
                let key = key.as_str().expect("key was not a string");
                let key_ident = format_ident!("{}", key);
                let struct_name = format_ident!("{}", capitalize(key));
                structs.push(gen_struct(Some(&struct_name), m));
                keys.push(quote!(
                    #key_ident: #struct_name
                ));
            }
            _ => panic!("value can only be string or mapping"),
        }
    }
    let default_name = format_ident!("{}", "I18n");
    let struct_name = name.unwrap_or(&default_name);
    quote!(
        #(#structs)*
        pub struct #struct_name {
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

fn gen_fn(
    lang: &LanguageIdentifier,
    default_mapping: &serde_yaml::Mapping,
    mapping: &serde_yaml::Mapping,
) -> proc_macro2::TokenStream {
    let construct = gen_construct(
        &format_ident!("{}", "Self"),
        Some(lang),
        default_mapping,
        mapping,
    );
    let lang_ident = format_ident!("{}", lang.to_string());
    quote!(
        pub fn #lang_ident() -> Self {
            #construct
        }
    )
}

fn gen_construct(
    ident: &Ident,
    lang_id: Option<&LanguageIdentifier>,
    default_mapping: &serde_yaml::Mapping,
    mapping: &serde_yaml::Mapping,
) -> proc_macro2::TokenStream {
    let mut values = vec![];
    if let Some(lang_id) = lang_id {
        let str = lang_id.to_string();
        values.push(quote!(
            lang_id: ::std::str::FromStr::from_str(#str).unwrap()
        ));
    }
    for (key, default_value) in default_mapping {
        let key = key.as_str().expect("failed to convert key to string");
        let key_ident = format_ident!("{}", key);
        if let Some(value) = mapping.get(key) {
            match default_value {
                Value::String(_) => {
                    let s = value.as_str().expect("failed to get value as string");
                    values.push(quote!(
                        #key_ident: #s
                    ));
                }
                Value::Mapping(m) => {
                    let construct = gen_construct(
                        &format_ident!("{}", capitalize(key)),
                        None,
                        m,
                        value.as_mapping().expect("failed to get value as mapping"),
                    );
                    values.push(quote!(
                        #key_ident: #construct
                    ));
                }
                _ => panic!("value can only be string or mapping"),
            }
        } else {
            match default_value {
                Value::String(s) => {
                    values.push(quote!(
                        #key_ident: #s
                    ));
                }
                Value::Mapping(m) => {
                    let construct =
                        gen_construct(&format_ident!("{}", capitalize(key)), None, m, m);
                    values.push(quote!(
                        #key_ident: #construct
                    ));
                }
                _ => panic!("value can only be string or mapping"),
            }
        }
    }
    quote!(
        #ident {
            #(#values),*
        }
    )
}
