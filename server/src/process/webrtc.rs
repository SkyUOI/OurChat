use std::time::Duration;

use base::{database::DbPool, wrapper::JobSchedulerWrapper};
use deadpool_redis::redis::AsyncTypedCommands;
use sea_orm::ActiveModelTrait;
use tokio::sync::MutexGuard;
use tokio_cron_scheduler::Job;
use tracing::error;

use crate::webrtc::{RoomInfo, empty_room_name, room_key};

pub mod create_room;

pub async fn move_room_from_redis_to_postgres(
    room_key: &str,
    room_info: RoomInfo,
    redis_conn: &mut deadpool_redis::Connection,
    db_conn: &impl sea_orm::ConnectionTrait,
) -> anyhow::Result<()> {
    let entity = entities::rtc_room::ActiveModel {
        room_id: sea_orm::ActiveValue::Set(room_info.room_id.into()),
        title: sea_orm::ActiveValue::Set(room_info.title.unwrap_or_default()),
        users_num: sea_orm::ActiveValue::Set(room_info.users_num as i32),
    };
    entity.insert(db_conn).await?;
    redis_conn.del(room_key).await?;
    Ok(())
}

pub async fn clean_rooms<'a>(
    duration: Duration,
    db_pool: DbPool,
    job_scheduler: MutexGuard<'a, JobSchedulerWrapper>,
) -> anyhow::Result<()> {
    let job = Job::new_repeated_async(duration, move |_uuid, _l| {
        let db_pool = db_pool.clone();
        Box::pin(async move {
            let logic = async move {
                let mut conn = db_pool.redis_pool.get().await?;
                let empty_rooms = conn.smembers(empty_room_name()).await?;
                for i in empty_rooms.iter() {
                    let room_id = i.parse()?;
                    let room_key = room_key(room_id);
                    let room_info = RoomInfo::from_redis(&mut conn, &room_key).await?;
                    if room_info.auto_delete {
                        let mut pipe = deadpool_redis::redis::pipe();
                        pipe.atomic();
                        pipe.srem(empty_room_name(), i);
                        pipe.del(room_key);
                        let _: () = pipe.query_async(&mut conn).await?;
                    } else {
                        move_room_from_redis_to_postgres(
                            &room_key,
                            room_info,
                            &mut conn,
                            &db_pool.db_pool,
                        )
                        .await?;
                    }
                }

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
