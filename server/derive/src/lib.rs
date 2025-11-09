use proc_macro::TokenStream;

mod helper;
mod path;
mod redis;

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
    redis::redis_hset_derive_internal(input)
}

#[proc_macro_derive(PathConvert, attributes(path_convert))]
pub fn path_convert_derive(input: TokenStream) -> TokenStream {
    path::path_convert_derive_internal(input)
}
