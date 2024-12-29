use sea_orm::{DbErr, RuntimeErr};

/// Check if a `DbErr` is a conflict error.
///
/// This function returns true if the error is either a
/// `DbErr::RecordNotInserted` or a `DbErr::Query` with a
/// `sqlx::Error::Database` containing a PostgreSQL error
/// code "23505", which is a unique constraint violation.
///
/// If the error is not a conflict error, the function returns
/// false.
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
