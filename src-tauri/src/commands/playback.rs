use crate::state::{AudioState, LibraryState, PlaybackCommand};
use audio_engine::device::list_devices;
use library::{
    apply_migrations, get_audio_output_fade, get_audio_output_mode, get_audio_output_policy,
    get_audio_output_timing, get_track_by_id, open_db, set_setting,
};
use serde::{Deserialize, Serialize};
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
    pub output_mode: String, // "exclusive" | "shared"
    pub policy: String,      // "strict" | "compatibility"
    pub conversion: String,  // "none" | "shared_fallback" | "pad_16_to_24"
    pub gain_mode: String,   // "unity" | "software"
    pub fade_enabled: bool,
    pub exclusive_active: bool,
    pub bit_perfect: String,        // "yes" | "no"
    pub bit_perfect_reason: String, // "" if yes, or specific reason string
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
    pub valid_bits: Option<u16>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioOutputSettings {
    pub mode: String,   // "exclusive" | "shared"
    pub policy: String, // "strict" | "compatibility"
    pub fade: bool,
    pub timing: String, // "event" | "polling"
}

#[tauri::command]
pub fn cmd_output_get_settings(
    library_state: State<'_, LibraryState>,
) -> Result<AudioOutputSettings, String> {
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;

    Ok(AudioOutputSettings {
        mode: get_audio_output_mode(&conn),
        policy: get_audio_output_policy(&conn),
        fade: get_audio_output_fade(&conn),
        timing: get_audio_output_timing(&conn),
    })
}

#[tauri::command]
pub fn cmd_output_set_settings(
    audio_state: State<'_, AudioState>,
    library_state: State<'_, LibraryState>,
    settings: AudioOutputSettings,
) -> Result<(), String> {
    // 1. Persist to DB
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;
    set_setting(&conn, "audio.output.mode", &settings.mode).map_err(|e| e.to_string())?;
    set_setting(&conn, "audio.output.policy", &settings.policy).map_err(|e| e.to_string())?;
    set_setting(
        &conn,
        "audio.output.fade",
        if settings.fade { "on" } else { "off" },
    )
    .map_err(|e| e.to_string())?;
    set_setting(&conn, "audio.output.timing", &settings.timing).map_err(|e| e.to_string())?;

    // 2. Notify audio thread
    audio_state
        .command_tx
        .send(PlaybackCommand::SetOutputSettings {
            mode: settings.mode,
            policy: settings.policy,
            fade: settings.fade,
            timing: settings.timing,
        })
        .map_err(|e| e.to_string())?;

    Ok(())
}
