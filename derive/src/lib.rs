use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse2, parse_macro_input, Block, Ident, Item, Stmt, UseTree};

fn db_replace(stmts: Vec<Stmt>, path: syn::Ident) -> Vec<Stmt> {
    let mut output = vec![];
    stmts.into_iter().for_each(|stmt| {
        if let syn::Stmt::Item(Item::Use(use_item)) = &stmt {
            if let UseTree::Path(ref item_use) = use_item.tree {
                if item_use.ident == "entities" {
                    let tree = &item_use.tree;
                    output.push(
                        syn::parse2(quote! {
                            use crate::entities::#path::#tree;
                        })
                        .unwrap(),
                    );
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
    let mut output: Vec<Block> = vec![];
    let mysql = db_replace(
        funcbody.block.stmts.clone(),
        Ident::new("mysql", Span::call_site()),
    );
    let sqlite = db_replace(
        funcbody.block.stmts.clone(),
        Ident::new("sqlite", Span::call_site()),
    );
    let ret: Block = parse2(quote! {
        {
            if static_keys::static_branch_unlikely!(crate::db::SQLITE_TYPE) {
                #(#sqlite)*
            }else if static_keys::static_branch_unlikely!(crate::db::MYSQL_TYPE) {
                #(#mysql)*
            }
        }
    })
    .unwrap();
    funcbody.block = Box::new(ret);
    let ret = quote! {#funcbody}.into();
    // println!("{}", ret);
    ret
}
