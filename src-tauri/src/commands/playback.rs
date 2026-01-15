use crate::state::{AudioState, LibraryState, PlaybackCommand};
use audio_engine::device::list_devices;
use library::{apply_migrations, get_track_by_id, open_db};
use serde::Serialize;
use tauri::State;

// -----------------
// Event payloads
// -----------------

#[derive(Debug, Clone, Serialize)]
pub struct PlaybackStateEvent {
    pub state: String, // "playing" | "paused" | "stopped"
    pub play_id: Option<String>,
    pub track_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NowPlayingEvent {
    pub play_id: String,
    pub track: TrackEventData,
    pub position_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackEventData {
    pub id: i64,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_ms: Option<u64>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u16>,
    pub channels: Option<u16>,
    pub codec: Option<String>,
    pub container: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaybackPositionEvent {
    pub play_id: String,
    pub position_ms: u64,
    pub played_ms: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueueChangedEvent {
    pub play_id: Option<String>,
    pub current_index: Option<usize>,
    pub queue: Vec<QueueItemData>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueueItemData {
    pub track_id: i64,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceChangedEvent {
    pub device_id: String,
    pub device_name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioDebugEvent {
    pub output_mode: String, // "shared"
    pub device_id: String,
    pub device_name: String,
    pub output_format: AudioFormatData,
    pub decode_format: AudioFormatData,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioFormatData {
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub channels: u16,
    pub codec: Option<String>,
    pub container: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaybackErrorEvent {
    pub code: String,
    pub message: String,
    pub track_id: Option<i64>,
    pub recoverable: bool,
    pub action: Option<String>, // "switch_to_default"
}

// -----------------
// Command payloads
// -----------------

#[derive(Debug, Clone, Serialize)]
pub struct AudioDeviceInfoResponse {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

// -----------------
// Commands
// -----------------

#[tauri::command]
pub fn cmd_playback_start(
    audio_state: State<'_, AudioState>,
    library_state: State<'_, LibraryState>,
    track_id: i64,
) -> Result<(), String> {
    // Validate/resolve track before starting.
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;
    let track = get_track_by_id(&conn, track_id).map_err(|e| e.to_string())?;

    if track.is_missing {
        return Err("track is missing".to_string());
    }

    audio_state
        .command_tx
        .send(PlaybackCommand::PlayNow { track_id })
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn cmd_playback_pause(audio_state: State<'_, AudioState>) -> Result<(), String> {
    audio_state
        .command_tx
        .send(PlaybackCommand::Pause)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn cmd_playback_resume(audio_state: State<'_, AudioState>) -> Result<(), String> {
    audio_state
        .command_tx
        .send(PlaybackCommand::Resume)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn cmd_playback_stop(audio_state: State<'_, AudioState>) -> Result<(), String> {
    audio_state
        .command_tx
        .send(PlaybackCommand::Stop)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn cmd_playback_seek(
    audio_state: State<'_, AudioState>,
    position_ms: u64,
) -> Result<(), String> {
    audio_state
        .command_tx
        .send(PlaybackCommand::Seek { position_ms })
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn cmd_playback_next(audio_state: State<'_, AudioState>) -> Result<(), String> {
    audio_state
        .command_tx
        .send(PlaybackCommand::Next)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn cmd_playback_previous(audio_state: State<'_, AudioState>) -> Result<(), String> {
    audio_state
        .command_tx
        .send(PlaybackCommand::Previous)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn cmd_queue_play_now(
    audio_state: State<'_, AudioState>,
    library_state: State<'_, LibraryState>,
    track_id: i64,
) -> Result<(), String> {
    // Same validation as playback start.
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;
    let track = get_track_by_id(&conn, track_id).map_err(|e| e.to_string())?;

    if track.is_missing {
        return Err("track is missing".to_string());
    }

    audio_state
        .command_tx
        .send(PlaybackCommand::PlayNow { track_id })
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn cmd_queue_add(
    audio_state: State<'_, AudioState>,
    library_state: State<'_, LibraryState>,
    track_id: i64,
) -> Result<(), String> {
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;
    let track = get_track_by_id(&conn, track_id).map_err(|e| e.to_string())?;

    if track.is_missing {
        return Err("track is missing".to_string());
    }

    audio_state
        .command_tx
        .send(PlaybackCommand::AddToQueue { track_id })
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn cmd_output_list_devices() -> Result<Vec<AudioDeviceInfoResponse>, String> {
    let devices = list_devices().map_err(|e| e.to_string())?;
    Ok(devices
        .into_iter()
        .map(|d| AudioDeviceInfoResponse {
            id: d.id,
            name: d.name,
            is_default: d.is_default,
        })
        .collect())
}

#[tauri::command]
pub fn cmd_output_set_device(
    audio_state: State<'_, AudioState>,
    device_id: String,
) -> Result<(), String> {
    audio_state
        .command_tx
        .send(PlaybackCommand::SetDevice { device_id })
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn cmd_volume_get(audio_state: State<'_, AudioState>) -> Result<f32, String> {
    Ok(audio_state.engine.lock().volume())
}

#[tauri::command]
pub fn cmd_volume_set(audio_state: State<'_, AudioState>, volume: f32) -> Result<(), String> {
    audio_state
        .command_tx
        .send(PlaybackCommand::SetVolume { volume })
        .map_err(|e| e.to_string())?;
    Ok(())
}
