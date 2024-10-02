use actix_web::{get, HttpResponse, Responder};

#[get("/status")]
pub async fn status() -> impl Responder {
    HttpResponse::Ok()
}
