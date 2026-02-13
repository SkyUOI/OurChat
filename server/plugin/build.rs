/// Build script for generating WIT bindings
///
/// This script generates Rust bindings from WIT interface definitions
/// located in the wit/ directory.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=wit");

    // The WIT bindings are generated at compile time via the wit_bindgen! macro
    // in src/bindings.rs. This build script ensures the project rebuilds when
    // the WIT files change.
    println!("cargo:info=WIT bindings will be generated from wit/ourchat.wit");

    Ok(())
}
