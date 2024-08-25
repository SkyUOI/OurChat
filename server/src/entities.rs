//! 实体的抽象数据库中间层定义

pub mod mysql;
pub mod sqlite;

pub macro entities($($entity:tt)*) {
        if static_keys::static_branch_unlikely!($crate::db::SQLITE_TYPE) {
            {
                use $crate::entities::sqlite;
                $($entity)*
            }
        }else if static_keys::static_branch_unlikely!($crate::db::MYSQL_TYPE) {
            {
                use $crate::entities::mysql;
                $($entity)*
            }
        } else {
            tracing::error!("unknown db type");
            panic!("unknown db type");
        }
}
