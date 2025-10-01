use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

/// Derive macro for generating Redis HSET and HGET operations for a struct.
///
/// This macro generates:
/// - A `hset_pipe` method that creates a Redis pipe with HSET commands for each field
/// - A `from_redis` async method that reads all fields from Redis and reconstructs the struct
///
/// # Example
///
/// ```ignore
/// #[derive(RedisHset)]
/// pub struct RoomInfo {
///     pub title: Option<String>,
///     pub room_id: RoomId,
///     pub users_num: u32,
///     pub auto_delete: bool,
/// }
///
/// // Writing to Redis:
/// let pipe = info.hset_pipe(&key);
/// pipe.query_async(&mut conn).await?;
///
/// // Reading from Redis:
/// let info = RoomInfo::from_redis(&mut conn, &key).await?;
/// ```
#[proc_macro_derive(RedisHset)]
pub fn redis_hset_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("RedisHset derive only supports structs with named fields"),
        },
        _ => panic!("RedisHset derive only supports structs"),
    };

    // Collect field identifiers and their string names
    let field_idents: Vec<_> = fields.iter().map(|field| &field.ident).collect();
    let field_name_strs: Vec<String> = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_string())
        .collect();
    let field_is_option: Vec<bool> = fields
        .iter()
        .map(|field| is_option_type(&field.ty))
        .collect();

    // Generate HSET calls for writing
    let hset_calls = field_idents
        .iter()
        .zip(field_name_strs.iter())
        .zip(field_is_option.iter())
        .map(|((ident, name_str), is_option)| {
            if *is_option {
                quote! {
                    if let Some(value) = &self.#ident {
                        pipe.hset(key, #name_str, value);
                    }
                }
            } else {
                quote! {
                    pipe.hset(key, #name_str, &self.#ident);
                }
            }
        });

    // Generate HGET calls for reading
    let hget_calls = field_name_strs.iter().map(|name_str| {
        quote! {
            pipe.hget(key, #name_str);
        }
    });

    // Generate field assignments for reading
    let field_assignments = field_idents.iter().map(|ident| {
        quote! {
            #ident: deadpool_redis::redis::FromRedisValue::from_redis_value(&values_iter.next().expect("number of values should match number of fields"))?
        }
    });

    let expanded = quote! {
        impl #name {
            /// Creates a Redis pipe with HSET commands for all fields of the struct.
            ///
            /// # Arguments
            ///
            /// * `key` - The Redis key for the hash
            ///
            /// # Returns
            ///
            /// A `deadpool_redis::redis::Pipeline` with all HSET commands chained.
            pub fn hset_pipe(&self, key: &str) -> deadpool_redis::redis::Pipeline {
                let mut pipe = deadpool_redis::redis::pipe();
                pipe.atomic();
                #(#hset_calls)*
                pipe
            }

            /// Reads all fields from Redis hash and reconstructs the struct.
            ///
            /// # Arguments
            ///
            /// * `conn` - A mutable reference to a Redis connection implementing AsyncCommands
            /// * `key` - The Redis key for the hash
            ///
            /// # Returns
            ///
            /// A `Result` containing the reconstructed struct or a Redis error.
            pub async fn from_redis(conn: &mut impl deadpool_redis::redis::AsyncCommands, key: &str) -> Result<Self, deadpool_redis::redis::RedisError> {
                use deadpool_redis::redis::FromRedisValue;
                let mut pipe = deadpool_redis::redis::pipe();
                #(#hget_calls)*
                let values: Vec<deadpool_redis::redis::Value> = pipe.query_async(conn).await?;
                let mut values_iter = values.into_iter();
                Ok(Self {
                    #(#field_assignments),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
    {
        return seg.ident == "Option";
    }
    false
}
