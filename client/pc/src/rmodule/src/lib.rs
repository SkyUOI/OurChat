use pyo3::prelude::*;

fn get_version_msg() -> String {
    format!(
        "{}({}+{})",
        base::build::PKG_VERSION,
        base::build::BRANCH,
        base::build::COMMIT_HASH
    )
}

fn get_version_msg_details() -> &'static str {
    base::build::VERSION
}

/// A Python module implemented in Rust.
#[pymodule]
fn rmodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("version", get_version_msg())?;
    m.add("version_details", get_version_msg_details())?;
    Ok(())
}
