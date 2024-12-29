//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    #[sea_orm(unique)]
    pub ocid: String,
    #[sea_orm(column_type = "Text")]
    pub passwd: String,
    pub name: String,
    #[sea_orm(unique)]
    pub email: String,
    pub time: DateTimeWithTimeZone,
    pub resource_used: i64,
    pub friend_limit: i32,
    pub friends_num: i32,
    pub avatar: Option<String>,
    pub public_update_time: DateTimeWithTimeZone,
    pub update_time: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::files::Entity")]
    Files,
    #[sea_orm(has_many = "super::operations::Entity")]
    Operations,
    #[sea_orm(has_many = "super::session_relation::Entity")]
    SessionRelation,
    #[sea_orm(has_many = "super::user_chat_msg::Entity")]
    UserChatMsg,
    #[sea_orm(has_many = "super::user_role_relation::Entity")]
    UserRoleRelation,
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Files.def()
    }
}

impl Related<super::operations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Operations.def()
    }
}

impl Related<super::session_relation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SessionRelation.def()
    }
}

impl Related<super::user_chat_msg::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserChatMsg.def()
    }
}

impl Related<super::user_role_relation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserRoleRelation.def()
    }
}

impl Related<super::session::Entity> for Entity {
    fn to() -> RelationDef {
        super::session_relation::Relation::Session.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::session_relation::Relation::User.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
