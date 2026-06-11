use sea_orm::{DbErr, RuntimeErr};

const POSTGRES_UNIQUE_VIOLATION: &str = "23505";

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
        DbErr::Query(RuntimeErr::SqlxError(e)) => {
            if let sqlx::Error::Database(ref db_err) = **e {
                if let Some(code) = db_err.code() {
                    code == POSTGRES_UNIQUE_VIOLATION
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false,
    }
}
