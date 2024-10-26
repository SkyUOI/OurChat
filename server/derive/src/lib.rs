use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse2, parse_macro_input, Block, Ident, Item, Stmt, UseTree};

fn db_replace(stmts: Vec<Stmt>, path: syn::Ident) -> Vec<Stmt> {
    let mut output = vec![];
    stmts.into_iter().for_each(|stmt| {
        if let Stmt::Item(Item::Use(use_item)) = &stmt {
            if let UseTree::Path(ref item_use) = use_item.tree {
                if item_use.ident == "entities" {
                    let tree = &item_use.tree;
                    output.push(syn::parse_quote! {
                        use crate::entities::#path::#tree;
                    });
                    return;
                }
            }
        }
        output.push(stmt);
    });
    output
}

#[proc_macro_attribute]
pub fn db_compatibility(_attr: TokenStream, tok: TokenStream) -> TokenStream {
    let mut funcbody = parse_macro_input!(tok as syn::ItemFn);
    let sqlite = db_replace(
        funcbody.block.stmts.clone(),
        Ident::new("sqlite", Span::call_site()),
    );
    let postgres = db_replace(
        funcbody.block.stmts.clone(),
        Ident::new("postgres", Span::call_site()),
    );
    let ret: Block = parse2(quote! {
        {
            if static_keys::static_branch_unlikely!(crate::db::SQLITE_TYPE) {
                #(#sqlite)*
            } else {
                #(#postgres)*
            }
        }
    })
    .unwrap();
    funcbody.block = Box::new(ret);
    funcbody
        .attrs
        .push(syn::parse_quote! {#[allow(clippy::useless_conversion)]});
    quote! {#funcbody}.into()
}

#[proc_macro_attribute]
pub fn db_entities_from(_attr: TokenStream, tok: TokenStream) -> TokenStream {
    let mut impl_body = parse_macro_input!(tok as syn::ItemImpl);
    todo!()
}
