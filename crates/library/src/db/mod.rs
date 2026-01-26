pub mod migrations;

use crate::error::LibraryError;
use crate::models::{
    AlbumCursor, AlbumListItem, ArtistCursor, ArtistListItem, LibraryFolder, LibraryStats,
    OffsetCursor, Page, SearchHit, SearchSuggestResponse, TrackRow,
};
use rusqlite::{params, Connection, OptionalExtension, Row};
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

/// Check if FTS5 module is available in the SQLite build
pub fn check_fts5_available(conn: &Connection) -> Result<bool, LibraryError> {
    let result: Result<i32, _> = conn.query_row(
        "SELECT 1 FROM pragma_module_list WHERE name = 'fts5'",
        [],
        |row| row.get(0),
    );
    Ok(result.is_ok())
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
        dsd_rate_hz: row.get("dsd_rate_hz")?,
        dsd_channels: row.get("dsd_channels")?,
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
                dsd_rate_hz = ?, dsd_channels = ?,
                is_missing = ?, missing_since_ms = ?
             WHERE id = ?",
            params![
                track.library_folder_id, track.path, track.path_display, track.path_lossy,
                track.hash, track.title, track.artist, track.album, track.album_artist,
                track.track_no, track.disc_no, track.year, track.genre,
                track.codec, track.container, track.sample_rate, track.bit_depth, track.channels, track.duration_ms,
                track.dsd_rate_hz, track.dsd_channels,
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
                dsd_rate_hz, dsd_channels,
                is_missing, missing_since_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                track.library_folder_id, track.path, track.path_display, track.path_lossy, track.identity_source,
                track.volume_serial, track.file_id, track.mtime_ms, track.size_bytes, track.hash,
                track.title, track.artist, track.album, track.album_artist, track.track_no, track.disc_no, track.year, track.genre,
                track.codec, track.container, track.sample_rate, track.bit_depth, track.channels, track.duration_ms,
                track.dsd_rate_hz, track.dsd_channels,
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

// Settings helpers

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, LibraryError> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?",
        params![key],
        |row| row.get(0),
    )
    .optional()
    .map_err(LibraryError::from)
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), LibraryError> {
    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, strftime('%s', 'now')) 
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_audio_volume(conn: &Connection) -> f32 {
    match get_setting(conn, "audio.volume") {
        Ok(Some(v)) => v.parse().unwrap_or(1.0),
        _ => 1.0,
    }
}

pub fn get_audio_device_preference(conn: &Connection) -> String {
    match get_setting(conn, "audio.device.preference") {
        Ok(Some(v)) => v,
        _ => "default".to_string(),
    }
}

pub fn get_audio_output_mode(conn: &Connection) -> String {
    match get_setting(conn, "audio.output.mode") {
        Ok(Some(v)) => v,
        _ => "exclusive".to_string(),
    }
}

pub fn get_audio_output_policy(conn: &Connection) -> String {
    match get_setting(conn, "audio.output.policy") {
        Ok(Some(v)) => v,
        _ => "strict".to_string(),
    }
}

pub fn get_audio_output_fade(conn: &Connection) -> bool {
    match get_setting(conn, "audio.output.fade") {
        Ok(Some(v)) => v == "on",
        _ => false,
    }
}

pub fn get_audio_output_timing(conn: &Connection) -> String {
    match get_setting(conn, "audio.output.timing") {
        Ok(Some(v)) => v,
        _ => "polling".to_string(),
    }
}

pub fn get_audio_output_asio_driver(conn: &Connection) -> Option<String> {
    match get_setting(conn, "audio.output.asio_driver") {
        Ok(Some(v)) if !v.is_empty() => Some(v),
        _ => None,
    }
}

pub fn get_track_by_id(conn: &Connection, id: i64) -> Result<TrackRow, LibraryError> {
    conn.query_row("SELECT * FROM tracks WHERE id = ?", params![id], map_track)
        .map_err(LibraryError::from)
}

// ============================================================================
// Folder Management (Library Preferences Redesign)
// ============================================================================

/// Get a folder by its ID
pub fn get_folder_by_id(
    conn: &Connection,
    folder_id: i64,
) -> Result<Option<LibraryFolder>, LibraryError> {
    conn.query_row(
        "SELECT * FROM library_folders WHERE id = ?",
        params![folder_id],
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

/// Remove a library folder and all its associated tracks
/// Also cleans up the scan_state entry for this folder
pub fn remove_folder(conn: &Connection, folder_id: i64) -> Result<(), LibraryError> {
    // Delete all tracks in this folder
    conn.execute(
        "DELETE FROM tracks WHERE library_folder_id = ?",
        params![folder_id],
    )?;

    // Delete scan_state entry (if exists)
    conn.execute(
        "DELETE FROM scan_state WHERE folder_id = ?",
        params![folder_id],
    )?;

    // Delete the folder itself
    let deleted = conn.execute(
        "DELETE FROM library_folders WHERE id = ?",
        params![folder_id],
    )?;

    if deleted == 0 {
        return Err(LibraryError::NotFound(format!("Folder ID {}", folder_id)));
    }

    Ok(())
}

/// Toggle the enabled status of a folder
/// Returns the updated folder
pub fn update_folder_enabled(
    conn: &Connection,
    folder_id: i64,
    enabled: bool,
) -> Result<LibraryFolder, LibraryError> {
    let updated = conn.execute(
        "UPDATE library_folders SET enabled = ? WHERE id = ?",
        params![enabled, folder_id],
    )?;

    if updated == 0 {
        return Err(LibraryError::NotFound(format!("Folder ID {}", folder_id)));
    }

    get_folder_by_id(conn, folder_id)?
        .ok_or_else(|| LibraryError::NotFound(format!("Folder ID {}", folder_id)))
}

/// Update folder options (exclude patterns, extensions, etc.)
/// Takes the FolderOptions struct, serializes to JSON, and updates options_json column
pub fn update_folder_options(
    conn: &Connection,
    folder_id: i64,
    options: &crate::models::FolderOptions,
) -> Result<LibraryFolder, LibraryError> {
    let options_json = serde_json::to_string(options)
        .map_err(|e| LibraryError::Other(format!("Failed to serialize options: {}", e)))?;

    let updated = conn.execute(
        "UPDATE library_folders SET options_json = ? WHERE id = ?",
        params![options_json, folder_id],
    )?;

    if updated == 0 {
        return Err(LibraryError::NotFound(format!("Folder ID {}", folder_id)));
    }

    get_folder_by_id(conn, folder_id)?
        .ok_or_else(|| LibraryError::NotFound(format!("Folder ID {}", folder_id)))
}

/// Get count of tracks in a folder
pub fn get_folder_track_count(conn: &Connection, folder_id: i64) -> Result<i64, LibraryError> {
    conn.query_row(
        "SELECT COUNT(*) FROM tracks WHERE library_folder_id = ? AND is_missing = 0",
        params![folder_id],
        |row| row.get(0),
    )
    .map_err(LibraryError::from)
}

// ============================================================================
// Browse Queries (Milestone 04) - Derived from tracks table
// ============================================================================

fn map_album_list_item(row: &Row) -> rusqlite::Result<AlbumListItem> {
    Ok(AlbumListItem {
        album_title_display: row.get("album_title_display")?,
        album_artist_display: row.get("album_artist_display")?,
        album_title_sort: row.get("album_title_sort")?,
        album_artist_sort: row.get("album_artist_sort")?,
        year: row.get("year")?,
        track_count: row.get("track_count")?,
    })
}

fn map_artist_list_item(row: &Row) -> rusqlite::Result<ArtistListItem> {
    Ok(ArtistListItem {
        artist_display: row.get("artist_display")?,
        artist_sort: row.get("artist_sort")?,
        track_count: row.get("track_count")?,
        album_count: row.get("album_count")?,
    })
}

/// List albums with cursor-based pagination
/// Albums are derived from tracks using (album_artist_sort, album_title_sort) as the key
pub fn list_albums_page(
    conn: &Connection,
    limit: i64,
    cursor: Option<&AlbumCursor>,
) -> Result<Page<AlbumListItem, AlbumCursor>, LibraryError> {
    let mut items: Vec<AlbumListItem> = if let Some(c) = cursor {
        let query = r#"
        SELECT 
            COALESCE(NULLIF(TRIM(album), ''), 'Unknown Album') as album_title_display,
            COALESCE(NULLIF(TRIM(album_artist), ''), NULLIF(TRIM(artist), ''), 'Unknown Artist') as album_artist_display,
            LOWER(COALESCE(NULLIF(TRIM(album), ''), 'Unknown Album')) as album_title_sort,
            LOWER(COALESCE(NULLIF(TRIM(album_artist), ''), NULLIF(TRIM(artist), ''), 'Unknown Artist')) as album_artist_sort,
            MIN(year) as year,
            COUNT(*) as track_count
        FROM tracks
        WHERE is_missing = 0
        GROUP BY album_artist_sort, album_title_sort
        HAVING (album_artist_sort, album_title_sort) > (?, ?)
        ORDER BY album_artist_sort ASC, album_title_sort ASC
        LIMIT ?
        "#;
        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map(
            params![&c.album_artist_sort, &c.album_title_sort, limit + 1],
            map_album_list_item,
        )?;
        rows.collect::<Result<Vec<_>, _>>()?
    } else {
        let query = r#"
        SELECT 
            COALESCE(NULLIF(TRIM(album), ''), 'Unknown Album') as album_title_display,
            COALESCE(NULLIF(TRIM(album_artist), ''), NULLIF(TRIM(artist), ''), 'Unknown Artist') as album_artist_display,
            LOWER(COALESCE(NULLIF(TRIM(album), ''), 'Unknown Album')) as album_title_sort,
            LOWER(COALESCE(NULLIF(TRIM(album_artist), ''), NULLIF(TRIM(artist), ''), 'Unknown Artist')) as album_artist_sort,
            MIN(year) as year,
            COUNT(*) as track_count
        FROM tracks
        WHERE is_missing = 0
        GROUP BY album_artist_sort, album_title_sort
        ORDER BY album_artist_sort ASC, album_title_sort ASC
        LIMIT ?
        "#;
        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map(params![limit + 1], map_album_list_item)?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let next_cursor = if items.len() > limit as usize {
        let last = items.pop().unwrap();
        Some(AlbumCursor {
            album_artist_sort: last.album_artist_sort,
            album_title_sort: last.album_title_sort,
        })
    } else {
        None
    };

    Ok(Page { items, next_cursor })
}

/// List artists with cursor-based pagination
/// Artists are derived from tracks using artist_sort as the key
pub fn list_artists_page(
    conn: &Connection,
    limit: i64,
    cursor: Option<&ArtistCursor>,
) -> Result<Page<ArtistListItem, ArtistCursor>, LibraryError> {
    let mut items: Vec<ArtistListItem> = if let Some(c) = cursor {
        let query = r#"
        SELECT 
            COALESCE(NULLIF(TRIM(artist), ''), 'Unknown Artist') as artist_display,
            LOWER(COALESCE(NULLIF(TRIM(artist), ''), 'Unknown Artist')) as artist_sort,
            COUNT(*) as track_count,
            COUNT(DISTINCT 
                LOWER(COALESCE(NULLIF(TRIM(album_artist), ''), NULLIF(TRIM(artist), ''), 'Unknown Artist')) || '||' ||
                LOWER(COALESCE(NULLIF(TRIM(album), ''), 'Unknown Album'))
            ) as album_count
        FROM tracks
        WHERE is_missing = 0
        GROUP BY artist_sort
        HAVING artist_sort > ?
        ORDER BY artist_sort ASC
        LIMIT ?
        "#;
        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map(params![&c.artist_sort, limit + 1], map_artist_list_item)?;
        rows.collect::<Result<Vec<_>, _>>()?
    } else {
        let query = r#"
        SELECT 
            COALESCE(NULLIF(TRIM(artist), ''), 'Unknown Artist') as artist_display,
            LOWER(COALESCE(NULLIF(TRIM(artist), ''), 'Unknown Artist')) as artist_sort,
            COUNT(*) as track_count,
            COUNT(DISTINCT 
                LOWER(COALESCE(NULLIF(TRIM(album_artist), ''), NULLIF(TRIM(artist), ''), 'Unknown Artist')) || '||' ||
                LOWER(COALESCE(NULLIF(TRIM(album), ''), 'Unknown Album'))
            ) as album_count
        FROM tracks
        WHERE is_missing = 0
        GROUP BY artist_sort
        ORDER BY artist_sort ASC
        LIMIT ?
        "#;
        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map(params![limit + 1], map_artist_list_item)?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let next_cursor = if items.len() > limit as usize {
        let last = items.pop().unwrap();
        Some(ArtistCursor {
            artist_sort: last.artist_sort,
        })
    } else {
        None
    };

    Ok(Page { items, next_cursor })
}

/// List tracks for an album with cursor-based pagination
pub fn list_album_tracks_page(
    conn: &Connection,
    album_artist_sort: &str,
    album_title_sort: &str,
    limit: i64,
    cursor: Option<(i32, i32, &str, i64)>, // (disc_no, track_no, title_sort, id)
) -> Result<Page<TrackRow, (i32, i32, String, i64)>, LibraryError> {
    let query = if cursor.is_some() {
        r#"
        SELECT *,
            COALESCE(disc_no, 0) as disc_no_sort,
            COALESCE(track_no, 0) as track_no_sort,
            LOWER(COALESCE(NULLIF(TRIM(title), ''), '')) as title_sort
        FROM tracks
        WHERE is_missing = 0
          AND LOWER(COALESCE(NULLIF(TRIM(album_artist), ''), NULLIF(TRIM(artist), ''), 'Unknown Artist')) = ?
          AND LOWER(COALESCE(NULLIF(TRIM(album), ''), 'Unknown Album')) = ?
          AND (disc_no_sort, track_no_sort, title_sort, id) > (?, ?, ?, ?)
        ORDER BY disc_no_sort ASC, track_no_sort ASC, title_sort ASC, id ASC
        LIMIT ?
        "#
    } else {
        r#"
        SELECT *,
            COALESCE(disc_no, 0) as disc_no_sort,
            COALESCE(track_no, 0) as track_no_sort,
            LOWER(COALESCE(NULLIF(TRIM(title), ''), '')) as title_sort
        FROM tracks
        WHERE is_missing = 0
          AND LOWER(COALESCE(NULLIF(TRIM(album_artist), ''), NULLIF(TRIM(artist), ''), 'Unknown Artist')) = ?
          AND LOWER(COALESCE(NULLIF(TRIM(album), ''), 'Unknown Album')) = ?
        ORDER BY disc_no_sort ASC, track_no_sort ASC, title_sort ASC, id ASC
        LIMIT ?
        "#
    };

    let mut stmt = conn.prepare(query)?;

    let rows = if let Some((disc, track, title, id)) = cursor {
        stmt.query_map(
            params![
                album_artist_sort,
                album_title_sort,
                disc,
                track,
                title,
                id,
                limit + 1
            ],
            map_track,
        )?
    } else {
        stmt.query_map(
            params![album_artist_sort, album_title_sort, limit + 1],
            map_track,
        )?
    };

    let mut items: Vec<TrackRow> = rows.collect::<Result<Vec<_>, _>>()?;

    let next_cursor = if items.len() > limit as usize {
        let last = items.pop().unwrap();
        let title_sort = last.title.as_deref().unwrap_or("").to_lowercase();
        Some((
            last.disc_no.unwrap_or(0),
            last.track_no.unwrap_or(0),
            title_sort,
            last.id.unwrap_or(0),
        ))
    } else {
        None
    };

    Ok(Page { items, next_cursor })
}

/// List tracks for an artist with cursor-based pagination
pub fn list_artist_tracks_page(
    conn: &Connection,
    artist_sort: &str,
    limit: i64,
    cursor: Option<(&str, i32, i32, &str, i64)>, // (album_title_sort, disc_no, track_no, title_sort, id)
) -> Result<Page<TrackRow, (String, i32, i32, String, i64)>, LibraryError> {
    let query = if cursor.is_some() {
        r#"
        SELECT *,
            LOWER(COALESCE(NULLIF(TRIM(album), ''), 'Unknown Album')) as album_title_sort_col,
            COALESCE(disc_no, 0) as disc_no_sort,
            COALESCE(track_no, 0) as track_no_sort,
            LOWER(COALESCE(NULLIF(TRIM(title), ''), '')) as title_sort
        FROM tracks
        WHERE is_missing = 0
          AND LOWER(COALESCE(NULLIF(TRIM(artist), ''), 'Unknown Artist')) = ?
          AND (album_title_sort_col, disc_no_sort, track_no_sort, title_sort, id) > (?, ?, ?, ?, ?)
        ORDER BY album_title_sort_col ASC, disc_no_sort ASC, track_no_sort ASC, title_sort ASC, id ASC
        LIMIT ?
        "#
    } else {
        r#"
        SELECT *,
            LOWER(COALESCE(NULLIF(TRIM(album), ''), 'Unknown Album')) as album_title_sort_col,
            COALESCE(disc_no, 0) as disc_no_sort,
            COALESCE(track_no, 0) as track_no_sort,
            LOWER(COALESCE(NULLIF(TRIM(title), ''), '')) as title_sort
        FROM tracks
        WHERE is_missing = 0
          AND LOWER(COALESCE(NULLIF(TRIM(artist), ''), 'Unknown Artist')) = ?
        ORDER BY album_title_sort_col ASC, disc_no_sort ASC, track_no_sort ASC, title_sort ASC, id ASC
        LIMIT ?
        "#
    };

    let mut stmt = conn.prepare(query)?;

    let rows = if let Some((album, disc, track, title, id)) = cursor {
        stmt.query_map(
            params![artist_sort, album, disc, track, title, id, limit + 1],
            map_track,
        )?
    } else {
        stmt.query_map(params![artist_sort, limit + 1], map_track)?
    };

    let mut items: Vec<TrackRow> = rows.collect::<Result<Vec<_>, _>>()?;

    let next_cursor = if items.len() > limit as usize {
        let last = items.pop().unwrap();
        let album_sort = last
            .album
            .as_deref()
            .unwrap_or("unknown album")
            .to_lowercase();
        let title_sort = last.title.as_deref().unwrap_or("").to_lowercase();
        Some((
            album_sort,
            last.disc_no.unwrap_or(0),
            last.track_no.unwrap_or(0),
            title_sort,
            last.id.unwrap_or(0),
        ))
    } else {
        None
    };

    Ok(Page { items, next_cursor })
}

/// Paginated tracks list with offset-based pagination
pub fn list_tracks_page(
    conn: &Connection,
    sort_by: &str,
    direction: &str,
    limit: i64,
    offset: i64,
) -> Result<Page<TrackRow, OffsetCursor>, LibraryError> {
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

    let query = format!(
        "SELECT * FROM tracks WHERE is_missing = 0 ORDER BY {} {}, id {} LIMIT ? OFFSET ?",
        safe_sort, safe_dir, safe_dir
    );

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map(params![limit + 1, offset], map_track)?;

    let mut items: Vec<TrackRow> = rows.collect::<Result<Vec<_>, _>>()?;

    let next_cursor = if items.len() > limit as usize {
        items.pop();
        Some(OffsetCursor {
            offset: offset + limit,
        })
    } else {
        None
    };

    Ok(Page { items, next_cursor })
}

/// Get library statistics
pub fn get_library_stats(conn: &Connection, db_path: &Path) -> Result<LibraryStats, LibraryError> {
    let track_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tracks WHERE is_missing = 0",
        [],
        |row| row.get(0),
    )?;

    let album_count: i64 = conn.query_row(
        r#"
        SELECT COUNT(DISTINCT 
            LOWER(COALESCE(NULLIF(TRIM(album_artist), ''), NULLIF(TRIM(artist), ''), 'Unknown Artist')) || '||' ||
            LOWER(COALESCE(NULLIF(TRIM(album), ''), 'Unknown Album'))
        )
        FROM tracks WHERE is_missing = 0
        "#,
        [],
        |row| row.get(0),
    )?;

    let artist_count: i64 = conn.query_row(
        r#"
        SELECT COUNT(DISTINCT LOWER(COALESCE(NULLIF(TRIM(artist), ''), 'Unknown Artist')))
        FROM tracks WHERE is_missing = 0
        "#,
        [],
        |row| row.get(0),
    )?;

    let last_scan_completed_ms: Option<i64> = conn
        .query_row(
            "SELECT MAX(last_scan_completed_ms) FROM scan_state",
            [],
            |row| row.get(0),
        )
        .optional()?
        .flatten();

    let db_size_bytes = std::fs::metadata(db_path)
        .map(|m| m.len() as i64)
        .unwrap_or(0);

    Ok(LibraryStats {
        track_count,
        album_count,
        artist_count,
        db_size_bytes,
        last_scan_completed_ms,
    })
}

// ============================================================================
// Search Queries (Milestone 04) - FTS5 backed
// ============================================================================

/// Normalize a search query for safe FTS5 MATCH usage
/// Returns None if the query is empty or invalid
pub fn normalize_search_query(input: &str) -> Option<String> {
    let q = input.trim();
    if q.is_empty() {
        return None;
    }

    let tokens: Vec<String> = q
        .split_whitespace()
        .filter_map(|token| {
            // Escape double quotes
            let mut t = token.replace('"', "\"\"");
            // Strip FTS operator characters from start/end
            t = t.trim_matches(|c| "():*^".contains(c)).to_string();
            if t.is_empty() {
                return None;
            }
            // Use prefix query for tokens >= 2 chars
            if t.len() >= 2 {
                Some(format!("\"{}\"*", t))
            } else {
                Some(format!("\"{}\"", t))
            }
        })
        .collect();

    if tokens.is_empty() {
        return None;
    }

    Some(tokens.join(" AND "))
}

/// Search for tracks, albums, and artists (dropdown suggestions)
/// Returns up to `limit` results ranked by relevance
pub fn search_suggest(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> Result<SearchSuggestResponse, LibraryError> {
    let match_query = match normalize_search_query(query) {
        Some(q) => q,
        None => return Ok(SearchSuggestResponse { results: vec![] }),
    };

    let mut results: Vec<SearchHit> = Vec::new();

    // 1. Search tracks (ranked by bm25)
    let track_query = r#"
        SELECT t.id, t.title, t.artist, t.album, bm25(tracks_fts) as rank
        FROM tracks_fts
        JOIN tracks t ON tracks_fts.rowid = t.id
        WHERE tracks_fts MATCH ? AND t.is_missing = 0
        ORDER BY rank
        LIMIT ?
    "#;

    let track_limit = (limit / 2).max(4); // Reserve some slots for albums/artists

    match conn.prepare(track_query) {
        Ok(mut stmt) => {
            if let Ok(rows) = stmt.query_map(params![&match_query, track_limit], |row| {
                Ok(SearchHit::Track {
                    track_id: row.get("id")?,
                    title: row.get("title")?,
                    artist: row.get("artist")?,
                    album: row.get("album")?,
                })
            }) {
                for row in rows.flatten() {
                    results.push(row);
                }
            }
        }
        Err(_) => {} // FTS query failed, continue with empty results
    }

    // 2. Search albums (aggregated from tracks)
    let album_query = r#"
        SELECT 
            COALESCE(NULLIF(TRIM(t.album), ''), 'Unknown Album') as album_title_display,
            COALESCE(NULLIF(TRIM(t.album_artist), ''), NULLIF(TRIM(t.artist), ''), 'Unknown Artist') as album_artist_display,
            LOWER(COALESCE(NULLIF(TRIM(t.album), ''), 'Unknown Album')) as album_title_sort,
            LOWER(COALESCE(NULLIF(TRIM(t.album_artist), ''), NULLIF(TRIM(t.artist), ''), 'Unknown Artist')) as album_artist_sort,
            MIN(t.year) as year,
            COUNT(*) as track_count
        FROM tracks_fts
        JOIN tracks t ON tracks_fts.rowid = t.id
        WHERE tracks_fts MATCH ? AND t.is_missing = 0
        GROUP BY album_artist_sort, album_title_sort
        ORDER BY track_count DESC, album_artist_sort ASC, album_title_sort ASC
        LIMIT ?
    "#;

    let album_limit = (limit / 4).max(2);

    match conn.prepare(album_query) {
        Ok(mut stmt) => {
            if let Ok(rows) = stmt.query_map(params![&match_query, album_limit], |row| {
                Ok(SearchHit::Album {
                    album_artist_sort: row.get("album_artist_sort")?,
                    album_title_sort: row.get("album_title_sort")?,
                    album_artist_display: row.get("album_artist_display")?,
                    album_title_display: row.get("album_title_display")?,
                    year: row.get("year")?,
                })
            }) {
                for row in rows.flatten() {
                    results.push(row);
                }
            }
        }
        Err(_) => {}
    }

    // 3. Search artists (aggregated from tracks)
    let artist_query = r#"
        SELECT 
            COALESCE(NULLIF(TRIM(t.artist), ''), 'Unknown Artist') as artist_display,
            LOWER(COALESCE(NULLIF(TRIM(t.artist), ''), 'Unknown Artist')) as artist_sort,
            COUNT(*) as track_count
        FROM tracks_fts
        JOIN tracks t ON tracks_fts.rowid = t.id
        WHERE tracks_fts MATCH ? AND t.is_missing = 0
        GROUP BY artist_sort
        ORDER BY track_count DESC, artist_sort ASC
        LIMIT ?
    "#;

    let artist_limit = (limit / 4).max(2);

    match conn.prepare(artist_query) {
        Ok(mut stmt) => {
            if let Ok(rows) = stmt.query_map(params![&match_query, artist_limit], |row| {
                Ok(SearchHit::Artist {
                    artist_sort: row.get("artist_sort")?,
                    artist_display: row.get("artist_display")?,
                })
            }) {
                for row in rows.flatten() {
                    results.push(row);
                }
            }
        }
        Err(_) => {}
    }

    // Truncate to limit
    results.truncate(limit as usize);

    Ok(SearchSuggestResponse { results })
}

/// Search tracks with pagination (for Search Results view)
pub fn search_tracks_page(
    conn: &Connection,
    query: &str,
    sort_by: &str,
    direction: &str,
    limit: i64,
    offset: i64,
) -> Result<Page<TrackRow, OffsetCursor>, LibraryError> {
    let match_query = match normalize_search_query(query) {
        Some(q) => q,
        None => {
            return Ok(Page {
                items: vec![],
                next_cursor: None,
            });
        }
    };

    let allowed_sorts = ["title", "artist", "album", "id"];
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

    let sql = format!(
        r#"
        SELECT t.*
        FROM tracks_fts
        JOIN tracks t ON tracks_fts.rowid = t.id
        WHERE tracks_fts MATCH ? AND t.is_missing = 0
        ORDER BY t.{} {}, t.id {}
        LIMIT ? OFFSET ?
        "#,
        safe_sort, safe_dir, safe_dir
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![&match_query, limit + 1, offset], map_track)?;
    let mut items: Vec<TrackRow> = rows.collect::<Result<Vec<_>, _>>()?;

    let next_cursor = if items.len() > limit as usize {
        items.pop();
        Some(OffsetCursor {
            offset: offset + limit,
        })
    } else {
        None
    };

    Ok(Page { items, next_cursor })
}

/// Search albums with pagination (for Search Results view)
pub fn search_albums_page(
    conn: &Connection,
    query: &str,
    limit: i64,
    cursor: Option<&AlbumCursor>,
) -> Result<Page<AlbumListItem, AlbumCursor>, LibraryError> {
    let match_query = match normalize_search_query(query) {
        Some(q) => q,
        None => {
            return Ok(Page {
                items: vec![],
                next_cursor: None,
            });
        }
    };

    let mut items: Vec<AlbumListItem> = if let Some(c) = cursor {
        let sql = r#"
            SELECT 
                COALESCE(NULLIF(TRIM(t.album), ''), 'Unknown Album') as album_title_display,
                COALESCE(NULLIF(TRIM(t.album_artist), ''), NULLIF(TRIM(t.artist), ''), 'Unknown Artist') as album_artist_display,
                LOWER(COALESCE(NULLIF(TRIM(t.album), ''), 'Unknown Album')) as album_title_sort,
                LOWER(COALESCE(NULLIF(TRIM(t.album_artist), ''), NULLIF(TRIM(t.artist), ''), 'Unknown Artist')) as album_artist_sort,
                MIN(t.year) as year,
                COUNT(*) as track_count
            FROM tracks_fts
            JOIN tracks t ON tracks_fts.rowid = t.id
            WHERE tracks_fts MATCH ? AND t.is_missing = 0
            GROUP BY album_artist_sort, album_title_sort
            HAVING (album_artist_sort, album_title_sort) > (?, ?)
            ORDER BY album_artist_sort ASC, album_title_sort ASC
            LIMIT ?
        "#;
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(
            params![
                &match_query,
                &c.album_artist_sort,
                &c.album_title_sort,
                limit + 1
            ],
            map_album_list_item,
        )?;
        rows.collect::<Result<Vec<_>, _>>()?
    } else {
        let sql = r#"
            SELECT 
                COALESCE(NULLIF(TRIM(t.album), ''), 'Unknown Album') as album_title_display,
                COALESCE(NULLIF(TRIM(t.album_artist), ''), NULLIF(TRIM(t.artist), ''), 'Unknown Artist') as album_artist_display,
                LOWER(COALESCE(NULLIF(TRIM(t.album), ''), 'Unknown Album')) as album_title_sort,
                LOWER(COALESCE(NULLIF(TRIM(t.album_artist), ''), NULLIF(TRIM(t.artist), ''), 'Unknown Artist')) as album_artist_sort,
                MIN(t.year) as year,
                COUNT(*) as track_count
            FROM tracks_fts
            JOIN tracks t ON tracks_fts.rowid = t.id
            WHERE tracks_fts MATCH ? AND t.is_missing = 0
            GROUP BY album_artist_sort, album_title_sort
            ORDER BY album_artist_sort ASC, album_title_sort ASC
            LIMIT ?
        "#;
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params![&match_query, limit + 1], map_album_list_item)?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let next_cursor = if items.len() > limit as usize {
        let last = items.pop().unwrap();
        Some(AlbumCursor {
            album_artist_sort: last.album_artist_sort,
            album_title_sort: last.album_title_sort,
        })
    } else {
        None
    };

    Ok(Page { items, next_cursor })
}

/// Search artists with pagination (for Search Results view)
pub fn search_artists_page(
    conn: &Connection,
    query: &str,
    limit: i64,
    cursor: Option<&ArtistCursor>,
) -> Result<Page<ArtistListItem, ArtistCursor>, LibraryError> {
    let match_query = match normalize_search_query(query) {
        Some(q) => q,
        None => {
            return Ok(Page {
                items: vec![],
                next_cursor: None,
            });
        }
    };

    let mut items: Vec<ArtistListItem> = if let Some(c) = cursor {
        let sql = r#"
            SELECT 
                COALESCE(NULLIF(TRIM(t.artist), ''), 'Unknown Artist') as artist_display,
                LOWER(COALESCE(NULLIF(TRIM(t.artist), ''), 'Unknown Artist')) as artist_sort,
                COUNT(*) as track_count,
                COUNT(DISTINCT 
                    LOWER(COALESCE(NULLIF(TRIM(t.album_artist), ''), NULLIF(TRIM(t.artist), ''), 'Unknown Artist')) || '||' ||
                    LOWER(COALESCE(NULLIF(TRIM(t.album), ''), 'Unknown Album'))
                ) as album_count
            FROM tracks_fts
            JOIN tracks t ON tracks_fts.rowid = t.id
            WHERE tracks_fts MATCH ? AND t.is_missing = 0
            GROUP BY artist_sort
            HAVING artist_sort > ?
            ORDER BY artist_sort ASC
            LIMIT ?
        "#;
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(
            params![&match_query, &c.artist_sort, limit + 1],
            map_artist_list_item,
        )?;
        rows.collect::<Result<Vec<_>, _>>()?
    } else {
        let sql = r#"
            SELECT 
                COALESCE(NULLIF(TRIM(t.artist), ''), 'Unknown Artist') as artist_display,
                LOWER(COALESCE(NULLIF(TRIM(t.artist), ''), 'Unknown Artist')) as artist_sort,
                COUNT(*) as track_count,
                COUNT(DISTINCT 
                    LOWER(COALESCE(NULLIF(TRIM(t.album_artist), ''), NULLIF(TRIM(t.artist), ''), 'Unknown Artist')) || '||' ||
                    LOWER(COALESCE(NULLIF(TRIM(t.album), ''), 'Unknown Album'))
                ) as album_count
            FROM tracks_fts
            JOIN tracks t ON tracks_fts.rowid = t.id
            WHERE tracks_fts MATCH ? AND t.is_missing = 0
            GROUP BY artist_sort
            ORDER BY artist_sort ASC
            LIMIT ?
        "#;
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params![&match_query, limit + 1], map_artist_list_item)?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let next_cursor = if items.len() > limit as usize {
        let last = items.pop().unwrap();
        Some(ArtistCursor {
            artist_sort: last.artist_sort,
        })
    } else {
        None
    };

    Ok(Page { items, next_cursor })
}
