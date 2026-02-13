// Test file to see what's generated
wit_bindgen::generate!({
    path: "../plugin/wit/ourchat.wit",
});

fn main() {
    // Try to see what's available
    let _ = exports::ourchat::plugin::hooks::HookResult::Continue;
}
