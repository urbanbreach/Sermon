use crate::state::LibraryState;
use library::db::{
    add_folder, get_library_stats, list_album_tracks_page, list_albums_page,
    list_artist_tracks_page, list_artists_page, list_folders, list_tracks_page, search_albums_page,
    search_artists_page, search_suggest, search_tracks_page,
};
use library::models::{
    AlbumCursor, AlbumListItem, ArtistCursor, ArtistListItem, LibraryStats, OffsetCursor, Page,
    SearchSuggestResponse,
};
use library::{LibraryFolder, TrackRow, apply_migrations, list_tracks, open_db, scan_folder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tracing::{error, info};

#[derive(Debug, Serialize)]
pub struct FolderResponse {
    pub id: i64,
    pub path: String,
    pub enabled: bool,
}

impl From<LibraryFolder> for FolderResponse {
    fn from(f: LibraryFolder) -> Self {
        Self {
            id: f.id,
            path: f.path,
            enabled: f.enabled,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ScanStartResponse {
    pub scan_id: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanProgressEvent {
    pub scanned: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanCompleteEvent {
    pub scan_id: u64,
    pub scanned: u32,
    pub total: u32,
    pub skipped: u32,
    pub errors: u32,
    pub elapsed_ms: u64,
}

#[tauri::command]
pub fn cmd_library_add_folder(
    state: State<'_, LibraryState>,
    path: String,
) -> Result<FolderResponse, String> {
    let conn = open_db(&state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;
    let folder = add_folder(&conn, &path).map_err(|e| e.to_string())?;
    Ok(folder.into())
}

#[tauri::command]
pub fn cmd_library_list_folders(
    state: State<'_, LibraryState>,
) -> Result<Vec<FolderResponse>, String> {
    let conn = open_db(&state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;
    let folders = list_folders(&conn).map_err(|e| e.to_string())?;
    Ok(folders.into_iter().map(|f| f.into()).collect())
}

#[tauri::command]
pub fn cmd_library_list_tracks(
    state: State<'_, LibraryState>,
    sort_by: String,
    direction: String,
) -> Result<Vec<TrackRow>, String> {
    let conn = open_db(&state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;
    let tracks = list_tracks(&conn, &sort_by, &direction).map_err(|e| e.to_string())?;
    Ok(tracks)
}

#[tauri::command]
pub async fn cmd_scan_start(
    state: State<'_, LibraryState>,
    app: AppHandle,
    path: String,
) -> Result<ScanStartResponse, String> {
    // Check if scan is already running
    {
        let lock = state.scan_lock.lock().map_err(|e| e.to_string())?;
        if lock.is_some() {
            return Err("scan already running".to_string());
        }
    }

    // Verify folder exists
    let folder_path = std::path::Path::new(&path);
    if !folder_path.exists() {
        return Err("folder not found".to_string());
    }
    if !folder_path.is_dir() {
        return Err("path is not a directory".to_string());
    }

    // Generate scan_id
    let scan_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // Set scan lock
    {
        let mut lock = state.scan_lock.lock().map_err(|e| e.to_string())?;
        *lock = Some(scan_id);
    }

    let db_path = state.db_path.clone();
    let path_clone = path.clone();
    let app_clone = app.clone();
    let state_db_path = state.db_path.clone();

    // Ensure folder is added to DB
    {
        let conn = open_db(&state_db_path).map_err(|e| e.to_string())?;
        apply_migrations(&conn).map_err(|e| e.to_string())?;
        add_folder(&conn, &path).map_err(|e| e.to_string())?;
    }

    // Spawn scan task
    tauri::async_runtime::spawn(async move {
        let last_emit = Arc::new(std::sync::Mutex::new(std::time::Instant::now()));
        let emit_interval = std::time::Duration::from_millis(100); // 10/sec max
        let progress_app = app_clone.clone();

        let result = scan_folder(&db_path, &path_clone, move |progress| {
            // Rate limit progress events to 10/sec
            let mut last = last_emit.lock().unwrap();
            let now = std::time::Instant::now();
            if now.duration_since(*last) >= emit_interval {
                *last = now;
                let _ = progress_app.emit(
                    "evt_scan_progress",
                    ScanProgressEvent {
                        scanned: progress.scanned,
                        total: progress.total,
                    },
                );
            }
        });

        match result {
            Ok(summary) => {
                info!("Scan completed: {:?}", summary);
                let _ = app_clone.emit(
                    "evt_scan_complete",
                    ScanCompleteEvent {
                        scan_id: summary.scan_id,
                        scanned: summary.scanned,
                        total: summary.total,
                        skipped: summary.skipped,
                        errors: summary.errors,
                        elapsed_ms: summary.elapsed_ms,
                    },
                );
            }
            Err(e) => {
                error!("Scan failed: {}", e);
                let _ = app_clone.emit(
                    "evt_scan_complete",
                    ScanCompleteEvent {
                        scan_id,
                        scanned: 0,
                        total: 0,
                        skipped: 0,
                        errors: 1,
                        elapsed_ms: 0,
                    },
                );
            }
        }

        // Clear scan lock
        let state = app_clone.state::<LibraryState>();
        if let Ok(mut lock) = state.scan_lock.lock() {
            *lock = None;
        }
    });

    Ok(ScanStartResponse { scan_id })
}

// ============================================================================
// Browse Commands (Milestone 04)
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTracksPageRequest {
    pub sort_by: String,
    pub direction: String,
    pub limit: i64,
    pub cursor: Option<OffsetCursor>,
}

#[tauri::command]
pub fn cmd_library_list_tracks_page(
    state: State<'_, LibraryState>,
    request: ListTracksPageRequest,
) -> Result<Page<TrackRow, OffsetCursor>, String> {
    let conn = open_db(&state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;
    let offset = request.cursor.map(|c| c.offset).unwrap_or(0);
    list_tracks_page(
        &conn,
        &request.sort_by,
        &request.direction,
        request.limit,
        offset,
    )
    .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAlbumsPageRequest {
    pub limit: i64,
    pub cursor: Option<AlbumCursor>,
}

#[tauri::command]
pub fn cmd_library_list_albums_page(
    state: State<'_, LibraryState>,
    request: ListAlbumsPageRequest,
) -> Result<Page<AlbumListItem, AlbumCursor>, String> {
    let conn = open_db(&state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;
    list_albums_page(&conn, request.limit, request.cursor.as_ref()).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListArtistsPageRequest {
    pub limit: i64,
    pub cursor: Option<ArtistCursor>,
}

#[tauri::command]
pub fn cmd_library_list_artists_page(
    state: State<'_, LibraryState>,
    request: ListArtistsPageRequest,
) -> Result<Page<ArtistListItem, ArtistCursor>, String> {
    let conn = open_db(&state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;
    list_artists_page(&conn, request.limit, request.cursor.as_ref()).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAlbumTracksPageRequest {
    pub album_artist_sort: String,
    pub album_title_sort: String,
    pub limit: i64,
    pub cursor: Option<AlbumTrackCursorRequest>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumTrackCursorRequest {
    pub disc_no: i32,
    pub track_no: i32,
    pub title_sort: String,
    pub id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumTrackCursorResponse {
    pub disc_no: i32,
    pub track_no: i32,
    pub title_sort: String,
    pub id: i64,
}

#[tauri::command]
pub fn cmd_library_list_album_tracks_page(
    state: State<'_, LibraryState>,
    request: ListAlbumTracksPageRequest,
) -> Result<Page<TrackRow, AlbumTrackCursorResponse>, String> {
    let conn = open_db(&state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;

    let cursor = request
        .cursor
        .as_ref()
        .map(|c| (c.disc_no, c.track_no, c.title_sort.as_str(), c.id));

    let result = list_album_tracks_page(
        &conn,
        &request.album_artist_sort,
        &request.album_title_sort,
        request.limit,
        cursor,
    )
    .map_err(|e| e.to_string())?;

    Ok(Page {
        items: result.items,
        next_cursor: result
            .next_cursor
            .map(|(d, t, ts, id)| AlbumTrackCursorResponse {
                disc_no: d,
                track_no: t,
                title_sort: ts,
                id,
            }),
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListArtistTracksPageRequest {
    pub artist_sort: String,
    pub limit: i64,
    pub cursor: Option<ArtistTrackCursorRequest>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistTrackCursorRequest {
    pub album_title_sort: String,
    pub disc_no: i32,
    pub track_no: i32,
    pub title_sort: String,
    pub id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistTrackCursorResponse {
    pub album_title_sort: String,
    pub disc_no: i32,
    pub track_no: i32,
    pub title_sort: String,
    pub id: i64,
}

#[tauri::command]
pub fn cmd_library_list_artist_tracks_page(
    state: State<'_, LibraryState>,
    request: ListArtistTracksPageRequest,
) -> Result<Page<TrackRow, ArtistTrackCursorResponse>, String> {
    let conn = open_db(&state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;

    let cursor = request.cursor.as_ref().map(|c| {
        (
            c.album_title_sort.as_str(),
            c.disc_no,
            c.track_no,
            c.title_sort.as_str(),
            c.id,
        )
    });

    let result = list_artist_tracks_page(&conn, &request.artist_sort, request.limit, cursor)
        .map_err(|e| e.to_string())?;

    Ok(Page {
        items: result.items,
        next_cursor: result
            .next_cursor
            .map(|(a, d, t, ts, id)| ArtistTrackCursorResponse {
                album_title_sort: a,
                disc_no: d,
                track_no: t,
                title_sort: ts,
                id,
            }),
    })
}

// ============================================================================
// Search Commands (Milestone 04)
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchSuggestRequest {
    pub query: String,
    pub limit: Option<i64>,
}

#[tauri::command]
pub fn cmd_library_search_suggest(
    state: State<'_, LibraryState>,
    request: SearchSuggestRequest,
) -> Result<SearchSuggestResponse, String> {
    let conn = open_db(&state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;
    let limit = request.limit.unwrap_or(12);
    search_suggest(&conn, &request.query, limit).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchTracksPageRequest {
    pub query: String,
    pub sort_by: String,
    pub direction: String,
    pub limit: i64,
    pub cursor: Option<OffsetCursor>,
}

#[tauri::command]
pub fn cmd_library_search_tracks_page(
    state: State<'_, LibraryState>,
    request: SearchTracksPageRequest,
) -> Result<Page<TrackRow, OffsetCursor>, String> {
    let conn = open_db(&state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;
    let offset = request.cursor.map(|c| c.offset).unwrap_or(0);
    search_tracks_page(
        &conn,
        &request.query,
        &request.sort_by,
        &request.direction,
        request.limit,
        offset,
    )
    .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchAlbumsPageRequest {
    pub query: String,
    pub limit: i64,
    pub cursor: Option<AlbumCursor>,
}

#[tauri::command]
pub fn cmd_library_search_albums_page(
    state: State<'_, LibraryState>,
    request: SearchAlbumsPageRequest,
) -> Result<Page<AlbumListItem, AlbumCursor>, String> {
    let conn = open_db(&state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;
    search_albums_page(
        &conn,
        &request.query,
        request.limit,
        request.cursor.as_ref(),
    )
    .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchArtistsPageRequest {
    pub query: String,
    pub limit: i64,
    pub cursor: Option<ArtistCursor>,
}

#[tauri::command]
pub fn cmd_library_search_artists_page(
    state: State<'_, LibraryState>,
    request: SearchArtistsPageRequest,
) -> Result<Page<ArtistListItem, ArtistCursor>, String> {
    let conn = open_db(&state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;
    search_artists_page(
        &conn,
        &request.query,
        request.limit,
        request.cursor.as_ref(),
    )
    .map_err(|e| e.to_string())
}

// ============================================================================
// Stats Command (Milestone 04)
// ============================================================================

#[tauri::command]
pub fn cmd_library_get_stats(state: State<'_, LibraryState>) -> Result<LibraryStats, String> {
    let conn = open_db(&state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;
    get_library_stats(&conn, &state.db_path).map_err(|e| e.to_string())
}
