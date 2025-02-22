use crate::Cfg;
use actix_web::{Responder, get, web};

#[get("/logo")]
pub async fn logo(config: web::Data<Cfg>) -> Result<impl Responder, actix_web::Error> {
    Ok(actix_files::NamedFile::open_async(&config.main_cfg.logo_path).await?)
}
