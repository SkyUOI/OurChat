use pb::{
    service::server_manage::monitoring::v1::{
        GetHistoricalMetricsRequest, GetHistoricalMetricsResponse, GetMonitoringMetricsRequest,
        GetMonitoringMetricsResponse,
    },
    time::TimeStampUtc,
};
use tonic::{Request, Response, Status};

use crate::{
    process::error_msg::{
        SERVER_ERROR,
        metrics::{INVALID_END_TIME, INVALID_INTERVAL, INVALID_START_TIME, METRICS_DISABLED},
    },
    server::ServerManageServiceProvider,
};

pub async fn get_monitoring_metrics(
    server: &ServerManageServiceProvider,
    request: Request<GetMonitoringMetricsRequest>,
) -> Result<Response<GetMonitoringMetricsResponse>, Status> {
    // Check if metrics are enabled
    if let Some(metrics) = server.shared_data.metrics.as_ref() {
        let req = request.into_inner();
        let include_system_metrics = req.include_system_metrics;
        let include_tokio_metrics = req.include_tokio_metrics;

        // Update system metrics if requested
        if include_system_metrics {
            metrics.update_system_metrics();
        }

        // Collect current metrics, passing database connection for accurate counts
        let metrics_data = metrics
            .get_monitoring_metrics(
                &server.db.db_pool,
                &server.db.pg_pool,
                include_system_metrics,
                include_tokio_metrics,
            )
            .await;

        Ok(Response::new(GetMonitoringMetricsResponse {
            metrics: Some(metrics_data),
        }))
    } else {
        Err(Status::unimplemented(
            "Metrics are disabled in configuration",
        ))
    }
}

#[derive(thiserror::Error, Debug)]
enum MetricsErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

pub async fn get_historical_metrics(
    server: &ServerManageServiceProvider,
    request: Request<GetHistoricalMetricsRequest>,
) -> Result<Response<GetHistoricalMetricsResponse>, Status> {
    match get_historical_metrics_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            MetricsErr::Db(_) | MetricsErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            MetricsErr::Status(status) => Err(status),
        },
    }
}

async fn get_historical_metrics_impl(
    server: &ServerManageServiceProvider,
    request: Request<GetHistoricalMetricsRequest>,
) -> Result<GetHistoricalMetricsResponse, MetricsErr> {
    // Check if metrics are enabled
    if !server.shared_data.cfg().main_cfg.enable_metrics {
        Err(Status::unimplemented(METRICS_DISABLED))?
    }

    let req = request.into_inner();

    // Convert protobuf timestamps to TimeStampUtc
    let start_time: Option<TimeStampUtc> = req
        .start_time
        .map(|t| t.try_into())
        .transpose()
        .map_err(|_| Status::invalid_argument(INVALID_START_TIME))?;
    let end_time: Option<TimeStampUtc> = req
        .end_time
        .map(|t| t.try_into())
        .transpose()
        .map_err(|_| Status::invalid_argument(INVALID_END_TIME))?;

    let interval = req
        .interval
        .map(|x| x.try_into())
        .transpose()
        .map_err(|_| Status::invalid_argument(INVALID_INTERVAL))?;

    // Query metrics_history table for requested time range
    let data_points = crate::db::metrics::get_historical_metrics(
        &server.db.db_pool,
        start_time,
        end_time,
        interval,
    )
    .await?;

    Ok(GetHistoricalMetricsResponse { data_points })
}
