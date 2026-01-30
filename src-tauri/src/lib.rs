mod commands;
mod state;

use serde::Serialize;
use audio_engine::decode::AudioDecoder;
use audio_engine::device::{get_default_device, get_device_by_id};
use audio_engine::gapless_decoder::{AudioFormat, GaplessDecoder, TransitionType};
use audio_engine::output::{
    convert_channels_interleaved_f32, AudioOutput, AudioRingBuffer, OutputBackend, WasapiOutput,
};
use audio_engine::{PlaybackState, TrackInfo};
use commands::{
    cmd_artwork_embed_to_file, cmd_artwork_extract_embedded, cmd_artwork_find_folder,
    cmd_artwork_get_best_for_album, cmd_artwork_get_best_for_track, cmd_artwork_get_bytes,
    cmd_artwork_search_candidates, cmd_artwork_select_candidate_for_album, cmd_library_add_folder,
    cmd_library_get_folder_track_count, cmd_library_get_raw_tags, cmd_library_get_stats,
    cmd_library_get_track_by_id, cmd_library_list_album_tracks_page, cmd_library_list_albums_page,
    cmd_library_list_artist_tracks_page, cmd_library_list_artists_page, cmd_library_list_folders,
    cmd_library_list_tracks, cmd_library_list_tracks_page, cmd_library_remove_folder,
    cmd_library_search_albums_page, cmd_library_search_artists_page, cmd_library_search_suggest,
    cmd_library_search_tracks_page, cmd_library_update_folder_enabled,
    cmd_library_update_folder_options, cmd_library_update_track_tags, cmd_list_asio_drivers,
    cmd_open_asio_control_panel, cmd_output_get_settings, cmd_output_list_devices,
    cmd_output_probe_capabilities, cmd_output_set_device, cmd_output_set_settings,
    cmd_playback_next, cmd_playback_pause, cmd_playback_previous, cmd_playback_resume,
    cmd_playback_seek, cmd_playback_start, cmd_playback_stop, cmd_queue_add, cmd_queue_play_now,
    cmd_queue_set_and_play, cmd_scan_start, cmd_settings_export_diagnostics, cmd_settings_get,
    cmd_settings_get_category, cmd_settings_reset_category, cmd_settings_set,
    cmd_settings_set_category, cmd_volume_get, cmd_volume_set, cmd_waveform_get_peaks,
    AudioDebugEvent, AudioFormatData, DeviceChangedEvent, NowPlayingEvent, PlaybackErrorEvent,
    PlaybackPositionEvent, PlaybackStateEvent, QueueChangedEvent, QueueItemData, TrackEventData,
};
use crossbeam_channel::{select, tick, unbounded, Receiver, Sender};
use parking_lot::Mutex;
use state::{ArtworkCacheState, AudioState, DiagnosticsState, LibraryState, PlaybackCommand, WaveformCacheState};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Listener, Manager};
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::{
    filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

pub fn init_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    // 1. Determine Log Level
    let is_debug = std::env::var("SERMON_DEBUG")
        .map(|v| v == "1")
        .unwrap_or(false);
    let level = if is_debug { Level::DEBUG } else { Level::INFO };

    // 2. Determine Log Path
    // Use temp directory to avoid triggering Tauri dev watcher
    let log_dir = std::env::temp_dir().join("sermon-logs");

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
    // Record startup start time
    let startup_start_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let _guard = init_tracing();

    // Initialize diagnostics state with startup timestamp
    let diagnostics = Arc::new(DiagnosticsState::new());
    diagnostics.startup_start_ms.store(startup_start_ms, std::sync::atomic::Ordering::SeqCst);

    let mut builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    let diagnostics_for_listener = diagnostics.clone();
    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            // Manage diagnostics state for Tauri commands
            app.manage(diagnostics.clone());

            // Register event listener for first-interactive timing
            let diag = diagnostics_for_listener.clone();
            app.listen("sermon://first-interactive", move |_event| {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                diag.startup_complete_ms.store(now_ms, std::sync::atomic::Ordering::SeqCst);
                info!("first_interactive (startup_ms={})", now_ms.saturating_sub(
                    diag.startup_start_ms.load(std::sync::atomic::Ordering::SeqCst)
                ));
            });

            // Resolve DB path
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|_| "Failed to get app data directory")?;

            // Ensure directory exists
            fs::create_dir_all(&app_data_dir)?;

            let db_path = app_data_dir.join("library.db");
            info!("Database path: {:?}", db_path);

            // Setup artwork cache directory
            let artwork_cache_dir = app_data_dir.join("artwork-cache");
            fs::create_dir_all(&artwork_cache_dir)?;
            info!("Artwork cache path: {:?}", artwork_cache_dir);
            app.manage(ArtworkCacheState::new(artwork_cache_dir));

            // Setup waveform cache directory
            let waveform_cache_dir = app_data_dir.join("waveform-cache");
            fs::create_dir_all(&waveform_cache_dir)?;
            info!("Waveform cache path: {:?}", waveform_cache_dir);
            app.manage(WaveformCacheState::new(waveform_cache_dir));

            let library_state = LibraryState::new(db_path.clone());

            // Apply migrations once at startup (flag gates all subsequent calls)
            let conn = library::open_db(&db_path)?;
            library::apply_migrations(&conn)?;
            library_state
                .migrations_applied
                .store(true, std::sync::atomic::Ordering::SeqCst);
            drop(conn);

            app.manage(library_state);

            // Run quick scan on startup to detect added/removed files
            // Use a small delay to ensure the app is fully initialized
            let db_path_clone = db_path.clone();
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                // Small delay to let the app fully initialize
                std::thread::sleep(std::time::Duration::from_millis(500));

                // Check if scan_on_startup is enabled (default: true if setting missing)
                let scan_enabled = library::open_db(&db_path_clone)
                    .ok()
                    .and_then(|conn| library::db::get_setting(&conn, "library.scan_on_startup").ok())
                    .flatten()
                    .map(|v| v == "on")
                    .unwrap_or(true); // Default: scan if setting missing

                if !scan_enabled {
                    info!("Startup scan disabled by user preference");
                    return;
                }

                match library::quick_scan(&db_path_clone) {
                    Ok(summary) => {
                        info!(
                            "Startup quick scan complete: {} folders, {} files checked, {} added, {} missing, {} restored in {}ms",
                            summary.folders_checked,
                            summary.files_checked,
                            summary.files_added,
                            summary.files_marked_missing,
                            summary.files_restored,
                            summary.elapsed_ms
                        );

                        // Emit event if any changes were made
                        if summary.files_added > 0 || summary.files_marked_missing > 0 || summary.files_restored > 0 {
                            let _ = app_handle.emit("evt_quick_scan_complete", &summary);
                        }
                    }
                    Err(e) => {
                        warn!("Startup quick scan failed: {}", e);
                    }
                }
            });

            // Audio state + command loop
            let engine = Arc::new(Mutex::new(audio_engine::EngineState::new()));
            let (command_tx, command_rx) = unbounded::<PlaybackCommand>();
            app.manage(AudioState {
                engine: engine.clone(),
                command_tx,
            });

            if let Ok(conn) = library::open_db(&db_path) {
                let buffer_size_ms = library::get_setting(&conn, "player.buffer_size_ms")
                    .ok()
                    .flatten()
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(500)
                    .clamp(100, 2000);
                let load_to_memory = library::get_setting(&conn, "player.load_to_memory")
                    .ok()
                    .flatten()
                    .map(|v| v != "off")
                    .unwrap_or(true);
                let preload_next = library::get_setting(&conn, "player.preload_next")
                    .ok()
                    .flatten()
                    .map(|v| v != "off")
                    .unwrap_or(true);

                let _ = app.state::<AudioState>().command_tx.send(
                    PlaybackCommand::SetPlayerSettings {
                        buffer_size_ms,
                        load_to_memory,
                        preload_next,
                    },
                );
            }

            spawn_audio_thread(app.handle().clone(), db_path, engine, command_rx, diagnostics.clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd_library_add_folder,
            cmd_library_list_folders,
            cmd_library_list_tracks,
            cmd_library_get_track_by_id,
            cmd_library_list_tracks_page,
            cmd_library_remove_folder,
            cmd_library_update_folder_enabled,
            cmd_library_update_folder_options,
            cmd_library_get_folder_track_count,

            cmd_library_list_albums_page,
            cmd_library_list_artists_page,
            cmd_library_list_album_tracks_page,
            cmd_library_list_artist_tracks_page,
            cmd_library_search_suggest,
            cmd_library_search_tracks_page,
            cmd_library_search_albums_page,
            cmd_library_search_artists_page,
            cmd_library_get_stats,
            cmd_library_update_track_tags,
            cmd_library_get_raw_tags,
            cmd_scan_start,
            cmd_playback_start,
            cmd_playback_pause,
            cmd_playback_resume,
            cmd_playback_stop,
            cmd_playback_seek,
            cmd_playback_next,
            cmd_playback_previous,
            cmd_queue_play_now,
            cmd_queue_add,
            cmd_queue_set_and_play,
            cmd_output_list_devices,
            cmd_output_probe_capabilities,
            cmd_list_asio_drivers,
            cmd_open_asio_control_panel,
            cmd_output_set_device,
            cmd_output_get_settings,
            cmd_output_set_settings,
            cmd_volume_get,
            cmd_volume_set,
            cmd_artwork_get_bytes,
            cmd_artwork_search_candidates,
            cmd_artwork_select_candidate_for_album,
            cmd_artwork_get_best_for_album,
            cmd_artwork_get_best_for_track,
            cmd_artwork_embed_to_file,
            cmd_artwork_extract_embedded,
            cmd_artwork_find_folder,
            cmd_waveform_get_peaks,
            cmd_settings_get,
            cmd_settings_set,
            cmd_settings_get_category,
            cmd_settings_set_category,
            cmd_settings_reset_category,
            cmd_settings_export_diagnostics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Audio playback state held by the audio thread
struct AudioPlayback {
    decoder: Option<AudioDecoder>,
    gapless_decoder: Option<GaplessDecoder>,
    preload_in_progress: bool,
    output: Option<OutputBackend>,
    ring_buffer: Option<AudioRingBuffer>,
    device_id: String, // "default" or specific device ID
    output_sample_rate: u32,
    output_channels: u16,
    end_of_track: bool,         // Track if decoder has finished
    current_track_id: Option<i64>, // Track ID currently loaded in decoder
    output_mode: String,        // "exclusive" or "shared" or "asio"
    policy: String,             // "strict" or "compatibility"
    gain_mode: String,          // "unity" or "software"
    conversion: Option<String>, // None, "pad_16_to_24", "shared_fallback"
    fade_enabled: bool,
    fade_state: Option<FadeState>,
    timing_mode: String, // "event" or "polling"
    buffer_size_ms: u32, // Ring buffer size in milliseconds (from preferences)
    load_to_memory: bool,
    preload_next: bool,
    // DSD playback state
    is_dsd_playback: bool,
    dsd_dop_enabled: bool,
    dsd_dop_strict: bool,
    dop_ring_buffer: Option<audio_engine::output::DopRingBuffer>,
    dsd_decoder: Option<audio_engine::DsdDecoder>,
    dop_packer: Option<audio_engine::DopPacker>,
    // ASIO state
    asio_driver: Option<String>,
    // Resampler for ASIO sample rate mismatch
    resampler: Option<audio_engine::Resampler>,
    source_sample_rate: u32,
}

struct PreloadRequest {
    path: PathBuf,
}

struct PreloadResult {
    path: PathBuf,
    bytes: Vec<u8>,
}

struct FadeState {
    direction: FadeDirection,
    samples_remaining: usize,
    total_samples: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum FadeDirection {
    In,
    Out,
}

impl AudioPlayback {
    fn new() -> Self {
        Self {
            decoder: None,
            gapless_decoder: None,
            preload_in_progress: false,
            output: None,
            ring_buffer: None,
            device_id: "default".to_string(),
            output_sample_rate: 0,
            output_channels: 0,
            end_of_track: false,
            current_track_id: None,
            output_mode: "shared".to_string(),
            policy: "compatibility".to_string(), // Default to compatibility
            gain_mode: "software".to_string(),   // Default to software volume
            conversion: None,
            fade_enabled: true, // Default to true
            fade_state: None,
            timing_mode: "polling".to_string(), // Default to polling for USB compatibility
            buffer_size_ms: 500, // Default buffer size, will be overwritten from settings
            load_to_memory: true,
            preload_next: true,
            is_dsd_playback: false,
            dsd_dop_enabled: false,
            dsd_dop_strict: true,
            dop_ring_buffer: None,
            dsd_decoder: None,
            dop_packer: None,
            asio_driver: None,
            resampler: None,
            source_sample_rate: 0,
        }
    }

    /// Open output device by ID ("default" or specific device ID).
    /// Uses the device's mix format initially; will be reconfigured when playback starts.
    fn open_output(&mut self, device_id: &str) -> Result<(), String> {
        // audio_engine::device functions imported locally where needed
        use audio_engine::output::WasapiOutput;

        // Close existing output
        if let Some(ref mut output) = self.output {
            let _ = output.stop();
        }
        self.output = None;

        // Validate the device exists (but we use open_default which gets the default device)
        let _device = if device_id == "default" {
            get_default_device().map_err(|e| e.to_string())?
        } else {
            get_device_by_id(device_id).map_err(|e| e.to_string())?
        };

        // Open with device's default format (will be reconfigured on playback start)
        let output = WasapiOutput::open_default().map_err(|e| e.to_string())?;

        self.device_id = device_id.to_string();
        self.output_sample_rate = output.sample_rate();
        self.output_channels = output.channels();
        self.output = Some(OutputBackend::Wasapi(output));

        info!(
            device_id = device_id,
            sample_rate = self.output_sample_rate,
            channels = self.output_channels,
            "Opened audio output device"
        );

        Ok(())
    }

    fn open_output_for_format(
        &mut self,
        sample_rate: u32,
        channels: u16,
        bit_depth: u16,
        _authoritative_bit_depth: Option<u16>,
    ) -> Result<(), String> {
        // Don't block playback for unknown bit depth - just mark as not bit-perfect
        // The compute_signal_path_checks function will handle the "unknown" case

        // Check if we need to reopen
        if let Some(ref output) = self.output {
            let mode_mismatch = if self.output_mode == "exclusive" {
                !output.is_exclusive()
            } else {
                output.is_exclusive()
            };

            if !mode_mismatch
                && output.sample_rate() == sample_rate
                && output.channels() == channels
                // For exclusive, check bit depth match. For shared, bit depth is always 32-float so we ignore source bit depth.
                && (!output.is_exclusive() || output.bit_depth() == bit_depth)
            {
                return Ok(());
            }
        }

        // Close existing output - must fully release before acquiring exclusive mode
        if let Some(ref mut output) = self.output {
            let _ = output.stop();
        }
        // Drop the old output to fully release WASAPI resources
        self.output = None;
        // Small delay to ensure WASAPI fully releases the device
        // This is necessary when switching between shared and exclusive modes
        std::thread::sleep(std::time::Duration::from_millis(50));

        info!(
            sample_rate = sample_rate,
            channels = channels,
            bit_depth = bit_depth,
            mode = self.output_mode,
            "Opening audio output"
        );

        if self.output_mode == "asio" {
            // ASIO mode - use AsioOutput
            use audio_engine::asio::AsioOutput;

            if let Some(ref driver_name) = self.asio_driver {
                info!(driver = %driver_name, "Opening ASIO output");
                match AsioOutput::new(driver_name, sample_rate, channels) {
                    Ok(output) => {
                        self.output_sample_rate = output.sample_rate();
                        self.output_channels = output.channels();
                        self.output = Some(OutputBackend::Asio(output));
                        self.conversion = None;
                        info!(driver = %driver_name, "ASIO output opened successfully");
                    }
                    Err(e) => {
                        warn!("ASIO output failed: {:?}", e);
                        return Err(format!("asio_error: {:?}", e));
                    }
                }
            } else {
                return Err("asio_no_driver: No ASIO driver selected".to_string());
            }
        } else if self.output_mode == "exclusive" {
            // audio_engine::device functions imported locally where needed
            use audio_engine::device::{get_default_device, get_device_by_id};
            use audio_engine::output::WasapiOutput;

            let device = if self.device_id == "default" {
                get_default_device().map_err(|e| e.to_string())?
            } else {
                get_device_by_id(&self.device_id).map_err(|e| e.to_string())?
            };

            match WasapiOutput::negotiate_exclusive_format(
                &device,
                sample_rate,
                channels,
                bit_depth,
                &self.policy,
                &self.timing_mode,
            ) {
                Ok((output, conversion)) => {
                    self.output_sample_rate = output.sample_rate();
                    self.output_channels = output.channels();
                    self.output = Some(OutputBackend::Wasapi(output));
                    self.conversion = conversion;
                    info!(
                        "Opened exclusive output (conversion: {:?})",
                        self.conversion
                    );
                }
                Err(e) => {
                    warn!("Exclusive mode failed: {}. Falling back.", e);
                    if self.policy == "strict" {
                        let code = if e.to_string().contains("Format not supported") {
                            "exclusive_unsupported_format"
                        } else {
                            "exclusive_unavailable"
                        };
                        return Err(format!("{}: {}", code, e));
                    }
                    // Fallback to shared
                    self.conversion = Some("shared_fallback".to_string());
                    // Proceed to shared block below...
                    // Wait, cannot fall through easily. Just duplicate shared logic here.
                    let output = WasapiOutput::open_with_format(sample_rate, channels)
                        .map_err(|e| e.to_string())?;
                    self.output_sample_rate = output.sample_rate();
                    self.output_channels = output.channels();
                    self.output = Some(OutputBackend::Wasapi(output));
                }
            }
        } else {
            // Shared mode
            let output =
                WasapiOutput::open_with_format(sample_rate, channels).map_err(|e| e.to_string())?;
            self.output_sample_rate = output.sample_rate();
            self.output_channels = output.channels();
            self.output = Some(OutputBackend::Wasapi(output));
            self.conversion = None;
        }

        // Initialize fade-in if enabled
        if self.fade_enabled && self.output.is_some() {
            let fade_frames = (self.output_sample_rate * 10) / 1000; // 10ms
            self.fade_state = Some(FadeState {
                direction: FadeDirection::In,
                samples_remaining: fade_frames as usize,
                total_samples: fade_frames as usize,
            });
        }

        Ok(())
    }

    fn open_output_for_dop_format(&mut self, dsd_rate: u32, channels: u16) -> Result<(), String> {
        use audio_engine::dop::dop_sample_rate;

        let dop_rate = dop_sample_rate(dsd_rate);

        if let Some(ref mut output) = self.output {
            let _ = output.stop();
        }
        self.output = None;
        std::thread::sleep(std::time::Duration::from_millis(50));

        info!(
            dsd_rate = dsd_rate,
            dop_rate = dop_rate,
            channels = channels,
            output_mode = %self.output_mode,
            "Opening DoP output"
        );

        if self.output_mode == "asio" {
            use audio_engine::asio::AsioOutput;

            if let Some(ref driver_name) = self.asio_driver {
                match AsioOutput::new(driver_name, dop_rate, channels) {
                    Ok(output) => {
                        self.output_sample_rate = output.sample_rate();
                        self.output_channels = output.channels();
                        self.output = Some(OutputBackend::Asio(output));
                        self.conversion = None;
                        info!(driver = %driver_name, "Opened DoP ASIO output successfully");
                        Ok(())
                    }
                    Err(e) => {
                        warn!("DoP ASIO output failed: {:?}", e);
                        Err(format!("dop_asio_error: {:?}", e))
                    }
                }
            } else {
                Err("dop_asio_no_driver: No ASIO driver selected".to_string())
            }
        } else {
            use audio_engine::device::{get_default_device, get_device_by_id};
            use audio_engine::output::WasapiOutput;

            let device = if self.device_id == "default" {
                get_default_device().map_err(|e| e.to_string())?
            } else {
                get_device_by_id(&self.device_id).map_err(|e| e.to_string())?
            };

            match WasapiOutput::open_device_exclusive_with_format(
                &device,
                dop_rate,
                channels,
                24,
                &self.timing_mode,
            ) {
                Ok(output) => {
                    self.output_sample_rate = output.sample_rate();
                    self.output_channels = output.channels();
                    self.output = Some(OutputBackend::Wasapi(output));
                    self.conversion = None;
                    info!("Opened DoP exclusive output successfully");
                    Ok(())
                }
                Err(e) => {
                    warn!("DoP exclusive mode failed: {}", e);
                    if self.dsd_dop_strict {
                        Err(format!("dop_unsupported_format: {}", e))
                    } else {
                        Err(format!(
                            "dop_conversion_unavailable: DSD->PCM conversion not implemented"
                        ))
                    }
                }
            }
        }
    }

    fn start_playback(&mut self, track: &TrackInfo, _db_path: &PathBuf) -> Result<(), String> {
        let track_path = Path::new(&track.path);

        if track.is_dsd() {
            return self.start_dsd_playback(track);
        }

        // Reset all format-specific state (critical for DSD→PCM transitions)
        self.reset_playback_state();

        // Clear ASIO ring buffer to prevent stale audio pops
        if let Some(ref output) = self.output {
            output.clear_ring_buffer();
        }

        // Check if memory loading is enabled (default: on)
        let load_to_memory_setting = self.load_to_memory;

        const GAPLESS_MAX_FILE_SIZE: u64 = 512 * 1024 * 1024;
        let file_size = fs::metadata(track_path).map(|m| m.len()).unwrap_or(0);
        let load_to_memory = load_to_memory_setting && file_size <= GAPLESS_MAX_FILE_SIZE;

        if !load_to_memory && load_to_memory_setting && file_size > GAPLESS_MAX_FILE_SIZE {
            info!(
                path = %track_path.display(),
                file_size = file_size,
                max_size = GAPLESS_MAX_FILE_SIZE,
                "File too large for RAM loading, using streaming decoder"
            );
        }

        let sample_rate: u32;
        let channels: u16;
        let bit_depth = track.bit_depth;

        if load_to_memory {
            // Use GaplessDecoder (loads to RAM)
            let gapless_decoder = GaplessDecoder::new(track_path).map_err(|e| e.to_string())?;
            let format = gapless_decoder.format();
            sample_rate = format.sample_rate;
            channels = format.channels;

            info!(
                path = %track_path.display(),
                sample_rate = sample_rate,
                channels = channels,
                bit_depth = ?bit_depth,
                mode = "gapless (RAM)",
                "Starting playback"
            );

            self.gapless_decoder = Some(gapless_decoder);
            self.preload_in_progress = false;
            self.decoder = None;
            self.current_track_id = Some(track.id);
        } else {
            // Use legacy AudioDecoder (streaming from disk)
            let decoder = AudioDecoder::open(track_path).map_err(|e| e.to_string())?;
            sample_rate = decoder.sample_rate();
            channels = decoder.channels() as u16;

            info!(
                path = %track_path.display(),
                sample_rate = sample_rate,
                channels = channels,
                bit_depth = ?bit_depth,
                mode = "streaming (disk)",
                "Starting playback"
            );

            self.decoder = Some(decoder);
            self.gapless_decoder = None;
            self.preload_in_progress = false;
            self.current_track_id = Some(track.id);
        }

        self.end_of_track = false;
        self.source_sample_rate = sample_rate;
        self.resampler = None;

        let output_bit_depth = bit_depth.unwrap_or(16);
        self.open_output_for_format(sample_rate, channels, output_bit_depth, track.bit_depth)?;

        // For ASIO: Start the output FIRST to load the driver and get actual sample rate
        // The driver may run at a fixed rate different from what we requested
        if let Some(ref mut output) = self.output {
            output.start().map_err(|e| e.to_string())?;

            // Re-read sample rate after start - ASIO drivers may report different actual rate
            let actual_output_rate = output.sample_rate();
            if actual_output_rate != self.output_sample_rate {
                info!(
                    before = self.output_sample_rate,
                    after = actual_output_rate,
                    "Output sample rate updated after driver start"
                );
                self.output_sample_rate = actual_output_rate;
            }
        }

        // NOW check if resampling is needed (after we know actual output rate)
        if sample_rate != self.output_sample_rate && self.output_sample_rate > 0 {
            info!(
                source = sample_rate,
                target = self.output_sample_rate,
                "Creating resampler for sample rate conversion"
            );
            let resampler = audio_engine::Resampler::new(
                sample_rate,
                self.output_sample_rate,
                self.output_channels as usize,
                audio_engine::ResamplerQuality::HighQuality,
            )?;
            self.resampler = Some(resampler);
        }

        let output_bit_depth = if let Some(o) = &self.output {
            o.bit_depth()
        } else {
            0
        };

        info!(
            track_sample_rate = sample_rate,
            track_bit_depth = bit_depth,
            track_channels = channels,
            output_sample_rate = self.output_sample_rate,
            output_bit_depth = output_bit_depth,
            output_channels = self.output_channels,
            conversion = ?self.conversion,
            resampling = self.resampler.is_some(),
            "Format negotiation result"
        );

        // Create ring buffer sized for OUTPUT rate (after resampling)
        self.ring_buffer = Some(AudioRingBuffer::new(
            self.output_sample_rate,
            self.output_channels as usize,
            self.buffer_size_ms,
        ));

        Ok(())
    }

    fn start_dsd_playback(&mut self, track: &TrackInfo) -> Result<(), String> {
        use audio_engine::output::DopRingBuffer;
        use audio_engine::{DopPacker, DsdDecoder};

        // Reset all format-specific state (critical for PCM→DSD transitions)
        self.reset_playback_state();

        // Clear ASIO ring buffer to prevent stale audio pops
        if let Some(ref output) = self.output {
            output.clear_ring_buffer();
        }

        if !self.dsd_dop_enabled {
            return Err("dop_disabled: DoP playback is disabled in settings".to_string());
        }

        let track_path = Path::new(&track.path);
        let dsd_rate = track.dsd_rate_hz.ok_or("Missing DSD rate")?;
        let dsd_channels = track.dsd_channels.unwrap_or(2);

        info!(
            path = %track_path.display(),
            dsd_rate = dsd_rate,
            dsd_channels = dsd_channels,
            "Starting DSD playback via DoP"
        );

        let dsd_decoder = DsdDecoder::open(track_path).map_err(|e| e.to_string())?;

        self.open_output_for_dop_format(dsd_rate, dsd_channels)?;

        let dop_rate = audio_engine::dop::dop_sample_rate(dsd_rate);
        self.dop_ring_buffer = Some(DopRingBuffer::new(
            dop_rate,
            dsd_channels as usize,
            self.buffer_size_ms,
        ));

        self.dsd_decoder = Some(dsd_decoder);
        self.dop_packer = Some(DopPacker::new(dsd_channels as usize));
        self.is_dsd_playback = true;
        self.end_of_track = false;

        if let Some(ref mut output) = self.output {
            output.start().map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn stop_playback(&mut self) {
        self.decoder = None;
        self.gapless_decoder = None;
        self.preload_in_progress = false;
        self.ring_buffer = None;
        self.end_of_track = false;
        self.current_track_id = None;
        self.is_dsd_playback = false;
        self.dsd_decoder = None;
        self.dop_packer = None;
        self.dop_ring_buffer = None;
        self.resampler = None;
        self.source_sample_rate = 0;
        if let Some(ref mut output) = self.output {
            let _ = output.stop();
        }
    }

    fn reset_playback_state(&mut self) {
        self.decoder = None;
        self.gapless_decoder = None;
        self.preload_in_progress = false;
        self.ring_buffer = None;
        self.resampler = None;
        self.source_sample_rate = 0;
        self.is_dsd_playback = false;
        self.dsd_decoder = None;
        self.dop_packer = None;
        self.dop_ring_buffer = None;
        self.end_of_track = false;
    }

    /// Cancel any preloaded next track (for skip/seek/shuffle actions).
    fn cancel_preload(&mut self) {
        if let Some(ref mut gd) = self.gapless_decoder {
            gd.cancel_preload();
        }
        self.preload_in_progress = false;
    }

    /// Pause the output stream without clearing decoder/buffer state.
    /// This prevents stale audio from looping in the callback.
    fn pause_output(&mut self) {
        if let Some(ref mut output) = self.output {
            let _ = output.stop();
        }
    }

    /// Resume the output stream after a pause.
    fn resume_output(&mut self) {
        if let Some(ref mut output) = self.output {
            let _ = output.start();
        }
    }

    fn seek(&mut self, position_ms: u64) -> Result<(), String> {
        if self.is_dsd_playback {
            // DSD seek path
            if let Some(ref mut dsd) = self.dsd_decoder {
                dsd.seek_ms(position_ms).map_err(|e| e.to_string())?;
            }
            if let Some(ref mut rb) = self.dop_ring_buffer {
                rb.clear();
            }
            if let Some(ref mut packer) = self.dop_packer {
                packer.reset();
            }
        } else if let Some(ref mut gd) = self.gapless_decoder {
            gd.seek(position_ms).map_err(|e| e.to_string())?;
        } else if let Some(ref mut decoder) = self.decoder {
            decoder.seek(position_ms).map_err(|e| e.to_string())?;
        }
        if let Some(ref mut ring_buffer) = self.ring_buffer {
            ring_buffer.clear();
        }
        if let Some(ref output) = self.output {
            output.clear_ring_buffer();
        }
        if let Some(ref mut resampler) = self.resampler {
            resampler.reset();
        }
        self.end_of_track = false;
        Ok(())
    }

    fn fill_ring_buffer(&mut self) -> Result<bool, String> {
        let gapless_decoder = match self.gapless_decoder.as_mut() {
            Some(d) => d,
            None => {
                // Fallback to legacy decoder if gapless_decoder not set
                return self.fill_ring_buffer_legacy();
            }
        };

        let ring_buffer = match self.ring_buffer.as_mut() {
            Some(rb) => rb,
            None => return Ok(false),
        };

        let output_channels = self.output_channels as usize;
        let target_frames = ring_buffer.capacity_frames() / 2;

        while ring_buffer.available_frames() < target_frames {
            let samples = match gapless_decoder.decode_next() {
                Ok(Some(samples)) => samples,
                Ok(None) => {
                    self.end_of_track = true;
                    return Ok(ring_buffer.available_frames() > 0);
                }
                Err(e) => {
                    return Err(format!("Decode error: {}", e));
                }
            };

            if samples.is_empty() {
                continue;
            }

            let input_channels = gapless_decoder.format().channels as usize;
            let converted = if input_channels != output_channels {
                convert_channels_interleaved_f32(&samples, input_channels, output_channels)
            } else {
                samples
            };

            let resampled = if let Some(ref mut resampler) = self.resampler {
                resampler.process_interleaved(&converted)?
            } else {
                converted
            };

            ring_buffer.push(&resampled);
        }

        // Trigger preload when ~2 seconds from end (or immediately for short tracks)
        if self.preload_next && !self.preload_in_progress {
            let samples_decoded = gapless_decoder.samples_decoded();
            if let Some(total) = gapless_decoder.total_samples() {
                let remaining = total.saturating_sub(samples_decoded);
                let preload_threshold = gapless_decoder.format().sample_rate as u64 * 2;
                if remaining < preload_threshold {
                    self.preload_in_progress = true;
                }
            }
        }

        Ok(true)
    }

    fn fill_ring_buffer_legacy(&mut self) -> Result<bool, String> {
        let decoder = match self.decoder.as_mut() {
            Some(d) => d,
            None => return Ok(false),
        };

        let ring_buffer = match self.ring_buffer.as_mut() {
            Some(rb) => rb,
            None => return Ok(false),
        };

        let output_channels = self.output_channels as usize;

        let target_frames = ring_buffer.capacity_frames() / 2;

        while ring_buffer.available_frames() < target_frames {
            let samples = match decoder.decode_next() {
                Ok(Some(samples)) => samples,
                Ok(None) => {
                    self.end_of_track = true;
                    return Ok(ring_buffer.available_frames() > 0);
                }
                Err(e) => {
                    return Err(format!("Decode error: {}", e));
                }
            };

            if samples.is_empty() {
                continue;
            }

            let input_channels = decoder.channels();
            let converted = if input_channels != output_channels {
                convert_channels_interleaved_f32(&samples, input_channels, output_channels)
            } else {
                samples
            };

            let resampled = if let Some(ref mut resampler) = self.resampler {
                resampler.process_interleaved(&converted)?
            } else {
                converted
            };

            ring_buffer.push(&resampled);
        }

        Ok(true)
    }

    fn fill_dop_ring_buffer(&mut self) -> Result<bool, String> {
        let dsd_decoder = match self.dsd_decoder.as_mut() {
            Some(d) => d,
            None => return Ok(false),
        };

        let dop_ring_buffer = match self.dop_ring_buffer.as_mut() {
            Some(rb) => rb,
            None => return Ok(false),
        };

        let dop_packer = match self.dop_packer.as_mut() {
            Some(p) => p,
            None => return Ok(false),
        };

        let target_frames = dop_ring_buffer.capacity_frames() / 2;

        while dop_ring_buffer.available_frames() < target_frames {
            let dsd_bytes = match dsd_decoder.read_block() {
                Ok(Some(bytes)) => bytes,
                Ok(None) => {
                    self.end_of_track = true;
                    return Ok(dop_ring_buffer.available_frames() > 0);
                }
                Err(e) => {
                    return Err(format!("DSD decode error: {}", e));
                }
            };

            if dsd_bytes.is_empty() {
                continue;
            }

            let dop_samples = dop_packer.pack(&dsd_bytes);
            dop_ring_buffer.push(&dop_samples);
        }

        Ok(true)
    }

    fn process_dop_audio(&mut self) -> Result<bool, String> {
        let output = match self.output.as_mut() {
            Some(o) => o,
            None => return Err("No output device".to_string()),
        };

        let dop_ring_buffer = match self.dop_ring_buffer.as_mut() {
            Some(rb) => rb,
            None => return Err("No DoP ring buffer".to_string()),
        };

        let available = dop_ring_buffer.available_frames();
        if available == 0 {
            if self.end_of_track {
                return Ok(false);
            }
            return Ok(true);
        }

        let channels = self.output_channels as usize;
        let space_samples = output.available_dop_space();
        let space_frames = space_samples / channels.max(1);

        if space_frames == 0 {
            return Ok(true);
        }

        let frames_to_write = available.min(space_frames);
        let samples_to_write = frames_to_write * channels;
        let dop_samples = dop_ring_buffer.pop(samples_to_write);

        match output.write_raw_dop(&dop_samples) {
            Ok(_frames_written) => {
                if self.end_of_track && dop_ring_buffer.available_frames() == 0 {
                    return Ok(false);
                }
                Ok(true)
            }
            Err(audio_engine::output::OutputError::DeviceInvalidated) => {
                Err("Device invalidated".to_string())
            }
            Err(e) => Err(format!("DoP output error: {}", e)),
        }
    }

    fn effective_volume(&self, volume: f32) -> f32 {
        if let Some(ref output) = self.output {
            if output.is_exclusive() && self.policy == "strict" {
                return 1.0;
            }
        }
        if self.gain_mode == "unity" {
            1.0
        } else {
            volume
        }
    }

    /// Process audio: fill buffer, then write to WASAPI. Returns true if playback should continue.
    fn process_audio(&mut self, volume: f32) -> Result<bool, String> {
        if self.is_dsd_playback {
            return self.process_dop_audio();
        }

        // Check if we have any active decoder (gapless or legacy)
        if self.decoder.is_none() && self.gapless_decoder.is_none() {
            return Ok(false);
        }

        let eff_vol = self.effective_volume(volume);

        let output = match self.output.as_mut() {
            Some(o) => o,
            None => return Err("No output device".to_string()),
        };

        let ring_buffer = match self.ring_buffer.as_mut() {
            Some(rb) => rb,
            None => return Err("No ring buffer".to_string()),
        };

        match output.write_from_buffer(ring_buffer, eff_vol) {
            Ok(_frames_written) => {
                if self.end_of_track && ring_buffer.available_frames() == 0 {
                    return Ok(false);
                }
                Ok(true)
            }
            Err(audio_engine::output::OutputError::DeviceInvalidated) => {
                Err("Device invalidated".to_string())
            }
            Err(e) => Err(format!("Output error: {}", e)),
        }
    }
}

fn spawn_audio_thread(
    app: tauri::AppHandle,
    db_path: PathBuf,
    engine: Arc<Mutex<audio_engine::EngineState>>,
    command_rx: Receiver<PlaybackCommand>,
    diagnostics: Arc<DiagnosticsState>,
) {
    std::thread::spawn(move || {
        // Initialize settings defaults
        if let Ok(conn) = library::open_db(&db_path) {
            if library::get_setting(&conn, "audio.output.mode")
                .unwrap_or(None)
                .is_none()
            {
                let _ = library::set_setting(&conn, "audio.output.mode", "exclusive");
            }
            if library::get_setting(&conn, "audio.output.policy")
                .unwrap_or(None)
                .is_none()
            {
                let _ = library::set_setting(&conn, "audio.output.policy", "strict");
            }
            if library::get_setting(&conn, "audio.output.fade")
                .unwrap_or(None)
                .is_none()
            {
                let _ = library::set_setting(&conn, "audio.output.fade", "off");
            }
        }

        let position_tick = tick(Duration::from_millis(250));
        // Audio processing tick - run at ~10ms for smooth playback
        let audio_tick = tick(Duration::from_millis(10));
        // Telemetry tick - 1 Hz for diagnostics UI
        let telemetry_tick = tick(Duration::from_secs(1));

        let (preload_tx, preload_request_rx): (Sender<PreloadRequest>, Receiver<PreloadRequest>) =
            unbounded();
        let (preload_result_tx, preload_rx): (Sender<PreloadResult>, Receiver<PreloadResult>) =
            unbounded();

        std::thread::spawn(move || {
            while let Ok(request) = preload_request_rx.recv() {
                let path = request.path;
                match fs::read(&path) {
                    Ok(bytes) => {
                        if preload_result_tx.send(PreloadResult { path, bytes }).is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        warn!(
                            path = %path.display(),
                            error = %err,
                            "Failed to preload next track bytes"
                        );
                        if preload_result_tx
                            .send(PreloadResult {
                                path,
                                bytes: Vec::new(),
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            }
        });

        let mut playback = AudioPlayback::new();
        let mut current_device_info: Option<audio_engine::device::AudioDeviceInfo> = None;
        let mut preload_request_path: Option<PathBuf> = None;

        // Load saved settings from DB into playback state
        if let Ok(conn) = library::open_db(&db_path) {
            if let Ok(Some(mode)) = library::get_setting(&conn, "audio.output.mode") {
                playback.output_mode = mode;
            }
            if let Ok(Some(policy)) = library::get_setting(&conn, "audio.output.policy") {
                playback.policy = policy;
            }
            if let Ok(Some(fade)) = library::get_setting(&conn, "audio.output.fade") {
                playback.fade_enabled = fade == "on";
            }
            // Set gain_mode based on output_mode and policy
            if playback.output_mode == "exclusive" && playback.policy == "strict" {
                playback.gain_mode = "unity".to_string();
            }
            // Load DSD settings
            if let Ok(Some(dop_enabled)) = library::get_setting(&conn, "devices.dsd_dop_enabled") {
                playback.dsd_dop_enabled = dop_enabled == "on";
            }
            if let Ok(Some(dop_strict)) = library::get_setting(&conn, "devices.dsd_dop_strict") {
                playback.dsd_dop_strict = dop_strict == "on";
            }
            // Load ASIO driver setting
            if let Ok(Some(asio_driver)) = library::get_setting(&conn, "audio.output.asio_driver") {
                if !asio_driver.is_empty() {
                    playback.asio_driver = Some(asio_driver);
                }
            }
        }

        // Try to open default output at startup
        if let Err(e) = playback.open_output("default") {
            warn!("Failed to open default audio output: {}", e);
        } else {
            // Get device info
            current_device_info = audio_engine::device::list_devices()
                .ok()
                .and_then(|devices| devices.into_iter().find(|d| d.is_default));
        }

        loop {
            select! {
                recv(command_rx) -> msg => {
                    let Ok(cmd) = msg else { break; };
                    handle_playback_command(
                        &app,
                        &db_path,
                        &engine,
                        &mut playback,
                        &mut current_device_info,
                        cmd,
                        &diagnostics,
                    );
                }
                recv(position_tick) -> _ => {
                    emit_position_tick(&app, &engine);
                }
                recv(telemetry_tick) -> _ => {
                    emit_audio_telemetry(&app, &engine, &playback, current_device_info.as_ref());
                }
                recv(audio_tick) -> _ => {
                    // Process audio if playing
                    let (is_playing, volume, _track_path) = {
                        let engine = engine.lock();
                        let is_playing = engine.state == PlaybackState::Playing;
                        let volume = engine.volume;
                        let track_path = engine.session.as_ref().map(|s| s.track.path.clone());
                        (is_playing, volume, track_path)
                    };

                    if is_playing {
                        let fill_result = if playback.is_dsd_playback {
                            playback.fill_dop_ring_buffer()
                        } else {
                            playback.fill_ring_buffer()
                        };

                        if let Err(e) = fill_result {
                            error!("Failed to fill ring buffer: {}", e);
                            let track_id = {
                                let engine = engine.lock();
                                engine.session.as_ref().map(|s| s.track_id)
                            };
                            emit_playback_error(&app, "decode_error", &e, track_id, false, None);
                            engine.lock().stop();
                            playback.stop_playback();
                            emit_playback_state(&app, &engine);
                            continue;
                        }

                        if !playback.preload_in_progress {
                            preload_request_path = None;
                        }

                        while let Ok(result) = preload_rx.try_recv() {
                            let should_apply = playback.preload_next
                                && playback.preload_in_progress
                                && playback
                                    .gapless_decoder
                                    .as_ref()
                                    .map_or(false, |d| !d.has_preloaded_next());

                            if should_apply
                                && preload_request_path
                                    .as_ref()
                                    .map_or(true, |path| path == &result.path)
                            {
                                if result.bytes.is_empty() {
                                    warn!(
                                        path = %result.path.display(),
                                        "Preload worker returned empty bytes"
                                    );
                                    playback.preload_in_progress = false;
                                } else if let Some(ref mut gd) = playback.gapless_decoder {
                                    if let Err(e) = gd.preload_next_from_bytes(result.bytes) {
                                        warn!(
                                            path = %result.path.display(),
                                            error = %e,
                                            "Failed to preload next track - will use non-gapless transition"
                                        );
                                        playback.preload_in_progress = false;
                                    }
                                }
                            }

                            preload_request_path = None;
                        }

                        // Preload next track when approaching end
                        if playback.preload_next
                            && playback.preload_in_progress
                            && playback
                                .gapless_decoder
                                .as_ref()
                                .map_or(false, |d| !d.has_preloaded_next())
                            && preload_request_path.is_none()
                        {
                            let next_track_path = {
                                let engine_guard = engine.lock();
                                if let Some(current_idx) = engine_guard.queue.current_index() {
                                    engine_guard.queue.items().get(current_idx + 1).map(|item| item.track.path.clone())
                                } else {
                                    None
                                }
                            };
                            if let Some(path) = next_track_path {
                                if preload_tx.send(PreloadRequest { path: path.clone().into() }).is_ok() {
                                    preload_request_path = Some(path.into());
                                } else {
                                    playback.preload_in_progress = false;
                                }
                            } else {
                                playback.preload_in_progress = false;
                            }
                        }

                        let transitioned = playback
                            .gapless_decoder
                            .as_mut()
                            .map(|d| d.take_just_transitioned())
                            .unwrap_or(false);

                        if transitioned {
                            {
                                let mut engine = engine.lock();
                                engine.next();
                            }
                            playback.preload_in_progress = false;
                            let new_track_id = {
                                let engine = engine.lock();
                                engine.session.as_ref().map(|s| s.track_id)
                            };
                            playback.current_track_id = new_track_id;
                            emit_now_playing(&app, &engine);
                            emit_playback_state(&app, &engine);
                            emit_queue_changed(&app, &engine);
                        }

                        match playback.process_audio(volume) {
                            Ok(true) => {
                                // Continue playing
                            }
                            Ok(false) => {
                                // Track ended - check transition type for gapless
                                let transition_type = playback.gapless_decoder
                                    .as_ref()
                                    .map(|d| d.transition_type())
                                    .unwrap_or(TransitionType::EndOfQueue);

                                match transition_type {
                                    TransitionType::Gapless => {
                                        // Seamless transition already happened in decode_next()
                                        info!("Gapless transition complete");
                                        {
                                            let mut engine = engine.lock();
                                            engine.next();
                                        }
                                        playback.preload_in_progress = false;
                                        let new_track_id = {
                                            let engine = engine.lock();
                                            engine.session.as_ref().map(|s| s.track_id)
                                        };
                                        playback.current_track_id = new_track_id;
                                        emit_now_playing(&app, &engine);
                                        emit_playback_state(&app, &engine);
                                        emit_queue_changed(&app, &engine);
                                    }
                                    TransitionType::FormatChange => {
                                        // Different format - need to reinit WASAPI
                                        info!("Format change - reinitializing output");
                                        {
                                            let mut engine = engine.lock();
                                            engine.next();
                                        }
                                        let next_track = {
                                            let engine = engine.lock();
                                            engine.session.as_ref().map(|s| s.track.clone())
                                        };
                                        if let Some(track) = next_track {
                                            if let Err(e) = playback.start_playback(&track, &db_path) {
                                                error!("Failed to start next track: {}", e);
                                                let (code, msg) = parse_playback_error(&e);
                                                emit_playback_error(
                                                    &app,
                                                    &code,
                                                    &msg,
                                                    Some(track.id),
                                                    false,
                                                    None,
                                                );
                                                engine.lock().stop();
                                            }
                                        }
                                        emit_now_playing(&app, &engine);
                                        emit_playback_state(&app, &engine);
                                        emit_queue_changed(&app, &engine);
                                    }
                                    TransitionType::EndOfQueue => {
                                        // No preloaded next track - try to advance queue normally
                                        {
                                            let mut engine = engine.lock();
                                            engine.next();
                                        }

                                        let next_track = {
                                            let engine = engine.lock();
                                            if engine.state == PlaybackState::Playing {
                                                engine.session.as_ref().map(|s| s.track.clone())
                                            } else {
                                                None
                                            }
                                        };

                                        if let Some(track) = next_track {
                                            if let Err(e) = playback.start_playback(&track, &db_path) {
                                                error!("Failed to start next track: {}", e);
                                                let (code, msg) = parse_playback_error(&e);
                                                emit_playback_error(&app, &code, &msg, Some(track.id), false, None);
                                                engine.lock().stop();
                                                playback.stop_playback();
                                            }
                                            emit_now_playing(&app, &engine);
                                            emit_playback_state(&app, &engine);
                                            emit_queue_changed(&app, &engine);
                                        } else {
                                            info!("Track ended, queue exhausted");
                                            playback.stop_playback();
                                            emit_playback_state(&app, &engine);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Audio processing error: {}", e);
                                let track_id = {
                                    let engine = engine.lock();
                                    engine.session.as_ref().map(|s| s.track_id)
                                };

                                if e.contains("Device invalidated") {
                                    emit_playback_error(&app, "device_invalidated", &e, track_id, true, Some("switch_to_default"));
                                    // Try to recover with default device
                                    if let Err(recover_err) = playback.open_output("default") {
                                        error!("Failed to recover to default device: {}", recover_err);
                                        engine.lock().stop();
                                        playback.stop_playback();
                                    }
                                } else {
                                    emit_playback_error(&app, "playback_error", &e, track_id, false, None);
                                    engine.lock().stop();
                                    playback.stop_playback();
                                }
                                emit_playback_state(&app, &engine);
                            }
                        }
                    }
                }
            }
        }
    });
}

fn handle_playback_command(
    app: &tauri::AppHandle,
    db_path: &PathBuf,
    engine: &Arc<Mutex<audio_engine::EngineState>>,
    playback: &mut AudioPlayback,
    current_device_info: &mut Option<audio_engine::device::AudioDeviceInfo>,
    command: PlaybackCommand,
    diagnostics: &Arc<DiagnosticsState>,
) {
    match command {
        PlaybackCommand::PlayNow { track_id } => {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            diagnostics.playback_start_ms.store(now_ms, std::sync::atomic::Ordering::SeqCst);

            playback.cancel_preload();
            let Some(track) = resolve_track(db_path, track_id)
                .map_err(|e| {
                    emit_playback_error(app, "db_track_lookup", &e, Some(track_id), true, None)
                })
                .ok()
            else {
                return;
            };

            // Update engine state
            {
                let mut engine = engine.lock();
                engine.play_now(track.clone());
            }

            // Start actual playback
            if let Err(e) = playback.start_playback(&track, db_path) {
                error!("Failed to start playback: {}", e);
                let (code, msg) = parse_playback_error(&e);
                emit_playback_error(app, &code, &msg, Some(track_id), false, None);
                engine.lock().stop();
                emit_playback_state(app, engine);
                return;
            }

            let complete_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            diagnostics.playback_complete_ms.store(complete_ms, std::sync::atomic::Ordering::SeqCst);

            emit_now_playing(app, engine);
            emit_playback_state(app, engine);
            emit_queue_changed(app, engine);
            emit_audio_debug(app, engine, playback, current_device_info.as_ref());
        }
        PlaybackCommand::AddToQueue { track_id } => {
            let Some(track) = resolve_track(db_path, track_id)
                .map_err(|e| {
                    emit_playback_error(app, "db_track_lookup", &e, Some(track_id), true, None)
                })
                .ok()
            else {
                return;
            };

            {
                let mut engine = engine.lock();
                engine.add_to_queue(track);
            }

            emit_queue_changed(app, engine);
        }
        PlaybackCommand::PlayNowWithQueue {
            track_ids,
            start_index,
        } => {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            diagnostics.playback_start_ms.store(now_ms, std::sync::atomic::Ordering::SeqCst);

            playback.cancel_preload();
            let mut tracks = Vec::new();
            for track_id in &track_ids {
                if let Ok(track) = resolve_track(db_path, *track_id) {
                    tracks.push(track);
                }
            }

            if tracks.is_empty() {
                emit_playback_error(
                    app,
                    "no_valid_tracks",
                    "No valid tracks to play",
                    None,
                    false,
                    None,
                );
                return;
            }

            let actual_start = start_index.min(tracks.len() - 1);
            let start_track = tracks[actual_start].clone();

            {
                let mut engine = engine.lock();
                engine.set_and_play(tracks, actual_start);
            }

            if let Err(e) = playback.start_playback(&start_track, db_path) {
                error!("Failed to start playback: {}", e);
                let (code, msg) = parse_playback_error(&e);
                emit_playback_error(app, &code, &msg, Some(start_track.id), false, None);
                engine.lock().stop();
                emit_playback_state(app, engine);
                return;
            }

            let complete_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            diagnostics.playback_complete_ms.store(complete_ms, std::sync::atomic::Ordering::SeqCst);

            emit_now_playing(app, engine);
            emit_playback_state(app, engine);
            emit_queue_changed(app, engine);
            emit_audio_debug(app, engine, playback, current_device_info.as_ref());
        }
        PlaybackCommand::Pause => {
            engine.lock().pause();
            playback.pause_output();
            emit_playback_state(app, engine);
        }
        PlaybackCommand::Resume => {
            engine.lock().resume();
            playback.resume_output();
            emit_playback_state(app, engine);
        }
        PlaybackCommand::Stop => {
            playback.cancel_preload();
            engine.lock().stop();
            playback.stop_playback();
            emit_playback_state(app, engine);
        }
        PlaybackCommand::Seek { position_ms } => {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            diagnostics.seek_start_ms.store(now_ms, std::sync::atomic::Ordering::SeqCst);

            playback.cancel_preload();
            
            // Check if decoder track matches engine track (fixes race condition)
            let engine_track_id = {
                let engine = engine.lock();
                engine.session.as_ref().map(|s| s.track_id)
            };
            
            if playback.current_track_id != engine_track_id {
                // Decoder has stale track - reload the correct track before seeking
                if let Some(track_id) = engine_track_id {
                    let track = {
                        let engine = engine.lock();
                        engine.session.as_ref().map(|s| s.track.clone())
                    };
                    if let Some(track) = track {
                        info!("Seek: reloading track {} (decoder had stale track)", track_id);
                        if let Err(e) = playback.start_playback(&track, db_path) {
                            error!("Failed to reload track for seek: {}", e);
                        }
                    }
                }
            }
            
            if let Err(e) = playback.seek(position_ms) {
                error!("Seek failed: {}", e);
            }
            engine.lock().seek(position_ms);

            let complete_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            diagnostics.seek_complete_ms.store(complete_ms, std::sync::atomic::Ordering::SeqCst);

            emit_position_now(app, engine);
        }
        PlaybackCommand::Next => {
            playback.cancel_preload();
            // Get current track before advancing
            {
                let mut engine = engine.lock();
                engine.next();
            }

            // Check if there's a next track to play
            let next_track = {
                let engine = engine.lock();
                if engine.state == PlaybackState::Playing {
                    engine.session.as_ref().map(|s| s.track.clone())
                } else {
                    None
                }
            };

            if let Some(track) = next_track {
                if let Err(e) = playback.start_playback(&track, db_path) {
                    error!("Failed to start next track: {}", e);
                    let (code, msg) = parse_playback_error(&e);
                    emit_playback_error(app, &code, &msg, Some(track.id), false, None);
                    engine.lock().stop();
                }
            } else {
                playback.stop_playback();
            }

            emit_now_playing(app, engine);
            emit_playback_state(app, engine);
            emit_queue_changed(app, engine);
        }
        PlaybackCommand::Previous => {
            playback.cancel_preload();
            {
                let mut engine = engine.lock();
                engine.previous();
            }

            // Restart playback on current/previous track
            let track = {
                let engine = engine.lock();
                engine.session.as_ref().map(|s| s.track.clone())
            };

            if let Some(track) = track {
                if let Err(e) = playback.start_playback(&track, db_path) {
                    error!("Failed to start previous track: {}", e);
                    let (code, msg) = parse_playback_error(&e);
                    emit_playback_error(app, &code, &msg, Some(track.id), false, None);
                    engine.lock().stop();
                }
            }

            emit_now_playing(app, engine);
            emit_playback_state(app, engine);
            emit_queue_changed(app, engine);
        }
        PlaybackCommand::SetVolume { volume } => {
            engine.lock().set_volume(volume);
        }
        PlaybackCommand::SetDevice { device_id } => {
            let devices = match audio_engine::device::list_devices() {
                Ok(devices) => devices,
                Err(e) => {
                    emit_playback_error(
                        app,
                        "device_list_failed",
                        &e.to_string(),
                        None,
                        true,
                        None,
                    );
                    return;
                }
            };

            let next_device = if device_id == "default" {
                devices.into_iter().find(|d| d.is_default)
            } else {
                devices.into_iter().find(|d| d.id == device_id)
            };

            let Some(device) = next_device else {
                emit_playback_error(
                    app,
                    "device_not_found",
                    "device not found",
                    None,
                    true,
                    Some("switch_to_default"),
                );
                return;
            };

            // Switch output device
            if let Err(e) = playback.open_output(&device.id) {
                emit_playback_error(
                    app,
                    "device_open_failed",
                    &e,
                    None,
                    true,
                    Some("switch_to_default"),
                );
                return;
            }

            *current_device_info = Some(device.clone());
            let _ = app.emit(
                "evt_device_changed",
                DeviceChangedEvent {
                    device_id: device.id,
                    device_name: device.name,
                    is_default: device.is_default,
                },
            );

            emit_audio_debug(app, engine, playback, current_device_info.as_ref());
        }
        PlaybackCommand::SetOutputSettings {
            mode,
            policy,
            fade,
            timing,
            asio_driver,
        } => {
            info!(
                "Received output settings update: mode={}, policy={}, fade={}, timing={}, asio_driver={:?}",
                mode, policy, fade, timing, asio_driver
            );

            let mut changed = false;

            if playback.output_mode != mode {
                playback.output_mode = mode.clone();
                changed = true;
            }

            if playback.timing_mode != timing {
                playback.timing_mode = timing.clone();
                changed = true;
            }

            if playback.asio_driver != asio_driver {
                playback.asio_driver = asio_driver.clone();
                changed = true;
            }

            playback.policy = policy.clone();

            // Update gain_mode based on output_mode and policy
            if mode == "exclusive" && policy == "strict" {
                playback.gain_mode = "unity".to_string();
            } else {
                playback.gain_mode = "software".to_string();
            }

            if playback.output_mode == "exclusive" {
                changed = true;
            }

            playback.fade_enabled = fade;

            if changed {
                // Trigger reopen if playing
                if engine.lock().state == PlaybackState::Playing {
                    if let Some(decoder) = &playback.decoder {
                        let sr = decoder.sample_rate();
                        let ch = decoder.channels() as u16;
                        let decoder_bd = decoder.bit_depth().map(|b| b as u16);

                        // Get track bit depth from session
                        let track_bd = engine
                            .lock()
                            .session
                            .as_ref()
                            .and_then(|s| s.track.bit_depth);

                        // Use decoder bit depth, fall back to track metadata, then default to 16
                        let bd = decoder_bd.or(track_bd).unwrap_or(16);

                        if let Err(e) = playback.open_output_for_format(sr, ch, bd, track_bd) {
                            error!("Failed to apply output settings: {}", e);
                            engine.lock().stop();
                            playback.stop_playback();
                            emit_playback_state(app, engine);

                            let (code, msg) = parse_playback_error(&e);
                            emit_playback_error(app, &code, &msg, None, false, None);
                        } else {
                            // Recreate ring buffer with decoder's sample rate and output's channels
                            // The ring buffer sits between decoder and output, storing decoded samples
                            // before they're written to the output device
                            playback.ring_buffer = Some(AudioRingBuffer::new(
                                sr, // decoder's sample rate
                                playback.output_channels as usize,
                                playback.buffer_size_ms,
                            ));
                            // Start output if needed
                            if let Some(output) = &mut playback.output {
                                if let Err(e) = output.start() {
                                    error!("Failed to restart output: {}", e);
                                }
                            }
                        }
                    }
                }
            }

            emit_audio_debug(app, engine, playback, current_device_info.as_ref());
        }
        PlaybackCommand::SetPlayerSettings {
            buffer_size_ms,
            load_to_memory,
            preload_next,
        } => {
            info!(
                "Received player settings update: buffer_size_ms={}, load_to_memory={}, preload_next={}",
                buffer_size_ms, load_to_memory, preload_next
            );

            playback.buffer_size_ms = buffer_size_ms;
            playback.load_to_memory = load_to_memory;
            playback.preload_next = preload_next;

            if !playback.preload_next {
                playback.cancel_preload();
            }

            emit_audio_debug(app, engine, playback, current_device_info.as_ref());
        }
        PlaybackCommand::OpenAsioControlPanel { driver_name } => {
            // First try to use the active ASIO output if available
            if let Some(ref output) = playback.output {
                if let Err(e) = output.open_asio_control_panel() {
                    warn!("Failed to open ASIO control panel via active output: {:?}", e);
                    // Fall through to direct method
                } else {
                    return; // Success via active output
                }
            }
            
            // No active ASIO output or it failed - use direct method
            info!(driver = %driver_name, "Opening ASIO control panel directly (no active output)");
            if let Err(e) = audio_engine::open_asio_control_panel_direct(&driver_name) {
                warn!("Failed to open ASIO control panel directly: {}", e);
            }
        }
    }
}

fn resolve_track(db_path: &PathBuf, track_id: i64) -> Result<TrackInfo, String> {
    let conn = library::open_db(db_path).map_err(|e| e.to_string())?;
    let row = library::get_track_by_id(&conn, track_id).map_err(|e| e.to_string())?;

    let id = row.id.unwrap_or(track_id);
    Ok(TrackInfo {
        id,
        path: row.path,
        title: row.title,
        artist: row.artist,
        album: row.album,
        duration_ms: row.duration_ms.and_then(|v| u64::try_from(v).ok()),
        sample_rate: row.sample_rate.and_then(|v| u32::try_from(v).ok()),
        bit_depth: row.bit_depth.and_then(|v| u16::try_from(v).ok()),
        channels: row.channels.and_then(|v| u16::try_from(v).ok()),
        codec: row.codec,
        container: row.container,
        dsd_rate_hz: row.dsd_rate_hz.and_then(|v| u32::try_from(v).ok()),
        dsd_channels: row.dsd_channels.and_then(|v| u16::try_from(v).ok()),
    })
}

fn emit_playback_state(app: &tauri::AppHandle, engine: &Arc<Mutex<audio_engine::EngineState>>) {
    let engine = engine.lock();

    let (state, play_id, track_id) = if let Some(session) = &engine.session {
        let state = match engine.state {
            audio_engine::PlaybackState::Playing => "playing",
            audio_engine::PlaybackState::Paused => "paused",
            audio_engine::PlaybackState::Stopped => "stopped",
        };

        (
            state.to_string(),
            Some(session.play_id.to_string()),
            Some(session.track_id),
        )
    } else {
        ("stopped".to_string(), None, None)
    };

    let _ = app.emit(
        "evt_playback_state",
        PlaybackStateEvent {
            state,
            play_id,
            track_id,
        },
    );
}

fn emit_now_playing(app: &tauri::AppHandle, engine: &Arc<Mutex<audio_engine::EngineState>>) {
    let engine = engine.lock();
    let Some(session) = &engine.session else {
        return;
    };

    let track = TrackEventData {
        id: session.track.id,
        title: session.track.title.clone(),
        artist: session.track.artist.clone(),
        album: session.track.album.clone(),
        duration_ms: session.track.duration_ms,
        sample_rate: session.track.sample_rate,
        bit_depth: session.track.bit_depth,
        channels: session.track.channels,
        codec: session.track.codec.clone(),
        container: session.track.container.clone(),
        dsd_rate_hz: session.track.dsd_rate_hz,
        dsd_channels: session.track.dsd_channels,
    };

    let _ = app.emit(
        "evt_now_playing",
        NowPlayingEvent {
            play_id: session.play_id.to_string(),
            track,
            position_ms: session.position_ms,
        },
    );
}

fn emit_queue_changed(app: &tauri::AppHandle, engine: &Arc<Mutex<audio_engine::EngineState>>) {
    let engine = engine.lock();

    let play_id = engine.session.as_ref().map(|s| s.play_id.to_string());
    let current_index = engine.queue.current_index();
    let queue = engine
        .queue
        .items()
        .iter()
        .map(|item| QueueItemData {
            track_id: item.track.id,
            title: item.track.title.clone(),
            artist: item.track.artist.clone(),
            album: item.track.album.clone(),
            duration_ms: item.track.duration_ms,
        })
        .collect();

    let _ = app.emit(
        "evt_queue_changed",
        QueueChangedEvent {
            play_id,
            current_index,
            queue,
        },
    );
}

fn emit_position_now(app: &tauri::AppHandle, engine: &Arc<Mutex<audio_engine::EngineState>>) {
    let engine = engine.lock();
    let Some(session) = &engine.session else {
        return;
    };

    let duration_ms = session.track.duration_ms.unwrap_or(0);

    let _ = app.emit(
        "evt_playback_position",
        PlaybackPositionEvent {
            play_id: session.play_id.to_string(),
            position_ms: session.position_ms,
            played_ms: session.played_ms,
            duration_ms,
        },
    );
}

fn emit_position_tick(app: &tauri::AppHandle, engine: &Arc<Mutex<audio_engine::EngineState>>) {
    {
        let mut engine = engine.lock();
        engine.update_time();
    }
    emit_position_now(app, engine);
}

const REASON_NO_OUTPUT_ACTIVE: &str = "no_output_active";
const REASON_OUTPUT_MODE_SHARED: &str = "output_mode_shared";
const REASON_EXCLUSIVE_INACTIVE: &str = "exclusive_inactive";
const REASON_POLICY_COMPATIBILITY: &str = "policy_compatibility";
const REASON_CONVERSION_SHARED_FALLBACK: &str = "conversion_shared_fallback";
const REASON_CONVERSION_PAD_16_TO_24: &str = "conversion_pad_16_to_24";
const REASON_CONVERSION_PAD_16_TO_32: &str = "conversion_pad_16_to_32";
const REASON_GAIN_SOFTWARE_APPLIED: &str = "gain_software_applied";
const REASON_GAIN_SOFTWARE_BYPASSED: &str = "gain_software_bypassed";
const REASON_FADE_ACTIVE: &str = "fade_active";
const REASON_SAMPLE_RATE_MISMATCH: &str = "sample_rate_mismatch";
const REASON_BIT_DEPTH_MISMATCH: &str = "bit_depth_mismatch";
const REASON_BIT_DEPTH_UNKNOWN: &str = "bit_depth_unknown";
const REASON_ASIO_RESAMPLER_ACTIVE: &str = "asio_resampler_active";
const REASON_DOP_DISABLED: &str = "dop_disabled";
const REASON_DOP_DEGRADED_DROPS: &str = "dop_degraded_drops";
const REASON_DOP_DEGRADED_CB_UNDERRUN: &str = "dop_degraded_cb_underrun";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SignalPathStage {
    Source,
    Decode,
    Resample,
    ChannelMap,
    Gain,
    Fade,
    Mixer,
    Transport,
    Device,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SignalPathStatus {
    Ok,
    TouchingBits,
    Unknown,
    Inactive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SignalPathCheck {
    stage: SignalPathStage,
    status: SignalPathStatus,
    reason_code: String,
    detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DopPayloadIntegrityStatus {
    Ok,
    Degraded,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DopIntegrity {
    status: DopPayloadIntegrityStatus,
    reason_code: String,
    detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IntegrityStatus {
    pcm_bit_perfect: String,
    dop_payload_integrity: DopIntegrity,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioTelemetryEvent {
    pub version: u32,
    pub timestamp_ms: u64,
    pub playback: TelemetryPlayback,
    pub device: TelemetryDevice,
    pub format: TelemetryFormat,
    pub stability: TelemetryStability,
    pub integrity: TelemetryIntegrity,
    pub signal_path_checks: Vec<TelemetrySignalPathCheck>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryPlayback {
    pub state: String,
    pub track_id: i64,
    pub is_dsd: bool,
    pub output_mode: String,
    pub policy: String,
    pub timing_mode: String,
    pub gain_mode: String,
    pub effective_volume_mode: String,
    pub fade_enabled: bool,
    pub fade_active: bool,
    pub conversion: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryDevice {
    pub device_id: String,
    pub device_name: String,
    pub exclusive_active: bool,
    pub backend: TelemetryBackend,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryBackend {
    pub kind: String,
    pub wasapi: Option<TelemetryWasapiBackend>,
    pub asio: Option<TelemetryAsioBackend>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryWasapiBackend {
    pub buffer_frames: u32,
    pub device_period_default_hns: i64,
    pub device_period_min_hns: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryAsioBackend {
    pub driver_name: String,
    pub buffer_size_frames: u32,
    pub sample_format: String,
    pub actual_sample_rate: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryFormat {
    pub decode: TelemetryDecodeFormat,
    pub output: TelemetryOutputFormat,
    pub resampler: TelemetryResamplerFormat,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryDecodeFormat {
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub channels: u16,
    pub codec: String,
    pub container: String,
    pub is_dsd: bool,
    pub dsd_rate_hz: u32,
    pub dop_rate_hz: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryOutputFormat {
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub valid_bits: u16,
    pub channels: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryResamplerFormat {
    pub active: bool,
    pub source_sample_rate: u32,
    pub output_sample_rate: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryStability {
    pub ring_buffer: TelemetryRingBufferStats,
    pub dop_ring_buffer: TelemetryRingBufferStats,
    pub asio: TelemetryAsioStats,
    pub recent_events: Vec<TelemetryRecentEvent>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryRingBufferStats {
    pub capacity_frames: usize,
    pub available_frames: usize,
    pub fill_percent: f32,
    pub underruns: TelemetryCounter,
    pub overflows: TelemetryCounter,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryCounter {
    pub track: u64,
    pub lifetime: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryAsioStats {
    pub callback_underruns: TelemetryCounter,
    pub dop_drops: TelemetryCounter,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryRecentEvent {
    pub ts_ms: u64,
    pub code: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryIntegrity {
    pub pcm_bit_perfect: TelemetryBitPerfect,
    pub dop_payload_integrity: TelemetryDopIntegrity,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryBitPerfect {
    pub status: String,
    pub reasons: Vec<String>,
    pub display: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryDopIntegrity {
    pub status: String,
    pub reasons: Vec<String>,
    pub display: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetrySignalPathCheck {
    pub stage: String,
    pub status: String,
    pub reason_code: String,
    pub detail: String,
}

fn signal_path_check(
    stage: SignalPathStage,
    status: SignalPathStatus,
    reason_code: Option<&str>,
    detail: Option<String>,
) -> SignalPathCheck {
    SignalPathCheck {
        stage,
        status,
        reason_code: reason_code.unwrap_or_default().to_string(),
        detail: detail.unwrap_or_default(),
    }
}

fn compute_signal_path_checks(playback: &AudioPlayback, track: Option<&TrackInfo>) -> Vec<SignalPathCheck> {
    let output = playback.output.as_ref();
    let exclusive_active = output.map(|o| o.is_exclusive()).unwrap_or(false);
    let track_sample_rate = track.and_then(|t| t.sample_rate);
    let track_bit_depth = track.and_then(|t| t.bit_depth);

    let source_check = if track.is_none() {
        signal_path_check(
            SignalPathStage::Source,
            SignalPathStatus::Unknown,
            Some(REASON_BIT_DEPTH_UNKNOWN),
            Some("No track info".to_string()),
        )
    } else {
        signal_path_check(SignalPathStage::Source, SignalPathStatus::Ok, None, None)
    };

    let decode_check = if track.is_some() && track_bit_depth.is_none() {
        signal_path_check(
            SignalPathStage::Decode,
            SignalPathStatus::Unknown,
            Some(REASON_BIT_DEPTH_UNKNOWN),
            Some("Bit depth unknown".to_string()),
        )
    } else if playback.conversion.as_deref() == Some("pad_16_to_24") {
        signal_path_check(
            SignalPathStage::Decode,
            SignalPathStatus::TouchingBits,
            Some(REASON_CONVERSION_PAD_16_TO_24),
            Some("Conversion: Zero-pad 16->24 (compat)".to_string()),
        )
    } else if playback.conversion.as_deref() == Some("pad_16_to_32") {
        signal_path_check(
            SignalPathStage::Decode,
            SignalPathStatus::TouchingBits,
            Some(REASON_CONVERSION_PAD_16_TO_32),
            Some("Conversion: Zero-pad 16->32 (compat)".to_string()),
        )
    } else if let (Some(track_bd), Some(output)) = (track_bit_depth, output) {
        if output.valid_bits() != track_bd && playback.conversion.is_none() {
            signal_path_check(
                SignalPathStage::Decode,
                SignalPathStatus::TouchingBits,
                Some(REASON_BIT_DEPTH_MISMATCH),
                Some(format!(
                    "Bit depth mismatch: {} vs {}",
                    track_bd,
                    output.valid_bits()
                )),
            )
        } else {
            signal_path_check(SignalPathStage::Decode, SignalPathStatus::Ok, None, None)
        }
    } else {
        signal_path_check(SignalPathStage::Decode, SignalPathStatus::Ok, None, None)
    };

    let resample_check = if playback.resampler.is_some() {
        let reason_code = if playback.output_mode == "asio" {
            REASON_ASIO_RESAMPLER_ACTIVE
        } else {
            REASON_SAMPLE_RATE_MISMATCH
        };
        let detail = if let Some(track_sr) = track_sample_rate {
            format!(
                "Sample rate mismatch: {} vs {}",
                track_sr, playback.output_sample_rate
            )
        } else {
            "Sample rate mismatch".to_string()
        };
        signal_path_check(
            SignalPathStage::Resample,
            SignalPathStatus::TouchingBits,
            Some(reason_code),
            Some(detail),
        )
    } else if let Some(track_sr) = track_sample_rate {
        if playback.output_sample_rate > 0 && track_sr != playback.output_sample_rate {
            signal_path_check(
                SignalPathStage::Resample,
                SignalPathStatus::TouchingBits,
                Some(REASON_SAMPLE_RATE_MISMATCH),
                Some(format!(
                    "Sample rate mismatch: {} vs {}",
                    track_sr, playback.output_sample_rate
                )),
            )
        } else {
            signal_path_check(SignalPathStage::Resample, SignalPathStatus::Ok, None, None)
        }
    } else if track.is_some() {
        signal_path_check(
            SignalPathStage::Resample,
            SignalPathStatus::Unknown,
            None,
            Some("Sample rate unknown".to_string()),
        )
    } else {
        signal_path_check(SignalPathStage::Resample, SignalPathStatus::Ok, None, None)
    };

    let channel_map_check = signal_path_check(SignalPathStage::ChannelMap, SignalPathStatus::Ok, None, None);

    let unity_forced = exclusive_active && playback.policy == "strict";
    let gain_check = if playback.gain_mode == "software" {
        if unity_forced {
            signal_path_check(
                SignalPathStage::Gain,
                SignalPathStatus::Ok,
                Some(REASON_GAIN_SOFTWARE_BYPASSED),
                Some("Gain: Software volume bypassed (exclusive strict)".to_string()),
            )
        } else {
            signal_path_check(
                SignalPathStage::Gain,
                SignalPathStatus::TouchingBits,
                Some(REASON_GAIN_SOFTWARE_APPLIED),
                Some("Gain: Software volume".to_string()),
            )
        }
    } else {
        signal_path_check(SignalPathStage::Gain, SignalPathStatus::Ok, None, None)
    };

    let fade_check = if playback.fade_enabled && playback.fade_state.is_some() {
        signal_path_check(
            SignalPathStage::Fade,
            SignalPathStatus::TouchingBits,
            Some(REASON_FADE_ACTIVE),
            Some("Fade enabled (not bit-perfect during fade window)".to_string()),
        )
    } else {
        signal_path_check(SignalPathStage::Fade, SignalPathStatus::Ok, None, None)
    };

    let mixer_check = if playback.output_mode == "shared" {
        signal_path_check(
            SignalPathStage::Mixer,
            SignalPathStatus::TouchingBits,
            Some(REASON_OUTPUT_MODE_SHARED),
            Some("Output mode: Shared fallback (Windows SRC)".to_string()),
        )
    } else if playback.conversion.as_deref() == Some("shared_fallback") {
        signal_path_check(
            SignalPathStage::Mixer,
            SignalPathStatus::TouchingBits,
            Some(REASON_CONVERSION_SHARED_FALLBACK),
            Some("Output mode: Shared fallback (Windows SRC)".to_string()),
        )
    } else {
        signal_path_check(SignalPathStage::Mixer, SignalPathStatus::Ok, None, None)
    };

    let transport_check = if playback.policy == "compatibility" {
        signal_path_check(
            SignalPathStage::Transport,
            SignalPathStatus::TouchingBits,
            Some(REASON_POLICY_COMPATIBILITY),
            Some("Policy: Compatibility".to_string()),
        )
    } else {
        signal_path_check(SignalPathStage::Transport, SignalPathStatus::Ok, None, None)
    };

    let device_check = if output.is_none() {
        signal_path_check(
            SignalPathStage::Device,
            SignalPathStatus::Inactive,
            Some(REASON_NO_OUTPUT_ACTIVE),
            Some("No output active".to_string()),
        )
    } else if playback.output_mode != "shared" && !exclusive_active {
        signal_path_check(
            SignalPathStage::Device,
            SignalPathStatus::TouchingBits,
            Some(REASON_EXCLUSIVE_INACTIVE),
            Some("Exclusive mode inactive".to_string()),
        )
    } else {
        signal_path_check(SignalPathStage::Device, SignalPathStatus::Ok, None, None)
    };

    vec![
        source_check,
        decode_check,
        resample_check,
        channel_map_check,
        gain_check,
        fade_check,
        mixer_check,
        transport_check,
        device_check,
    ]
}

fn derive_pcm_bit_perfect(
    playback: &AudioPlayback,
    checks: &[SignalPathCheck],
) -> (String, String) {
    let exclusive_active = playback.output.as_ref().map(|o| o.is_exclusive()).unwrap_or(false);
    let all_relevant_ok = checks
        .iter()
        .filter(|check| check.status != SignalPathStatus::Inactive)
        .all(|check| check.status == SignalPathStatus::Ok);

    if exclusive_active && all_relevant_ok {
        return ("yes".to_string(), "".to_string());
    }

    let reason = checks
        .iter()
        .find_map(|check| {
            let should_report = match check.status {
                SignalPathStatus::TouchingBits | SignalPathStatus::Unknown => true,
                SignalPathStatus::Inactive => !check.reason_code.is_empty(),
                SignalPathStatus::Ok => false,
            };

            if !should_report {
                return None;
            }

            let detail = check.detail.trim();
            if !detail.is_empty() {
                Some(detail.to_string())
            } else if !check.reason_code.is_empty() {
                Some(check.reason_code.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            if !exclusive_active {
                "Output not exclusive".to_string()
            } else {
                "Signal path not bit-perfect".to_string()
            }
        });

    ("no".to_string(), reason)
}

fn compute_dop_payload_integrity(playback: &AudioPlayback) -> DopIntegrity {
    if !playback.is_dsd_playback {
        return DopIntegrity {
            status: DopPayloadIntegrityStatus::Unknown,
            reason_code: String::new(),
            detail: "Not DSD playback".to_string(),
        };
    }

    if !playback.dsd_dop_enabled {
        return DopIntegrity {
            status: DopPayloadIntegrityStatus::Degraded,
            reason_code: REASON_DOP_DISABLED.to_string(),
            detail: "DoP disabled".to_string(),
        };
    }

    let dop_active = playback.dop_ring_buffer.is_some() && playback.dop_packer.is_some();
    if !dop_active {
        return DopIntegrity {
            status: DopPayloadIntegrityStatus::Unknown,
            reason_code: String::new(),
            detail: "DoP path inactive".to_string(),
        };
    }

    let mut drops = 0u64;
    let mut cb_underruns = 0u64;

    if let Some(rb) = playback.dop_ring_buffer.as_ref() {
        drops = drops.saturating_add(rb.overflow_count());
        cb_underruns = cb_underruns.saturating_add(rb.underrun_count());
    }

    if let Some(output) = playback.output.as_ref() {
        if let Some(count) = output.asio_dop_drops() {
            drops = drops.saturating_add(count);
        }
        if let Some(count) = output.asio_callback_underruns() {
            cb_underruns = cb_underruns.saturating_add(count);
        }
    }

    if drops > 0 {
        return DopIntegrity {
            status: DopPayloadIntegrityStatus::Degraded,
            reason_code: REASON_DOP_DEGRADED_DROPS.to_string(),
            detail: format!("DoP drops detected: {}", drops),
        };
    }

    if cb_underruns > 0 {
        return DopIntegrity {
            status: DopPayloadIntegrityStatus::Degraded,
            reason_code: REASON_DOP_DEGRADED_CB_UNDERRUN.to_string(),
            detail: format!("DoP callback underruns: {}", cb_underruns),
        };
    }

    DopIntegrity {
        status: DopPayloadIntegrityStatus::Ok,
        reason_code: String::new(),
        detail: "DoP payload intact".to_string(),
    }
}

fn emit_audio_debug(
    app: &tauri::AppHandle,
    engine: &Arc<Mutex<audio_engine::EngineState>>,
    playback: &AudioPlayback,
    device: Option<&audio_engine::device::AudioDeviceInfo>,
) {
    let engine = engine.lock();
    let session = engine.session.as_ref();
    let track = session.map(|s| &s.track);

    let decode_format = AudioFormatData {
        sample_rate: track.and_then(|t| t.sample_rate).unwrap_or(0),
        bit_depth: track.and_then(|t| t.bit_depth).unwrap_or(0),
        channels: track.and_then(|t| t.channels).unwrap_or(0),
        codec: track.and_then(|t| t.codec.clone()),
        container: track.and_then(|t| t.container.clone()),
        valid_bits: None, // Decode format doesn't distinguish valid bits
        is_dsd: track.and_then(|t| t.dsd_rate_hz).is_some(),
        dsd_rate_hz: track.and_then(|t| t.dsd_rate_hz),
        dop_rate_hz: track.and_then(|t| t.dsd_rate_hz).map(|r| r / 16),
    };

    let output_format = AudioFormatData {
        sample_rate: playback.output_sample_rate,
        bit_depth: if let Some(o) = &playback.output {
            o.bit_depth()
        } else {
            0
        },
        channels: playback.output_channels,
        codec: None,
        container: playback.conversion.clone(),
        valid_bits: playback.output.as_ref().map(|o| o.valid_bits()),
        is_dsd: playback.is_dsd_playback,
        dsd_rate_hz: if playback.is_dsd_playback {
            track.and_then(|t| t.dsd_rate_hz)
        } else {
            None
        },
        dop_rate_hz: if playback.is_dsd_playback {
            Some(playback.output_sample_rate)
        } else {
            None
        },
    };

    let (device_id, device_name) = device
        .map(|d| (d.id.clone(), d.name.clone()))
        .unwrap_or_else(|| ("default".to_string(), "Default".to_string()));

    let exclusive_active = playback
        .output
        .as_ref()
        .map(|o| o.is_exclusive())
        .unwrap_or(false);

    let signal_checks = compute_signal_path_checks(playback, track);
    let (bit_perfect, bit_perfect_reason) = derive_pcm_bit_perfect(playback, &signal_checks);
    let _integrity_status = IntegrityStatus {
        pcm_bit_perfect: bit_perfect.clone(),
        dop_payload_integrity: compute_dop_payload_integrity(playback),
    };

    let _ = app.emit(
        "evt_audio_debug",
        AudioDebugEvent {
            output_mode: playback.output_mode.clone(),
            policy: playback.policy.clone(),
            conversion: playback
                .conversion
                .clone()
                .unwrap_or_else(|| "none".to_string()),
            gain_mode: playback.gain_mode.clone(),
            fade_enabled: playback.fade_enabled,
            exclusive_active,
            bit_perfect,
            bit_perfect_reason,
            device_id,
            device_name,
            output_format,
            decode_format,
        },
    );
}

fn build_telemetry_snapshot(
    engine: &Arc<Mutex<audio_engine::EngineState>>,
    playback: &AudioPlayback,
    device: Option<&audio_engine::device::AudioDeviceInfo>,
) -> AudioTelemetryEvent {
    let engine_guard = engine.lock();
    let session = engine_guard.session.as_ref();
    let track = session.map(|s| &s.track);

    let state_str = match engine_guard.state {
        PlaybackState::Playing => "playing",
        PlaybackState::Paused => "paused",
        PlaybackState::Stopped => "stopped",
    };

    let track_id = session.map(|s| s.track_id).unwrap_or(0);

    let exclusive_active = playback
        .output
        .as_ref()
        .map(|o| o.is_exclusive())
        .unwrap_or(false);

    let unity_forced = exclusive_active && playback.policy == "strict";
    let effective_volume_mode = if unity_forced {
        "unity_forced"
    } else if playback.gain_mode == "unity" {
        "unity"
    } else {
        "scaled"
    };

    let telemetry_playback = TelemetryPlayback {
        state: state_str.to_string(),
        track_id,
        is_dsd: playback.is_dsd_playback,
        output_mode: playback.output_mode.clone(),
        policy: playback.policy.clone(),
        timing_mode: playback.timing_mode.clone(),
        gain_mode: playback.gain_mode.clone(),
        effective_volume_mode: effective_volume_mode.to_string(),
        fade_enabled: playback.fade_enabled,
        fade_active: playback.fade_state.is_some(),
        conversion: playback
            .conversion
            .clone()
            .unwrap_or_else(|| "none".to_string()),
    };

    let (device_id, device_name) = device
        .map(|d| (d.id.clone(), d.name.clone()))
        .unwrap_or_else(|| ("default".to_string(), "Default".to_string()));

    let backend_kind = if playback.output_mode == "asio" {
        "asio"
    } else {
        "wasapi"
    };

    let wasapi_backend = if backend_kind == "wasapi" {
        Some(TelemetryWasapiBackend {
            buffer_frames: playback
                .output
                .as_ref()
                .map(|o| o.buffer_frames())
                .unwrap_or(0),
            device_period_default_hns: 0,
            device_period_min_hns: 0,
        })
    } else {
        None
    };

    let asio_backend = if backend_kind == "asio" {
        Some(TelemetryAsioBackend {
            driver_name: playback.asio_driver.clone().unwrap_or_default(),
            buffer_size_frames: playback
                .output
                .as_ref()
                .map(|o| o.buffer_frames())
                .unwrap_or(0),
            sample_format: playback
                .output
                .as_ref()
                .map(|o| o.sample_format_name())
                .unwrap_or_else(|| "unknown".to_string()),
            actual_sample_rate: playback.output_sample_rate,
        })
    } else {
        None
    };

    let telemetry_device = TelemetryDevice {
        device_id,
        device_name,
        exclusive_active,
        backend: TelemetryBackend {
            kind: backend_kind.to_string(),
            wasapi: wasapi_backend,
            asio: asio_backend,
        },
    };

    let decode_format = TelemetryDecodeFormat {
        sample_rate: track.and_then(|t| t.sample_rate).unwrap_or(0),
        bit_depth: track.and_then(|t| t.bit_depth).unwrap_or(0),
        channels: track.and_then(|t| t.channels).unwrap_or(0),
        codec: track.and_then(|t| t.codec.clone()).unwrap_or_default(),
        container: track.and_then(|t| t.container.clone()).unwrap_or_default(),
        is_dsd: track.and_then(|t| t.dsd_rate_hz).is_some(),
        dsd_rate_hz: track.and_then(|t| t.dsd_rate_hz).unwrap_or(0),
        dop_rate_hz: track.and_then(|t| t.dsd_rate_hz).map(|r| r / 16).unwrap_or(0),
    };

    let output_format = TelemetryOutputFormat {
        sample_rate: playback.output_sample_rate,
        bit_depth: playback.output.as_ref().map(|o| o.bit_depth()).unwrap_or(0),
        valid_bits: playback.output.as_ref().map(|o| o.valid_bits()).unwrap_or(0),
        channels: playback.output_channels,
    };

    let resampler_format = TelemetryResamplerFormat {
        active: playback.resampler.is_some(),
        source_sample_rate: playback.source_sample_rate,
        output_sample_rate: if playback.resampler.is_some() {
            playback.output_sample_rate
        } else {
            0
        },
    };

    let ring_buffer_stats = if let Some(rb) = playback.ring_buffer.as_ref() {
        TelemetryRingBufferStats {
            capacity_frames: rb.capacity_frames(),
            available_frames: rb.available_frames(),
            fill_percent: rb.fill_percent(),
            underruns: TelemetryCounter {
                track: rb.underrun_count(),
                lifetime: rb.underrun_count(),
            },
            overflows: TelemetryCounter {
                track: rb.overflow_count(),
                lifetime: rb.overflow_count(),
            },
        }
    } else {
        TelemetryRingBufferStats {
            capacity_frames: 0,
            available_frames: 0,
            fill_percent: 0.0,
            underruns: TelemetryCounter { track: 0, lifetime: 0 },
            overflows: TelemetryCounter { track: 0, lifetime: 0 },
        }
    };

    let dop_ring_buffer_stats = if let Some(rb) = playback.dop_ring_buffer.as_ref() {
        TelemetryRingBufferStats {
            capacity_frames: rb.capacity_frames(),
            available_frames: rb.available_frames(),
            fill_percent: rb.fill_percent(),
            underruns: TelemetryCounter {
                track: rb.underrun_count(),
                lifetime: rb.underrun_count(),
            },
            overflows: TelemetryCounter {
                track: rb.overflow_count(),
                lifetime: rb.overflow_count(),
            },
        }
    } else {
        TelemetryRingBufferStats {
            capacity_frames: 0,
            available_frames: 0,
            fill_percent: 0.0,
            underruns: TelemetryCounter { track: 0, lifetime: 0 },
            overflows: TelemetryCounter { track: 0, lifetime: 0 },
        }
    };

    let asio_stats = TelemetryAsioStats {
        callback_underruns: TelemetryCounter {
            track: playback.output.as_ref().and_then(|o| o.asio_callback_underruns()).unwrap_or(0),
            lifetime: playback.output.as_ref().and_then(|o| o.asio_callback_underruns()).unwrap_or(0),
        },
        dop_drops: TelemetryCounter {
            track: playback.output.as_ref().and_then(|o| o.asio_dop_drops()).unwrap_or(0),
            lifetime: playback.output.as_ref().and_then(|o| o.asio_dop_drops()).unwrap_or(0),
        },
    };

    let signal_checks = compute_signal_path_checks(playback, track);
    let (bit_perfect_status, bit_perfect_reason) = derive_pcm_bit_perfect(playback, &signal_checks);
    let dop_integrity = compute_dop_payload_integrity(playback);

    let telemetry_signal_checks: Vec<TelemetrySignalPathCheck> = signal_checks
        .iter()
        .map(|check| TelemetrySignalPathCheck {
            stage: format!("{:?}", check.stage).to_lowercase(),
            status: match check.status {
                SignalPathStatus::Ok => "ok".to_string(),
                SignalPathStatus::TouchingBits => "touching_bits".to_string(),
                SignalPathStatus::Unknown => "unknown".to_string(),
                SignalPathStatus::Inactive => "inactive".to_string(),
            },
            reason_code: check.reason_code.clone(),
            detail: check.detail.clone(),
        })
        .collect();

    let dop_status_str = match dop_integrity.status {
        DopPayloadIntegrityStatus::Ok => "ok",
        DopPayloadIntegrityStatus::Degraded => "degraded",
        DopPayloadIntegrityStatus::Unknown => "unknown",
    };

    let dop_reasons = if dop_integrity.reason_code.is_empty() {
        vec![]
    } else {
        vec![dop_integrity.reason_code.clone()]
    };

    let bit_perfect_reasons = if bit_perfect_reason.is_empty() {
        vec![]
    } else {
        vec![bit_perfect_reason.clone()]
    };

    AudioTelemetryEvent {
        version: 1,
        timestamp_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        playback: telemetry_playback,
        device: telemetry_device,
        format: TelemetryFormat {
            decode: decode_format,
            output: output_format,
            resampler: resampler_format,
        },
        stability: TelemetryStability {
            ring_buffer: ring_buffer_stats,
            dop_ring_buffer: dop_ring_buffer_stats,
            asio: asio_stats,
            recent_events: vec![],
        },
        integrity: TelemetryIntegrity {
            pcm_bit_perfect: TelemetryBitPerfect {
                status: bit_perfect_status.clone(),
                reasons: bit_perfect_reasons,
                display: if bit_perfect_status == "yes" {
                    "Bit-perfect".to_string()
                } else {
                    bit_perfect_reason.clone()
                },
            },
            dop_payload_integrity: TelemetryDopIntegrity {
                status: dop_status_str.to_string(),
                reasons: dop_reasons,
                display: dop_integrity.detail.clone(),
            },
        },
        signal_path_checks: telemetry_signal_checks,
    }
}

fn emit_audio_telemetry(
    app: &tauri::AppHandle,
    engine: &Arc<Mutex<audio_engine::EngineState>>,
    playback: &AudioPlayback,
    device: Option<&audio_engine::device::AudioDeviceInfo>,
) {
    let telemetry = build_telemetry_snapshot(engine, playback, device);
    let _ = app.emit("evt_audio_telemetry", telemetry);
}

#[cfg(test)]
mod tests {
    use super::*;
    use audio_engine::output::{NullSinkOutput, OutputBackend};

    #[test]
    fn gain_software_bypassed_when_unity_forced() {
        let mut playback = AudioPlayback::new();
        playback.output_mode = "asio".to_string();
        playback.policy = "strict".to_string();
        playback.gain_mode = "software".to_string();
        playback.fade_enabled = false;
        playback.fade_state = None;
        playback.output_sample_rate = 44_100;
        playback.output_channels = 2;
        playback.output = Some(OutputBackend::NullSink(NullSinkOutput::new(44_100, 2, 24, true)));

        let track = TrackInfo {
            id: 1,
            path: "test".to_string(),
            title: None,
            artist: None,
            album: None,
            duration_ms: None,
            sample_rate: Some(44_100),
            bit_depth: Some(24),
            channels: Some(2),
            codec: None,
            container: None,
            dsd_rate_hz: None,
            dsd_channels: None,
        };

        let checks = compute_signal_path_checks(&playback, Some(&track));
        let gain_check = checks
            .iter()
            .find(|check| check.stage == SignalPathStage::Gain)
            .expect("gain stage check should exist");

        assert_eq!(gain_check.status, SignalPathStatus::Ok);
        assert_eq!(gain_check.reason_code, REASON_GAIN_SOFTWARE_BYPASSED);

        let (bit_perfect, reason) = derive_pcm_bit_perfect(&playback, &checks);
        assert_eq!(bit_perfect, "yes");
        assert!(reason.is_empty());
    }
}

fn parse_playback_error(e: &str) -> (String, String) {
    if let Some(msg) = e.strip_prefix("bit_depth_unknown:") {
        return ("bit_depth_unknown".to_string(), msg.trim().to_string());
    }
    if let Some(msg) = e.strip_prefix("exclusive_unavailable:") {
        return ("exclusive_unavailable".to_string(), msg.trim().to_string());
    }
    if let Some(msg) = e.strip_prefix("exclusive_unsupported_format:") {
        return (
            "exclusive_unsupported_format".to_string(),
            msg.trim().to_string(),
        );
    }
    if let Some(msg) = e.strip_prefix("dop_disabled:") {
        return ("dop_disabled".to_string(), msg.trim().to_string());
    }
    if let Some(msg) = e.strip_prefix("dop_unsupported_format:") {
        return ("dop_unsupported_format".to_string(), msg.trim().to_string());
    }
    if let Some(msg) = e.strip_prefix("dop_conversion_unavailable:") {
        return (
            "dop_conversion_unavailable".to_string(),
            msg.trim().to_string(),
        );
    }
    if let Some(msg) = e.strip_prefix("dop_dst_unsupported:") {
        return ("dop_dst_unsupported".to_string(), msg.trim().to_string());
    }
    ("decode_error".to_string(), e.to_string())
}

fn emit_playback_error(
    app: &tauri::AppHandle,
    code: &str,
    message: &str,
    track_id: Option<i64>,
    recoverable: bool,
    action: Option<&str>,
) {
    error!(code, message, "playback error");
    let _ = app.emit(
        "evt_playback_error",
        PlaybackErrorEvent {
            code: code.to_string(),
            message: message.to_string(),
            track_id,
            recoverable,
            action: action.map(|v| v.to_string()),
        },
    );
}
