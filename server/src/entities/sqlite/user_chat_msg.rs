//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user_chat_msg")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub chat_msg_id: i64,
    pub msg_type: i32,
    pub msg_data: String,
    pub sender_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::SenderId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    User,
    #[sea_orm(has_one = "super::user_chat_msg_relation::Entity")]
    UserChatMsgRelation,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::user_chat_msg_relation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserChatMsgRelation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
