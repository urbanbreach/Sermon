use crate::db::{self, open_db};
use crate::error::LibraryError;
use crate::identity::{compute_partial_hash, get_file_identity, IdentitySource};
use crate::models::{FolderOptions, QuickScanSummary, ScanProgress, ScanSummary, TrackRow};
use globset::{Glob, GlobSetBuilder};
use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;
use tracing::{info, warn};
use walkdir::WalkDir;

/// Scan a folder and upsert tracks to database
/// Returns scan summary and calls progress callback during scan
pub fn scan_folder<F>(
    db_path: &Path,
    folder_path: &str,
    on_progress: F,
) -> Result<ScanSummary, LibraryError>
where
    F: Fn(ScanProgress) + Send + 'static,
{
    let start = Instant::now();
    let scan_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let conn = open_db(db_path)?;

    // Get or create folder
    let folder = match db::get_folder_by_path(&conn, folder_path)? {
        Some(f) => f,
        None => db::add_folder(&conn, folder_path)?,
    };

    // Check folder accessibility
    let folder_path_obj = Path::new(folder_path);
    if !folder_path_obj.exists() {
        set_folder_unavailable(&conn, folder.id, "Folder does not exist")?;
        return Err(LibraryError::NotFound(format!(
            "Folder not found: {}",
            folder_path
        )));
    }

    if !folder_path_obj.is_dir() {
        set_folder_unavailable(&conn, folder.id, "Path is not a directory")?;
        return Err(LibraryError::NotFound(format!(
            "Not a directory: {}",
            folder_path
        )));
    }

    // Parse folder options
    let options: FolderOptions = serde_json::from_str(&folder.options_json).unwrap_or_default();

    // Build exclude glob set
    let exclude_set = build_glob_set(&options.exclude_patterns);

    // Record scan start
    set_scan_started(&conn, folder.id, scan_id as i64)?;

    // Clear folder status to available
    set_folder_available(&conn, folder.id)?;

    // Collect eligible files first (for total count)
    let files: Vec<_> = collect_eligible_files(folder_path_obj, &options, &exclude_set);
    let total = files.len() as u32;

    let mut scanned = 0u32;
    let mut skipped = 0u32;
    let mut errors = 0u32;
    let mut seen_track_ids: Vec<i64> = Vec::new();

    // Process each file
    for file_path in files {
        match process_file(&conn, folder.id, &file_path) {
            Ok(ProcessResult::Scanned(track_id)) => {
                scanned += 1;
                seen_track_ids.push(track_id);
            }
            Ok(ProcessResult::Skipped(track_id)) => {
                skipped += 1;
                seen_track_ids.push(track_id);
            }
            Err(e) => {
                errors += 1;
                warn!("Error processing {:?}: {}", file_path, e);
            }
        }

        // Report progress
        on_progress(ScanProgress {
            scanned: scanned + skipped,
            total,
            errors,
        });

        info!(
            "scan_progress scanned={} total={} errors={}",
            scanned + skipped,
            total,
            errors
        );
    }

    // Mark missing files (files in DB but not seen during scan)
    mark_missing_files(&conn, folder.id, &seen_track_ids)?;

    // Record scan completion
    set_scan_completed(&conn, folder.id)?;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    let summary = ScanSummary {
        scan_id,
        scanned,
        total,
        skipped,
        errors,
        elapsed_ms,
    };

    info!(
        "scan_complete scanned={} total={} skipped={} errors={} elapsed_ms={}",
        scanned, total, skipped, errors, elapsed_ms
    );

    Ok(summary)
}

enum ProcessResult {
    Scanned(i64),
    Skipped(i64),
}

fn process_file(
    conn: &Connection,
    folder_id: i64,
    path: &Path,
) -> Result<ProcessResult, LibraryError> {
    let identity = get_file_identity(path)?;

    // Check if file already exists and is unchanged
    let existing = find_existing_track(conn, &identity, path)?;

    if let Some((track_id, existing_mtime, existing_size)) = existing {
        // File exists - check if unchanged
        if existing_mtime == identity.mtime_ms && existing_size == identity.size_bytes {
            // Clear missing flag if set
            db::set_missing(conn, track_id, false)?;
            return Ok(ProcessResult::Skipped(track_id));
        }
    }

    // Read metadata
    let metadata = tags::read_metadata(path);

    // Compute hash for fallback identity if needed
    let hash = if identity.source == IdentitySource::Fallback {
        compute_partial_hash(path).ok()
    } else {
        None
    };

    // Build track row
    let path_str = path.to_string_lossy().to_string();
    let path_display = path.file_name().map(|n| n.to_string_lossy().to_string());

    let track = TrackRow {
        id: None,
        library_folder_id: folder_id,
        path: path_str.clone(),
        path_display,
        path_lossy: !path.as_os_str().to_str().is_some(),
        identity_source: identity.source.as_str().to_string(),
        volume_serial: identity.volume_serial.map(|v| v as i64),
        file_id: identity.file_id.map(|v| v as i64),
        mtime_ms: identity.mtime_ms,
        size_bytes: identity.size_bytes,
        hash,
        title: metadata.title,
        artist: metadata.artist,
        album: metadata.album,
        album_artist: metadata.album_artist,
        track_no: metadata.track_no.map(|v| v as i32),
        disc_no: metadata.disc_no.map(|v| v as i32),
        year: metadata.year.map(|v| v as i32),
        genre: metadata.genre,
        codec: metadata.codec,
        container: metadata.container,
        sample_rate: metadata.sample_rate.map(|v| v as i32),
        bit_depth: metadata.bit_depth.map(|v| v as i32),
        channels: metadata.channels.map(|v| v as i32),
        duration_ms: metadata.duration_ms.map(|v| v as i64),
        loudness_db: metadata.loudness_db.map(|v| v as f64),
        dsd_rate_hz: metadata.dsd_rate_hz.map(|v| v as i32),
        dsd_channels: metadata.dsd_channels.map(|v| v as i32),
        is_missing: false,
        missing_since_ms: None,
    };

    let track_id = db::upsert_track(conn, &track)?;
    Ok(ProcessResult::Scanned(track_id))
}

fn find_existing_track(
    conn: &Connection,
    identity: &crate::identity::FileIdentity,
    path: &Path,
) -> Result<Option<(i64, i64, i64)>, LibraryError> {
    use rusqlite::OptionalExtension;

    let path_str = path.to_string_lossy();

    // Try identity-based lookup first
    let result: Option<(i64, i64, i64)> = if identity.source == IdentitySource::Ntfs {
        conn.query_row(
            "SELECT id, mtime_ms, size_bytes FROM tracks 
             WHERE identity_source = 'ntfs' AND volume_serial = ? AND file_id = ?",
            params![
                identity.volume_serial.map(|v| v as i64),
                identity.file_id.map(|v| v as i64)
            ],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
    } else {
        conn.query_row(
            "SELECT id, mtime_ms, size_bytes FROM tracks WHERE path = ?",
            params![path_str.as_ref()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
    };

    Ok(result)
}

fn collect_eligible_files(
    folder: &Path,
    options: &FolderOptions,
    exclude_set: &Option<globset::GlobSet>,
) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();

    let walker = if options.recursive {
        WalkDir::new(folder).follow_links(options.follow_symlinks)
    } else {
        WalkDir::new(folder)
            .max_depth(1)
            .follow_links(options.follow_symlinks)
    };

    let extensions: Vec<String> = if options.include_extensions.is_empty() {
        vec![
            ".flac".into(),
            ".mp3".into(),
            ".m4a".into(),
            ".mp4".into(),
            ".wav".into(),
            ".ogg".into(),
            ".opus".into(),
            ".aiff".into(),
            ".aif".into(),
            ".dsf".into(),
            ".dff".into(),
        ]
    } else {
        options
            .include_extensions
            .iter()
            .map(|e| e.to_lowercase())
            .collect()
    };

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        // Check extension
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e.to_lowercase()))
            .unwrap_or_default();

        if !extensions.contains(&ext) {
            continue;
        }

        // Check exclude patterns
        if let Some(exclude) = exclude_set {
            let normalized = path.to_string_lossy().replace('\\', "/");
            if exclude.is_match(&normalized) {
                continue;
            }
        }

        files.push(path.to_path_buf());
    }

    files
}

fn build_glob_set(patterns: &[String]) -> Option<globset::GlobSet> {
    if patterns.is_empty() {
        return None;
    }

    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        match Glob::new(pattern) {
            Ok(glob) => {
                builder.add(glob);
            }
            Err(e) => {
                warn!("Invalid glob pattern '{}': {}", pattern, e);
            }
        }
    }

    builder.build().ok()
}

fn set_folder_unavailable(
    conn: &Connection,
    folder_id: i64,
    error: &str,
) -> Result<(), LibraryError> {
    conn.execute(
        "UPDATE library_folders SET status = 'unavailable', last_error = ? WHERE id = ?",
        params![error, folder_id],
    )?;
    Ok(())
}

fn set_folder_available(conn: &Connection, folder_id: i64) -> Result<(), LibraryError> {
    conn.execute(
        "UPDATE library_folders SET status = 'available', last_error = NULL WHERE id = ?",
        params![folder_id],
    )?;
    Ok(())
}

fn set_scan_started(conn: &Connection, folder_id: i64, scan_id: i64) -> Result<(), LibraryError> {
    conn.execute(
        "INSERT INTO scan_state (folder_id, last_scan_started_ms) VALUES (?, ?)
         ON CONFLICT(folder_id) DO UPDATE SET last_scan_started_ms = ?",
        params![folder_id, scan_id, scan_id],
    )?;
    Ok(())
}

fn set_scan_completed(conn: &Connection, folder_id: i64) -> Result<(), LibraryError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    conn.execute(
        "UPDATE scan_state SET last_scan_completed_ms = ? WHERE folder_id = ?",
        params![now, folder_id],
    )?;
    Ok(())
}

fn mark_missing_files(
    conn: &Connection,
    folder_id: i64,
    seen_ids: &[i64],
) -> Result<(), LibraryError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    if seen_ids.is_empty() {
        // Mark ALL tracks in folder as missing
        conn.execute(
            "UPDATE tracks SET is_missing = 1, missing_since_ms = ? 
             WHERE library_folder_id = ? AND is_missing = 0",
            params![now, folder_id],
        )?;
    } else {
        // Mark tracks NOT in seen_ids as missing
        let placeholders: Vec<String> = seen_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "UPDATE tracks SET is_missing = 1, missing_since_ms = ? 
             WHERE library_folder_id = ? AND id NOT IN ({}) AND is_missing = 0",
            placeholders.join(",")
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(now), Box::new(folder_id)];
        for id in seen_ids {
            params_vec.push(Box::new(*id));
        }

        conn.execute(
            &sql,
            rusqlite::params_from_iter(params_vec.iter().map(|b| b.as_ref())),
        )?;
    }

    Ok(())
}

/// Quick scan all library folders on startup
/// - Checks if existing tracks still exist on disk (marks missing if not)
/// - Detects new files in folders (adds them to library)
/// - Restores previously missing files that reappear
/// This is faster than a full scan as it skips unchanged files
pub fn quick_scan(db_path: &Path) -> Result<QuickScanSummary, LibraryError> {
    let start = Instant::now();
    let conn = open_db(db_path)?;

    let mut folders_checked = 0u32;
    let mut files_checked = 0u32;
    let mut files_added = 0u32;
    let mut files_marked_missing = 0u32;
    let mut files_restored = 0u32;

    // Get all enabled folders
    let folders = db::list_folders(&conn)?;

    for folder in folders.iter().filter(|f| f.enabled) {
        let folder_path = Path::new(&folder.path);

        if !folder_path.exists() || !folder_path.is_dir() {
            warn!("Quick scan: folder unavailable: {}", folder.path);
            continue;
        }

        folders_checked += 1;

        // Parse folder options
        let options: FolderOptions = serde_json::from_str(&folder.options_json).unwrap_or_default();
        let exclude_set = build_glob_set(&options.exclude_patterns);

        // 1. Check existing tracks in this folder for missing files
        let existing_tracks = get_folder_tracks(&conn, folder.id)?;

        for (track_id, track_path, is_missing) in &existing_tracks {
            files_checked += 1;
            let path = Path::new(track_path);

            if path.exists() {
                // File exists - restore if was marked missing
                if *is_missing {
                    db::set_missing(&conn, *track_id, false)?;
                    files_restored += 1;
                    info!("Quick scan: restored track {} ({})", track_id, track_path);
                }
            } else {
                // File doesn't exist - mark as missing if not already
                if !*is_missing {
                    db::set_missing(&conn, *track_id, true)?;
                    files_marked_missing += 1;
                    info!(
                        "Quick scan: marked missing track {} ({})",
                        track_id, track_path
                    );
                }
            }
        }

        // 2. Check for new files in folder
        let existing_paths: HashSet<String> = existing_tracks
            .iter()
            .map(|(_, path, _)| path.clone())
            .collect();

        let disk_files = collect_eligible_files(folder_path, &options, &exclude_set);

        for file_path in disk_files {
            let path_str = file_path.to_string_lossy().to_string();

            if !existing_paths.contains(&path_str) {
                // New file found - add it
                match process_file(&conn, folder.id, &file_path) {
                    Ok(_) => {
                        files_added += 1;
                        info!("Quick scan: added new track {}", path_str);
                    }
                    Err(e) => {
                        warn!("Quick scan: failed to add {}: {}", path_str, e);
                    }
                }
            }
        }
    }

    let elapsed_ms = start.elapsed().as_millis() as u64;

    let summary = QuickScanSummary {
        folders_checked,
        files_checked,
        files_added,
        files_marked_missing,
        files_restored,
        elapsed_ms,
    };

    info!(
        "quick_scan_complete folders={} checked={} added={} missing={} restored={} elapsed_ms={}",
        folders_checked,
        files_checked,
        files_added,
        files_marked_missing,
        files_restored,
        elapsed_ms
    );

    Ok(summary)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LoudnessBackfillSummary {
    pub candidates: u64,
    pub checked: u64,
    pub updated: u64,
    pub skipped: bool,
}

/// One-time backfill for tracks that predate loudness_db extraction.
///
/// This reads metadata for tracks with NULL loudness_db and writes parsed
/// ReplayGain values when available. The operation is guarded by a setting
/// key so it runs at most once per database.
pub fn backfill_loudness_metadata_once(
    db_path: &Path,
) -> Result<LoudnessBackfillSummary, LibraryError> {
    let conn = open_db(db_path)?;
    let marker_key = "library.loudness_backfill_v1_done";

    if matches!(db::get_setting(&conn, marker_key)?, Some(v) if v == "on") {
        return Ok(LoudnessBackfillSummary {
            skipped: true,
            ..Default::default()
        });
    }

    let candidates = get_tracks_without_loudness(&conn)?;
    let mut summary = LoudnessBackfillSummary {
        candidates: candidates.len() as u64,
        checked: 0,
        updated: 0,
        skipped: false,
    };

    for (track_id, track_path) in candidates {
        let path = Path::new(&track_path);
        if !path.exists() {
            continue;
        }

        summary.checked += 1;
        let metadata = tags::read_metadata(path);
        if let Some(loudness_db) = metadata.loudness_db {
            db::update_track_loudness(&conn, track_id, loudness_db as f64)?;
            summary.updated += 1;
        }
    }

    db::set_setting(&conn, marker_key, "on")?;

    info!(
        candidates = summary.candidates,
        checked = summary.checked,
        updated = summary.updated,
        "loudness_backfill_complete"
    );

    Ok(summary)
}

/// Get all tracks for a folder with their paths and missing status
fn get_folder_tracks(
    conn: &Connection,
    folder_id: i64,
) -> Result<Vec<(i64, String, bool)>, LibraryError> {
    let mut stmt =
        conn.prepare("SELECT id, path, is_missing FROM tracks WHERE library_folder_id = ?")?;

    let rows = stmt.query_map(params![folder_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?;

    let mut tracks = Vec::new();
    for row in rows {
        tracks.push(row?);
    }

    Ok(tracks)
}

fn get_tracks_without_loudness(conn: &Connection) -> Result<Vec<(i64, String)>, LibraryError> {
    let mut stmt =
        conn.prepare("SELECT id, path FROM tracks WHERE is_missing = 0 AND loudness_db IS NULL")?;

    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

    let mut tracks = Vec::new();
    for row in rows {
        tracks.push(row?);
    }

    Ok(tracks)
}
