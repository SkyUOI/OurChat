use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

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

    // Generate convert code for Option<PathBuf>
    let option_pathbuf_fields: Vec<_> = match &fields {
        Fields::Named(fields_named) => fields_named
            .named
            .iter()
            .filter_map(|field| {
                let field_name = field.ident.as_ref()?;

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

    let output = quote! {
        impl base::setting::PathConvert for #name {
            fn convert_to_abs_path(&mut self, full_basepath: &std::path::Path) -> anyhow::Result<()> {
                #(#field_conversions)*
                #(#option_field_conversions)*
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
