use actix_web::{HttpResponse, Responder, get, http::header::ContentType};

#[get("/limuy")]
pub async fn easter_egg() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("easter_egg.html"))
}
