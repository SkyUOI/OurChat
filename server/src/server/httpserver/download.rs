use crate::server::httpserver::KEY;
use actix_web::{HttpRequest, HttpResponse, Responder, get};

#[get("/download")]
pub async fn download(req: HttpRequest) -> impl Responder {
    let _key = match req.headers().get(KEY).and_then(|key| key.to_str().ok()) {
        None => {
            return HttpResponse::BadRequest();
        }
        Some(key) => key,
    };
    HttpResponse::Ok()
}
