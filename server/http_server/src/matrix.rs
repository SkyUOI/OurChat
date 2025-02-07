pub mod defines;
pub mod route;

use actix_web::web;

pub fn configure_matrix(config: &mut web::ServiceConfig) {
    let scope = web::scope("/_matrix").configure(route::client::configure_route);
    config
        .service(scope)
        .configure(route::wellknown::configure_route);
}
