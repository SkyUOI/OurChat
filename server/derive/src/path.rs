use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Type, parse_macro_input};

use crate::helper::is_option_type;

pub fn path_convert_derive_internal(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => panic!("Only accept struct"),
    };

    // Collect all PathBuf fields
    let pathbuf_fields: Vec<_> = match &fields {
        Fields::Named(fields_named) => {
            fields_named
                .named
                .iter()
                .filter_map(|field| {
                    let field_name = field.ident.as_ref()?;

                    // Skip if marked with #[path_convert]
                    if has_path_convert_attr(&field.attrs) {
                        return None;
                    }

                    // check if it is PathBuf
                    if is_pathbuf_type(&field.ty) {
                        Some((field_name, field.ty.clone()))
                    } else {
                        None
                    }
                })
                .collect()
        }
        Fields::Unnamed(_) => vec![], // tuple struct is not supported now
        Fields::Unit => vec![],       // no field in unit struct
    };

    // Generate convert code for PathBuf
    let field_conversions: Vec<_> = pathbuf_fields
        .iter()
        .map(|(field_name, _field_type)| {
            quote! {
                self.#field_name = utils::resolve_relative_path(full_basepath, &self.#field_name)?;
            }
        })
        .collect();

    // Collect all Option<PathBuf> fields
    let option_pathbuf_fields: Vec<_> = match &fields {
        Fields::Named(fields_named) => fields_named
            .named
            .iter()
            .filter_map(|field| {
                let field_name = field.ident.as_ref()?;

                // Skip if marked with #[path_convert]
                if has_path_convert_attr(&field.attrs) {
                    return None;
                }

                if is_option_pathbuf_type(&field.ty) {
                    Some(field_name)
                } else {
                    None
                }
            })
            .collect(),
        Fields::Unnamed(_) => vec![],
        Fields::Unit => vec![],
    };

    let option_field_conversions: Vec<_> = option_pathbuf_fields
        .iter()
        .map(|field_name| {
            quote! {
                self.#field_name = match &self.#field_name {
                    Some(path) => Some(utils::resolve_relative_path(full_basepath, path)?),
                    None => None,
                };
            }
        })
        .collect();

    // Collect all fields marked with #[path_convert]
    let path_convert_fields: Vec<_> = match &fields {
        Fields::Named(fields_named) => fields_named
            .named
            .iter()
            .filter_map(|field| {
                let field_name = field.ident.as_ref()?;

                // Only include if marked with #[path_convert]
                if has_path_convert_attr(&field.attrs) {
                    Some(field_name)
                } else {
                    None
                }
            })
            .collect(),
        Fields::Unnamed(_) => vec![],
        Fields::Unit => vec![],
    };

    // Generate conversion code for #[path_convert] fields
    let path_convert_field_conversions: Vec<_> = path_convert_fields
        .iter()
        .map(|field_name| {
            quote! {
                self.#field_name.convert_to_abs_path(full_basepath)?;
            }
        })
        .collect();

    // Collect all Option<T> fields marked with #[path_convert] where T: PathConvert
    let option_path_convert_fields: Vec<_> = match &fields {
        Fields::Named(fields_named) => fields_named
            .named
            .iter()
            .filter_map(|field| {
                let field_name = field.ident.as_ref()?;

                // Only include if marked with #[path_convert] and is Option<T>
                if has_path_convert_attr(&field.attrs) && is_option_type(&field.ty) {
                    Some(field_name)
                } else {
                    None
                }
            })
            .collect(),
        Fields::Unnamed(_) => vec![],
        Fields::Unit => vec![],
    };

    // Generate conversion code for Option<T> fields marked with #[path_convert]
    let option_path_convert_field_conversions: Vec<_> = option_path_convert_fields
        .iter()
        .map(|field_name| {
            quote! {
                if let Some(ref mut value) = self.#field_name {
                    value.convert_to_abs_path(full_basepath)?;
                }
            }
        })
        .collect();

    let output = quote! {
        impl base::setting::PathConvert for #name {
            fn convert_to_abs_path(&mut self, full_basepath: &std::path::Path) -> anyhow::Result<()> {
                #(#field_conversions)*
                #(#option_field_conversions)*
                #(#path_convert_field_conversions)*
                #(#option_path_convert_field_conversions)*
                Ok(())
            }
        }
    };

    output.into()
}

/// Check if the type is PathBuf
fn is_pathbuf_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            segment.ident == "PathBuf"
        } else {
            false
        }
    } else {
        false
    }
}

/// Check if the type is Option<PathBuf>
fn is_option_pathbuf_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return is_pathbuf_type(inner_ty);
    }
    false
}

/// Check if a field has the #[path_convert] attribute
fn has_path_convert_attr(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| attr.path().is_ident("path_convert"))
}
