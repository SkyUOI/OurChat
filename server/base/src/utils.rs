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
    };
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
}
