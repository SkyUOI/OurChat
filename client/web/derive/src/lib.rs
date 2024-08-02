use proc_macro::TokenStream;
use quote::quote;
use std::{fs, str::FromStr};
use syn::parse_macro_input;

#[proc_macro]
pub fn file_html(tok: TokenStream) -> TokenStream {
    let path = parse_macro_input!(tok as syn::LitStr);
    let path = path.value();
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let path = manifest_dir.join(path);
    println!("path:{}", path.display());
    let html_file =
        proc_macro2::TokenStream::from_str(&fs::read_to_string(&path).expect("path is no exists"))
            .unwrap();
    let ret = quote! {
        yew::html! {
            #html_file
        }
    }
    .into();
    println!("ret:{}", ret);
    ret
}
