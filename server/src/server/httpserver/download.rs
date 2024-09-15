use crate::server::httpserver::KEY;
use actix_web::{get, HttpRequest, HttpResponse, Responder};

#[get("/download")]
pub async fn download(req: HttpRequest) -> impl Responder {
    let key = match req.headers().get(KEY).and_then(|key| key.to_str().ok()) {
        None => {
            return HttpResponse::BadRequest();
        }
        Some(key) => key,
    };
    HttpResponse::Ok()
}
