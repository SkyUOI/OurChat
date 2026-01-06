//! Attribute macro for automatic test registration in stress test framework

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, parse_macro_input};

/// Attribute macro to automatically register a stress test at program startup
///
/// # Syntax
///
/// ```ignore
/// use derive::register_test;
///
/// #[register_test("Authentication Test", WithUsers)]
/// pub async fn test_auth(users: &UsersGroup, report: &mut Report) {
///     // test implementation
/// }
///
/// #[register_test("Get Server Info", AppOnly)]
/// pub async fn test_get_server_info(app: &mut client::ClientCore, report: &mut Report) {
///     // test implementation
/// }
/// ```
///
/// # Test Types
///
/// - `AppOnly`: Tests that only need the app (no users)
/// - `WithUsers`: Tests that need users registered (default)
/// - `WithSessions`: Tests that need sessions created
///
/// # How it works
///
/// This macro:
/// 1. Keeps the original function intact
/// 2. Generates a `#[ctor]` function that runs at program startup
/// 3. The ctor function automatically registers the test with the global registry
/// 4. Auto-infers test name from function name (`test_auth` → `auth`)
/// 5. Auto-detects the module name
///
/// **No `register_tests()` function needed - registration is automatic!**
pub fn register_test(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let function_name = &input_fn.sig.ident;
    let function_name_str = function_name.to_string();

    // Infer test name from function name (remove "test_" prefix if present)
    let test_name = if function_name_str.starts_with("test_") {
        function_name_str[5..].to_string()
    } else {
        function_name_str.clone()
    };

    // Parse the attribute arguments
    let args_str = args.to_string();
    let mut parts = args_str.split(',');
    let display_name = parts
        .next()
        .unwrap_or(&test_name)
        .trim()
        .trim_matches('"')
        .to_string();
    let test_type = parts
        .next()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "WithUsers".to_string());

    // Get visibility and other properties
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;

    // Remove our custom attribute from the output
    let clean_attrs: Vec<_> = input_fn
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("register_test"))
        .collect();

    // Create a unique identifier for the ctor function
    let ctor_fn_name = format_ident!("__register_test_ctor_{}", function_name);

    // Expand the macro
    let expanded = quote! {
        #(#clean_attrs)*
        #vis #sig {
            #block
        }

        // Generate a constructor function that runs at program startup
        // This automatically registers the test with the global registry
        #[doc(hidden)]
        #[ctor::ctor]
        fn #ctor_fn_name() {
            // Get module name from the current module path
            let module_name = module_path!()
                .split("::")
                .last()
                .unwrap_or("unknown")
                .to_string();

            // Map string to TestType
            let test_type = match #test_type {
                "AppOnly" => crate::tests::registry::TestType::AppOnly,
                "WithUsers" => crate::tests::registry::TestType::WithUsers,
                "WithSessions" => crate::tests::registry::TestType::WithSessions,
                _ => crate::tests::registry::TestType::WithUsers,
            };

            let test_info = crate::tests::registry::TestInfo::new(
                #test_name,
                #display_name,
                module_name.leak(), // leak for &'static str
                test_type,
            );

            // Register with the global registry
            let mut registry = crate::tests::registry::registry_mut().lock().unwrap();
            registry.register(test_info);
        }
    };

    TokenStream::from(expanded)
}
