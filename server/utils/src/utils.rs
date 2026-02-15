use std::path::{Path, PathBuf};

/// Change the relative path to the full path relative to the base path
pub fn resolve_relative_path<P: AsRef<Path>>(
    base_path: P,
    relative_path: &Path,
) -> std::io::Result<PathBuf> {
    let base_path = base_path.as_ref();
    let mut path_buf = PathBuf::from(base_path);
    // merge the relative path to the base path
    path_buf.push(relative_path);
    Ok(path_buf)
}

#[macro_export]
macro_rules! impl_newtype {
    ($name:ident, $type:ty, $($derive:tt)*) => {
        $($derive)*
        pub struct $name(pub $type);
        impl std::ops::Deref for $name {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

#[macro_export]
macro_rules! impl_newtype_int {
    ($name:ident, $type:ty, $($derive:tt)*) => {
        $crate::impl_newtype!($name, $type, #[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, $($derive)*)]);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = std::num::ParseIntError;
            fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(Self(s.parse()?)) }
        }
    };
}

#[macro_export]
macro_rules! impl_newtype_string {
    ($name:ident, $($derive:tt)*) => {
        $crate::impl_newtype!($name, String, #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, $($derive)*)]);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    }
}

pub macro impl_redis_value_from_for_newint($name:ident) {
    impl deadpool_redis::redis::ToRedisArgs for $name {
        fn write_redis_args<W>(&self, out: &mut W)
        where
            W: ?Sized + deadpool_redis::redis::RedisWrite,
        {
            self.0.write_redis_args(out)
        }
    }

    impl deadpool_redis::redis::FromRedisValue for $name {
        fn from_redis_value(
            v: &deadpool_redis::redis::Value,
        ) -> deadpool_redis::redis::RedisResult<Self> {
            Ok($name(
                deadpool_redis::redis::FromRedisValue::from_redis_value(v)?,
            ))
        }
    }
}

pub fn oaep_padding() -> rsa::Oaep<rsa::sha2::Sha256> {
    rsa::Oaep::<rsa::sha2::Sha256>::new()
}

pub fn merge_json(origin: serde_json::Value, new: serde_json::Value) -> serde_json::Value {
    match (origin, new) {
        // If both are objects, merge them recursively
        (serde_json::Value::Object(mut origin_map), serde_json::Value::Object(new_map)) => {
            for (key, new_value) in new_map {
                match origin_map.get_mut(&key) {
                    Some(origin_value) => {
                        // Recursively merge values with the same key
                        *origin_value = merge_json(origin_value.clone(), new_value);
                    }
                    None => {
                        // If the key doesn't exist in the original object, insert it
                        origin_map.insert(key, new_value);
                    }
                }
            }
            serde_json::Value::Object(origin_map)
        }
        // For all other cases, use the new value
        (_, new) => new,
    }
}

pub macro serde_default($name:ty) {
    impl Default for $name {
        fn default() -> Self {
            let empty = serde_json::json!({});
            // It use serde's default value. It must be a valid value. It is safe to unwrap
            serde_json::from_value(empty).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impl_newtype() {
        impl_newtype!(T1, String, #[derive(Eq, PartialEq)]);
        let a = T1("hello".to_string());
        assert_eq!(*a, "hello");
    }

    #[test]
    fn test_impl_newtype_int() {
        impl_newtype_int!(T1, i32,);
        let a = T1(1);
        assert_eq!(*a, 1);
        println!("{}{}", a, *a + 1);
    }

    #[test]
    fn test_resolve_relative_path() {
        let base_path = Path::new("/home/limuy");
        let relative_path = Path::new("a/b/c");
        let path = resolve_relative_path(base_path, relative_path).unwrap();
        assert_eq!(path, Path::new("/home/limuy/a/b/c"));
    }

    #[test]
    fn test_merge_json() {
        use serde_json::json;

        // Test basic merging
        let origin = json!({
            "name": "Alice",
            "age": 30,
            "address": {
                "city": "Beijing",
                "street": "Main St"
            }
        });

        let new = json!({
            "age": 31,
            "address": {
                "street": "Second St",
                "zip": "100000"
            },
            "phone": "123456"
        });

        let merged = merge_json(origin, new);

        assert_eq!(
            merged,
            json!({
                "name": "Alice",
                "age": 31,
                "address": {
                    "city": "Beijing",
                    "street": "Second St",
                    "zip": "100000"
                },
                "phone": "123456"
            })
        );

        // Test array handling
        let origin = json!({
            "tags": ["a", "b", "c"],
            "data": 1
        });

        let new = json!({
            "tags": ["d", "e"],
            "data": 2
        });

        let merged = merge_json(origin, new);

        assert_eq!(
            merged,
            json!({
                "tags": ["d", "e"],
                "data": 2
            })
        );
    }
}
