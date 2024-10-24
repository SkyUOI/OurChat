use crate::server::httpserver::FileUploadMetadata;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};

#[get("/download")]
pub async fn download(req: HttpRequest, key: web::Form<FileUploadMetadata>) -> impl Responder {
    let _key = key.into_inner().key;
    HttpResponse::Ok()
}
