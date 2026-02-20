use crate::{
    google::protobuf::Timestamp,
    service::server_manage::monitoring::v1::{
        MetricDataPoint, MonitoringMetrics, TokioMetrics, TokioRuntimeMetrics, TokioTaskMetrics,
    },
};

pub mod delete_account {
    pub mod v1 {
        include!("../generated/service.server_manage.delete_account.v1.rs");
    }
}

pub mod set_server_status {
    pub mod v1 {
        include!("../generated/service.server_manage.set_server_status.v1.rs");
    }
}

pub mod publish_announcement {
    pub mod v1 {
        include!("../generated/service.server_manage.publish_announcement.v1.rs");
    }
}

pub mod config {
    pub mod v1 {
        include!("../generated/service.server_manage.config.v1.rs");
    }
}

pub mod monitoring {
    pub mod v1 {
        include!("../generated/service.server_manage.monitoring.v1.rs");
    }
}

pub mod session_manage {
    pub mod v1 {
        include!("../generated/service.server_manage.session_manage.v1.rs");
    }
}

pub mod user_manage {
    pub mod v1 {
        include!("../generated/service.server_manage.user_manage.v1.rs");
    }
}

pub mod v1 {
    include!("../generated/service.server_manage.v1.rs");
}

impl From<entities::metrics_history::Model> for MetricDataPoint {
    fn from(record: entities::metrics_history::Model) -> MetricDataPoint {
        MetricDataPoint {
            timestamp: Some(Timestamp {
                seconds: record.timestamp.timestamp(),
                nanos: 0,
            }),
            metrics: Some(MonitoringMetrics {
                active_connections: record.active_connections,
                total_users: record.total_users,
                messages_per_second: record.messages_per_second,
                uptime_seconds: record.uptime_seconds,
                timestamp: record.timestamp.timestamp(),
                cpu_usage_percent: record.cpu_usage_percent,
                memory_usage_percent: record.memory_usage_percent,
                disk_usage_percent: record.disk_usage_percent,
                total_sessions: record.total_sessions,
                active_sessions: record.active_sessions,
                database_connections: record.database_connections,
                rabbitmq_connections: record.rabbitmq_connections,
                tokio: Some(TokioMetrics {
                    runtime: Some(model_to_runtime_metrics(&record)),
                    task: Some(model_to_task_metrics(&record)),
                }),
            }),
        }
    }
}

/// Convert tokio runtime metrics from database model
fn model_to_runtime_metrics(record: &entities::metrics_history::Model) -> TokioRuntimeMetrics {
    TokioRuntimeMetrics {
        workers_count: record.tokio_workers_count.unwrap_or(0) as u32,
        total_park_count: record.tokio_total_park_count.unwrap_or(0) as u64,
        max_park_count: record.tokio_max_park_count.unwrap_or(0) as u64,
        min_park_count: record.tokio_min_park_count.unwrap_or(0) as u64,
        total_busy_duration: record.tokio_total_busy_duration.unwrap_or(0) as u64,
        max_busy_duration: record.tokio_max_busy_duration.unwrap_or(0) as u64,
        min_busy_duration: record.tokio_min_busy_duration.unwrap_or(0) as u64,
        global_queue_depth: record.tokio_global_queue_depth.unwrap_or(0) as u64,
        elapsed: record.tokio_elapsed.unwrap_or(0) as u64,
        live_tasks_count: record.tokio_live_tasks_count.unwrap_or(0) as u64,
        mean_poll_duration: record.tokio_mean_poll_duration.unwrap_or(0) as u64,
        mean_poll_duration_worker_min: record.tokio_mean_poll_duration_worker_min.unwrap_or(0)
            as u64,
        mean_poll_duration_worker_max: record.tokio_mean_poll_duration_worker_max.unwrap_or(0)
            as u64,
        total_noop_count: record.tokio_total_noop_count.unwrap_or(0) as u64,
        max_noop_count: record.tokio_max_noop_count.unwrap_or(0) as u64,
        min_noop_count: record.tokio_min_noop_count.unwrap_or(0) as u64,
        total_steal_count: record.tokio_total_steal_count.unwrap_or(0) as u64,
        max_steal_count: record.tokio_max_steal_count.unwrap_or(0) as u64,
        min_steal_count: record.tokio_min_steal_count.unwrap_or(0) as u64,
        total_steal_operations: record.tokio_total_steal_operations.unwrap_or(0) as u64,
        max_steal_operations: record.tokio_max_steal_operations.unwrap_or(0) as u64,
        min_steal_operations: record.tokio_min_steal_operations.unwrap_or(0) as u64,
        num_remote_schedules: record.tokio_num_remote_schedules.unwrap_or(0) as u64,
        total_local_schedule_count: record.tokio_total_local_schedule_count.unwrap_or(0) as u64,
        max_local_schedule_count: record.tokio_max_local_schedule_count.unwrap_or(0) as u64,
        min_local_schedule_count: record.tokio_min_local_schedule_count.unwrap_or(0) as u64,
        total_overflow_count: record.tokio_total_overflow_count.unwrap_or(0) as u64,
        max_overflow_count: record.tokio_max_overflow_count.unwrap_or(0) as u64,
        min_overflow_count: record.tokio_min_overflow_count.unwrap_or(0) as u64,
        total_polls_count: record.tokio_total_polls_count.unwrap_or(0) as u64,
        max_polls_count: record.tokio_max_polls_count.unwrap_or(0) as u64,
        min_polls_count: record.tokio_min_polls_count.unwrap_or(0) as u64,
        total_local_queue_depth: record.tokio_total_local_queue_depth.unwrap_or(0) as u64,
        max_local_queue_depth: record.tokio_max_local_queue_depth.unwrap_or(0) as u64,
        min_local_queue_depth: record.tokio_min_local_queue_depth.unwrap_or(0) as u64,
        blocking_queue_depth: record.tokio_blocking_queue_depth.unwrap_or(0) as u64,
        blocking_threads_count: record.tokio_blocking_threads_count.unwrap_or(0) as u64,
        idle_blocking_threads_count: record.tokio_idle_blocking_threads_count.unwrap_or(0) as u64,
        budget_forced_yield_count: record.tokio_budget_forced_yield_count.unwrap_or(0) as u64,
        io_driver_ready_count: record.tokio_io_driver_ready_count.unwrap_or(0) as u64,
        busy_ratio: record.tokio_busy_ratio.unwrap_or(0.0),
        mean_polls_per_park: record.tokio_mean_polls_per_park.unwrap_or(0.0),
    }
}

/// Convert tokio task metrics from database model
fn model_to_task_metrics(record: &entities::metrics_history::Model) -> TokioTaskMetrics {
    TokioTaskMetrics {
        instrumented_count: record.tokio_task_instrumented_count.unwrap_or(0) as u64,
        dropped_count: record.tokio_task_dropped_count.unwrap_or(0) as u64,
        first_poll_count: record.tokio_task_first_poll_count.unwrap_or(0) as u64,
        total_first_poll_delay: record.tokio_task_total_first_poll_delay.unwrap_or(0) as u64,
        total_idled_count: record.tokio_task_total_idled_count.unwrap_or(0) as u64,
        total_idle_duration: record.tokio_task_total_idle_duration.unwrap_or(0) as u64,
        max_idle_duration: record.tokio_task_max_idle_duration.unwrap_or(0) as u64,
        total_scheduled_count: record.tokio_task_total_scheduled_count.unwrap_or(0) as u64,
        total_scheduled_duration: record.tokio_task_total_scheduled_duration.unwrap_or(0) as u64,
        total_poll_count: record.tokio_task_total_poll_count.unwrap_or(0) as u64,
        total_poll_duration: record.tokio_task_total_poll_duration.unwrap_or(0) as u64,
        total_fast_poll_count: record.tokio_task_total_fast_poll_count.unwrap_or(0) as u64,
        total_fast_poll_duration: record.tokio_task_total_fast_poll_duration.unwrap_or(0) as u64,
        total_slow_poll_count: record.tokio_task_total_slow_poll_count.unwrap_or(0) as u64,
        total_slow_poll_duration: record.tokio_task_total_slow_poll_duration.unwrap_or(0) as u64,
        total_short_delay_count: record.tokio_task_total_short_delay_count.unwrap_or(0) as u64,
        total_short_delay_duration: record.tokio_task_total_short_delay_duration.unwrap_or(0)
            as u64,
        total_long_delay_count: record.tokio_task_total_long_delay_count.unwrap_or(0) as u64,
        total_long_delay_duration: record.tokio_task_total_long_delay_duration.unwrap_or(0) as u64,
        mean_first_poll_delay: record.tokio_task_mean_first_poll_delay.unwrap_or(0) as u64,
        mean_idle_duration: record.tokio_task_mean_idle_duration.unwrap_or(0) as u64,
        mean_scheduled_duration: record.tokio_task_mean_scheduled_duration.unwrap_or(0) as u64,
        mean_poll_duration: record.tokio_task_mean_poll_duration.unwrap_or(0) as u64,
        slow_poll_ratio: record.tokio_task_slow_poll_ratio.unwrap_or(0.0),
        long_delay_ratio: record.tokio_task_long_delay_ratio.unwrap_or(0.0),
        mean_fast_poll_duration: record.tokio_task_mean_fast_poll_duration.unwrap_or(0) as u64,
        mean_slow_poll_duration: record.tokio_task_mean_slow_poll_duration.unwrap_or(0) as u64,
        mean_short_delay_duration: record.tokio_task_mean_short_delay_duration.unwrap_or(0) as u64,
        mean_long_delay_duration: record.tokio_task_mean_long_delay_duration.unwrap_or(0) as u64,
    }
}
