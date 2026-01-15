use crate::error::LibraryError;
use rusqlite::Connection;
use tracing::info;

pub fn apply_migrations(conn: &Connection) -> Result<(), LibraryError> {
    let version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if version < 1 {
        info!("Applying migration 0001_init");
        conn.execute_batch(include_str!("../../migrations/0001_init.sql"))?;
        conn.pragma_update(None, "user_version", 1)?;
    }

    Ok(())
}
