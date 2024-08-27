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
