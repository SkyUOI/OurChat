use actix_web::{HttpResponse, Responder, get};

#[get("/timestamp")]
async fn timestamp() -> impl Responder {
    HttpResponse::Ok().body(chrono::Utc::now().to_rfc3339())
}
