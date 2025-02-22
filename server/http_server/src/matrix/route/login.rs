use actix_web::{HttpResponse, Responder, get};

#[get("/login")]
async fn login() -> impl Responder {
    HttpResponse::Ok()
}
