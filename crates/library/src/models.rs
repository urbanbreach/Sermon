use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryFolder {
    pub id: i64,
    pub path: String,
    pub enabled: bool,
    pub status: String,
    pub last_error: Option<String>,
    pub options_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TrackRow {
    pub id: Option<i64>,
    pub library_folder_id: i64,
    pub path: String,
    pub path_display: Option<String>,
    pub path_lossy: bool,
    pub identity_source: String,
    pub volume_serial: Option<i64>,
    pub file_id: Option<i64>,
    pub mtime_ms: i64,
    pub size_bytes: i64,
    pub hash: Option<String>,
    // Metadata
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_no: Option<i32>,
    pub disc_no: Option<i32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    // Technical
    pub codec: Option<String>,
    pub container: Option<String>,
    pub sample_rate: Option<i32>,
    pub bit_depth: Option<i32>,
    pub channels: Option<i32>,
    pub duration_ms: Option<i64>,
    pub loudness_db: Option<f64>,
    // DSD-specific
    pub dsd_rate_hz: Option<i32>,
    pub dsd_channels: Option<i32>,
    // Missing
    pub is_missing: bool,
    pub missing_since_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanState {
    pub folder_id: i64,
    pub last_scan_started_ms: Option<i64>,
    pub last_scan_completed_ms: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct ScanProgress {
    pub scanned: u32,
    pub total: u32,
    pub errors: u32,
}

#[derive(Debug, Clone)]
pub struct ScanSummary {
    pub scan_id: u64,
    pub scanned: u32,
    pub total: u32,
    pub skipped: u32,
    pub errors: u32,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderOptions {
    #[serde(default = "default_recursive")]
    pub recursive: bool,
    #[serde(default)]
    pub include_extensions: Vec<String>,
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    #[serde(default)]
    pub follow_symlinks: bool,
}

fn default_recursive() -> bool {
    true
}

impl Default for FolderOptions {
    fn default() -> Self {
        Self {
            recursive: true,
            include_extensions: vec![
                ".flac".into(),
                ".mp3".into(),
                ".m4a".into(),
                ".wav".into(),
                ".ogg".into(),
                ".dsf".into(),
                ".dff".into(),
            ],
            exclude_patterns: vec![],
            follow_symlinks: false,
        }
    }
}

// ============================================================================
// Browse DTOs (Milestone 04)
// ============================================================================

/// Album list item derived from tracks (no normalized album table)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumListItem {
    pub album_title_display: String,
    pub album_artist_display: String,
    pub album_title_sort: String,
    pub album_artist_sort: String,
    pub year: Option<i32>,
    pub track_count: i64,
}

/// Artist list item derived from tracks (no normalized artist table)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistListItem {
    pub artist_display: String,
    pub artist_sort: String,
    pub track_count: i64,
    pub album_count: i64,
}

/// Cursor for album pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumCursor {
    pub album_artist_sort: String,
    pub album_title_sort: String,
}

/// Cursor for artist pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistCursor {
    pub artist_sort: String,
}

/// Cursor for album detail tracks pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumTrackCursor {
    pub disc_no: i32,
    pub track_no: i32,
    pub title_sort: String,
    pub id: i64,
}

/// Cursor for artist tracks pagination  
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistTrackCursor {
    pub album_title_sort: String,
    pub disc_no: i32,
    pub track_no: i32,
    pub title_sort: String,
    pub id: i64,
}

/// Generic paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T, C> {
    pub items: Vec<T>,
    pub next_cursor: Option<C>,
}

/// Offset-based cursor for simple pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OffsetCursor {
    pub offset: i64,
}

/// Library statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryStats {
    pub track_count: i64,
    pub album_count: i64,
    pub artist_count: i64,
    pub db_size_bytes: i64,
    pub last_scan_completed_ms: Option<i64>,
}

/// Quick scan summary (startup scan)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickScanSummary {
    pub folders_checked: u32,
    pub files_checked: u32,
    pub files_added: u32,
    pub files_marked_missing: u32,
    pub files_restored: u32,
    pub elapsed_ms: u64,
}

// ============================================================================
// Search DTOs (Milestone 04)
// ============================================================================

/// Search hit - discriminated union for track/album/artist results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SearchHit {
    #[serde(rename = "track")]
    Track {
        #[serde(rename = "trackId")]
        track_id: i64,
        title: Option<String>,
        artist: Option<String>,
        album: Option<String>,
    },
    #[serde(rename = "album")]
    Album {
        #[serde(rename = "albumArtistSort")]
        album_artist_sort: String,
        #[serde(rename = "albumTitleSort")]
        album_title_sort: String,
        #[serde(rename = "albumArtistDisplay")]
        album_artist_display: String,
        #[serde(rename = "albumTitleDisplay")]
        album_title_display: String,
        year: Option<i32>,
    },
    #[serde(rename = "artist")]
    Artist {
        #[serde(rename = "artistSort")]
        artist_sort: String,
        #[serde(rename = "artistDisplay")]
        artist_display: String,
    },
}

/// Search suggestion response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchSuggestResponse {
    pub results: Vec<SearchHit>,
}
