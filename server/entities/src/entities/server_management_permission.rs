//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.6

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "server_management_permission")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub description: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::server_management_role_permissions::Entity")]
    ServerManagementRolePermissions,
}

impl Related<super::server_management_role_permissions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ServerManagementRolePermissions.def()
    }
}

impl Related<super::server_management_role::Entity> for Entity {
    fn to() -> RelationDef {
        super::server_management_role_permissions::Relation::ServerManagementRole.def()
    }
    fn via() -> Option<RelationDef> {
        Some(
            super::server_management_role_permissions::Relation::ServerManagementPermission
                .def()
                .rev(),
        )
    }
}

impl ActiveModelBehavior for ActiveModel {}
