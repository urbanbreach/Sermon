pub mod migrations;

use crate::error::LibraryError;
use crate::models::{LibraryFolder, TrackRow};
use rusqlite::{Connection, OptionalExtension, Row, params};
use std::path::Path;

pub fn open_db(path: &Path) -> Result<Connection, LibraryError> {
    let conn = Connection::open(path)?;

    // Set some safe defaults
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;

    Ok(conn)
}

fn map_track(row: &Row) -> rusqlite::Result<TrackRow> {
    Ok(TrackRow {
        id: Some(row.get("id")?),
        library_folder_id: row.get("library_folder_id")?,
        path: row.get("path")?,
        path_display: row.get("path_display")?,
        path_lossy: row.get("path_lossy")?,
        identity_source: row.get("identity_source")?,
        volume_serial: row.get("volume_serial")?,
        file_id: row.get("file_id")?,
        mtime_ms: row.get("mtime_ms")?,
        size_bytes: row.get("size_bytes")?,
        hash: row.get("hash")?,
        title: row.get("title")?,
        artist: row.get("artist")?,
        album: row.get("album")?,
        album_artist: row.get("album_artist")?,
        track_no: row.get("track_no")?,
        disc_no: row.get("disc_no")?,
        year: row.get("year")?,
        genre: row.get("genre")?,
        codec: row.get("codec")?,
        container: row.get("container")?,
        sample_rate: row.get("sample_rate")?,
        bit_depth: row.get("bit_depth")?,
        channels: row.get("channels")?,
        duration_ms: row.get("duration_ms")?,
        is_missing: row.get("is_missing")?,
        missing_since_ms: row.get("missing_since_ms")?,
    })
}

pub fn list_tracks(
    conn: &Connection,
    sort_by: &str,
    direction: &str,
) -> Result<Vec<TrackRow>, LibraryError> {
    // Validate sort params to prevent injection
    let allowed_sorts = ["title", "artist", "album", "path", "id"];
    let safe_sort = if allowed_sorts.contains(&sort_by) {
        sort_by
    } else {
        "id"
    };
    let safe_dir = if direction.to_uppercase() == "DESC" {
        "DESC"
    } else {
        "ASC"
    };

    let query = format!("SELECT * FROM tracks ORDER BY {} {}", safe_sort, safe_dir);

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map([], map_track)?;

    let mut tracks = Vec::new();
    for row in rows {
        tracks.push(row?);
    }

    Ok(tracks)
}

pub fn upsert_track(conn: &Connection, track: &TrackRow) -> Result<i64, LibraryError> {
    // 1. Try to find existing ID based on identity
    let existing_id: Option<i64> = if track.identity_source == "ntfs" {
        conn.query_row(
            "SELECT id FROM tracks WHERE identity_source = 'ntfs' AND volume_serial = ? AND file_id = ?",
            params![track.volume_serial, track.file_id],
            |row| row.get(0)
        ).optional()?
    } else {
        conn.query_row(
            "SELECT id FROM tracks WHERE identity_source = 'fallback' AND path = ? AND mtime_ms = ? AND size_bytes = ?",
            params![track.path, track.mtime_ms, track.size_bytes],
            |row| row.get(0)
        ).optional()?
    };

    if let Some(id) = existing_id {
        // UPDATE
        conn.execute(
            "UPDATE tracks SET 
                library_folder_id = ?, path = ?, path_display = ?, path_lossy = ?,
                hash = ?, title = ?, artist = ?, album = ?, album_artist = ?,
                track_no = ?, disc_no = ?, year = ?, genre = ?,
                codec = ?, container = ?, sample_rate = ?, bit_depth = ?, channels = ?, duration_ms = ?,
                is_missing = ?, missing_since_ms = ?
             WHERE id = ?",
            params![
                track.library_folder_id, track.path, track.path_display, track.path_lossy,
                track.hash, track.title, track.artist, track.album, track.album_artist,
                track.track_no, track.disc_no, track.year, track.genre,
                track.codec, track.container, track.sample_rate, track.bit_depth, track.channels, track.duration_ms,
                track.is_missing, track.missing_since_ms,
                id
            ]
        )?;
        Ok(id)
    } else {
        // INSERT
        conn.execute(
            "INSERT INTO tracks (
                library_folder_id, path, path_display, path_lossy, identity_source,
                volume_serial, file_id, mtime_ms, size_bytes, hash,
                title, artist, album, album_artist, track_no, disc_no, year, genre,
                codec, container, sample_rate, bit_depth, channels, duration_ms,
                is_missing, missing_since_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                track.library_folder_id, track.path, track.path_display, track.path_lossy, track.identity_source,
                track.volume_serial, track.file_id, track.mtime_ms, track.size_bytes, track.hash,
                track.title, track.artist, track.album, track.album_artist, track.track_no, track.disc_no, track.year, track.genre,
                track.codec, track.container, track.sample_rate, track.bit_depth, track.channels, track.duration_ms,
                track.is_missing, track.missing_since_ms
            ]
        )?;
        Ok(conn.last_insert_rowid())
    }
}

pub fn set_missing(conn: &Connection, track_id: i64, missing: bool) -> Result<(), LibraryError> {
    let missing_since = if missing {
        Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        )
    } else {
        None
    };

    conn.execute(
        "UPDATE tracks SET is_missing = ?, missing_since_ms = ? WHERE id = ?",
        params![missing, missing_since, track_id],
    )?;
    Ok(())
}

pub fn add_folder(conn: &Connection, path: &str) -> Result<LibraryFolder, LibraryError> {
    // Upsert the folder
    conn.execute(
        "INSERT INTO library_folders (path) VALUES (?) ON CONFLICT(path) DO UPDATE SET enabled=1",
        params![path],
    )?;

    // Fetch it back
    get_folder_by_path(conn, path)?
        .ok_or_else(|| LibraryError::NotFound(format!("Folder {}", path)))
}

pub fn list_folders(conn: &Connection) -> Result<Vec<LibraryFolder>, LibraryError> {
    let mut stmt = conn.prepare("SELECT * FROM library_folders")?;
    let rows = stmt.query_map([], |row| {
        Ok(LibraryFolder {
            id: row.get("id")?,
            path: row.get("path")?,
            enabled: row.get("enabled")?,
            status: row.get("status")?,
            last_error: row.get("last_error")?,
            options_json: row.get("options_json")?,
        })
    })?;

    let mut folders = Vec::new();
    for row in rows {
        folders.push(row?);
    }
    Ok(folders)
}

pub fn get_folder_by_path(
    conn: &Connection,
    path: &str,
) -> Result<Option<LibraryFolder>, LibraryError> {
    conn.query_row(
        "SELECT * FROM library_folders WHERE path = ?",
        params![path],
        |row| {
            Ok(LibraryFolder {
                id: row.get("id")?,
                path: row.get("path")?,
                enabled: row.get("enabled")?,
                status: row.get("status")?,
                last_error: row.get("last_error")?,
                options_json: row.get("options_json")?,
            })
        },
    )
    .optional()
    .map_err(LibraryError::from)
}
