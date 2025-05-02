mod logo;
mod status;
pub mod verify;

use crate::{Cfg, EmailClientType};
use actix_web::{
    App,
    web::{self},
};
use anyhow::anyhow;
use base::{
    database::DbPool,
    shutdown::{ShutdownRev, ShutdownSdr},
};
use deadpool_lapin::lapin::options::{BasicAckOptions, BasicRejectOptions};
use rustls::{
    RootCertStore, ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio_stream::StreamExt;
use tracing::{debug, info};

pub struct HttpServer {}

impl HttpServer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run_forever(
        &mut self,
        listener: tokio::net::TcpListener,
        email_client: Option<EmailClientType>,
        cfg: Arc<Cfg>,
        rabbitmq: deadpool_lapin::Pool,
        db_conn: DbPool,
        shutdown_sdr: ShutdownSdr,
    ) -> anyhow::Result<()> {
        let cfg = web::Data::from(cfg);
        let cfg_clone = cfg.clone();
        let rabbitmq_clone = rabbitmq.clone();
        let db_conn_clone = db_conn.clone();
        info!("Start building Server");
        let enable_matrix = cfg.main_cfg.enable_matrix;
        let mut tls_config = None;
        if cfg.main_cfg.tls.is_tls_on()? {
            tls_config = Some(self.load_rustls_config(cfg.clone().into_inner())?);
        }
        let http_server = actix_web::HttpServer::new(move || {
            let v1 = web::scope("/v1")
                .service(status::status)
                .service(logo::logo)
                .configure(verify::config);
            let cors = actix_cors::Cors::default()
                .allow_any_origin()
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec![
                    actix_web::http::header::HeaderName::from_str("X-Requested-With").unwrap(),
                    actix_web::http::header::CONTENT_TYPE,
                    actix_web::http::header::AUTHORIZATION,
                ]);
            let mut app = App::new()
                .wrap(actix_web::middleware::NormalizePath::default())
                .wrap(cors)
                .wrap(actix_web::middleware::Logger::default())
                .app_data(db_conn_clone.clone())
                .app_data(cfg_clone.clone())
                .app_data(rabbitmq_clone.clone())
                .service(v1);
            if enable_matrix {
                info!("matrix api enabled");
                app = app.configure(crate::matrix::configure_matrix)
            }
            app
        });
        let http_server: actix_web::dev::Server = if cfg.main_cfg.tls.is_tls_on()? {
            drop(listener);
            http_server.bind_rustls_0_23(
                (cfg.main_cfg.ip.clone(), cfg.main_cfg.port),
                tls_config.unwrap(),
            )?
        } else {
            http_server.listen(listener.into_std()?)?
        }
        .run();
        info!("Start creating rabbitmq consumer");
        let connection = rabbitmq.get().await?;
        debug!("Get connection to rabbitmq");
        let channel = connection.create_channel().await?;
        debug!("Get channel to rabbitmq");
        let rabbit_listen_rev =
            shutdown_sdr.new_receiver("rabbitmq verify", "listen to rabbitmq to get verify record");
        tokio::spawn(async move {
            match Self::listen_rabbitmq(channel, db_conn, cfg, email_client, rabbit_listen_rev)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("{}", e);
                }
            }
        });
        info!("Http server setup done");
        let mut rev = shutdown_sdr.new_receiver("http server", "http server");
        select! {
            _ = rev.wait_shutting_down() => {
                Ok(())
            }
            e = http_server => {
                e?;
                Ok(())
            }
        }
    }

    fn load_rustls_config(&self, cfg: Arc<Cfg>) -> anyhow::Result<rustls::ServerConfig> {
        let mut cert_store = RootCertStore::empty();
        CertificateDer::pem_file_iter(cfg.main_cfg.tls.ca_tls_cert_path.as_ref().unwrap())?
            .flatten()
            .for_each(|der| cert_store.add(der).unwrap());

        // let client_auth = WebPkiClientVerifier::builder(Arc::new(cert_store)).build()?;

        let key_der =
            PrivateKeyDer::from_pem_file(cfg.main_cfg.tls.server_key_cert_path.as_ref().unwrap())?;
        let cert_chain =
            CertificateDer::pem_file_iter(cfg.main_cfg.tls.server_tls_cert_path.as_ref().unwrap())
                .unwrap()
                .flatten()
                .collect();
        Ok(ServerConfig::builder()
            // .with_client_cert_verifier(client_auth)
            .with_no_client_auth()
            .with_single_cert(cert_chain, key_der)?)
    }

    async fn listen_rabbitmq(
        mq_channel: deadpool_lapin::lapin::Channel,
        db_pool: DbPool,
        cfg: web::Data<Cfg>,
        email_client: Option<EmailClientType>,
        mut shutdown_rev: ShutdownRev,
    ) -> anyhow::Result<()> {
        let logic = async {
            debug!("Starting set channel");
            // TODO:add this to config file
            mq_channel
                .basic_qos(
                    70,
                    deadpool_lapin::lapin::options::BasicQosOptions::default(),
                )
                .await?;
            // Wait for the channel to be set
            let mut try_cnt = 0;
            let mut consumer = loop {
                match mq_channel
                    .basic_consume(
                        base::rabbitmq::http_server::VERIFY_QUEUE,
                        "http_server",
                        deadpool_lapin::lapin::options::BasicConsumeOptions::default(),
                        deadpool_lapin::lapin::types::FieldTable::default(),
                    )
                    .await
                {
                    Ok(c) => {
                        break c;
                    }
                    Err(e) => {
                        tracing::error!("try {} to get consumer failed:{}", try_cnt, e);
                        if try_cnt == 9 {
                            return Err(anyhow!(e));
                        }
                    }
                }
                tokio::time::sleep(Duration::from_secs(3)).await;
                try_cnt += 1;
            };
            debug!("Starting to consume verification");
            while let Some(data) = consumer.next().await {
                let delivery = match data {
                    Ok(data) => data,
                    Err(e) => {
                        tracing::error!("{}", e);
                        continue;
                    }
                };
                let verify_record = serde_json::from_slice::<
                    base::rabbitmq::http_server::VerifyRecord,
                >(&delivery.data[..])?;
                match verify::verify_client(&db_pool, &email_client, verify_record.clone(), &cfg)
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("{}", e);
                    }
                }
                let redis_conn = db_pool.redis_pool.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(base::consts::VERIFY_EMAIL_EXPIRE).await;
                    let reject = async {
                        match delivery.reject(BasicRejectOptions { requeue: false }).await {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("{}", e);
                            }
                        }
                    };
                    let status = match verify::check_token_exist_and_del_token(
                        &verify_record.token,
                        &redis_conn,
                    )
                    .await
                    {
                        Ok(data) => data,
                        Err(e) => {
                            reject.await;
                            tracing::error!("check token error:{e}");
                            return;
                        }
                    };
                    if status {
                        reject.await;
                    } else {
                        match delivery.ack(BasicAckOptions::default()).await {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("ack verify failed:{}", e);
                            }
                        }
                    }
                });
            }
            anyhow::Ok(())
        };
        select! {
            ret = logic => {
                ret
            }
            _ = shutdown_rev.wait_shutting_down() => {
                Ok(())
            }
        }
    }
}

impl Default for HttpServer {
    fn default() -> Self {
        Self::new()
    }
}
