//! Private sticker collections (#147).
//!
//! A sticker is a reference to a file the user uploaded themselves. Only the
//! owner of a file may add it to their collection, and every user manages
//! their own collection only.

use crate::process::error_msg::{PERMISSION_DENIED, SERVER_ERROR, exist, not_found};
use crate::server::RpcServer;
use base::constants::ID;
use entities::stickers;
use pb::service::ourchat::sticker::v1::{
    AddStickerRequest, AddStickerResponse, GetStickersRequest, GetStickersResponse,
    RemoveStickerRequest, RemoveStickerResponse, Sticker,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder,
};
use tonic::{Request, Response, Status};

pub async fn add_sticker(
    server: &RpcServer,
    id: ID,
    request: Request<AddStickerRequest>,
) -> Result<Response<AddStickerResponse>, Status> {
    match add_sticker_internal(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => Err(map_err(e)),
    }
}

pub async fn remove_sticker(
    server: &RpcServer,
    id: ID,
    request: Request<RemoveStickerRequest>,
) -> Result<Response<RemoveStickerResponse>, Status> {
    match remove_sticker_internal(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => Err(map_err(e)),
    }
}

pub async fn get_stickers(
    server: &RpcServer,
    id: ID,
    request: Request<GetStickersRequest>,
) -> Result<Response<GetStickersResponse>, Status> {
    match get_stickers_internal(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => Err(map_err(e)),
    }
}

#[derive(Debug, thiserror::Error)]
enum StickerErr {
    #[error("database error:{0:?}")]
    Db(#[from] DbErr),
    #[error("status:{0:?}")]
    Status(#[from] Status),
}

fn map_err(e: StickerErr) -> Status {
    match e {
        StickerErr::Db(db_err) => {
            tracing::error!("database error:{db_err:?}");
            Status::internal(SERVER_ERROR)
        }
        StickerErr::Status(status) => status,
    }
}

async fn add_sticker_internal(
    server: &RpcServer,
    id: ID,
    request: Request<AddStickerRequest>,
) -> Result<AddStickerResponse, StickerErr> {
    let req = request.into_inner();
    let db_conn = &server.db.db_pool;
    // The file must exist and belong to the requesting user: stickers cannot
    // reference other people's files
    let file = entities::files::Entity::find_by_id(&req.file_key)
        .one(db_conn)
        .await?
        .ok_or_else(|| Status::not_found(not_found::FILE))?;
    if file.user_id != i64::from(id) {
        return Err(Status::permission_denied(PERMISSION_DENIED).into());
    }
    let sticker = stickers::ActiveModel {
        user_id: ActiveValue::Set(id.into()),
        file_key: ActiveValue::Set(req.file_key),
        ..Default::default()
    };
    if let Err(e) = sticker.insert(db_conn).await {
        // the composite primary key (user_id, file_key) rejects duplicates
        return match crate::db::helper::is_conflict(&e) {
            true => Err(Status::already_exists(exist::STICKER).into()),
            false => Err(e.into()),
        };
    }
    Ok(AddStickerResponse {})
}

async fn remove_sticker_internal(
    server: &RpcServer,
    id: ID,
    request: Request<RemoveStickerRequest>,
) -> Result<RemoveStickerResponse, StickerErr> {
    let req = request.into_inner();
    let db_conn = &server.db.db_pool;
    // Users can only delete their own stickers: the composite primary key
    // scopes the deletion to (id, file_key), so removing another user's
    // sticker simply matches no row
    let result = stickers::Entity::delete_by_id((id.into(), req.file_key))
        .exec(db_conn)
        .await?;
    if result.rows_affected == 0 {
        return Err(Status::not_found(not_found::STICKER).into());
    }
    Ok(RemoveStickerResponse {})
}

async fn get_stickers_internal(
    server: &RpcServer,
    id: ID,
    _request: Request<GetStickersRequest>,
) -> Result<GetStickersResponse, StickerErr> {
    let db_conn = &server.db.db_pool;
    let own_stickers = stickers::Entity::find()
        .filter(stickers::Column::UserId.eq(id))
        .order_by_asc(stickers::Column::AddedAt)
        .all(db_conn)
        .await?;
    Ok(GetStickersResponse {
        stickers: own_stickers
            .into_iter()
            .map(|s| Sticker {
                file_key: s.file_key,
                added_at: Some(s.added_at.into()),
            })
            .collect(),
    })
}
