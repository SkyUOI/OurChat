use actix_web::{HttpResponse, Responder, get};

#[get("/status")]
pub async fn status() -> impl Responder {
    HttpResponse::Ok()
}
