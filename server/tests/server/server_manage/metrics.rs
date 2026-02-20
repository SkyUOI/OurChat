use std::time::Duration;

use claims::{assert_ge, assert_gt, assert_le, assert_some};
use client::TestApp;
use pb::google::protobuf::{Duration as ProtoDuration, Timestamp as ProtoTimestamp};
use pb::service::server_manage::monitoring::v1::{
    GetHistoricalMetricsRequest, GetMonitoringMetricsRequest,
};
use pb::time::TimeStampUtc;
use tonic::Request;

/// Single comprehensive metrics test to avoid global recorder conflicts across tests.
#[tokio::test]
async fn metrics_comprehensive_test() {
    let (mut config, args) = TestApp::get_test_config().unwrap();
    config.main_cfg.enable_metrics = true;
    config.main_cfg.metrics_snapshot_interval = Duration::from_millis(500);

    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    tracing::info!("Testing basic metrics retrieval with system metrics");
    let user = app.new_user().await.unwrap();

    let response = user
        .lock()
        .await
        .server_manage()
        .get_monitoring_metrics(Request::new(GetMonitoringMetricsRequest {
            include_system_metrics: true,
            include_tokio_metrics: true,
        }))
        .await
        .unwrap();

    let metrics = response.into_inner().metrics.unwrap();

    assert!(metrics.uptime_seconds > 0, "Uptime should be positive");
    assert!(metrics.timestamp > 0, "Timestamp should be present");
    assert_ge!(metrics.active_connections, 0);
    assert_ge!(metrics.total_users, 0);
    assert!(metrics.messages_per_second >= 0.0);
    assert_ge!(metrics.total_sessions, 0);
    assert_ge!(metrics.active_sessions, 0);
    assert_ge!(metrics.database_connections, 0);
    assert_ge!(metrics.rabbitmq_connections, 0);

    assert!(metrics.cpu_usage_percent.is_some());
    assert!(metrics.memory_usage_percent.is_some());
    assert!(metrics.disk_usage_percent.is_some());

    let cpu = metrics.cpu_usage_percent.unwrap();
    assert!((0.0..=100.0).contains(&cpu), "CPU should be 0-100");
    let memory = metrics.memory_usage_percent.unwrap();
    assert!((0.0..=100.0).contains(&memory), "Memory should be 0-100");
    let disk = metrics.disk_usage_percent.unwrap();
    assert!((0.0..=100.0).contains(&disk), "Disk should be 0-100");

    tracing::info!("Testing metrics without system metrics");
    let response = user
        .lock()
        .await
        .server_manage()
        .get_monitoring_metrics(Request::new(GetMonitoringMetricsRequest {
            include_system_metrics: false,
            include_tokio_metrics: true,
        }))
        .await
        .unwrap();

    let metrics_no_system = response.into_inner().metrics.unwrap();
    assert!(metrics_no_system.cpu_usage_percent.is_none());
    assert!(metrics_no_system.memory_usage_percent.is_none());
    assert!(metrics_no_system.disk_usage_percent.is_none());
    assert_ge!(metrics_no_system.uptime_seconds, 0);
    assert_ge!(metrics_no_system.timestamp, 0);

    tracing::info!("Testing tokio metrics");
    tokio::time::sleep(Duration::from_millis(600)).await;

    let response = user
        .lock()
        .await
        .server_manage()
        .get_monitoring_metrics(Request::new(GetMonitoringMetricsRequest {
            include_system_metrics: false,
            include_tokio_metrics: true,
        }))
        .await
        .unwrap();

    let tokio_metrics = response.into_inner().metrics.unwrap().tokio.unwrap();
    let runtime = tokio_metrics.runtime.unwrap();

    assert_gt!(runtime.workers_count, 0);
    assert_ge!(runtime.total_park_count, 0);
    assert_ge!(runtime.max_park_count, 0);
    assert_ge!(runtime.min_park_count, 0);
    assert_ge!(runtime.total_busy_duration, 0);
    assert_ge!(runtime.global_queue_depth, 0);
    assert_gt!(runtime.elapsed, 0);
    assert_ge!(runtime.live_tasks_count, 0);
    assert!(runtime.mean_polls_per_park >= 0.0);
    assert_some!(tokio_metrics.task);

    let response = user
        .lock()
        .await
        .server_manage()
        .get_monitoring_metrics(Request::new(GetMonitoringMetricsRequest {
            include_system_metrics: false,
            include_tokio_metrics: false,
        }))
        .await
        .unwrap();

    let metrics_no_tokio = response.into_inner().metrics.unwrap();
    assert!(
        metrics_no_tokio.tokio.is_none(),
        "Tokio metrics should not be included when not requested"
    );

    tracing::info!("Testing uptime increases over time");
    let uptime1 = {
        let response = user
            .lock()
            .await
            .server_manage()
            .get_monitoring_metrics(Request::new(GetMonitoringMetricsRequest {
                include_system_metrics: false,
                include_tokio_metrics: false,
            }))
            .await
            .unwrap();
        response.into_inner().metrics.unwrap().uptime_seconds
    };

    tokio::time::sleep(Duration::from_millis(100)).await;

    let uptime2 = {
        let response = user
            .lock()
            .await
            .server_manage()
            .get_monitoring_metrics(Request::new(GetMonitoringMetricsRequest {
                include_system_metrics: false,
                include_tokio_metrics: false,
            }))
            .await
            .unwrap();
        response.into_inner().metrics.unwrap().uptime_seconds
    };

    assert_ge!(uptime2, uptime1, "Uptime should increase over time");

    tracing::info!("Testing metrics consistency");
    let response1 = user
        .lock()
        .await
        .server_manage()
        .get_monitoring_metrics(Request::new(GetMonitoringMetricsRequest {
            include_system_metrics: false,
            include_tokio_metrics: false,
        }))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    let response2 = user
        .lock()
        .await
        .server_manage()
        .get_monitoring_metrics(Request::new(GetMonitoringMetricsRequest {
            include_system_metrics: false,
            include_tokio_metrics: false,
        }))
        .await
        .unwrap();

    let m1 = response1.into_inner().metrics.unwrap();
    let m2 = response2.into_inner().metrics.unwrap();

    assert_eq!(m1.active_connections, m2.active_connections);
    assert_eq!(m1.total_users, m2.total_users);
    assert_eq!(m1.total_sessions, m2.total_sessions);
    assert_eq!(m1.active_sessions, m2.active_sessions);
    assert_ge!(m2.uptime_seconds, m1.uptime_seconds);
    assert_ge!(m2.timestamp, m1.timestamp);

    tracing::info!("Verifying tokio metrics continue to work");
    {
        let response = user
            .lock()
            .await
            .server_manage()
            .get_monitoring_metrics(Request::new(GetMonitoringMetricsRequest {
                include_system_metrics: false,
                include_tokio_metrics: true,
            }))
            .await
            .unwrap();
        let rt = response
            .into_inner()
            .metrics
            .unwrap()
            .tokio
            .unwrap()
            .runtime
            .unwrap();
        assert_gt!(rt.elapsed, 0);
    }

    tracing::info!("Testing historical metrics with snapshots");
    let before_snapshot = chrono::Utc::now() - chrono::Duration::seconds(5);

    tokio::time::sleep(Duration::from_millis(2500)).await;

    let after_snapshot = chrono::Utc::now() + chrono::Duration::seconds(1);

    let history_response = user
        .lock()
        .await
        .server_manage()
        .get_historical_metrics(Request::new(GetHistoricalMetricsRequest {
            start_time: Some(ProtoTimestamp {
                seconds: before_snapshot.timestamp(),
                nanos: 0,
            }),
            end_time: Some(ProtoTimestamp {
                seconds: after_snapshot.timestamp(),
                nanos: 0,
            }),
            interval: Some(ProtoDuration {
                seconds: 0,
                nanos: 0,
            }),
        }))
        .await
        .unwrap();

    let data_points = history_response.into_inner().data_points;
    assert!(
        !data_points.is_empty(),
        "Should have historical data points"
    );
    assert_ge!(data_points.len(), 1, "Should have at least 1 snapshot");

    for i in 0..data_points.len() {
        let dp = &data_points[i];
        let ts = dp.timestamp.as_ref().unwrap().seconds;
        assert_ge!(ts, before_snapshot.timestamp());
        assert_le!(ts, after_snapshot.timestamp());
        assert_some!(dp.metrics);
        assert_ge!(dp.metrics.as_ref().unwrap().uptime_seconds, 0);

        if i > 0 {
            let ts_prev = data_points[i - 1].timestamp.as_ref().unwrap().seconds;
            assert_gt!(ts, ts_prev, "Data points should be ordered");
        }
    }

    tracing::info!("Testing historical metrics aggregation");
    let intervals = [(0, 0), (1, 0), (2, 0), (5, 0), (10, 0)];

    for (secs, nanos) in intervals {
        let response = user
            .lock()
            .await
            .server_manage()
            .get_historical_metrics(Request::new(GetHistoricalMetricsRequest {
                start_time: Some(ProtoTimestamp {
                    seconds: before_snapshot.timestamp(),
                    nanos: 0,
                }),
                end_time: Some(ProtoTimestamp {
                    seconds: after_snapshot.timestamp(),
                    nanos: 0,
                }),
                interval: Some(ProtoDuration {
                    seconds: secs,
                    nanos,
                }),
            }))
            .await
            .unwrap();

        let points = response.into_inner().data_points;
        for dp in &points {
            assert_some!(dp.metrics);
            let ts: TimeStampUtc = dp.timestamp.clone().unwrap().try_into().unwrap();
            assert_le!(ts, after_snapshot);
        }
        assert_le!(points.len(), 10, "Should have reasonable number of points");
    }

    tracing::info!("Testing historical metrics time range validation");
    let now = chrono::Utc::now().timestamp();

    let response = user
        .lock()
        .await
        .server_manage()
        .get_historical_metrics(Request::new(GetHistoricalMetricsRequest {
            start_time: Some(ProtoTimestamp {
                seconds: now,
                nanos: 0,
            }),
            end_time: Some(ProtoTimestamp {
                seconds: now - 3600,
                nanos: 0,
            }),
            interval: Some(ProtoDuration {
                seconds: 60,
                nanos: 0,
            }),
        }))
        .await
        .unwrap();

    assert!(
        response.into_inner().data_points.is_empty(),
        "Inverted time range should return empty"
    );

    let response = user
        .lock()
        .await
        .server_manage()
        .get_historical_metrics(Request::new(GetHistoricalMetricsRequest {
            start_time: Some(ProtoTimestamp {
                seconds: now - 7200,
                nanos: 0,
            }),
            end_time: Some(ProtoTimestamp {
                seconds: now - 3600,
                nanos: 0,
            }),
            interval: Some(ProtoDuration {
                seconds: 60,
                nanos: 0,
            }),
        }))
        .await
        .unwrap();

    assert!(
        response.into_inner().data_points.is_empty(),
        "Should return empty for range with no data"
    );

    tracing::info!("Testing multiple users accessing metrics");
    let user2 = app.new_user().await.unwrap();
    let user3 = app.new_user().await.unwrap();

    for u in [&user, &user2, &user3] {
        let response = u
            .lock()
            .await
            .server_manage()
            .get_monitoring_metrics(Request::new(GetMonitoringMetricsRequest {
                include_system_metrics: false,
                include_tokio_metrics: false,
            }))
            .await
            .unwrap();
        assert_gt!(response.into_inner().metrics.unwrap().uptime_seconds, 0);
    }

    tracing::info!("Testing active connections tracking");
    let (session_users, session) = app
        .new_session_db_level(2, "test_session", false)
        .await
        .unwrap();
    let user_a = session_users[0].clone();
    let user_b = session_users[1].clone();

    user_a
        .lock()
        .await
        .send_msg(session.session_id, "test message", vec![], false)
        .await
        .unwrap();

    let initial_metrics = user_a
        .lock()
        .await
        .server_manage()
        .get_monitoring_metrics(Request::new(GetMonitoringMetricsRequest {
            include_system_metrics: false,
            include_tokio_metrics: false,
        }))
        .await
        .unwrap()
        .into_inner()
        .metrics
        .unwrap();
    assert_eq!(initial_metrics.active_connections, 0);

    let messages = user_b.lock().await.fetch_msgs().fetch(1).await.unwrap();
    assert_eq!(messages.len(), 1);

    let final_metrics = user_a
        .lock()
        .await
        .server_manage()
        .get_monitoring_metrics(Request::new(GetMonitoringMetricsRequest {
            include_system_metrics: false,
            include_tokio_metrics: false,
        }))
        .await
        .unwrap()
        .into_inner()
        .metrics
        .unwrap();
    assert_eq!(final_metrics.active_connections, 0);

    app.async_drop().await;
}
