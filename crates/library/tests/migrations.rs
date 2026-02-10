use library::db::{migrations::apply_migrations, open_db};
use tempfile::tempdir;

#[test]
fn test_migration_idempotency() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    // First run
    let conn = open_db(&db_path).unwrap();
    apply_migrations(&conn).unwrap();

    // Second run should not error
    apply_migrations(&conn).unwrap();

    // Verify tables exist
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tracks'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);

    let count_settings: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='settings'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count_settings, 1);
}

#[test]
fn test_migration_second_run_preserves_existing_rows() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_preserve_rows.db");
    let conn = open_db(&db_path).unwrap();

    apply_migrations(&conn).unwrap();

    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?, ?)",
        ["audio.volume", "0.75"],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO library_folders (path) VALUES (?)",
        ["C:\\Music"],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO tracks (library_folder_id, path, identity_source, mtime_ms, size_bytes, title)
         VALUES (1, 'before_second_run.flac', 'fallback', 0, 1000, 'Before')",
        [],
    )
    .unwrap();

    let settings_before: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM settings WHERE key = 'audio.volume'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let tracks_before: i64 = conn
        .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))
        .unwrap();

    assert_eq!(settings_before, 1);
    assert_eq!(tracks_before, 1);

    // Second run should be a no-op for schema and preserve data.
    apply_migrations(&conn).unwrap();

    let version: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 9);

    let settings_after: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM settings WHERE key = 'audio.volume'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let tracks_after: i64 = conn
        .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))
        .unwrap();

    assert_eq!(settings_after, 1);
    assert_eq!(tracks_after, 1);
}

#[test]
fn test_in_memory_migration() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    apply_migrations(&conn).unwrap();

    // Check user_version
    let version: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 9);
}

#[test]
fn test_artwork_cache_tables_exist() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    apply_migrations(&conn).unwrap();

    // Verify artwork_cache_map_album table exists
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='artwork_cache_map_album'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);

    // Verify artwork_cache_map_track table exists
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='artwork_cache_map_track'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_lyrics_cache_table_exists() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    apply_migrations(&conn).unwrap();

    // Verify lyrics_cache table exists
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='lyrics_cache'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_fts_migration_creates_virtual_table() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    apply_migrations(&conn).unwrap();

    // Verify tracks_fts virtual table exists
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tracks_fts'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_fts_search_after_insert() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    apply_migrations(&conn).unwrap();

    // Insert a test track
    conn.execute(
        "INSERT INTO library_folders (path) VALUES (?)",
        ["C:\\Music"],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO tracks (library_folder_id, path, identity_source, mtime_ms, size_bytes, title, artist, album)
         VALUES (1, 'test.flac', 'fallback', 0, 1000, 'Test Song', 'Test Artist', 'Test Album')",
        [],
    ).unwrap();

    // Search should find the track
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tracks_fts WHERE tracks_fts MATCH 'Test'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_fts_rebuild_existing_tracks() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();

    // Apply only migrations 1 and 2 (simulating existing v2 database)
    conn.execute_batch(include_str!("../migrations/0001_init.sql"))
        .unwrap();
    conn.execute_batch(include_str!("../migrations/0002_settings.sql"))
        .unwrap();

    // Insert a track BEFORE FTS migration
    conn.execute(
        "INSERT INTO library_folders (path) VALUES (?)",
        ["C:\\Music"],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO tracks (library_folder_id, path, identity_source, mtime_ms, size_bytes, title, artist, album)
         VALUES (1, 'existing.flac', 'fallback', 0, 1000, 'Existing Song', 'Existing Artist', 'Existing Album')",
        [],
    ).unwrap();

    // Now apply migration 3
    conn.execute_batch(include_str!("../migrations/0003_fts.sql"))
        .unwrap();

    // Search should find the pre-existing track (proves rebuild worked)
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tracks_fts WHERE tracks_fts MATCH 'Existing'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}
