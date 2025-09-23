pub mod defines;
pub mod route;

pub fn configure_matrix() -> axum::Router {
    axum::Router::new().nest("/client", route::client::configure_route())
}
