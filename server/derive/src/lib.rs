use proc_macro::TokenStream;

mod helper;
mod path;
mod stress_test;

#[proc_macro_derive(PathConvert, attributes(path_convert))]
pub fn path_convert_derive(input: TokenStream) -> TokenStream {
    path::path_convert_derive_internal(input)
}

/// Attribute macro for automatic stress test registration
#[proc_macro_attribute]
pub fn register_test(args: TokenStream, input: TokenStream) -> TokenStream {
    stress_test::register_test(args, input)
}
