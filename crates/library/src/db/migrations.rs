use crate::error::LibraryError;
use rusqlite::Connection;
use tracing::info;

pub fn apply_migrations(conn: &Connection) -> Result<(), LibraryError> {
    let mut version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if version < 1 {
        info!("Applying migration 0001_init");
        conn.execute_batch(include_str!("../../migrations/0001_init.sql"))?;
        version = 1;
    }

    if version < 2 {
        info!("Applying migration 0002_settings");
        conn.execute_batch(include_str!("../../migrations/0002_settings.sql"))?;
        version = 2;
    }

    if version < 3 {
        info!("Applying migration 0003_fts");
        conn.execute_batch(include_str!("../../migrations/0003_fts.sql"))?;
        version = 3;
    }

    if version < 4 {
        info!("Applying migration 0004_artwork_cache");
        conn.execute_batch(include_str!("../../migrations/0004_artwork_cache.sql"))?;
        version = 4;
    }

    if version < 5 {
        info!("Applying migration 0005_preferences");
        conn.execute_batch(include_str!("../../migrations/0005_preferences.sql"))?;
        version = 5;
    }

    Ok(())
}
