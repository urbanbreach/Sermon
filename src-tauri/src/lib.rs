mod commands;
mod state;

use audio_engine::decode::AudioDecoder;
use audio_engine::device::{get_default_device, get_device_by_id};
use audio_engine::output::{
    AudioOutput, AudioRingBuffer, OutputBackend, WasapiOutput, convert_channels_interleaved_f32,
};
use audio_engine::{PlaybackState, TrackInfo};
use commands::{
    AudioDebugEvent, AudioFormatData, DeviceChangedEvent, NowPlayingEvent, PlaybackErrorEvent,
    PlaybackPositionEvent, PlaybackStateEvent, QueueChangedEvent, QueueItemData, TrackEventData,
    cmd_artwork_embed_to_file, cmd_artwork_extract_embedded, cmd_artwork_find_folder,
    cmd_artwork_get_best_for_album, cmd_artwork_get_best_for_track, cmd_artwork_get_bytes,
    cmd_artwork_search_candidates, cmd_artwork_select_candidate_for_album, cmd_library_add_folder,
    cmd_library_get_raw_tags, cmd_library_get_stats, cmd_library_get_track_by_id, cmd_library_list_album_tracks_page,
    cmd_library_list_albums_page, cmd_library_list_artist_tracks_page,
    cmd_library_list_artists_page, cmd_library_list_folders, cmd_library_list_tracks,
    cmd_library_list_tracks_page, cmd_library_remove_folder, cmd_library_update_folder_enabled, cmd_library_update_folder_options,
    cmd_library_get_folder_track_count, cmd_list_asio_drivers, cmd_open_asio_control_panel,
    cmd_output_get_settings, cmd_output_list_devices, cmd_output_set_device,
    cmd_output_set_settings, cmd_playback_next, cmd_playback_pause, cmd_playback_previous,
    cmd_playback_resume, cmd_playback_seek, cmd_playback_start, cmd_playback_stop, cmd_queue_add,
    cmd_queue_play_now, cmd_queue_set_and_play, cmd_scan_start, cmd_settings_export_diagnostics, cmd_settings_get,
    cmd_settings_get_category, cmd_settings_reset_category, cmd_settings_set,
    cmd_settings_set_category, cmd_volume_get, cmd_volume_set, cmd_waveform_get_peaks,
};
use crossbeam_channel::{Receiver, select, tick, unbounded};
use parking_lot::Mutex;
use state::{ArtworkCacheState, AudioState, LibraryState, PlaybackCommand, WaveformCacheState};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Listener, Manager};
use tracing::{Level, debug, error, info, warn};
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
    let _guard = init_tracing();

    let mut builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
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

            // Setup artwork cache directory
            let artwork_cache_dir = app_data_dir.join("artwork-cache");
            fs::create_dir_all(&artwork_cache_dir)
                .expect("Failed to create artwork cache directory");
            info!("Artwork cache path: {:?}", artwork_cache_dir);
            app.manage(ArtworkCacheState::new(artwork_cache_dir));

            // Setup waveform cache directory
            let waveform_cache_dir = app_data_dir.join("waveform-cache");
            fs::create_dir_all(&waveform_cache_dir)
                .expect("Failed to create waveform cache directory");
            info!("Waveform cache path: {:?}", waveform_cache_dir);
            app.manage(WaveformCacheState::new(waveform_cache_dir));

            // Initialize database
            let conn = library::open_db(&db_path).expect("Failed to open database");
            library::apply_migrations(&conn).expect("Failed to apply migrations");
            drop(conn);

            // Register state
            app.manage(LibraryState::new(db_path.clone()));

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

            spawn_audio_thread(app.handle().clone(), db_path, engine, command_rx);

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
    output: Option<OutputBackend>,
    ring_buffer: Option<AudioRingBuffer>,
    device_id: String, // "default" or specific device ID
    output_sample_rate: u32,
    output_channels: u16,
    end_of_track: bool,         // Track if decoder has finished
    output_mode: String,        // "exclusive" or "shared" or "asio"
    policy: String,             // "strict" or "compatibility"
    gain_mode: String,          // "unity" or "software"
    conversion: Option<String>, // None, "pad_16_to_24", "shared_fallback"
    fade_enabled: bool,
    fade_state: Option<FadeState>,
    timing_mode: String, // "event" or "polling"
    buffer_size_ms: u32, // Ring buffer size in milliseconds (from preferences)
    // DSD playback state
    is_dsd_playback: bool,
    dsd_dop_enabled: bool,
    dsd_dop_strict: bool,
    dop_ring_buffer: Option<audio_engine::output::DopRingBuffer>,
    dsd_decoder: Option<audio_engine::DsdDecoder>,
    dop_packer: Option<audio_engine::DopPacker>,
    // ASIO state
    asio_driver: Option<String>,
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
            output: None,
            ring_buffer: None,
            device_id: "default".to_string(),
            output_sample_rate: 0,
            output_channels: 0,
            end_of_track: false,
            output_mode: "shared".to_string(),
            policy: "compatibility".to_string(), // Default to compatibility
            gain_mode: "software".to_string(),   // Default to software volume
            conversion: None,
            fade_enabled: true, // Default to true
            fade_state: None,
            timing_mode: "polling".to_string(), // Default to polling for USB compatibility
            buffer_size_ms: 500, // Default buffer size, will be overwritten from settings
            is_dsd_playback: false,
            dsd_dop_enabled: false,
            dsd_dop_strict: true,
            dop_ring_buffer: None,
            dsd_decoder: None,
            dop_packer: None,
            asio_driver: None,
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
        // The determine_bit_perfect function will handle the "unknown" case

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

        if self.output_mode == "exclusive" {
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

    fn open_output_for_dop_format(
        &mut self,
        dsd_rate: u32,
        channels: u16,
    ) -> Result<(), String> {
        use audio_engine::device::{get_default_device, get_device_by_id};
        use audio_engine::dop::dop_sample_rate;
        use audio_engine::output::WasapiOutput;

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
            "Opening DoP output (exclusive 24-bit)"
        );

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
                    Err(format!("dop_conversion_unavailable: DSD->PCM conversion not implemented"))
                }
            }
        }
    }

    fn start_playback(&mut self, track: &TrackInfo) -> Result<(), String> {
        let track_path = Path::new(&track.path);

        if track.is_dsd() {
            return self.start_dsd_playback(track);
        }

        let decoder = AudioDecoder::open(track_path).map_err(|e| e.to_string())?;

        // Get decoder format
        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels() as u16;
        let bit_depth = decoder.bit_depth().map(|b| b as u16);

        info!(
            path = %track_path.display(),
            sample_rate = sample_rate,
            channels = channels,
            bit_depth = ?bit_depth,
            "Starting playback"
        );

        self.decoder = Some(decoder);
        self.end_of_track = false;

        // Open output with decoder's format (use track metadata bit_depth if decoder doesn't know)
        let output_bit_depth = bit_depth.or(track.bit_depth).unwrap_or(16);
        self.open_output_for_format(sample_rate, channels, output_bit_depth, track.bit_depth)?;

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
            "Format negotiation result"
        );

        // Create ring buffer sized for configured buffer duration
        // Use output_channels because fill_ring_buffer converts to output channels before pushing
        self.ring_buffer = Some(AudioRingBuffer::new(
            sample_rate,
            self.output_channels as usize,
            self.buffer_size_ms,
        ));

        // Start the output stream
        if let Some(ref mut output) = self.output {
            output.start().map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn start_dsd_playback(&mut self, track: &TrackInfo) -> Result<(), String> {
        use audio_engine::output::DopRingBuffer;
        use audio_engine::{DopPacker, DsdDecoder};

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
        self.decoder = None;
        self.ring_buffer = None;
        self.end_of_track = false;

        if let Some(ref mut output) = self.output {
            output.start().map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn stop_playback(&mut self) {
        self.decoder = None;
        self.ring_buffer = None;
        self.end_of_track = false;
        self.is_dsd_playback = false;
        self.dsd_decoder = None;
        self.dop_packer = None;
        self.dop_ring_buffer = None;
        if let Some(ref mut output) = self.output {
            let _ = output.stop();
        }
    }

    fn seek(&mut self, position_ms: u64) -> Result<(), String> {
        if let Some(ref mut decoder) = self.decoder {
            decoder.seek(position_ms).map_err(|e| e.to_string())?;
        }
        // Clear the ring buffer on seek to avoid stale audio
        if let Some(ref mut ring_buffer) = self.ring_buffer {
            ring_buffer.clear();
        }
        self.end_of_track = false;
        Ok(())
    }

    /// Fill the ring buffer with decoded samples. Returns false if track ended.
    fn fill_ring_buffer(&mut self) -> Result<bool, String> {
        let decoder = match self.decoder.as_mut() {
            Some(d) => d,
            None => return Ok(false),
        };

        let ring_buffer = match self.ring_buffer.as_mut() {
            Some(rb) => rb,
            None => return Ok(false),
        };

        let output_channels = self.output_channels as usize;

        // Decode packets until ring buffer is reasonably full or we hit end of track
        // Target: keep buffer at least 50% full
        let target_frames = ring_buffer.capacity_frames() / 2;

        while ring_buffer.available_frames() < target_frames {
            let samples = match decoder.decode_next() {
                Ok(Some(samples)) => samples,
                Ok(None) => {
                    // End of track
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

            // Channel conversion if needed
            let input_channels = decoder.channels();
            let converted = if input_channels != output_channels {
                convert_channels_interleaved_f32(&samples, input_channels, output_channels)
            } else {
                samples
            };

            ring_buffer.push(&converted);
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

        let samples_to_write = available * self.output_channels as usize;
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

        if self.decoder.is_none() {
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

        let mut playback = AudioPlayback::new();
        let mut current_device_info: Option<audio_engine::device::AudioDeviceInfo> = None;

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
            // Load buffer size from preferences (default 500ms)
            if let Ok(Some(buffer_str)) = library::get_setting(&conn, "player.buffer_size_ms") {
                if let Ok(buffer_ms) = buffer_str.parse::<u32>() {
                    // Clamp to valid range: 100-2000ms
                    playback.buffer_size_ms = buffer_ms.clamp(100, 2000);
                }
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
                    );
                }
                recv(position_tick) -> _ => {
                    emit_position_tick(&app, &engine);
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

                        match playback.process_audio(volume) {
                            Ok(true) => {
                                // Continue playing
                            }
                            Ok(false) => {
                                // Track ended - advance to next
                                info!("Track ended, advancing to next");
                                {
                                    let mut engine = engine.lock();
                                    engine.next();
                                }

                                // Check if there's a next track
                                let next_track = {
                                    let engine = engine.lock();
                                    engine.session.as_ref().map(|s| s.track.clone())
                                };

                                if let Some(track) = next_track {
                                    if let Err(e) = playback.start_playback(&track) {
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
                                    emit_now_playing(&app, &engine);
                                    emit_playback_state(&app, &engine);
                                    emit_queue_changed(&app, &engine);
                                } else {
                                    // Queue exhausted
                                    playback.stop_playback();
                                    emit_playback_state(&app, &engine);
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
) {
    match command {
        PlaybackCommand::PlayNow { track_id } => {
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
            if let Err(e) = playback.start_playback(&track) {
                error!("Failed to start playback: {}", e);
                let (code, msg) = parse_playback_error(&e);
                emit_playback_error(app, &code, &msg, Some(track_id), false, None);
                engine.lock().stop();
                emit_playback_state(app, engine);
                return;
            }

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
            let mut tracks = Vec::new();
            for track_id in &track_ids {
                if let Ok(track) = resolve_track(db_path, *track_id) {
                    tracks.push(track);
                }
            }

            if tracks.is_empty() {
                emit_playback_error(app, "no_valid_tracks", "No valid tracks to play", None, false, None);
                return;
            }

            let actual_start = start_index.min(tracks.len() - 1);
            let start_track = tracks[actual_start].clone();

            {
                let mut engine = engine.lock();
                engine.set_and_play(tracks, actual_start);
            }

            if let Err(e) = playback.start_playback(&start_track) {
                error!("Failed to start playback: {}", e);
                let (code, msg) = parse_playback_error(&e);
                emit_playback_error(app, &code, &msg, Some(start_track.id), false, None);
                engine.lock().stop();
                emit_playback_state(app, engine);
                return;
            }

            emit_now_playing(app, engine);
            emit_playback_state(app, engine);
            emit_queue_changed(app, engine);
            emit_audio_debug(app, engine, playback, current_device_info.as_ref());
        }
        PlaybackCommand::Pause => {
            engine.lock().pause();
            // Don't stop the output stream - just stop decoding
            emit_playback_state(app, engine);
        }
        PlaybackCommand::Resume => {
            engine.lock().resume();
            emit_playback_state(app, engine);
        }
        PlaybackCommand::Stop => {
            engine.lock().stop();
            playback.stop_playback();
            emit_playback_state(app, engine);
        }
        PlaybackCommand::Seek { position_ms } => {
            if let Err(e) = playback.seek(position_ms) {
                error!("Seek failed: {}", e);
            }
            engine.lock().seek(position_ms);
            emit_position_now(app, engine);
        }
        PlaybackCommand::Next => {
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
                if let Err(e) = playback.start_playback(&track) {
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
                if let Err(e) = playback.start_playback(&track) {
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
    }
}

fn resolve_track(db_path: &PathBuf, track_id: i64) -> Result<TrackInfo, String> {
    let conn = library::open_db(db_path).map_err(|e| e.to_string())?;
    library::apply_migrations(&conn).map_err(|e| e.to_string())?;
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

fn determine_bit_perfect(playback: &AudioPlayback, track: Option<&TrackInfo>) -> (String, String) {
    // Precedence order (first matching wins):
    // 1. Output mode: Shared fallback (Windows SRC)
    if playback.output_mode == "shared" {
        return (
            "no".to_string(),
            "Output mode: Shared fallback (Windows SRC)".to_string(),
        );
    }

    if let Some(c) = &playback.conversion {
        if c == "shared_fallback" {
            return (
                "no".to_string(),
                "Output mode: Shared fallback (Windows SRC)".to_string(),
            );
        }
    }

    // Check if we are actually in exclusive mode
    if let Some(output) = &playback.output {
        if !output.is_exclusive() {
            return ("no".to_string(), "Output mode: Shared".to_string());
        }
    } else {
        return ("no".to_string(), "No output active".to_string());
    }

    // 2. Policy: Compatibility
    if playback.policy == "compatibility" {
        return ("no".to_string(), "Policy: Compatibility".to_string());
    }

    // 3. Conversion: Zero-pad 16->24 (compat)
    if playback.conversion.as_deref() == Some("pad_16_to_24") {
        return (
            "no".to_string(),
            "Conversion: Zero-pad 16->24 (compat)".to_string(),
        );
    }

    // 4. Gain: Software volume
    if playback.gain_mode == "software" {
        return ("no".to_string(), "Gain: Software volume".to_string());
    }

    // 5. Fade enabled (not bit-perfect during fade window)
    if playback.fade_enabled && playback.fade_state.is_some() {
        return (
            "no".to_string(),
            "Fade enabled (not bit-perfect during fade window)".to_string(),
        );
    }

    // 6. Format mismatch (track vs output)
    if let Some(track) = track {
        if let Some(track_sr) = track.sample_rate {
            if track_sr != playback.output_sample_rate {
                return (
                    "no".to_string(),
                    format!(
                        "Sample rate mismatch: {} vs {}",
                        track_sr, playback.output_sample_rate
                    ),
                );
            }
        }

        // 7. Bit depth unknown
        if let Some(track_bd) = track.bit_depth {
            if let Some(output) = &playback.output {
                // Compare to valid_bits, not container bit_depth
                // This correctly handles 24-bit in 32-bit container
                if output.valid_bits() != track_bd && playback.conversion.is_none() {
                    return (
                        "no".to_string(),
                        format!(
                            "Bit depth mismatch: {} vs {}",
                            track_bd,
                            output.valid_bits()
                        ),
                    );
                }
            }
        } else {
            return ("no".to_string(), "Bit depth unknown".to_string());
        }
    } else {
        return ("no".to_string(), "No track info".to_string());
    }

    ("yes".to_string(), "".to_string())
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

    let (bit_perfect, bit_perfect_reason) = determine_bit_perfect(playback, track);

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
        return ("dop_conversion_unavailable".to_string(), msg.trim().to_string());
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
