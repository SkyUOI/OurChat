use std::path::{Path, PathBuf};

/// 将相对路径转换为相对于基准路径的完整路径
pub fn resolve_relative_path<P: AsRef<Path>>(
    base_path: P,
    relative_path: &Path,
) -> std::io::Result<PathBuf> {
    let base_path = base_path.as_ref();
    let mut path_buf = PathBuf::from(base_path);
    // 将相对路径合并到基准路径
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
    ($name:ident, $type:ty) => {
        $crate::impl_newtype!($name, $type, #[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]);
    };
}
