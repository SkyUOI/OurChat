mod status;
pub mod verify;

use crate::{Config, EmailClientType};
use actix_web::{
    App,
    web::{self},
};
use base::database::DbPool;
use deadpool_lapin::lapin::options::{BasicAckOptions, BasicRejectOptions};
use tokio_stream::StreamExt;

pub struct HttpServer {}

impl HttpServer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run_forever(
        &mut self,
        listener: tokio::net::TcpListener,
        email_client: Option<EmailClientType>,
        cfg: Config,
        rabbitmq: deadpool_lapin::Pool,
        db_conn: DbPool,
    ) -> anyhow::Result<()> {
        let cfg = web::Data::new(cfg);
        let cfg_clone = cfg.clone();
        let rabbitmq_clone = rabbitmq.clone();
        let db_conn_clone = db_conn.clone();
        let http_server = actix_web::HttpServer::new(move || {
            let v1 = web::scope("/v1")
                .service(status::status)
                .configure(verify::config);
            App::new()
                .wrap(actix_web::middleware::Logger::default())
                .app_data(db_conn_clone.clone())
                .app_data(cfg_clone.clone())
                .app_data(rabbitmq_clone.clone())
                .service(v1)
        })
        .listen(listener.into_std()?)?
        .run();
        let connection = rabbitmq.get().await?;
        let channel = connection.create_channel().await?;
        tokio::spawn(async move {
            match Self::listen_rabbitmq(channel, db_conn, cfg, email_client).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("{}", e);
                }
            }
        });
        http_server.await?;
        Ok(())
    }

    async fn listen_rabbitmq(
        mq_channel: deadpool_lapin::lapin::Channel,
        db_pool: DbPool,
        cfg: web::Data<Config>,
        email_client: Option<EmailClientType>,
    ) -> anyhow::Result<()> {
        // TODO:add this to config file
        mq_channel
            .basic_qos(
                70,
                deadpool_lapin::lapin::options::BasicQosOptions::default(),
            )
            .await?;
        let mut consumer = mq_channel
            .basic_consume(
                base::rabbitmq::http_server::VERIFY_QUEUE,
                "http_server",
                deadpool_lapin::lapin::options::BasicConsumeOptions::default(),
                deadpool_lapin::lapin::types::FieldTable::default(),
            )
            .await
            .expect("basic_consume");
        while let Some(data) = consumer.next().await {
            let delivery = match data {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!("{}", e);
                    continue;
                }
            };
            let verify_record = serde_json::from_slice::<base::rabbitmq::http_server::VerifyRecord>(
                &delivery.data[..],
            )?;
            match verify::verify_client(&db_pool, &email_client, verify_record.clone(), &cfg).await
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
        Ok(())
    }
}

impl Default for HttpServer {
    fn default() -> Self {
        Self::new()
    }
}
