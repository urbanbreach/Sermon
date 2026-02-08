use crate::state::{AudioState, DiagnosticsState, LibraryState, PlaybackCommand};
use chrono::Utc;
use library::db::{get_setting, set_setting};
use library::open_db;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use sysinfo::{Pid, System};
use tauri::State;

/// Valid category names
const VALID_CATEGORIES: [&str; 7] = [
    "general",
    "player",
    "nowplaying",
    "library",
    "tags",
    "internet",
    "devices",
];

/// Get setting keys for a category
fn get_category_keys(category: &str) -> Vec<&'static str> {
    match category {
        "general" => vec!["general.startup.with_windows", "general.startup.minimized"],
        "player" => vec![
            "player.buffer_size_ms",
            "player.preload_next",
            "player.load_to_memory",
            "player.resume_on_startup",
        ],
        "nowplaying" => vec![
            "nowplaying.double_click",
            "nowplaying.queue_add_position",
            "nowplaying.shuffle_mode",
        ],
        "library" => vec!["library.scan_on_startup", "library.continuous_monitoring"],
        "tags" => vec!["tags.backup_before_write", "tags.write_behavior"],
        "internet" => vec!["internet.lastfm_enabled"],
        "devices" => vec!["devices.dsd_dop_enabled", "devices.dsd_dop_strict"],
        _ => vec![],
    }
}

/// Get default values for a category
fn get_category_defaults(category: &str) -> HashMap<&'static str, &'static str> {
    match category {
        "general" => HashMap::from([
            ("general.startup.with_windows", "off"),
            ("general.startup.minimized", "off"),
        ]),
        "player" => HashMap::from([
            ("player.buffer_size_ms", "500"),
            ("player.preload_next", "on"),
            ("player.load_to_memory", "on"),
            ("player.resume_on_startup", "off"),
        ]),
        "nowplaying" => HashMap::from([
            ("nowplaying.double_click", "play_now"),
            ("nowplaying.queue_add_position", "end"),
            ("nowplaying.shuffle_mode", "off"),
        ]),
        "library" => HashMap::from([
            ("library.scan_on_startup", "on"),
            ("library.continuous_monitoring", "off"),
        ]),
        "tags" => HashMap::from([
            ("tags.backup_before_write", "on"),
            ("tags.write_behavior", "prompt"),
        ]),
        "internet" => HashMap::from([("internet.lastfm_enabled", "off")]),
        "devices" => HashMap::from([
            ("devices.dsd_dop_enabled", "off"),
            ("devices.dsd_dop_strict", "on"),
        ]),
        _ => HashMap::new(),
    }
}

#[tauri::command]
pub async fn cmd_settings_get_category(
    category: String,
    state: State<'_, LibraryState>,
) -> Result<Value, String> {
    // Validate category
    if !VALID_CATEGORIES.contains(&category.as_str()) {
        return Err(format!(
            "Invalid category: {}. Valid: general, player, nowplaying, library, tags, internet, devices",
            category
        ));
    }

    let db_path = state.db_path.clone();
    let cat = category.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let conn = open_db(&db_path).map_err(|e| e.to_string())?;

        let keys = get_category_keys(&cat);
        let defaults = get_category_defaults(&cat);
        let mut result = serde_json::Map::new();

        for key in keys {
            let value = get_setting(&conn, key)
                .map_err(|e| e.to_string())?
                .unwrap_or_else(|| defaults.get(key).unwrap_or(&"").to_string());
            result.insert(key.to_string(), Value::String(value));
        }

        Ok(Value::Object(result))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn cmd_settings_set_category(
    category: String,
    settings: Value,
    audio_state: State<'_, AudioState>,
    state: State<'_, LibraryState>,
) -> Result<(), String> {
    // Validate category
    if !VALID_CATEGORIES.contains(&category.as_str()) {
        return Err(format!(
            "Invalid category: {}. Valid: general, player, nowplaying, library, tags, internet, devices",
            category
        ));
    }

    let db_path = state.db_path.clone();
    let command_tx = audio_state.command_tx.clone();
    let category_name = category.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let conn = open_db(&db_path).map_err(|e| e.to_string())?;

        if let Value::Object(map) = settings {
            for (key, value) in map {
                if let Value::String(v) = value {
                    set_setting(&conn, &key, &v).map_err(|e| e.to_string())?;
                }
            }
        }

        if category_name == "player" {
            let buffer_size_ms = get_setting(&conn, "player.buffer_size_ms")
                .map_err(|e| e.to_string())?
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(500)
                .clamp(100, 2000);
            let load_to_memory = get_setting(&conn, "player.load_to_memory")
                .map_err(|e| e.to_string())?
                .map(|v| v != "off")
                .unwrap_or(true);
            let preload_next = get_setting(&conn, "player.preload_next")
                .map_err(|e| e.to_string())?
                .map(|v| v != "off")
                .unwrap_or(true);

            command_tx
                .send(PlaybackCommand::SetPlayerSettings {
                    buffer_size_ms,
                    load_to_memory,
                    preload_next,
                })
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn cmd_settings_reset_category(
    category: String,
    state: State<'_, LibraryState>,
) -> Result<(), String> {
    // Validate category
    if !VALID_CATEGORIES.contains(&category.as_str()) {
        return Err(format!(
            "Invalid category: {}. Valid: general, player, nowplaying, library, tags, internet, devices",
            category
        ));
    }

    let db_path = state.db_path.clone();
    let cat = category.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let conn = open_db(&db_path).map_err(|e| e.to_string())?;

        let defaults = get_category_defaults(&cat);
        for (key, value) in defaults {
            set_setting(&conn, key, value).map_err(|e| e.to_string())?;
        }

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Export diagnostics JSON for debugging and support
#[tauri::command]
pub async fn cmd_settings_export_diagnostics(
    state: State<'_, LibraryState>,
    diagnostics_state: State<'_, Arc<DiagnosticsState>>,
) -> Result<String, String> {
    let db_path = state.db_path.clone();
    let diagnostics = diagnostics_state.inner().clone();

    tauri::async_runtime::spawn_blocking(move || {
        let conn = open_db(&db_path).map_err(|e| e.to_string())?;

        // Collect all settings
        let mut settings = serde_json::Map::new();

        // Get all category settings
        for category in VALID_CATEGORIES {
            let keys = get_category_keys(category);
            let defaults = get_category_defaults(category);
            for key in keys {
                let value = get_setting(&conn, key)
                    .map_err(|e| e.to_string())?
                    .unwrap_or_else(|| defaults.get(key).unwrap_or(&"").to_string());
                settings.insert(key.to_string(), Value::String(value));
            }
        }

        // Get audio settings
        let audio_device = get_setting(&conn, "audio.device.preference")
            .ok()
            .flatten()
            .unwrap_or_else(|| "default".to_string());
        let output_mode = get_setting(&conn, "audio.output.mode")
            .ok()
            .flatten()
            .unwrap_or_else(|| "exclusive".to_string());

        // Get process metrics via sysinfo
        let pid = Pid::from_u32(std::process::id());
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
        
        let (cpu_pct, rss_bytes) = sys.process(pid)
            .map(|p| (p.cpu_usage() as f64, p.memory()))
            .unwrap_or((0.0, 0));

        // Build metrics from DiagnosticsState
        let startup_ms = diagnostics.startup_duration_ms();
        let playback_start_ms = diagnostics.playback_start_duration_ms();
        let seek_ms = diagnostics.seek_duration_ms();
        let underruns = diagnostics.underruns();

        // Build diagnostics JSON
        let diagnostics_json = json!({
            "version": "0.1.0",
            "timestamp": Utc::now().to_rfc3339(),
            "system": {
                "os": std::env::consts::OS,
                "tauri_version": tauri::VERSION,
                "app_version": "0.1.0"
            },
            "audio": {
                "current_device": audio_device,
                "output_mode": output_mode,
                "wasapi_available": true,
                "underruns": underruns
            },
            "metrics": {
                "startup_ms": startup_ms,
                "playback_start_ms": playback_start_ms,
                "seek_ms": seek_ms
            },
            "process": {
                "cpu_pct": cpu_pct,
                "rss_bytes": rss_bytes
            },
            "settings": Value::Object(settings)
        });

        serde_json::to_string_pretty(&diagnostics_json).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
