use std::time::Duration;

use base::wrapper::JobSchedulerWrapper;
use deadpool_redis::redis::AsyncTypedCommands;
use tokio::sync::MutexGuard;
use tokio_cron_scheduler::Job;
use tracing::error;

use crate::webrtc::zero_room_name;

pub mod create_room;

pub async fn clean_rooms<'a>(
    duration: Duration,
    redis_pool: deadpool_redis::Pool,
    job_scheduler: MutexGuard<'a, JobSchedulerWrapper>,
) -> anyhow::Result<()> {
    let job = Job::new_repeated_async(duration, move |_uuid, _l| {
        let redis_pool = redis_pool.clone();
        Box::pin(async move {
            let logic = async move {
                let mut conn = redis_pool.get().await?;
                let empty_rooms = conn.smembers(zero_room_name()).await?;

                anyhow::Ok(())
            };
            if let Err(e) = logic.await {
                error!("Failed to clean rooms: {}", e);
            }
        })
    })?;
    job_scheduler.add(job).await?;
    Ok(())
}
