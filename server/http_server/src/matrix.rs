pub mod route;

use actix_web::web;

pub fn configure_matrix(config: &mut web::ServiceConfig) {
    let scope = web::scope("/_matrix").configure(route::client::configure_client);
    config.service(scope);
}
