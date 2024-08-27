//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user_chat_id")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub chat_msg_id: i64,
    pub msg_type: i32,
    pub msg_data: String,
    pub sender_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
