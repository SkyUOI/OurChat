//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.6

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user_chat_msg")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub chat_msg_id: i64,
    #[sea_orm(column_type = "JsonBinary")]
    pub msg_data: Json,
    pub sender_id: i64,
    pub session_id: Option<i64>,
    pub time: DateTimeWithTimeZone,
    pub is_encrypted: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::session::Entity",
        from = "Column::SessionId",
        to = "super::session::Column::SessionId",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Session,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::SenderId",
        to = "super::user::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Session.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
