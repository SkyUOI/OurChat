use actix_web::{HttpResponse, get, web};
use parking_lot::RwLock;
use std::{path::Path, sync::OnceLock, time::SystemTime};
use tokio::fs::read;

use crate::Cfg;

struct LogoCache {
    data: Vec<u8>,
    time: SystemTime,
    imginfo: imageinfo::ImageInfo,
}

impl LogoCache {
    fn new(data: Vec<u8>) -> anyhow::Result<Self> {
        let img = imageinfo::ImageInfo::from_raw_data(data.as_slice())?;
        Ok(Self {
            data,
            time: SystemTime::now(),
            imginfo: img,
        })
    }

    fn update_logo(&mut self, file_path: &Path) -> anyhow::Result<()> {
        let data = std::fs::read(file_path)?;
        let img = imageinfo::ImageInfo::from_raw_data(&data)?;
        self.data = data;
        self.time = SystemTime::now();
        self.imginfo = img;
        Ok(())
    }

    fn detect_update(&self, file_path: &Path) -> std::io::Result<bool> {
        let meta = std::fs::metadata(file_path)?;
        Ok(match meta.modified() {
            Ok(time) => time > self.time,
            Err(_) => false,
        })
    }
}

#[get("/logo")]
pub async fn logo(config: web::Data<Cfg>) -> Result<HttpResponse, actix_web::Error> {
    static TMP: OnceLock<RwLock<LogoCache>> = OnceLock::new();
    if TMP.get().is_none() {
        let logo_data = read(&config.main_cfg.logo_path).await?;
        let logo_cache = match LogoCache::new(logo_data) {
            Ok(cache) => cache,
            Err(e) => {
                tracing::error!("Failed to load logo: {}", e);
                return Ok(HttpResponse::InternalServerError().finish());
            }
        };
        TMP.set(RwLock::new(logo_cache)).ok();
    }
    let tmp = TMP.get();
    let rw = tmp.as_ref().unwrap();
    let mut rlock = Some(rw.read());
    if rlock
        .as_ref()
        .unwrap()
        .detect_update(&config.main_cfg.logo_path)?
    {
        drop(rlock.take());
        let mut wlock = rw.write();
        match wlock.update_logo(&config.main_cfg.logo_path) {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Failed to update logo: {}", e);
                return Ok(HttpResponse::InternalServerError().finish());
            }
        }
        drop(wlock);
        rlock = Some(rw.read());
    }
    let rlock = rlock.unwrap();
    let ret = HttpResponse::Ok()
        .content_type(rlock.imginfo.mimetype)
        .body(rlock.data.clone());
    Ok(ret)
}
