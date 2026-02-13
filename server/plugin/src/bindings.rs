//! WIT bindings for OurChat plugin system
//!
//! This module uses the wit-bindgen macro to generate bindings at compile time.

wit_bindgen::generate!({
    path: "wit/ourchat.wit",
});

// The bindings will be generated in a module named `ourchat`
// We'll re-export common types for convenience

