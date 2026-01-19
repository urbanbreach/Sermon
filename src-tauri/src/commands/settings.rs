use crate::state::LibraryState;
use chrono::Utc;
use library::db::{get_setting, set_setting};
use library::{apply_migrations, open_db};
use serde_json::{Value, json};
use std::collections::HashMap;
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
        "player" => vec!["player.buffer_size_ms", "player.preload_next"],
        "nowplaying" => vec![
            "nowplaying.double_click",
            "nowplaying.queue_add_position",
            "nowplaying.shuffle_mode",
        ],
        "library" => vec!["library.scan_on_startup", "library.continuous_monitoring"],
        "tags" => vec!["tags.backup_before_write", "tags.write_behavior"],
        "internet" => vec!["internet.lastfm_enabled"],
        "devices" => vec!["devices.dsd_dop_enabled"],
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
        "devices" => HashMap::from([("devices.dsd_dop_enabled", "off")]),
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
        apply_migrations(&conn).map_err(|e| e.to_string())?;

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

    tauri::async_runtime::spawn_blocking(move || {
        let conn = open_db(&db_path).map_err(|e| e.to_string())?;
        apply_migrations(&conn).map_err(|e| e.to_string())?;

        if let Value::Object(map) = settings {
            for (key, value) in map {
                if let Value::String(v) = value {
                    set_setting(&conn, &key, &v).map_err(|e| e.to_string())?;
                }
            }
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
        apply_migrations(&conn).map_err(|e| e.to_string())?;

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
) -> Result<String, String> {
    let db_path = state.db_path.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let conn = open_db(&db_path).map_err(|e| e.to_string())?;
        apply_migrations(&conn).map_err(|e| e.to_string())?;

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

        // Build diagnostics JSON
        let diagnostics = json!({
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
                "wasapi_available": true
            },
            "settings": Value::Object(settings)
        });

        serde_json::to_string_pretty(&diagnostics).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
