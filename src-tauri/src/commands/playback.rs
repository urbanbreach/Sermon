use crate::state::{AudioState, LibraryState, PlaybackCommand};
use audio_engine::device::list_devices;
use library::{
    get_audio_output_asio_driver, get_audio_output_fade, get_audio_output_mode,
    get_audio_output_policy, get_audio_output_timing, get_track_by_id, open_db, set_missing,
    set_setting,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::{Emitter, State};

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
    pub dsd_rate_hz: Option<u32>,
    pub dsd_channels: Option<u16>,
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
    pub output_mode: String, // "exclusive" | "shared" | "asio"
    pub policy: String,      // "strict" | "compatibility"
    pub conversion: String,  // "none" | "shared_fallback" | "pad_16_to_24" | "pad_16_to_32"
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
    pub is_dsd: bool,
    pub dsd_rate_hz: Option<u32>,
    pub dop_rate_hz: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaybackErrorEvent {
    pub code: String,
    pub message: String,
    pub track_id: Option<i64>,
    pub recoverable: bool,
    pub action: Option<String>, // "switch_to_default"
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackMarkedMissingEvent {
    pub track_id: i64,
    pub path: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsioDriverInfo {
    pub name: String,
}

#[tauri::command]
#[cfg(windows)]
pub fn cmd_list_asio_drivers() -> Vec<AsioDriverInfo> {
    use audio_engine::asio_device::list_asio_drivers;
    list_asio_drivers()
        .into_iter()
        .map(|d| AsioDriverInfo { name: d.name })
        .collect()
}

#[tauri::command]
#[cfg(not(windows))]
pub fn cmd_list_asio_drivers() -> Vec<AsioDriverInfo> {
    Vec::new() // ASIO not available on non-Windows
}

#[tauri::command]
#[cfg(windows)]
pub fn cmd_open_asio_control_panel(
    audio_state: State<'_, AudioState>,
    driver_name: String,
) -> Result<(), String> {
    audio_state
        .command_tx
        .send(PlaybackCommand::OpenAsioControlPanel { driver_name })
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn cmd_output_probe_capabilities(
    audio_state: State<'_, AudioState>,
    backend: String,
    device_id: Option<String>,
    asio_driver: Option<String>,
    channels: u16,
    sample_rates: Vec<u32>,
    bit_depths: Vec<u16>,
) -> Result<ProbeCapabilitiesResult, String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let probed_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let key = ProbeCapabilitiesKey {
        backend: backend.clone(),
        device_id: device_id.clone(),
        asio_driver: asio_driver.clone(),
        channels,
    };

    let dimensions = ProbeDimensions {
        sample_rates: sample_rates.clone(),
        bit_depths: bit_depths.clone(),
        channels: vec![channels],
    };

    let mut cells = Vec::new();

    match backend.as_str() {
        "wasapi" => {
            cells = probe_wasapi_capabilities(
                device_id.as_deref(),
                channels,
                &sample_rates,
                &bit_depths,
            )?;
        }
        "asio" => {
            let driver_name = asio_driver
                .as_ref()
                .ok_or("asio_driver required for ASIO probing")?;

            let is_stopped = {
                let engine = audio_state.engine.lock();
                engine.state == audio_engine::PlaybackState::Stopped
            };

            if !is_stopped {
                return Err("probe_not_allowed_during_playback".to_string());
            }

            cells = probe_asio_capabilities(driver_name, channels, &sample_rates, &bit_depths)?;
        }
        _ => {
            return Err(format!("Unknown backend: {}", backend));
        }
    }

    Ok(ProbeCapabilitiesResult {
        version: 1,
        key,
        probed_at_ms,
        dimensions,
        cells,
    })
}

#[cfg(windows)]
fn probe_wasapi_capabilities(
    device_id: Option<&str>,
    channels: u16,
    sample_rates: &[u32],
    bit_depths: &[u16],
) -> Result<Vec<ProbeCell>, String> {
    use audio_engine::device::{get_default_device, get_device_by_id};
    use wasapi::{SampleType, WaveFormat};

    let device = match device_id {
        Some("default") | None => get_default_device().map_err(|e| e.to_string())?,
        Some(id) => get_device_by_id(id).map_err(|e| e.to_string())?,
    };

    let channel_mask = if channels == 2 {
        Some(0x3)
    } else if channels == 1 {
        Some(0x4)
    } else {
        None
    };

    let mut cells = Vec::new();

    for &sample_rate in sample_rates {
        for &bit_depth in bit_depths {
            let format_attempts: Vec<(usize, usize, SampleType)> = match bit_depth {
                24 => vec![(32, 24, SampleType::Int), (24, 24, SampleType::Int)],
                16 => vec![(16, 16, SampleType::Int)],
                32 => vec![(32, 32, SampleType::Float), (32, 32, SampleType::Int)],
                _ => vec![(bit_depth as usize, bit_depth as usize, SampleType::Int)],
            };

            let mut supported = false;
            let mut reason_code = "unsupported".to_string();
            let mut detail: Option<String> = None;

            for (store_bits, valid_bits, sample_type) in &format_attempts {
                let wave_format = WaveFormat::new(
                    *store_bits,
                    *valid_bits,
                    sample_type,
                    sample_rate as usize,
                    channels as usize,
                    channel_mask,
                );

                let client = match device.get_iaudioclient() {
                    Ok(c) => c,
                    Err(e) => {
                        reason_code = "device_error".to_string();
                        detail = Some(e.to_string());
                        break;
                    }
                };

                match client.is_supported_exclusive_with_quirks(&wave_format) {
                    Ok(_) => {
                        supported = true;
                        reason_code = "ok".to_string();
                        detail = Some(format!(
                            "{}bit in {}bit container, {:?}",
                            valid_bits, store_bits, sample_type
                        ));
                        break;
                    }
                    Err(e) => {
                        detail = Some(e.to_string());
                    }
                }
            }

            cells.push(ProbeCell {
                sample_rate,
                bit_depth,
                channels,
                supported,
                reason_code,
                detail,
            });
        }
    }

    Ok(cells)
}

#[cfg(not(windows))]
fn probe_wasapi_capabilities(
    _device_id: Option<&str>,
    _channels: u16,
    _sample_rates: &[u32],
    _bit_depths: &[u16],
) -> Result<Vec<ProbeCell>, String> {
    Err("WASAPI is only available on Windows".to_string())
}

#[cfg(windows)]
fn probe_asio_capabilities(
    driver_name: &str,
    channels: u16,
    sample_rates: &[u32],
    _bit_depths: &[u16],
) -> Result<Vec<ProbeCell>, String> {
    use asio_sys::Asio;

    let asio = Asio::new();
    let driver = asio
        .load_driver(driver_name)
        .map_err(|e| format!("Failed to load ASIO driver: {:?}", e))?;

    let mut cells = Vec::new();

    for &sample_rate in sample_rates {
        let mut supported = false;
        let mut reason_code = "unsupported".to_string();
        let mut detail: Option<String> = None;

        if let Err(e) = driver.set_sample_rate(sample_rate as f64) {
            reason_code = "sample_rate_not_supported".to_string();
            detail = Some(format!("{:?}", e));
        } else {
            match driver.sample_rate() {
                Ok(actual_rate) => {
                    if (actual_rate as u32) == sample_rate {
                        supported = true;
                        reason_code = "ok".to_string();
                        detail = Some(format!("Driver accepted {}Hz", sample_rate));
                    } else {
                        reason_code = "sample_rate_mismatch".to_string();
                        detail = Some(format!(
                            "Requested {}Hz, driver set {}Hz",
                            sample_rate, actual_rate as u32
                        ));
                    }
                }
                Err(e) => {
                    reason_code = "sample_rate_query_failed".to_string();
                    detail = Some(format!("{:?}", e));
                }
            }
        }

        for &bit_depth in &[16u16, 24, 32] {
            cells.push(ProbeCell {
                sample_rate,
                bit_depth,
                channels,
                supported,
                reason_code: reason_code.clone(),
                detail: detail.clone(),
            });
        }
    }

    Ok(cells)
}

#[cfg(not(windows))]
fn probe_asio_capabilities(
    _driver_name: &str,
    _channels: u16,
    _sample_rates: &[u32],
    _bit_depths: &[u16],
) -> Result<Vec<ProbeCell>, String> {
    Err("ASIO is only available on Windows".to_string())
}

// -----------------
// Commands
// -----------------

#[tauri::command]
pub fn cmd_playback_start(
    app: tauri::AppHandle,
    audio_state: State<'_, AudioState>,
    library_state: State<'_, LibraryState>,
    track_id: i64,
) -> Result<(), String> {
    // Validate/resolve track before starting.
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;
    let track = get_track_by_id(&conn, track_id).map_err(|e| e.to_string())?;

    if track.is_missing {
        return Err("track is missing".to_string());
    }

    // Check if file actually exists on disk
    if !Path::new(&track.path).exists() {
        // Mark as missing in database
        set_missing(&conn, track_id, true).map_err(|e| e.to_string())?;

        // Emit event so UI can refresh
        let _ = app.emit(
            "evt_track_marked_missing",
            TrackMarkedMissingEvent {
                track_id,
                path: track.path.clone(),
            },
        );

        return Err(format!("File not found: {}", track.path));
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
    app: tauri::AppHandle,
    audio_state: State<'_, AudioState>,
    library_state: State<'_, LibraryState>,
    track_id: i64,
) -> Result<(), String> {
    // Same validation as playback start.
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;
    let track = get_track_by_id(&conn, track_id).map_err(|e| e.to_string())?;

    if track.is_missing {
        return Err("track is missing".to_string());
    }

    // Check if file actually exists on disk
    if !Path::new(&track.path).exists() {
        // Mark as missing in database
        set_missing(&conn, track_id, true).map_err(|e| e.to_string())?;

        // Emit event so UI can refresh
        let _ = app.emit(
            "evt_track_marked_missing",
            TrackMarkedMissingEvent {
                track_id,
                path: track.path.clone(),
            },
        );

        return Err(format!("File not found: {}", track.path));
    }

    audio_state
        .command_tx
        .send(PlaybackCommand::PlayNow { track_id })
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn cmd_queue_add(
    app: tauri::AppHandle,
    audio_state: State<'_, AudioState>,
    library_state: State<'_, LibraryState>,
    track_id: i64,
) -> Result<(), String> {
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;
    let track = get_track_by_id(&conn, track_id).map_err(|e| e.to_string())?;

    if track.is_missing {
        return Err("track is missing".to_string());
    }

    // Check if file actually exists on disk
    if !Path::new(&track.path).exists() {
        // Mark as missing in database
        set_missing(&conn, track_id, true).map_err(|e| e.to_string())?;

        // Emit event so UI can refresh
        let _ = app.emit(
            "evt_track_marked_missing",
            TrackMarkedMissingEvent {
                track_id,
                path: track.path.clone(),
            },
        );

        return Err(format!("File not found: {}", track.path));
    }

    audio_state
        .command_tx
        .send(PlaybackCommand::AddToQueue { track_id })
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn cmd_queue_set_and_play(
    app: tauri::AppHandle,
    audio_state: State<'_, AudioState>,
    library_state: State<'_, LibraryState>,
    track_ids: Vec<i64>,
    start_index: usize,
) -> Result<(), String> {
    if track_ids.is_empty() {
        return Err("No tracks provided".to_string());
    }

    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;

    // Validate at least the starting track exists and is not missing
    let start_track_id = track_ids.get(start_index).ok_or("Invalid start index")?;
    let track = get_track_by_id(&conn, *start_track_id).map_err(|e| e.to_string())?;

    if track.is_missing {
        return Err("Starting track is missing".to_string());
    }

    if !Path::new(&track.path).exists() {
        set_missing(&conn, *start_track_id, true).map_err(|e| e.to_string())?;
        let _ = app.emit(
            "evt_track_marked_missing",
            TrackMarkedMissingEvent {
                track_id: *start_track_id,
                path: track.path.clone(),
            },
        );
        return Err(format!("File not found: {}", track.path));
    }

    audio_state
        .command_tx
        .send(PlaybackCommand::PlayNowWithQueue {
            track_ids,
            start_index,
        })
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
#[serde(rename_all = "camelCase")]
pub struct AudioOutputSettings {
    pub mode: String,   // "exclusive" | "shared" | "asio"
    pub policy: String, // "strict" | "compatibility"
    pub fade: bool,
    pub timing: String,              // "event" | "polling"
    pub asio_driver: Option<String>, // ASIO driver name when mode is "asio"
}

// -----------------
// Capability Probing
// -----------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeCapabilitiesKey {
    pub backend: String,
    pub device_id: Option<String>,
    pub asio_driver: Option<String>,
    pub channels: u16,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeDimensions {
    pub sample_rates: Vec<u32>,
    pub bit_depths: Vec<u16>,
    pub channels: Vec<u16>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeCell {
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub channels: u16,
    pub supported: bool,
    pub reason_code: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeCapabilitiesResult {
    pub version: u32,
    pub key: ProbeCapabilitiesKey,
    pub probed_at_ms: u64,
    pub dimensions: ProbeDimensions,
    pub cells: Vec<ProbeCell>,
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
        asio_driver: get_audio_output_asio_driver(&conn),
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
    if let Some(ref driver) = settings.asio_driver {
        set_setting(&conn, "audio.output.asio_driver", driver).map_err(|e| e.to_string())?;
    }

    // 2. Notify audio thread
    audio_state
        .command_tx
        .send(PlaybackCommand::SetOutputSettings {
            mode: settings.mode,
            policy: settings.policy,
            fade: settings.fade,
            timing: settings.timing,
            asio_driver: settings.asio_driver,
        })
        .map_err(|e| e.to_string())?;

    Ok(())
}
