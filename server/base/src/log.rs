use crate::consts::{LOG_ENV_VAR, LOG_OUTPUT_DIR};
use crate::setting::debug::DebugCfg;
use crate::wrapper::JobSchedulerWrapper;
use anyhow::anyhow;
use chrono::TimeDelta;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::fs;
use tokio::sync::MutexGuard;
use tokio_cron_scheduler::Job;
use tracing::error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::writer::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry, fmt};

/// Initialize the logger.
///
/// If `test_mode` is `true`, it will always set the log level to "trace".
/// Otherwise, it will read the log level from the environment variable
/// specified by [`LOG_ENV_VAR`] and set it to "info" if not present.
/// The log will be written to a file in the directory specified by
/// [`LOG_OUTPUT_DIR`], and the file name will be "test" if `test_mode` is
/// `true` and "ourchat" otherwise.
/// If `debug_cfg` is `Some` and `debug_console` is `true`, it will also
/// write the log to the console at the address specified by
/// `debug_cfg.debug_console_port`.
///
/// # Warning
/// This function should be called only once.
/// The second one will be ignored
pub fn logger_init<Sink>(
    test_mode: bool,
    debug_cfg: Option<&DebugCfg>,
    output: Sink,
    output_file: impl AsRef<Path>,
) where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    static INIT: OnceLock<Option<WorkerGuard>> = OnceLock::new();
    INIT.get_or_init(|| {
        let env = if test_mode {
            || EnvFilter::try_from_env(LOG_ENV_VAR).unwrap_or("trace".into())
        } else {
            || EnvFilter::try_from_env(LOG_ENV_VAR).unwrap_or("info".into())
        };
        let formatting_layer = fmt::layer().pretty().with_writer(output);
        let file_appender = if test_mode {
            tracing_appender::rolling::never(LOG_OUTPUT_DIR, "test")
        } else {
            tracing_appender::rolling::daily(LOG_OUTPUT_DIR, output_file)
        };
        let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);
        let tmp = Registry::default()
            .with(env())
            .with(formatting_layer)
            .with(fmt::layer().with_ansi(false).with_writer(non_blocking));
        if let Some(debug_cfg) = debug_cfg {
            if debug_cfg.debug_console {
                // TODO:move this to "debug" section of config
                let console_layer = console_subscriber::ConsoleLayer::builder()
                    .retention(Duration::from_secs(60))
                    .server_addr(([0, 0, 0, 0], debug_cfg.debug_console_port))
                    .spawn();
                tmp.with(console_layer).init();
            }
        } else {
            tmp.init();
        }
        Some(file_guard)
    });
}

pub async fn add_clean_to_scheduler<'a>(
    log_file_prefix: impl Into<String>,
    log_keep: Duration,
    duration: Duration,
    sched: MutexGuard<'a, JobSchedulerWrapper>,
) -> anyhow::Result<()> {
    let log_file_prefix = log_file_prefix.into();
    let job = Job::new_repeated_async(duration, move |_uuid, _l| {
        let log_file_prefix = log_file_prefix.clone();
        Box::pin(async move {
            let now = chrono::Local::now().date_naive();
            let logic = async {
                let mut tmp = fs::read_dir(LOG_OUTPUT_DIR).await?;
                while let Some(i) = tmp.next_entry().await? {
                    let path = i.path();
                    if path.is_file()
                        && path.file_prefix() == Some(OsStr::new(&log_file_prefix))
                        && let Some(date) = path.clone().extension()
                    {
                        let remove_logic = async {
                            let mut date = date
                                .to_str()
                                .ok_or_else(|| anyhow!("no date info"))?
                                .split(".");
                            let date = chrono::NaiveDate::from_ymd_opt(
                                date.next()
                                    .ok_or_else(|| anyhow!("missing year"))?
                                    .parse()?,
                                date.next()
                                    .ok_or_else(|| anyhow!("missing month"))?
                                    .parse()?,
                                date.next().ok_or_else(|| anyhow!("missing day"))?.parse()?,
                            )
                            .ok_or_else(|| anyhow!("date invalid"))?;
                            let date_diff = now.signed_duration_since(date);
                            if date_diff >= TimeDelta::from_std(log_keep)? {
                                fs::remove_file(path).await?;
                            }
                            anyhow::Ok(())
                        };
                        if let Err(e) = remove_logic.await {
                            error!("Error when delete log file: {e}")
                        }
                    }
                }
                anyhow::Ok(())
            };
            if let Err(e) = logic.await {
                error!("Error when cleaning log: {e}")
            }
        })
    })?;
    sched.add(job).await?;
    Ok(())
}
