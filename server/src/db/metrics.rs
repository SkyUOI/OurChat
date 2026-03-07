use pb::google::protobuf::Timestamp;
use pb::{
    service::server_manage::monitoring::v1::{
        MetricDataPoint, MonitoringMetrics, TokioMetrics, TokioRuntimeMetrics, TokioTaskMetrics,
    },
    time::TimeStampUtc,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, Order, QueryFilter, QueryOrder,
    Set,
};
use std::collections::HashMap;
use std::time::Duration;

use entities::metrics_history as MetricsHistoryEntity;

/// Get historical metrics from the database
///
/// # Arguments
///
/// * `db` - Database connection
/// * `start_time` - Start timestamp
/// * `end_time` - End timestamp
/// * `interval` - Aggregation interval (0 = no aggregation)
///
/// # Errors
///
/// Returns Status::internal on database errors
///
/// # Note
///
/// This function requires the `metrics_history` entity to be generated.
/// Run the migration and regenerate entities first:
/// ```bash
/// sea migrate generate create_metrics_history_table
/// python scripts/db_migration.py
/// python scripts/regenerate_entities.py
/// ```
pub async fn get_historical_metrics(
    db: &impl ConnectionTrait,
    start_time: Option<TimeStampUtc>,
    end_time: Option<TimeStampUtc>,
    interval: Option<Duration>,
) -> Result<Vec<MetricDataPoint>, sea_orm::DbErr> {
    use entities::metrics_history;

    // Convert Option<DateTime<Utc>> to Option<DateTime<FixedOffset>> for database comparison
    let start_time_fixed: Option<chrono::DateTime<chrono::FixedOffset>> =
        start_time.map(|t| t.into());
    let end_time_fixed: Option<chrono::DateTime<chrono::FixedOffset>> = end_time.map(|t| t.into());

    // Query metrics within the time range
    let records = metrics_history::Entity::find()
        .filter(metrics_history::Column::Timestamp.gte(start_time_fixed))
        .filter(metrics_history::Column::Timestamp.lte(end_time_fixed))
        .order_by(metrics_history::Column::Timestamp, Order::Asc)
        .all(db)
        .await?;

    let data_points = if let Some(interval) = interval
        && interval.as_secs() > 0
    {
        aggregate_by_interval(records, interval)
    } else {
        records.into_iter().map(MetricDataPoint::from).collect()
    };

    Ok(data_points)
}

/// Aggregate metrics by time interval
fn aggregate_by_interval(
    records: Vec<entities::metrics_history::Model>,
    interval: Duration,
) -> Vec<MetricDataPoint> {
    let mut buckets: HashMap<i64, Vec<entities::metrics_history::Model>> = HashMap::new();

    // Group records into buckets
    for record in records {
        let bucket_key =
            record.timestamp.timestamp() / interval.as_secs() as i64 * interval.as_secs() as i64;
        buckets.entry(bucket_key).or_default().push(record);
    }

    // Collect and sort by timestamp
    let mut sorted_buckets: Vec<_> = buckets.into_iter().collect();
    sorted_buckets.sort_by_key(|(k, _)| *k);

    // Aggregate each bucket
    let mut result = Vec::new();
    for (timestamp, bucket_records) in sorted_buckets {
        let aggregated = MonitoringMetrics {
            active_connections: average(&bucket_records, |r| r.active_connections),
            total_users: average(&bucket_records, |r| r.total_users),
            messages_per_second: average_double(&bucket_records, |r| r.messages_per_second),
            uptime_seconds: latest(&bucket_records, |r| r.uptime_seconds),
            timestamp,
            cpu_usage_percent: average_double_option(&bucket_records, |r| r.cpu_usage_percent),
            memory_usage_percent: average_double_option(&bucket_records, |r| {
                r.memory_usage_percent
            }),
            disk_usage_percent: average_double_option(&bucket_records, |r| r.disk_usage_percent),
            total_sessions: average(&bucket_records, |r| r.total_sessions),
            active_sessions: average(&bucket_records, |r| r.active_sessions),
            database_connections: average(&bucket_records, |r| r.database_connections),
            rabbitmq_connections: average(&bucket_records, |r| r.rabbitmq_connections),
            tokio: aggregate_tokio_metrics(&bucket_records),
        };

        result.push(MetricDataPoint {
            timestamp: Some(Timestamp {
                seconds: timestamp,
                nanos: 0,
            }),
            metrics: Some(aggregated),
        });
    }

    result
}

/// Aggregate tokio metrics from a bucket of records
fn aggregate_tokio_metrics(records: &[entities::metrics_history::Model]) -> Option<TokioMetrics> {
    if records.is_empty() {
        return None;
    }

    // Only aggregate tokio metrics if at least one record has them
    let has_tokio = records.iter().any(|r| r.tokio_workers_count.is_some());
    if !has_tokio {
        return None;
    }

    Some(TokioMetrics {
        runtime: Some(aggregate_runtime_metrics(records)),
        task: Some(aggregate_task_metrics(records)),
    })
}

/// Aggregate runtime metrics from a bucket of records
fn aggregate_runtime_metrics(records: &[entities::metrics_history::Model]) -> TokioRuntimeMetrics {
    TokioRuntimeMetrics {
        workers_count: average_option(records, |r| r.tokio_workers_count.map(|v| v as u32))
            .unwrap_or(0),
        total_park_count: average_option_bigint(records, |r| r.tokio_total_park_count).unwrap_or(0),
        max_park_count: average_option_bigint(records, |r| r.tokio_max_park_count).unwrap_or(0),
        min_park_count: average_option_bigint(records, |r| r.tokio_min_park_count).unwrap_or(0),
        total_busy_duration: average_option_bigint(records, |r| r.tokio_total_busy_duration)
            .unwrap_or(0),
        max_busy_duration: average_option_bigint(records, |r| r.tokio_max_busy_duration)
            .unwrap_or(0),
        min_busy_duration: average_option_bigint(records, |r| r.tokio_min_busy_duration)
            .unwrap_or(0),
        global_queue_depth: average_option_bigint(records, |r| r.tokio_global_queue_depth)
            .unwrap_or(0),
        elapsed: average_option_bigint(records, |r| r.tokio_elapsed).unwrap_or(0),
        live_tasks_count: average_option_bigint(records, |r| r.tokio_live_tasks_count).unwrap_or(0),
        mean_poll_duration: average_option_bigint(records, |r| r.tokio_mean_poll_duration)
            .unwrap_or(0),
        mean_poll_duration_worker_min: average_option_bigint(records, |r| {
            r.tokio_mean_poll_duration_worker_min
        })
        .unwrap_or(0),
        mean_poll_duration_worker_max: average_option_bigint(records, |r| {
            r.tokio_mean_poll_duration_worker_max
        })
        .unwrap_or(0),
        total_noop_count: average_option_bigint(records, |r| r.tokio_total_noop_count).unwrap_or(0),
        max_noop_count: average_option_bigint(records, |r| r.tokio_max_noop_count).unwrap_or(0),
        min_noop_count: average_option_bigint(records, |r| r.tokio_min_noop_count).unwrap_or(0),
        total_steal_count: average_option_bigint(records, |r| r.tokio_total_steal_count)
            .unwrap_or(0),
        max_steal_count: average_option_bigint(records, |r| r.tokio_max_steal_count).unwrap_or(0),
        min_steal_count: average_option_bigint(records, |r| r.tokio_min_steal_count).unwrap_or(0),
        total_steal_operations: average_option_bigint(records, |r| r.tokio_total_steal_operations)
            .unwrap_or(0),
        max_steal_operations: average_option_bigint(records, |r| r.tokio_max_steal_operations)
            .unwrap_or(0),
        min_steal_operations: average_option_bigint(records, |r| r.tokio_min_steal_operations)
            .unwrap_or(0),
        num_remote_schedules: average_option_bigint(records, |r| r.tokio_num_remote_schedules)
            .unwrap_or(0),
        total_local_schedule_count: average_option_bigint(records, |r| {
            r.tokio_total_local_schedule_count
        })
        .unwrap_or(0),
        max_local_schedule_count: average_option_bigint(records, |r| {
            r.tokio_max_local_schedule_count
        })
        .unwrap_or(0),
        min_local_schedule_count: average_option_bigint(records, |r| {
            r.tokio_min_local_schedule_count
        })
        .unwrap_or(0),
        total_overflow_count: average_option_bigint(records, |r| r.tokio_total_overflow_count)
            .unwrap_or(0),
        max_overflow_count: average_option_bigint(records, |r| r.tokio_max_overflow_count)
            .unwrap_or(0),
        min_overflow_count: average_option_bigint(records, |r| r.tokio_min_overflow_count)
            .unwrap_or(0),
        total_polls_count: average_option_bigint(records, |r| r.tokio_total_polls_count)
            .unwrap_or(0),
        max_polls_count: average_option_bigint(records, |r| r.tokio_max_polls_count).unwrap_or(0),
        min_polls_count: average_option_bigint(records, |r| r.tokio_min_polls_count).unwrap_or(0),
        total_local_queue_depth: average_option_bigint(records, |r| {
            r.tokio_total_local_queue_depth
        })
        .unwrap_or(0),
        max_local_queue_depth: average_option_bigint(records, |r| r.tokio_max_local_queue_depth)
            .unwrap_or(0),
        min_local_queue_depth: average_option_bigint(records, |r| r.tokio_min_local_queue_depth)
            .unwrap_or(0),
        blocking_queue_depth: average_option_bigint(records, |r| r.tokio_blocking_queue_depth)
            .unwrap_or(0),
        blocking_threads_count: average_option_bigint(records, |r| r.tokio_blocking_threads_count)
            .unwrap_or(0),
        idle_blocking_threads_count: average_option_bigint(records, |r| {
            r.tokio_idle_blocking_threads_count
        })
        .unwrap_or(0),
        budget_forced_yield_count: average_option_bigint(records, |r| {
            r.tokio_budget_forced_yield_count
        })
        .unwrap_or(0),
        io_driver_ready_count: average_option_bigint(records, |r| r.tokio_io_driver_ready_count)
            .unwrap_or(0),
        busy_ratio: average_double_option(records, |r| r.tokio_busy_ratio).unwrap_or(0.0),
        mean_polls_per_park: average_double_option(records, |r| r.tokio_mean_polls_per_park)
            .unwrap_or(0.0),
    }
}

/// Aggregate task metrics from a bucket of records
fn aggregate_task_metrics(records: &[entities::metrics_history::Model]) -> TokioTaskMetrics {
    TokioTaskMetrics {
        instrumented_count: average_option_bigint(records, |r| r.tokio_task_instrumented_count)
            .unwrap_or(0),
        dropped_count: average_option_bigint(records, |r| r.tokio_task_dropped_count).unwrap_or(0),
        first_poll_count: average_option_bigint(records, |r| r.tokio_task_first_poll_count)
            .unwrap_or(0),
        total_first_poll_delay: average_option_bigint(records, |r| {
            r.tokio_task_total_first_poll_delay
        })
        .unwrap_or(0),
        total_idled_count: average_option_bigint(records, |r| r.tokio_task_total_idled_count)
            .unwrap_or(0),
        total_idle_duration: average_option_bigint(records, |r| r.tokio_task_total_idle_duration)
            .unwrap_or(0),
        max_idle_duration: average_option_bigint(records, |r| r.tokio_task_max_idle_duration)
            .unwrap_or(0),
        total_scheduled_count: average_option_bigint(records, |r| {
            r.tokio_task_total_scheduled_count
        })
        .unwrap_or(0),
        total_scheduled_duration: average_option_bigint(records, |r| {
            r.tokio_task_total_scheduled_duration
        })
        .unwrap_or(0),
        total_poll_count: average_option_bigint(records, |r| r.tokio_task_total_poll_count)
            .unwrap_or(0),
        total_poll_duration: average_option_bigint(records, |r| r.tokio_task_total_poll_duration)
            .unwrap_or(0),
        total_fast_poll_count: average_option_bigint(records, |r| {
            r.tokio_task_total_fast_poll_count
        })
        .unwrap_or(0),
        total_fast_poll_duration: average_option_bigint(records, |r| {
            r.tokio_task_total_fast_poll_duration
        })
        .unwrap_or(0),
        total_slow_poll_count: average_option_bigint(records, |r| {
            r.tokio_task_total_slow_poll_count
        })
        .unwrap_or(0),
        total_slow_poll_duration: average_option_bigint(records, |r| {
            r.tokio_task_total_slow_poll_duration
        })
        .unwrap_or(0),
        total_short_delay_count: average_option_bigint(records, |r| {
            r.tokio_task_total_short_delay_count
        })
        .unwrap_or(0),
        total_short_delay_duration: average_option_bigint(records, |r| {
            r.tokio_task_total_short_delay_duration
        })
        .unwrap_or(0),
        total_long_delay_count: average_option_bigint(records, |r| {
            r.tokio_task_total_long_delay_count
        })
        .unwrap_or(0),
        total_long_delay_duration: average_option_bigint(records, |r| {
            r.tokio_task_total_long_delay_duration
        })
        .unwrap_or(0),
        mean_first_poll_delay: average_option_bigint(records, |r| {
            r.tokio_task_mean_first_poll_delay
        })
        .unwrap_or(0),
        mean_idle_duration: average_option_bigint(records, |r| r.tokio_task_mean_idle_duration)
            .unwrap_or(0),
        mean_scheduled_duration: average_option_bigint(records, |r| {
            r.tokio_task_mean_scheduled_duration
        })
        .unwrap_or(0),
        mean_poll_duration: average_option_bigint(records, |r| r.tokio_task_mean_poll_duration)
            .unwrap_or(0),
        slow_poll_ratio: average_double_option(records, |r| r.tokio_task_slow_poll_ratio)
            .unwrap_or(0.0),
        long_delay_ratio: average_double_option(records, |r| r.tokio_task_long_delay_ratio)
            .unwrap_or(0.0),
        mean_fast_poll_duration: average_option_bigint(records, |r| {
            r.tokio_task_mean_fast_poll_duration
        })
        .unwrap_or(0),
        mean_slow_poll_duration: average_option_bigint(records, |r| {
            r.tokio_task_mean_slow_poll_duration
        })
        .unwrap_or(0),
        mean_short_delay_duration: average_option_bigint(records, |r| {
            r.tokio_task_mean_short_delay_duration
        })
        .unwrap_or(0),
        mean_long_delay_duration: average_option_bigint(records, |r| {
            r.tokio_task_mean_long_delay_duration
        })
        .unwrap_or(0),
    }
}

/// Calculate average of an integer field across records
fn average<T, F>(records: &[T], f: F) -> i32
where
    F: Fn(&T) -> i32,
{
    if records.is_empty() {
        return 0;
    }
    let sum: i32 = records.iter().map(&f).sum();
    sum / records.len() as i32
}

/// Calculate average of an optional integer field across records
fn average_option<T, F>(records: &[T], f: F) -> Option<u32>
where
    F: Fn(&T) -> Option<u32>,
{
    let values: Vec<u32> = records.iter().filter_map(f).collect();
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<u32>() / values.len() as u32)
    }
}

/// Calculate average of an optional bigint field across records
fn average_option_bigint<T, F>(records: &[T], f: F) -> Option<u64>
where
    F: Fn(&T) -> Option<i64>,
{
    let values: Vec<i64> = records.iter().filter_map(f).collect();
    if values.is_empty() {
        None
    } else {
        let sum: i64 = values.iter().sum();
        Some((sum / values.len() as i64) as u64)
    }
}

/// Calculate average of a double field across records
fn average_double<T, F>(records: &[T], f: F) -> f64
where
    F: Fn(&T) -> f64,
{
    if records.is_empty() {
        return 0.0;
    }
    let sum: f64 = records.iter().map(&f).sum();
    sum / records.len() as f64
}

/// Calculate average of an optional double field across records
fn average_double_option<T, F>(records: &[T], f: F) -> Option<f64>
where
    F: Fn(&T) -> Option<f64>,
{
    let values: Vec<f64> = records.iter().filter_map(f).collect();
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

/// Get the latest (maximum) value of an integer field across records
fn latest<T, F>(records: &[T], f: F) -> i64
where
    F: Fn(&T) -> i64,
{
    records.iter().map(&f).max().unwrap_or(0)
}

/// Save metrics snapshot to database
///
/// # Errors
///
/// Returns database errors
///
/// # Note
///
/// This function requires the `metrics_history` entity to be generated.
/// Run the migration and regenerate entities first.
pub async fn save_metrics_snapshot(
    db: &impl ConnectionTrait,
    metrics: &MonitoringMetrics,
    timestamp: TimeStampUtc,
) -> Result<(), sea_orm::DbErr> {
    use entities::metrics_history;

    // Generate a unique ID using timestamp (which should be unique for each snapshot)
    // If there could be multiple snapshots per second, we could use timestamp + random
    let id = timestamp.timestamp();

    // Convert i64 timestamp to DateTime<FixedOffset> for database storage
    let timestamp_dt = timestamp.into();

    let runtime_fields =
        tokio_runtime_to_active_model(metrics.tokio.as_ref().and_then(|t| t.runtime.as_ref()));
    let task_fields =
        tokio_task_to_active_model(metrics.tokio.as_ref().and_then(|t| t.task.as_ref()));

    let snapshot = metrics_history::ActiveModel {
        id: Set(id),
        timestamp: Set(timestamp_dt),
        active_connections: Set(metrics.active_connections),
        total_users: Set(metrics.total_users),
        messages_per_second: Set(metrics.messages_per_second),
        uptime_seconds: Set(metrics.uptime_seconds),
        cpu_usage_percent: Set(metrics.cpu_usage_percent),
        memory_usage_percent: Set(metrics.memory_usage_percent),
        disk_usage_percent: Set(metrics.disk_usage_percent),
        total_sessions: Set(metrics.total_sessions),
        active_sessions: Set(metrics.active_sessions),
        database_connections: Set(metrics.database_connections),
        rabbitmq_connections: Set(metrics.rabbitmq_connections),
        // Combine tokio fields
        tokio_workers_count: runtime_fields.tokio_workers_count,
        tokio_total_park_count: runtime_fields.tokio_total_park_count,
        tokio_max_park_count: runtime_fields.tokio_max_park_count,
        tokio_min_park_count: runtime_fields.tokio_min_park_count,
        tokio_total_busy_duration: runtime_fields.tokio_total_busy_duration,
        tokio_max_busy_duration: runtime_fields.tokio_max_busy_duration,
        tokio_min_busy_duration: runtime_fields.tokio_min_busy_duration,
        tokio_global_queue_depth: runtime_fields.tokio_global_queue_depth,
        tokio_elapsed: runtime_fields.tokio_elapsed,
        tokio_live_tasks_count: runtime_fields.tokio_live_tasks_count,
        tokio_mean_poll_duration: runtime_fields.tokio_mean_poll_duration,
        tokio_mean_poll_duration_worker_min: runtime_fields.tokio_mean_poll_duration_worker_min,
        tokio_mean_poll_duration_worker_max: runtime_fields.tokio_mean_poll_duration_worker_max,
        tokio_total_noop_count: runtime_fields.tokio_total_noop_count,
        tokio_max_noop_count: runtime_fields.tokio_max_noop_count,
        tokio_min_noop_count: runtime_fields.tokio_min_noop_count,
        tokio_total_steal_count: runtime_fields.tokio_total_steal_count,
        tokio_max_steal_count: runtime_fields.tokio_max_steal_count,
        tokio_min_steal_count: runtime_fields.tokio_min_steal_count,
        tokio_total_steal_operations: runtime_fields.tokio_total_steal_operations,
        tokio_max_steal_operations: runtime_fields.tokio_max_steal_operations,
        tokio_min_steal_operations: runtime_fields.tokio_min_steal_operations,
        tokio_num_remote_schedules: runtime_fields.tokio_num_remote_schedules,
        tokio_total_local_schedule_count: runtime_fields.tokio_total_local_schedule_count,
        tokio_max_local_schedule_count: runtime_fields.tokio_max_local_schedule_count,
        tokio_min_local_schedule_count: runtime_fields.tokio_min_local_schedule_count,
        tokio_total_overflow_count: runtime_fields.tokio_total_overflow_count,
        tokio_max_overflow_count: runtime_fields.tokio_max_overflow_count,
        tokio_min_overflow_count: runtime_fields.tokio_min_overflow_count,
        tokio_total_polls_count: runtime_fields.tokio_total_polls_count,
        tokio_max_polls_count: runtime_fields.tokio_max_polls_count,
        tokio_min_polls_count: runtime_fields.tokio_min_polls_count,
        tokio_total_local_queue_depth: runtime_fields.tokio_total_local_queue_depth,
        tokio_max_local_queue_depth: runtime_fields.tokio_max_local_queue_depth,
        tokio_min_local_queue_depth: runtime_fields.tokio_min_local_queue_depth,
        tokio_blocking_queue_depth: runtime_fields.tokio_blocking_queue_depth,
        tokio_blocking_threads_count: runtime_fields.tokio_blocking_threads_count,
        tokio_idle_blocking_threads_count: runtime_fields.tokio_idle_blocking_threads_count,
        tokio_budget_forced_yield_count: runtime_fields.tokio_budget_forced_yield_count,
        tokio_io_driver_ready_count: runtime_fields.tokio_io_driver_ready_count,
        tokio_busy_ratio: runtime_fields.tokio_busy_ratio,
        tokio_mean_polls_per_park: runtime_fields.tokio_mean_polls_per_park,
        // Task metrics
        tokio_task_instrumented_count: task_fields.tokio_task_instrumented_count,
        tokio_task_dropped_count: task_fields.tokio_task_dropped_count,
        tokio_task_first_poll_count: task_fields.tokio_task_first_poll_count,
        tokio_task_total_first_poll_delay: task_fields.tokio_task_total_first_poll_delay,
        tokio_task_total_idled_count: task_fields.tokio_task_total_idled_count,
        tokio_task_total_idle_duration: task_fields.tokio_task_total_idle_duration,
        tokio_task_max_idle_duration: task_fields.tokio_task_max_idle_duration,
        tokio_task_total_scheduled_count: task_fields.tokio_task_total_scheduled_count,
        tokio_task_total_scheduled_duration: task_fields.tokio_task_total_scheduled_duration,
        tokio_task_total_poll_count: task_fields.tokio_task_total_poll_count,
        tokio_task_total_poll_duration: task_fields.tokio_task_total_poll_duration,
        tokio_task_total_fast_poll_count: task_fields.tokio_task_total_fast_poll_count,
        tokio_task_total_fast_poll_duration: task_fields.tokio_task_total_fast_poll_duration,
        tokio_task_total_slow_poll_count: task_fields.tokio_task_total_slow_poll_count,
        tokio_task_total_slow_poll_duration: task_fields.tokio_task_total_slow_poll_duration,
        tokio_task_total_short_delay_count: task_fields.tokio_task_total_short_delay_count,
        tokio_task_total_short_delay_duration: task_fields.tokio_task_total_short_delay_duration,
        tokio_task_total_long_delay_count: task_fields.tokio_task_total_long_delay_count,
        tokio_task_total_long_delay_duration: task_fields.tokio_task_total_long_delay_duration,
        tokio_task_mean_first_poll_delay: task_fields.tokio_task_mean_first_poll_delay,
        tokio_task_mean_idle_duration: task_fields.tokio_task_mean_idle_duration,
        tokio_task_mean_scheduled_duration: task_fields.tokio_task_mean_scheduled_duration,
        tokio_task_mean_poll_duration: task_fields.tokio_task_mean_poll_duration,
        tokio_task_slow_poll_ratio: task_fields.tokio_task_slow_poll_ratio,
        tokio_task_long_delay_ratio: task_fields.tokio_task_long_delay_ratio,
        tokio_task_mean_fast_poll_duration: task_fields.tokio_task_mean_fast_poll_duration,
        tokio_task_mean_slow_poll_duration: task_fields.tokio_task_mean_slow_poll_duration,
        tokio_task_mean_short_delay_duration: task_fields.tokio_task_mean_short_delay_duration,
        tokio_task_mean_long_delay_duration: task_fields.tokio_task_mean_long_delay_duration,
        ..Default::default()
    };

    snapshot.insert(db).await?;

    Ok(())
}

/// Convert TokioRuntimeMetrics to ActiveModel fields
fn tokio_runtime_to_active_model(
    runtime: Option<&TokioRuntimeMetrics>,
) -> MetricsHistoryEntity::ActiveModel {
    if let Some(r) = runtime {
        MetricsHistoryEntity::ActiveModel {
            tokio_workers_count: Set(Some(r.workers_count as i32)),
            tokio_total_park_count: Set(Some(r.total_park_count as i64)),
            tokio_max_park_count: Set(Some(r.max_park_count as i64)),
            tokio_min_park_count: Set(Some(r.min_park_count as i64)),
            tokio_total_busy_duration: Set(Some(r.total_busy_duration as i64)),
            tokio_max_busy_duration: Set(Some(r.max_busy_duration as i64)),
            tokio_min_busy_duration: Set(Some(r.min_busy_duration as i64)),
            tokio_global_queue_depth: Set(Some(r.global_queue_depth as i64)),
            tokio_elapsed: Set(Some(r.elapsed as i64)),
            tokio_live_tasks_count: Set(Some(r.live_tasks_count as i64)),
            tokio_mean_poll_duration: Set(Some(r.mean_poll_duration as i64)),
            tokio_mean_poll_duration_worker_min: Set(Some(r.mean_poll_duration_worker_min as i64)),
            tokio_mean_poll_duration_worker_max: Set(Some(r.mean_poll_duration_worker_max as i64)),
            tokio_total_noop_count: Set(Some(r.total_noop_count as i64)),
            tokio_max_noop_count: Set(Some(r.max_noop_count as i64)),
            tokio_min_noop_count: Set(Some(r.min_noop_count as i64)),
            tokio_total_steal_count: Set(Some(r.total_steal_count as i64)),
            tokio_max_steal_count: Set(Some(r.max_steal_count as i64)),
            tokio_min_steal_count: Set(Some(r.min_steal_count as i64)),
            tokio_total_steal_operations: Set(Some(r.total_steal_operations as i64)),
            tokio_max_steal_operations: Set(Some(r.max_steal_operations as i64)),
            tokio_min_steal_operations: Set(Some(r.min_steal_operations as i64)),
            tokio_num_remote_schedules: Set(Some(r.num_remote_schedules as i64)),
            tokio_total_local_schedule_count: Set(Some(r.total_local_schedule_count as i64)),
            tokio_max_local_schedule_count: Set(Some(r.max_local_schedule_count as i64)),
            tokio_min_local_schedule_count: Set(Some(r.min_local_schedule_count as i64)),
            tokio_total_overflow_count: Set(Some(r.total_overflow_count as i64)),
            tokio_max_overflow_count: Set(Some(r.max_overflow_count as i64)),
            tokio_min_overflow_count: Set(Some(r.min_overflow_count as i64)),
            tokio_total_polls_count: Set(Some(r.total_polls_count as i64)),
            tokio_max_polls_count: Set(Some(r.max_polls_count as i64)),
            tokio_min_polls_count: Set(Some(r.min_polls_count as i64)),
            tokio_total_local_queue_depth: Set(Some(r.total_local_queue_depth as i64)),
            tokio_max_local_queue_depth: Set(Some(r.max_local_queue_depth as i64)),
            tokio_min_local_queue_depth: Set(Some(r.min_local_queue_depth as i64)),
            tokio_blocking_queue_depth: Set(Some(r.blocking_queue_depth as i64)),
            tokio_blocking_threads_count: Set(Some(r.blocking_threads_count as i64)),
            tokio_idle_blocking_threads_count: Set(Some(r.idle_blocking_threads_count as i64)),
            tokio_budget_forced_yield_count: Set(Some(r.budget_forced_yield_count as i64)),
            tokio_io_driver_ready_count: Set(Some(r.io_driver_ready_count as i64)),
            tokio_busy_ratio: Set(Some(r.busy_ratio)),
            tokio_mean_polls_per_park: Set(Some(r.mean_polls_per_park)),
            ..Default::default()
        }
    } else {
        Default::default()
    }
}

/// Convert TokioTaskMetrics to ActiveModel fields
fn tokio_task_to_active_model(
    task: Option<&TokioTaskMetrics>,
) -> MetricsHistoryEntity::ActiveModel {
    if let Some(t) = task {
        MetricsHistoryEntity::ActiveModel {
            tokio_task_instrumented_count: Set(Some(t.instrumented_count as i64)),
            tokio_task_dropped_count: Set(Some(t.dropped_count as i64)),
            tokio_task_first_poll_count: Set(Some(t.first_poll_count as i64)),
            tokio_task_total_first_poll_delay: Set(Some(t.total_first_poll_delay as i64)),
            tokio_task_total_idled_count: Set(Some(t.total_idled_count as i64)),
            tokio_task_total_idle_duration: Set(Some(t.total_idle_duration as i64)),
            tokio_task_max_idle_duration: Set(Some(t.max_idle_duration as i64)),
            tokio_task_total_scheduled_count: Set(Some(t.total_scheduled_count as i64)),
            tokio_task_total_scheduled_duration: Set(Some(t.total_scheduled_duration as i64)),
            tokio_task_total_poll_count: Set(Some(t.total_poll_count as i64)),
            tokio_task_total_poll_duration: Set(Some(t.total_poll_duration as i64)),
            tokio_task_total_fast_poll_count: Set(Some(t.total_fast_poll_count as i64)),
            tokio_task_total_fast_poll_duration: Set(Some(t.total_fast_poll_duration as i64)),
            tokio_task_total_slow_poll_count: Set(Some(t.total_slow_poll_count as i64)),
            tokio_task_total_slow_poll_duration: Set(Some(t.total_slow_poll_duration as i64)),
            tokio_task_total_short_delay_count: Set(Some(t.total_short_delay_count as i64)),
            tokio_task_total_short_delay_duration: Set(Some(t.total_short_delay_duration as i64)),
            tokio_task_total_long_delay_count: Set(Some(t.total_long_delay_count as i64)),
            tokio_task_total_long_delay_duration: Set(Some(t.total_long_delay_duration as i64)),
            tokio_task_mean_first_poll_delay: Set(Some(t.mean_first_poll_delay as i64)),
            tokio_task_mean_idle_duration: Set(Some(t.mean_idle_duration as i64)),
            tokio_task_mean_scheduled_duration: Set(Some(t.mean_scheduled_duration as i64)),
            tokio_task_mean_poll_duration: Set(Some(t.mean_poll_duration as i64)),
            tokio_task_slow_poll_ratio: Set(Some(t.slow_poll_ratio)),
            tokio_task_long_delay_ratio: Set(Some(t.long_delay_ratio)),
            tokio_task_mean_fast_poll_duration: Set(Some(t.mean_fast_poll_duration as i64)),
            tokio_task_mean_slow_poll_duration: Set(Some(t.mean_slow_poll_duration as i64)),
            tokio_task_mean_short_delay_duration: Set(Some(t.mean_short_delay_duration as i64)),
            tokio_task_mean_long_delay_duration: Set(Some(t.mean_long_delay_duration as i64)),
            ..Default::default()
        }
    } else {
        Default::default()
    }
}
