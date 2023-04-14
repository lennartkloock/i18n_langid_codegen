extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn i18n(input: TokenStream) -> TokenStream {
    "".parse().unwrap()
}
