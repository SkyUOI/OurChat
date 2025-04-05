//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.8

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "announcement_msg")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub announcement_id: i64,
    pub msg_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::announcement::Entity",
        from = "Column::AnnouncementId",
        to = "super::announcement::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Announcement,
    #[sea_orm(
        belongs_to = "super::message_records::Entity",
        from = "Column::MsgId",
        to = "super::message_records::Column::MsgId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    MessageRecords,
}

impl Related<super::announcement::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Announcement.def()
    }
}

impl Related<super::message_records::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MessageRecords.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
