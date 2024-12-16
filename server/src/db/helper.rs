use sea_orm::{DbErr, RuntimeErr};

pub fn is_conflict(e: &DbErr) -> bool {
    match e {
        DbErr::RecordNotInserted => true,
        DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(e))) => {
            if let Some(code) = e.code() {
                code == "23505"
            } else {
                false
            }
        }
        _ => false,
    }
}
