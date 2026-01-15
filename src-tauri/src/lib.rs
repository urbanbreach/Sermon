mod commands;
mod state;

use audio_engine::decode::AudioDecoder;
use audio_engine::device::{get_default_device, get_device_by_id};
use audio_engine::output::{AudioRingBuffer, WasapiOutput, convert_channels_interleaved_f32};
use audio_engine::{PlaybackState, TrackInfo};
use commands::{
    AudioDebugEvent, AudioFormatData, DeviceChangedEvent, NowPlayingEvent, PlaybackErrorEvent,
    PlaybackPositionEvent, PlaybackStateEvent, QueueChangedEvent, QueueItemData, TrackEventData,
    cmd_library_add_folder, cmd_library_list_folders, cmd_library_list_tracks,
    cmd_output_list_devices, cmd_output_set_device, cmd_playback_next, cmd_playback_pause,
    cmd_playback_previous, cmd_playback_resume, cmd_playback_seek, cmd_playback_start,
    cmd_playback_stop, cmd_queue_add, cmd_queue_play_now, cmd_scan_start, cmd_volume_get,
    cmd_volume_set,
};
use crossbeam_channel::{Receiver, select, tick, unbounded};
use parking_lot::Mutex;
use state::{AudioState, LibraryState, PlaybackCommand};
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
            app.manage(LibraryState::new(db_path.clone()));

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
            cmd_output_list_devices,
            cmd_output_set_device,
            cmd_volume_get,
            cmd_volume_set,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Audio playback state held by the audio thread
struct AudioPlayback {
    decoder: Option<AudioDecoder>,
    output: Option<WasapiOutput>,
    ring_buffer: Option<AudioRingBuffer>,
    device_id: String, // "default" or specific device ID
    output_sample_rate: u32,
    output_channels: u16,
    end_of_track: bool, // Track if decoder has finished
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
        self.output = Some(output);

        info!(
            device_id = device_id,
            sample_rate = self.output_sample_rate,
            channels = self.output_channels,
            "Opened audio output device"
        );

        Ok(())
    }

    fn open_output_for_format(&mut self, sample_rate: u32, channels: u16) -> Result<(), String> {
        // Close existing output if format changed
        if let Some(ref mut output) = self.output {
            if output.sample_rate() != sample_rate || output.channels() != channels {
                let _ = output.stop();
                self.output = None;
            }
        }

        if self.output.is_some() {
            return Ok(()); // Already open with correct format
        }

        info!(
            sample_rate = sample_rate,
            channels = channels,
            "Opening WASAPI output for source format"
        );

        let output =
            WasapiOutput::open_with_format(sample_rate, channels).map_err(|e| e.to_string())?;

        self.output_sample_rate = output.sample_rate();
        self.output_channels = output.channels();
        self.output = Some(output);

        Ok(())
    }

    fn start_playback(&mut self, track_path: &Path) -> Result<(), String> {
        // Open decoder first to get its format
        let decoder = AudioDecoder::open(track_path).map_err(|e| e.to_string())?;

        // Get decoder format
        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels() as u16;

        info!(
            path = %track_path.display(),
            sample_rate = sample_rate,
            channels = channels,
            "Starting playback"
        );

        self.decoder = Some(decoder);
        self.end_of_track = false;

        // Open output with decoder's format (Windows will handle conversion)
        self.open_output_for_format(sample_rate, channels)?;

        // Create ring buffer sized for ~500ms of audio
        self.ring_buffer = Some(AudioRingBuffer::new(sample_rate, channels as usize, 500));

        // Start the output stream
        if let Some(ref mut output) = self.output {
            output.start().map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn stop_playback(&mut self) {
        self.decoder = None;
        self.ring_buffer = None;
        self.end_of_track = false;
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

    /// Process audio: fill buffer, then write to WASAPI. Returns true if playback should continue.
    fn process_audio(&mut self, volume: f32) -> Result<bool, String> {
        if self.decoder.is_none() {
            return Ok(false);
        }

        let output = match self.output.as_mut() {
            Some(o) => o,
            None => return Err("No output device".to_string()),
        };

        let ring_buffer = match self.ring_buffer.as_mut() {
            Some(rb) => rb,
            None => return Err("No ring buffer".to_string()),
        };

        // Write from ring buffer to WASAPI
        match output.write_from_buffer(ring_buffer, volume) {
            Ok(_frames_written) => {
                // Check if we're done (end of track and buffer empty)
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
        let position_tick = tick(Duration::from_millis(250));
        // Audio processing tick - run at ~10ms for smooth playback
        let audio_tick = tick(Duration::from_millis(10));

        let mut playback = AudioPlayback::new();
        let mut current_device_info: Option<audio_engine::device::AudioDeviceInfo> = None;

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
                        // First, fill the ring buffer with decoded samples
                        if let Err(e) = playback.fill_ring_buffer() {
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

                        // Then write from ring buffer to WASAPI
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
                                    let path = Path::new(&track.path);
                                    if let Err(e) = playback.start_playback(path) {
                                        error!("Failed to start next track: {}", e);
                                        emit_playback_error(&app, "decode_error", &e, Some(track.id), false, None);
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

            let track_path = track.path.clone();

            // Update engine state
            {
                let mut engine = engine.lock();
                engine.play_now(track);
            }

            // Start actual playback
            let path = Path::new(&track_path);
            if let Err(e) = playback.start_playback(path) {
                error!("Failed to start playback: {}", e);
                emit_playback_error(app, "decode_error", &e, Some(track_id), false, None);
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
                let path = Path::new(&track.path);
                if let Err(e) = playback.start_playback(path) {
                    error!("Failed to start next track: {}", e);
                    emit_playback_error(app, "decode_error", &e, Some(track.id), false, None);
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
                let path = Path::new(&track.path);
                if let Err(e) = playback.start_playback(path) {
                    error!("Failed to start previous track: {}", e);
                    emit_playback_error(app, "decode_error", &e, Some(track.id), false, None);
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

fn emit_audio_debug(
    app: &tauri::AppHandle,
    engine: &Arc<Mutex<audio_engine::EngineState>>,
    playback: &AudioPlayback,
    device: Option<&audio_engine::device::AudioDeviceInfo>,
) {
    let engine = engine.lock();
    let session = engine.session.as_ref();

    let decode_format = AudioFormatData {
        sample_rate: session.and_then(|s| s.track.sample_rate).unwrap_or(0),
        bit_depth: session.and_then(|s| s.track.bit_depth).unwrap_or(0),
        channels: session.and_then(|s| s.track.channels).unwrap_or(0),
        codec: session.and_then(|s| s.track.codec.clone()),
        container: session.and_then(|s| s.track.container.clone()),
    };

    let output_format = AudioFormatData {
        sample_rate: playback.output_sample_rate,
        bit_depth: 32, // WASAPI shared mode typically uses 32-bit float
        channels: playback.output_channels,
        codec: None,
        container: None,
    };

    let (device_id, device_name) = device
        .map(|d| (d.id.clone(), d.name.clone()))
        .unwrap_or_else(|| ("default".to_string(), "Default".to_string()));

    let _ = app.emit(
        "evt_audio_debug",
        AudioDebugEvent {
            output_mode: "shared".to_string(),
            device_id,
            device_name,
            output_format,
            decode_format,
        },
    );
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
