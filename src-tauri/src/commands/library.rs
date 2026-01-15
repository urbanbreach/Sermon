use crate::state::LibraryState;
use library::db::{add_folder, list_folders};
use library::{LibraryFolder, TrackRow, apply_migrations, list_tracks, open_db, scan_folder};
use serde::Serialize;
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
