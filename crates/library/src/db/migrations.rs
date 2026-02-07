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

    if version < 6 {
        info!("Applying migration 0006_dsd_metadata");
        conn.execute_batch(include_str!("../../migrations/0006_dsd_metadata.sql"))?;
        version = 6;
    }

    if version < 7 {
        info!("Applying migration 0007_browse_indexes");
        conn.execute_batch(include_str!("../../migrations/0007_browse_indexes.sql"))?;
        version = 7;
    }

    if version < 8 {
        info!("Applying migration 0008_waveform_cache");
        conn.execute_batch(include_str!("../../migrations/0008_waveform_cache.sql"))?;
        version = 8;
    }

    if version < 9 {
        info!("Applying migration 0009_lyrics_cache");
        conn.execute_batch(include_str!("../../migrations/0009_lyrics_cache.sql"))?;
        version = 9;
    }

    let _ = version;

    Ok(())
}
