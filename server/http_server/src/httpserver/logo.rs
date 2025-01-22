use actix_web::{HttpResponse, Responder, get};

#[get("/logo")]
pub async fn logo() -> impl Responder {
    HttpResponse::Ok()
}
