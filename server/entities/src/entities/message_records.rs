//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.13

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "message_records")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub msg_id: i64,
    #[sea_orm(column_type = "JsonBinary")]
    pub msg_data: Json,
    pub sender_id: Option<i64>,
    pub session_id: Option<i64>,
    pub time: DateTimeWithTimeZone,
    pub is_encrypted: bool,
    pub is_all_user: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::announcement_msg::Entity")]
    AnnouncementMsg,
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

impl Related<super::announcement_msg::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AnnouncementMsg.def()
    }
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
