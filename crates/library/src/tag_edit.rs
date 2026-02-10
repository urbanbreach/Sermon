//! Tag editing functionality for the library crate
//!
//! Provides the update_track_tags function that coordinates:
//! - Safe write with backup
//! - Tag mutation via the tags crate
//! - DB identity and metadata refresh

use crate::error::LibraryError;
use crate::identity::{get_file_identity, FileIdentity};
use crate::safe_write::{safe_write_with_callback, SafeWriteError, SafeWriteOptions, WriteStatus};
use rusqlite::{params, Connection};
use std::path::Path;
use tags::{read_metadata_result, write_tags, NumberPatch, TagPatch, TagPatches, TagWriteOptions};
use tracing::{debug, info};

/// Error type for tag update operations
#[derive(Debug)]
pub enum TagUpdateError {
    TrackNotFound(i64),
    FileNotFound(String),
    SafeWriteFailed(SafeWriteError),
    TagWriteFailed(String),
    MetadataReadFailed(String),
    DbUpdateFailed(String),
    DuplicationHazard(String),
}

impl std::fmt::Display for TagUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagUpdateError::TrackNotFound(id) => write!(f, "Track not found: {}", id),
            TagUpdateError::FileNotFound(path) => write!(f, "File not found: {}", path),
            TagUpdateError::SafeWriteFailed(e) => write!(f, "Safe write failed: {}", e),
            TagUpdateError::TagWriteFailed(e) => write!(f, "Tag write failed: {}", e),
            TagUpdateError::MetadataReadFailed(e) => write!(f, "Metadata read failed: {}", e),
            TagUpdateError::DbUpdateFailed(e) => write!(f, "DB update failed: {}", e),
            TagUpdateError::DuplicationHazard(e) => write!(f, "Duplication hazard: {}", e),
        }
    }
}

impl std::error::Error for TagUpdateError {}

impl From<SafeWriteError> for TagUpdateError {
    fn from(e: SafeWriteError) -> Self {
        TagUpdateError::SafeWriteFailed(e)
    }
}

impl From<LibraryError> for TagUpdateError {
    fn from(e: LibraryError) -> Self {
        TagUpdateError::DbUpdateFailed(e.to_string())
    }
}

/// Request to update track tags
#[derive(Debug, Clone)]
pub struct UpdateTagsRequest {
    pub track_id: i64,
    pub create_backup: bool,
    pub title: TagPatch,
    pub artist: TagPatch,
    pub album: TagPatch,
    pub album_artist: TagPatch,
    pub genre: TagPatch,
    pub publisher: TagPatch,
    pub composer: TagPatch,
    pub conductor: TagPatch,
    pub comments: TagPatch,
    pub grouping: TagPatch,
    pub lyricist: TagPatch,
    pub plain_lyrics: TagPatch,
    pub synced_lyrics: TagPatch,
    pub track_no: NumberPatch,
    pub disc_no: NumberPatch,
    pub year: NumberPatch,
}

impl UpdateTagsRequest {
    /// Convert to TagPatches for the tags crate
    pub fn to_patches(&self) -> TagPatches {
        TagPatches {
            title: self.title.clone(),
            artist: self.artist.clone(),
            album: self.album.clone(),
            album_artist: self.album_artist.clone(),
            genre: self.genre.clone(),
            publisher: self.publisher.clone(),
            composer: self.composer.clone(),
            conductor: self.conductor.clone(),
            comments: self.comments.clone(),
            grouping: self.grouping.clone(),
            lyricist: self.lyricist.clone(),
            plain_lyrics: self.plain_lyrics.clone(),
            synced_lyrics: self.synced_lyrics.clone(),
            track_no: self.track_no.clone(),
            disc_no: self.disc_no.clone(),
            year: self.year.clone(),
            picture: Default::default(),
        }
    }

    pub fn has_lyrics_changes(&self) -> bool {
        !matches!(&self.lyricist, TagPatch::Leave)
            || !matches!(&self.plain_lyrics, TagPatch::Leave)
            || !matches!(&self.synced_lyrics, TagPatch::Leave)
    }
}

/// Update track tags with safe write and DB sync
///
/// # Algorithm
/// 1. Look up track by ID
/// 2. Verify file exists on disk
/// 3. Perform safe write (copy to temp, apply patches, atomic commit)
/// 4. Re-read identity after commit (NTFS File ID may change)
/// 5. Re-read metadata for DB refresh
/// 6. Update DB row with new identity and metadata
///
/// # Status Callback
/// The callback is invoked during retry attempts to allow UI feedback.
pub fn update_track_tags<F>(
    conn: &Connection,
    request: &UpdateTagsRequest,
    status_callback: Option<F>,
) -> Result<(), TagUpdateError>
where
    F: Fn(WriteStatus) + Clone,
{
    // Step 1: Look up track by ID
    let track = crate::db::get_track_by_id(conn, request.track_id)
        .map_err(|_| TagUpdateError::TrackNotFound(request.track_id))?;

    let file_path = Path::new(&track.path);

    // Step 2: Verify file exists
    if !file_path.exists() {
        return Err(TagUpdateError::FileNotFound(track.path.clone()));
    }

    info!(
        "Updating tags for track {}: {}",
        request.track_id, track.path
    );

    // Step 3: Perform safe write
    let patches = request.to_patches();
    let options = SafeWriteOptions {
        create_backup: request.create_backup,
    };

    safe_write_with_callback(file_path, &options, status_callback, |tmp_path| {
        // Apply tag patches to the temp file
        let tag_options = TagWriteOptions::new();
        write_tags(tmp_path, &patches, &tag_options)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    })?;

    debug!("Safe write completed for track {}", request.track_id);

    // Step 4: Re-read identity after commit
    let new_identity = get_file_identity(file_path)?;

    // Check for duplication hazard (only for NTFS identity)
    if new_identity.source == crate::identity::IdentitySource::Ntfs {
        check_duplication_hazard(conn, request.track_id, &new_identity)?;
    }

    // Step 5: Re-read metadata
    let new_metadata = read_metadata_result(file_path)
        .map_err(|e| TagUpdateError::MetadataReadFailed(e.to_string()))?;

    // Step 6: Update DB row with new identity and metadata
    update_track_identity_and_metadata(conn, request.track_id, &new_identity, &new_metadata)?;

    if request.has_lyrics_changes() {
        sync_lyrics_cache(conn, request.track_id, &new_metadata)?;
    }

    info!(
        "Identity updated after atomic commit for track {}",
        request.track_id
    );

    Ok(())
}

/// Check if the new identity would conflict with an existing track
fn check_duplication_hazard(
    conn: &Connection,
    current_track_id: i64,
    new_identity: &FileIdentity,
) -> Result<(), TagUpdateError> {
    if let (Some(vol), Some(file_id)) = (new_identity.volume_serial, new_identity.file_id) {
        let existing_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM tracks WHERE identity_source = 'ntfs' AND volume_serial = ? AND file_id = ? AND id != ?",
                params![vol, file_id, current_track_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| TagUpdateError::DbUpdateFailed(e.to_string()))?;

        if let Some(conflict_id) = existing_id {
            return Err(TagUpdateError::DuplicationHazard(format!(
                "New identity conflicts with existing track ID {}",
                conflict_id
            )));
        }
    }
    Ok(())
}

/// Update track identity and metadata in DB
fn update_track_identity_and_metadata(
    conn: &Connection,
    track_id: i64,
    identity: &FileIdentity,
    metadata: &tags::AudioMetadata,
) -> Result<(), TagUpdateError> {
    conn.execute(
        "UPDATE tracks SET 
            identity_source = ?,
            volume_serial = ?,
            file_id = ?,
            mtime_ms = ?,
            size_bytes = ?,
            hash = ?,
            title = ?,
            artist = ?,
            album = ?,
            album_artist = ?,
            track_no = ?,
            disc_no = ?,
            year = ?,
            genre = ?,
            codec = ?,
            container = ?,
            sample_rate = ?,
            bit_depth = ?,
            channels = ?,
            duration_ms = ?,
            loudness_db = ?
         WHERE id = ?",
        params![
            identity.source.as_str(),
            identity.volume_serial,
            identity.file_id,
            identity.mtime_ms,
            identity.size_bytes,
            identity.hash,
            metadata.title,
            metadata.artist,
            metadata.album,
            metadata.album_artist,
            metadata.track_no,
            metadata.disc_no,
            metadata.year,
            metadata.genre,
            metadata.codec,
            metadata.container,
            metadata.sample_rate,
            metadata.bit_depth,
            metadata.channels,
            metadata.duration_ms,
            metadata.loudness_db,
            track_id
        ],
    )
    .map_err(|e| TagUpdateError::DbUpdateFailed(e.to_string()))?;

    Ok(())
}

fn sync_lyrics_cache(
    conn: &Connection,
    track_id: i64,
    metadata: &tags::AudioMetadata,
) -> Result<(), TagUpdateError> {
    let synced = metadata.synced_lyrics.as_deref();
    let plain = metadata.lyrics.as_deref();
    let source = if synced.is_some() || plain.is_some() {
        "manual"
    } else {
        "manual-none"
    };

    conn.execute(
        "INSERT OR REPLACE INTO lyrics_cache (track_id, synced_lyrics, plain_lyrics, source, fetched_at) VALUES (?1, ?2, ?3, ?4, (strftime('%s','now') * 1000))",
        params![track_id, synced, plain, source],
    )
    .map_err(|e| TagUpdateError::DbUpdateFailed(e.to_string()))?;

    Ok(())
}

// Re-export for optional trait
use rusqlite::OptionalExtension;
