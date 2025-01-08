mod http;
mod verify;

#[ctor::ctor]
fn init() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}
