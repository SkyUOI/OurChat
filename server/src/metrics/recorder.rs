//! Custom metrics recorder that stores metrics for gRPC API responses
//!
//! This recorder implements the `metrics::Recorder` trait to capture metrics
//! throughout the application and store them for retrieval via the gRPC monitoring API.

use base::shutdown::ShutdownSdr;
use metrics::gauge;
use metrics::{Counter, Gauge, Histogram, Key, KeyName, Metadata, Recorder, SharedString, Unit};
use parking_lot::{RwLock, RwLockWriteGuard};
use sea_orm::{ConnectionTrait, EntityTrait, PaginatorTrait};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Instant;
use sysinfo::System;
use tokio::sync::Mutex as TokioMutex;

use pb::service::server_manage::monitoring::v1::{
    MonitoringMetrics, TokioMetrics, TokioRuntimeMetrics, TokioTaskMetrics,
};

/// Latest tokio runtime metrics captured by the monitor
#[derive(Debug, Default)]
struct LatestTokioMetrics {
    runtime_metrics: Option<tokio_metrics::RuntimeMetrics>,
    task_metrics: Option<tokio_metrics::TaskMetrics>,
}

/// Internal metrics storage
#[derive(Debug, Default)]
struct MetricsStorage {
    // Basic metrics
    active_connections: i64,
    total_users: i64,
    active_sessions: i64,
    database_connections: i64,
    rabbitmq_connections: i64,
    messages_sent: u64,
    messages_received: u64,
    last_message_rate_update: Option<Instant>,
    previous_message_count: u64,
    current_message_rate: f64,

    // System metrics (updated via sysinfo)
    cpu_usage_percent: f64,
    memory_usage_percent: f64,
    disk_usage_percent: f64,
}

/// Counter function that stores values in shared storage
struct StorageCounter {
    key: String,
    storage: Arc<RwLock<MetricsStorage>>,
}

impl metrics::CounterFn for StorageCounter {
    fn increment(&self, value: u64) {
        let mut storage = self.storage.write();
        let key = self.key.as_str();

        match key {
            "messages_sent_total" | "messages_sent" => {
                storage.messages_sent = storage.messages_sent.saturating_add(value);
                // Drop lock before calling update_message_rate
                drop(storage);
                self.update_message_rate();
            }
            "messages_received_total" | "messages_received" => {
                storage.messages_received = storage.messages_received.saturating_add(value);
                drop(storage);
                self.update_message_rate();
            }
            _ => {}
        }
    }

    fn absolute(&self, value: u64) {
        let mut storage = self.storage.write();
        let key = self.key.as_str();

        match key {
            "messages_sent_total" | "messages_sent" => {
                storage.messages_sent = value;
                drop(storage);
                self.update_message_rate();
            }
            "messages_received_total" | "messages_received" => {
                storage.messages_received = value;
                drop(storage);
                self.update_message_rate();
            }
            _ => {}
        }
    }
}

impl StorageCounter {
    fn update_message_rate(&self) {
        let mut storage = self.storage.write();
        let total_messages = storage.messages_sent + storage.messages_received;

        let now = Instant::now();
        if let Some(last_update) = storage.last_message_rate_update {
            let elapsed = now.duration_since(last_update).as_secs_f64();
            if elapsed >= 1.0 {
                let message_delta = total_messages.saturating_sub(storage.previous_message_count);
                storage.current_message_rate = if elapsed > 0.0 {
                    message_delta as f64 / elapsed
                } else {
                    0.0
                };
                storage.last_message_rate_update = Some(now);
                storage.previous_message_count = total_messages;
            }
        } else {
            storage.last_message_rate_update = Some(now);
            storage.previous_message_count = total_messages;
        }
    }
}

/// Gauge function that stores values in shared storage
struct StorageGauge {
    key: String,
    storage: Arc<RwLock<MetricsStorage>>,
}

impl metrics::GaugeFn for StorageGauge {
    fn increment(&self, value: f64) {
        let mut storage = self.storage.write();
        self.update_gauge_locked(&mut storage, |current| current + value);
    }

    fn decrement(&self, value: f64) {
        let mut storage = self.storage.write();
        self.update_gauge_locked(&mut storage, |current| current - value);
    }

    fn set(&self, value: f64) {
        let mut storage = self.storage.write();
        self.update_gauge_locked(&mut storage, |_| value);
    }
}

impl StorageGauge {
    fn update_gauge_locked(
        &self,
        storage: &mut RwLockWriteGuard<MetricsStorage>,
        f: impl FnOnce(f64) -> f64,
    ) {
        let key = self.key.as_str();

        match key {
            "active_sessions" => {
                storage.active_sessions = f(storage.active_sessions as f64) as i64;
            }
            "total_users" => {
                storage.total_users = f(storage.total_users as f64) as i64;
            }
            "database_connections" => {
                storage.database_connections = f(storage.database_connections as f64) as i64;
            }
            "rabbitmq_connections" => {
                storage.rabbitmq_connections = f(storage.rabbitmq_connections as f64) as i64;
            }
            "active_connections" => {
                storage.active_connections = f(storage.active_connections as f64) as i64;
            }
            "cpu_usage_percent" => {
                storage.cpu_usage_percent = f(storage.cpu_usage_percent);
            }
            "memory_usage_percent" => {
                storage.memory_usage_percent = f(storage.memory_usage_percent);
            }
            "disk_usage_percent" => {
                storage.disk_usage_percent = f(storage.disk_usage_percent);
            }
            _ => {}
        }
    }
}

/// Histogram function (no-op for now)
struct NoopHistogram;

impl metrics::HistogramFn for NoopHistogram {
    fn record(&self, _value: f64) {
        // No-op for now
    }
}

/// Custom recorder that stores metrics for gRPC responses
#[derive(Debug)]
pub struct OurChatRecorder {
    storage: Arc<RwLock<MetricsStorage>>,

    /// System metrics collector (still needed for sysinfo)
    system: Arc<RwLock<System>>,

    /// Server start time for uptime calculation
    start_time: Instant,

    /// tokio runtime metrics (collected via background task)
    tokio_metrics: Arc<TokioMutex<LatestTokioMetrics>>,

    /// Shutdown sender to register and stop the background metrics collection task
    shutdown_sdr: ShutdownSdr,
}

impl OurChatRecorder {
    /// Create a new recorder with a shutdown sender for task registration
    pub fn new(shutdown_sdr: ShutdownSdr) -> Self {
        let system = System::new_all();
        let tokio_metrics = Arc::new(TokioMutex::new(LatestTokioMetrics::default()));

        Self {
            storage: Arc::new(RwLock::new(MetricsStorage::default())),
            system: Arc::new(RwLock::new(system)),
            start_time: Instant::now(),
            tokio_metrics,
            shutdown_sdr,
        }
    }

    /// Start background tokio metrics collection
    ///
    /// This registers the collection task with the shutdown system.
    /// The task will exit gracefully when `shutdown_all_tasks()` is called.
    pub fn start_tokio_metrics_collection(&self, sample_interval: std::time::Duration) {
        let handle = tokio::runtime::Handle::current();
        let runtime_monitor = tokio_metrics::RuntimeMonitor::new(&handle);
        let task_monitor = tokio_metrics::TaskMonitor::new();

        let tokio_metrics_clone = self.tokio_metrics.clone();
        // Register this task with the shutdown system
        let mut shutdown_rev = self.shutdown_sdr.new_receiver(
            "metrics_collection",
            "Background tokio metrics collection task",
        );

        tokio::spawn(async move {
            let mut intervals = tokio::time::interval(sample_interval);
            let mut runtime_intervals = runtime_monitor.intervals();
            let mut task_intervals = task_monitor.intervals();

            loop {
                tokio::select! {
                    _ = shutdown_rev.wait_shutting_down() => {
                        tracing::debug!("Metrics collection task shutting down");
                        break;
                    }
                    _ = intervals.tick() => {
                        // Get the next runtime metrics
                        if let Some(runtime_metrics) = runtime_intervals.next() {
                            let mut latest = tokio_metrics_clone.lock().await;
                            latest.runtime_metrics = Some(runtime_metrics);
                        }

                        // Get the next task metrics
                        if let Some(task_metrics) = task_intervals.next() {
                            let mut latest = tokio_metrics_clone.lock().await;
                            latest.task_metrics = Some(task_metrics);
                        }
                    }
                }
            }
        });
    }

    /// Update system metrics (CPU, memory, disk usage)
    pub fn update_system_metrics(&self) {
        let mut sys = self.system.write();
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_usage();
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let memory_usage = if total_memory > 0 {
            (used_memory as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };

        // TODO: Implement proper disk tracking using sysinfo::System::disks()
        // For now, use 0.0 as a placeholder
        let disk_usage = 0.0;

        let mut storage = self.storage.write();
        storage.cpu_usage_percent = cpu_usage as f64;
        storage.memory_usage_percent = memory_usage;
        storage.disk_usage_percent = disk_usage;
    }

    /// Get current metrics as a protobuf struct
    ///
    /// When a database connection is provided, certain metrics (like total_users,
    /// total_sessions, active_sessions) are fetched from the database for accuracy.
    /// Otherwise, falls back to internally tracked values.
    pub async fn get_monitoring_metrics(
        &self,
        db: &(impl ConnectionTrait + Send),
        pg_pool: &PgPool,
        include_system_metrics: bool,
        include_tokio_metrics: bool,
    ) -> MonitoringMetrics {
        // Query database for certain metrics if available
        let (total_users, total_sessions, active_sessions) = self.query_db_metrics(db).await;

        // Get actual database pool statistics and update gauge BEFORE acquiring read lock
        // to avoid deadlock (gauge!().set() tries to acquire write lock)
        let database_connections = pg_pool.size() as i32;
        gauge!("database_connections").set(database_connections as f64);

        let storage = self.storage.read();

        MonitoringMetrics {
            active_connections: storage.active_connections as i32,
            total_users,
            messages_per_second: storage.current_message_rate,
            uptime_seconds: self.start_time.elapsed().as_secs() as i64,
            timestamp: chrono::Utc::now().timestamp(),
            total_sessions,
            active_sessions,
            database_connections,
            rabbitmq_connections: storage.rabbitmq_connections as i32,
            cpu_usage_percent: if include_system_metrics {
                Some(storage.cpu_usage_percent)
            } else {
                None
            },
            memory_usage_percent: if include_system_metrics {
                Some(storage.memory_usage_percent)
            } else {
                None
            },
            disk_usage_percent: if include_system_metrics {
                Some(storage.disk_usage_percent)
            } else {
                None
            },
            tokio: if include_tokio_metrics {
                Some(self.collect_tokio_metrics())
            } else {
                None
            },
        }
    }

    /// Query the database for certain metrics
    async fn query_db_metrics(&self, db: &(impl ConnectionTrait + Send)) -> (i32, i32, i32) {
        // Query total users (excluding soft-deleted users)
        let total_users = self.query_total_users(db).await.unwrap_or(0) as i32;

        // Query total sessions
        let total_sessions = self.query_total_sessions(db).await.unwrap_or(0) as i32;

        // For active sessions, we use the internally tracked value since
        // there's no reliable way to determine "active" from DB alone
        let storage = self.storage.read();
        let active_sessions = storage.active_sessions as i32;
        drop(storage);

        (total_users, total_sessions, active_sessions)
    }

    /// Query the total number of users from the database
    async fn query_total_users(
        &self,
        db: &(impl ConnectionTrait + Send),
    ) -> Result<u64, sea_orm::DbErr> {
        use entities::user;
        use sea_orm::{ColumnTrait, QueryFilter, QuerySelect};

        // Count users where deleted_at is NULL
        let count = user::Entity::find()
            .filter(user::Column::DeletedAt.is_null())
            .select_only()
            .count(db)
            .await?;
        Ok(count)
    }

    /// Query the total number of sessions from the database
    async fn query_total_sessions(
        &self,
        db: &(impl ConnectionTrait + Send),
    ) -> Result<u64, sea_orm::DbErr> {
        use entities::session;
        use sea_orm::QuerySelect;

        let count = session::Entity::find().select_only().count(db).await?;
        Ok(count)
    }

    /// Collect current tokio runtime metrics
    fn collect_tokio_metrics(&self) -> TokioMetrics {
        // Try to get the latest tokio metrics without blocking
        let runtime = if let Ok(latest) = self.tokio_metrics.try_lock() {
            latest
                .runtime_metrics
                .as_ref()
                .map(|m| self.runtime_metrics_to_protobuf(m))
        } else {
            None
        };

        let task = if let Ok(latest) = self.tokio_metrics.try_lock() {
            latest
                .task_metrics
                .as_ref()
                .map(|m| self.task_metrics_to_protobuf(m))
        } else {
            None
        };

        TokioMetrics {
            runtime: Some(runtime.unwrap_or_default()),
            task: Some(task.unwrap_or_default()),
        }
    }

    fn runtime_metrics_to_protobuf(
        &self,
        m: &tokio_metrics::RuntimeMetrics,
    ) -> TokioRuntimeMetrics {
        TokioRuntimeMetrics {
            workers_count: m.workers_count as u32,
            total_park_count: m.total_park_count,
            max_park_count: m.max_park_count,
            min_park_count: m.min_park_count,
            total_busy_duration: m.total_busy_duration.as_nanos() as u64,
            max_busy_duration: m.max_busy_duration.as_nanos() as u64,
            min_busy_duration: m.min_busy_duration.as_nanos() as u64,
            global_queue_depth: m.global_queue_depth as u64,
            elapsed: m.elapsed.as_nanos() as u64,
            live_tasks_count: m.live_tasks_count as u64,
            mean_poll_duration: m.mean_poll_duration.as_nanos() as u64,
            mean_poll_duration_worker_min: m.mean_poll_duration_worker_min.as_nanos() as u64,
            mean_poll_duration_worker_max: m.mean_poll_duration_worker_max.as_nanos() as u64,
            total_noop_count: m.total_noop_count,
            max_noop_count: m.max_noop_count,
            min_noop_count: m.min_noop_count,
            total_steal_count: m.total_steal_count,
            max_steal_count: m.max_steal_count,
            min_steal_count: m.min_steal_count,
            total_steal_operations: m.total_steal_operations,
            max_steal_operations: m.max_steal_operations,
            min_steal_operations: m.min_steal_operations,
            num_remote_schedules: m.num_remote_schedules,
            total_local_schedule_count: m.total_local_schedule_count,
            max_local_schedule_count: m.max_local_schedule_count,
            min_local_schedule_count: m.min_local_schedule_count,
            total_overflow_count: m.total_overflow_count,
            max_overflow_count: m.max_overflow_count,
            min_overflow_count: m.min_overflow_count,
            total_polls_count: m.total_polls_count,
            max_polls_count: m.max_polls_count,
            min_polls_count: m.min_polls_count,
            total_local_queue_depth: m.total_local_queue_depth as u64,
            max_local_queue_depth: m.max_local_queue_depth as u64,
            min_local_queue_depth: m.min_local_queue_depth as u64,
            blocking_queue_depth: m.blocking_queue_depth as u64,
            blocking_threads_count: m.blocking_threads_count as u64,
            idle_blocking_threads_count: m.idle_blocking_threads_count as u64,
            budget_forced_yield_count: m.budget_forced_yield_count,
            io_driver_ready_count: m.io_driver_ready_count,
            busy_ratio: m.busy_ratio(),
            mean_polls_per_park: m.mean_polls_per_park(),
        }
    }

    fn task_metrics_to_protobuf(&self, m: &tokio_metrics::TaskMetrics) -> TokioTaskMetrics {
        TokioTaskMetrics {
            instrumented_count: m.instrumented_count,
            dropped_count: m.dropped_count,
            first_poll_count: m.first_poll_count,
            total_first_poll_delay: m.total_first_poll_delay.as_nanos() as u64,
            total_idled_count: m.total_idled_count,
            total_idle_duration: m.total_idle_duration.as_nanos() as u64,
            max_idle_duration: m.max_idle_duration.as_nanos() as u64,
            total_scheduled_count: m.total_scheduled_count,
            total_scheduled_duration: m.total_scheduled_duration.as_nanos() as u64,
            total_poll_count: m.total_poll_count,
            total_poll_duration: m.total_poll_duration.as_nanos() as u64,
            total_fast_poll_count: m.total_fast_poll_count,
            total_fast_poll_duration: m.total_fast_poll_duration.as_nanos() as u64,
            total_slow_poll_count: m.total_slow_poll_count,
            total_slow_poll_duration: m.total_slow_poll_duration.as_nanos() as u64,
            total_short_delay_count: m.total_short_delay_count,
            total_short_delay_duration: m.total_short_delay_duration.as_nanos() as u64,
            total_long_delay_count: m.total_long_delay_count,
            total_long_delay_duration: m.total_long_delay_duration.as_nanos() as u64,
            mean_first_poll_delay: m.mean_first_poll_delay().as_nanos() as u64,
            mean_idle_duration: m.mean_idle_duration().as_nanos() as u64,
            mean_scheduled_duration: m.mean_scheduled_duration().as_nanos() as u64,
            mean_poll_duration: m.mean_poll_duration().as_nanos() as u64,
            slow_poll_ratio: m.slow_poll_ratio(),
            long_delay_ratio: m.long_delay_ratio(),
            mean_fast_poll_duration: m.mean_fast_poll_duration().as_nanos() as u64,
            mean_slow_poll_duration: m.mean_slow_poll_duration().as_nanos() as u64,
            mean_short_delay_duration: m.mean_short_delay_duration().as_nanos() as u64,
            mean_long_delay_duration: m.mean_long_delay_duration().as_nanos() as u64,
        }
    }
}

// Implement the Recorder trait
impl Recorder for OurChatRecorder {
    fn describe_counter(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        // No-op for now
    }

    fn describe_gauge(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        // No-op for now
    }

    fn describe_histogram(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        // No-op for now
    }

    fn register_counter(&self, key: &Key, _metadata: &Metadata<'_>) -> Counter {
        let key_str = key.to_string();
        let counter = StorageCounter {
            key: key_str.clone(),
            storage: self.storage.clone(),
        };
        Counter::from_arc(Arc::new(counter))
    }

    fn register_gauge(&self, key: &Key, _metadata: &Metadata<'_>) -> Gauge {
        let key_str = key.to_string();
        let gauge = StorageGauge {
            key: key_str,
            storage: self.storage.clone(),
        };
        Gauge::from_arc(Arc::new(gauge))
    }

    fn register_histogram(&self, _key: &Key, _metadata: &Metadata<'_>) -> Histogram {
        Histogram::from_arc(Arc::new(NoopHistogram))
    }
}

/// Helper to set up the global recorder
///
/// This creates a new recorder, installs it globally, and returns an Arc for our own use.
/// The returned Arc has tokio metrics collection running on it.
///
/// # Arguments
///
/// * `interval` - The interval at which to collect tokio metrics
/// * `shutdown_sdr` - The shutdown sender to register the background task with
pub fn setup_global_recorder(
    interval: std::time::Duration,
    shutdown_sdr: ShutdownSdr,
) -> Arc<OurChatRecorder> {
    // Create our recorder that we'll use for gRPC access
    let recorder = Arc::new(OurChatRecorder::new(shutdown_sdr));
    recorder.start_tokio_metrics_collection(interval);

    // Install the recorder globally for the metrics facade
    // We clone the Arc so both the global recorder and our returned Arc point to the same storage
    if let Err(e) = metrics::set_global_recorder(recorder.clone()) {
        tracing::warn!("Failed to set global metrics recorder: {}", e);
    }

    recorder
}
