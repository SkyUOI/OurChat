//! For Data-Driven Development

use migration::m20241229_022701_add_role_for_session::{PredefinedPermissions, PredefinedRoles};
use utils::impl_newtype_int;

impl_newtype_int!(RoleId, u64,);
impl_newtype_int!(PermissionId, u64,);

impl From<PredefinedRoles> for RoleId {
    fn from(value: PredefinedRoles) -> Self {
        RoleId(value.into())
    }
}

impl From<RoleId> for sea_orm::Value {
    fn from(value: RoleId) -> Self {
        sea_orm::Value::BigUnsigned(Some(value.0))
    }
}

impl From<PredefinedPermissions> for PermissionId {
    fn from(value: PredefinedPermissions) -> Self {
        PermissionId(value.into())
    }
}

impl From<PermissionId> for sea_orm::Value {
    fn from(value: PermissionId) -> Self {
        sea_orm::Value::BigUnsigned(Some(value.0))
    }
}
