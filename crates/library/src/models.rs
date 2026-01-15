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

#[derive(Debug, Clone, Deserialize)]
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
            ],
            exclude_patterns: vec![],
            follow_symlinks: false,
        }
    }
}
