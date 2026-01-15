mod commands;
mod state;

use commands::{
    cmd_library_add_folder, cmd_library_list_folders, cmd_library_list_tracks, cmd_scan_start,
};
use state::LibraryState;
use std::fs;
use tauri::{Listener, Manager};
use tracing::{Level, debug, info};
use tracing_subscriber::{
    Layer, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

pub fn init_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    // 1. Determine Log Level
    let is_debug = std::env::var("SERMON_DEBUG")
        .map(|v| v == "1")
        .unwrap_or(false);
    let level = if is_debug { Level::DEBUG } else { Level::INFO };

    // 2. Determine Log Path
    // Resolved from the repo root (std::env::current_dir)
    let cwd = std::env::current_dir().unwrap_or_default();
    let log_dir = cwd.join("artifacts").join("logs");

    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory {:?}: {}", log_dir, e);
    }

    // 3. Setup Tracing
    let file_appender = tracing_appender::rolling::never(&log_dir, "sermon.log");
    // We must keep the guard alive.
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_filter(LevelFilter::from_level(level));

    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_filter(LevelFilter::from_level(level));

    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .init();

    if is_debug {
        debug!("Debug mode enabled. Log level: DEBUG");
        debug!("Log directory: {:?}", log_dir);
    }

    guard
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _guard = init_tracing();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Register event listener
            app.listen("sermon://first-interactive", |_event| {
                info!("first_interactive");
            });

            // Resolve DB path
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            // Ensure directory exists
            fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

            let db_path = app_data_dir.join("library.db");
            info!("Database path: {:?}", db_path);

            // Initialize database
            let conn = library::open_db(&db_path).expect("Failed to open database");
            library::apply_migrations(&conn).expect("Failed to apply migrations");
            drop(conn);

            // Register state
            app.manage(LibraryState::new(db_path));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd_library_add_folder,
            cmd_library_list_folders,
            cmd_library_list_tracks,
            cmd_scan_start,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
