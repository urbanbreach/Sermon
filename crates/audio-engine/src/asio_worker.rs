//! ASIO worker thread with STA COM initialization
//!
//! Solves COM threading conflict: WASAPI uses MTA, ASIO SDK requires STA.
//! This module provides a dedicated worker thread that initializes COM as STA
//! and owns all ASIO driver operations.

use std::os::raw::c_void;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use asio_sys::{Asio, AsioSampleType, AsioStreams, CallbackId, Driver};
use parking_lot::Mutex;
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::HeapRb;
use tracing::{debug, error, info, trace, warn};
use windows_sys::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};

use crate::output::OutputError;

type RingProducer = ringbuf::HeapProd<f32>;
type RingConsumer = ringbuf::HeapCons<f32>;

/// Wrapper for raw buffer pointers that are Send+Sync.
/// SAFETY: The pointers are valid for the lifetime of the AsioStreams and are only
/// accessed from the ASIO callback thread which is the intended use case.
#[derive(Clone, Copy)]
struct SendPtr(*mut c_void);
unsafe impl Send for SendPtr {}
unsafe impl Sync for SendPtr {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
    Int32,
    Int32Lsb24,
    Float32,
}

impl SampleFormat {
    pub fn from_asio(st: &AsioSampleType) -> Option<Self> {
        match st {
            AsioSampleType::ASIOSTInt32LSB => Some(Self::Int32),
            AsioSampleType::ASIOSTInt32LSB24 => Some(Self::Int32Lsb24),
            AsioSampleType::ASIOSTFloat32LSB => Some(Self::Float32),
            _ => None,
        }
    }
}

/// Commands sent to the ASIO worker thread
#[derive(Debug)]
pub enum AsioCommand {
    /// Load the ASIO driver and prepare streams
    Load {
        driver_name: String,
        sample_rate: u32,
        channels: u16,
    },
    /// Start playback
    Start,
    /// Stop playback
    Stop,
    /// Open the ASIO driver's control panel
    OpenControlPanel,
    /// Shutdown the worker thread
    Shutdown,
}

/// Responses from the ASIO worker thread
#[derive(Debug)]
pub enum AsioResponse {
    /// Driver loaded successfully
    Loaded {
        buffer_size: i32,
        sample_format: SampleFormat,
        /// The actual sample rate the driver is running at (may differ from requested)
        actual_sample_rate: u32,
    },
    /// Playback started
    Started,
    /// Playback stopped
    Stopped,
    /// Control panel opened (fire-and-forget)
    ControlPanelOpened,
    /// Error occurred
    Error(String),
}

/// State shared between worker thread callback and producer
struct CallbackState {
    consumer: RingConsumer,
    sample_format: SampleFormat,
    channels: usize,
    buffer_size: i32,
    temp_buffer: Vec<f32>,
    buffer_ptrs: Vec<[SendPtr; 2]>,
    callback_underruns: Arc<AtomicU64>,
}

/// Handle to the ASIO worker thread
pub struct AsioWorker {
    cmd_tx: Sender<AsioCommand>,
    resp_rx: Receiver<AsioResponse>,
    handle: Option<JoinHandle<()>>,
    /// Ring buffer producer for main thread to push samples
    producer: Option<RingProducer>,
    /// Shared callback state
    callback_state: Arc<Mutex<Option<CallbackState>>>,
    /// Callback underrun counter (callback reads less than needed, zero-fills)
    callback_underruns: Arc<AtomicU64>,
    /// DoP sample drop counter (ring buffer full when pushing DoP)
    dop_drops: AtomicU64,
}

impl AsioWorker {
    /// Create a new ASIO worker with dedicated STA COM thread.
    ///
    /// The worker thread initializes COM as STA before any ASIO operations.
    pub fn new(driver_name: String, sample_rate: u32, channels: u16) -> Result<Self, OutputError> {
        let (cmd_tx, cmd_rx) = mpsc::channel::<AsioCommand>();
        let (resp_tx, resp_rx) = mpsc::channel::<AsioResponse>();

        // Create ring buffer: ~500ms of audio at sample_rate * channels
        let buffer_samples = (sample_rate as usize * channels as usize) / 2;
        let rb = HeapRb::<f32>::new(buffer_samples);
        let (producer, consumer) = rb.split();

        let callback_underruns = Arc::new(AtomicU64::new(0));

        let callback_state = Arc::new(Mutex::new(Some(CallbackState {
            consumer,
            sample_format: SampleFormat::Int32,
            channels: channels as usize,
            buffer_size: 0,
            temp_buffer: Vec::new(),
            buffer_ptrs: Vec::new(),
            callback_underruns: Arc::clone(&callback_underruns),
        })));

        let callback_state_clone = Arc::clone(&callback_state);
        let driver_name_clone = driver_name.clone();

        // Spawn dedicated worker thread with STA COM initialization
        let handle = thread::Builder::new()
            .name("asio-worker".to_string())
            .spawn(move || {
                Self::worker_loop(cmd_rx, resp_tx, callback_state_clone, driver_name_clone);
            })
            .map_err(|e| OutputError::Asio(format!("Failed to spawn ASIO worker thread: {}", e)))?;

        info!(
            driver = driver_name,
            sample_rate = sample_rate,
            channels = channels,
            buffer_samples = buffer_samples,
            "Created ASIO worker thread"
        );

        Ok(Self {
            cmd_tx,
            resp_rx,
            handle: Some(handle),
            producer: Some(producer),
            callback_state,
            callback_underruns,
            dop_drops: AtomicU64::new(0),
        })
    }

    /// Worker thread main loop - runs on dedicated STA COM thread
    fn worker_loop(
        cmd_rx: Receiver<AsioCommand>,
        resp_tx: Sender<AsioResponse>,
        callback_state: Arc<Mutex<Option<CallbackState>>>,
        expected_driver: String,
    ) {
        // CRITICAL: Initialize COM as STA before any ASIO calls
        let com_initialized = unsafe {
            let hr = CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED as u32);
            if hr < 0 {
                error!(hr = hr, "Failed to initialize COM as STA");
                let _ = resp_tx.send(AsioResponse::Error(format!(
                    "COM initialization failed with HRESULT: {:#010x}",
                    hr
                )));
                return;
            }
            debug!(hr = hr, "COM initialized as STA (COINIT_APARTMENTTHREADED)");
            true
        };

        // Verify driver exists before entering main loop
        let asio = Asio::new();
        let driver_names = asio.driver_names();
        if !driver_names.iter().any(|n| n == &expected_driver) {
            let _ = resp_tx.send(AsioResponse::Error(format!(
                "ASIO driver '{}' not found",
                expected_driver
            )));
            if com_initialized {
                unsafe { CoUninitialize() };
            }
            return;
        }

        // ASIO state - all owned by this thread
        let mut driver: Option<Driver> = None;
        let mut streams: Option<AsioStreams> = None;
        let mut callback_id: Option<CallbackId> = None;
        let mut is_playing = false;

        info!("ASIO worker thread started, waiting for commands");

        // Main command loop
        while let Ok(cmd) = cmd_rx.recv() {
            match cmd {
                AsioCommand::Load {
                    driver_name,
                    sample_rate,
                    channels,
                } => {
                    info!(driver = %driver_name, "Loading ASIO driver");

                    // Stop and cleanup any existing driver first
                    if is_playing {
                        if let Some(ref d) = driver {
                            if let Err(e) = d.stop() {
                                warn!("Failed to stop previous driver: {:?}", e);
                            }
                        }
                        is_playing = false;
                    }

                    if let Some(cb_id) = callback_id.take() {
                        if let Some(ref d) = driver {
                            d.remove_callback(cb_id);
                        }
                    }

                    if let Some(ref d) = driver {
                        if let Err(e) = d.dispose_buffers() {
                            warn!("Failed to dispose previous buffers: {:?}", e);
                        }
                    }

                    drop(streams.take());
                    drop(driver.take());

                    // Load new driver
                    let loaded_driver = match asio.load_driver(&driver_name) {
                        Ok(d) => d,
                        Err(e) => {
                            error!(error = ?e, "Failed to load ASIO driver");
                            let _ = resp_tx.send(AsioResponse::Error(format!(
                                "Failed to load driver: {:?}",
                                e
                            )));
                            continue;
                        }
                    };

                    // Set sample rate if needed, then verify actual rate
                    let actual_sample_rate: u32;
                    match loaded_driver.sample_rate() {
                        Ok(current_rate) => {
                            if (current_rate as u32) != sample_rate {
                                debug!(
                                    current = current_rate,
                                    requested = sample_rate,
                                    "Setting sample rate"
                                );
                                if let Err(e) = loaded_driver.set_sample_rate(sample_rate as f64) {
                                    warn!(
                                        error = ?e,
                                        requested = sample_rate,
                                        "Failed to set sample rate, will use driver's rate"
                                    );
                                }
                            }

                            match loaded_driver.sample_rate() {
                                Ok(rate) => {
                                    actual_sample_rate = rate as u32;
                                    if actual_sample_rate != sample_rate {
                                        warn!(
                                            requested = sample_rate,
                                            actual = actual_sample_rate,
                                            "ASIO driver sample rate differs from requested - resampling required"
                                        );
                                    } else {
                                        info!(
                                            sample_rate = actual_sample_rate,
                                            "ASIO sample rate set successfully"
                                        );
                                    }
                                }
                                Err(e) => {
                                    let _ = resp_tx.send(AsioResponse::Error(format!(
                                        "Failed to verify sample rate: {:?}",
                                        e
                                    )));
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            let _ = resp_tx.send(AsioResponse::Error(format!(
                                "Failed to get sample rate: {:?}",
                                e
                            )));
                            continue;
                        }
                    }

                    // Get output data type
                    let sample_type = match loaded_driver.output_data_type() {
                        Ok(st) => st,
                        Err(e) => {
                            let _ = resp_tx.send(AsioResponse::Error(format!(
                                "Failed to get output data type: {:?}",
                                e
                            )));
                            continue;
                        }
                    };

                    let sample_format = match SampleFormat::from_asio(&sample_type) {
                        Some(sf) => sf,
                        None => {
                            let _ = resp_tx.send(AsioResponse::Error(format!(
                                "Unsupported sample format: {:?}",
                                sample_type
                            )));
                            continue;
                        }
                    };

                    info!(
                        sample_type = ?sample_type,
                        channels = channels,
                        "Preparing ASIO output stream"
                    );

                    // Prepare output stream
                    let prepared_streams =
                        match loaded_driver.prepare_output_stream(None, channels as usize, None) {
                            Ok(s) => s,
                            Err(e) => {
                                let _ = resp_tx.send(AsioResponse::Error(format!(
                                    "Failed to prepare output stream: {:?}",
                                    e
                                )));
                                continue;
                            }
                        };

                    let output_stream = match prepared_streams.output.as_ref() {
                        Some(os) => os,
                        None => {
                            let _ = resp_tx.send(AsioResponse::Error(
                                "prepare_output_stream returned no output stream".to_string(),
                            ));
                            continue;
                        }
                    };

                    let buffer_size = output_stream.buffer_size;
                    info!(buffer_size = buffer_size, "ASIO stream prepared");

                    // Extract buffer pointers
                    let buffer_ptrs: Vec<[SendPtr; 2]> = output_stream
                        .buffer_infos
                        .iter()
                        .map(|info| [SendPtr(info.buffers[0]), SendPtr(info.buffers[1])])
                        .collect();

                    // Update callback state
                    {
                        let mut state_guard = callback_state.lock();
                        if let Some(ref mut state) = *state_guard {
                            state.sample_format = sample_format;
                            state.buffer_size = buffer_size;
                            state.channels = channels as usize;
                            state.temp_buffer =
                                vec![0.0f32; channels as usize * buffer_size as usize];
                            state.buffer_ptrs = buffer_ptrs;
                        }
                    }

                    // Register callback
                    let cb_state = Arc::clone(&callback_state);
                    let cb_id = loaded_driver.add_callback(move |info| {
                        Self::audio_callback(&cb_state, info.buffer_index as usize);
                    });

                    callback_id = Some(cb_id);
                    streams = Some(prepared_streams);
                    driver = Some(loaded_driver);

                    let _ = resp_tx.send(AsioResponse::Loaded {
                        buffer_size,
                        sample_format,
                        actual_sample_rate,
                    });
                }

                AsioCommand::Start => {
                    if is_playing {
                        let _ = resp_tx.send(AsioResponse::Started);
                        continue;
                    }

                    match driver.as_ref() {
                        Some(d) => match d.start() {
                            Ok(()) => {
                                is_playing = true;
                                info!("ASIO playback started");
                                let _ = resp_tx.send(AsioResponse::Started);
                            }
                            Err(e) => {
                                error!(error = ?e, "Failed to start ASIO");
                                let _ = resp_tx
                                    .send(AsioResponse::Error(format!("Failed to start: {:?}", e)));
                            }
                        },
                        None => {
                            let _ =
                                resp_tx.send(AsioResponse::Error("No driver loaded".to_string()));
                        }
                    }
                }

                AsioCommand::Stop => {
                    if !is_playing {
                        let _ = resp_tx.send(AsioResponse::Stopped);
                        continue;
                    }

                    if let Some(ref d) = driver {
                        if let Err(e) = d.stop() {
                            warn!("Failed to stop ASIO: {:?}", e);
                        }
                    }

                    is_playing = false;
                    info!("ASIO playback stopped");
                    let _ = resp_tx.send(AsioResponse::Stopped);
                }

                AsioCommand::OpenControlPanel => {
                    unsafe extern "C" {
                        #[link_name = "?ASIOControlPanel@@YAJXZ"]
                        fn ASIOControlPanel() -> i32;
                    }

                    if driver.is_some() {
                        info!("Opening ASIO control panel");
                        let _ = resp_tx.send(AsioResponse::ControlPanelOpened);
                        let result = unsafe { ASIOControlPanel() };
                        if result != 0 && result != -1000 {
                            warn!(error_code = result, "ASIOControlPanel returned error");
                        }
                    } else {
                        let _ = resp_tx.send(AsioResponse::Error("No driver loaded".to_string()));
                    }
                }

                AsioCommand::Shutdown => {
                    info!("ASIO worker shutting down");
                    break;
                }
            }
        }

        // Cleanup on same thread that created resources
        info!("Cleaning up ASIO resources on worker thread");

        if is_playing {
            if let Some(ref d) = driver {
                if let Err(e) = d.stop() {
                    warn!("Failed to stop ASIO during shutdown: {:?}", e);
                }
            }
        }

        if let Some(cb_id) = callback_id.take() {
            if let Some(ref d) = driver {
                d.remove_callback(cb_id);
            }
        }

        if let Some(ref d) = driver {
            if let Err(e) = d.dispose_buffers() {
                warn!("Failed to dispose ASIO buffers: {:?}", e);
            }
        }

        drop(streams);
        drop(driver);

        // Uninitialize COM
        if com_initialized {
            unsafe { CoUninitialize() };
            debug!("COM uninitialized");
        }

        info!("ASIO worker thread exited");
    }

    /// Audio callback - runs on ASIO driver's high-priority thread
    fn audio_callback(callback_state: &Arc<Mutex<Option<CallbackState>>>, buffer_index: usize) {
        let mut state_guard = match callback_state.try_lock() {
            Some(guard) => guard,
            None => return,
        };

        let state = match state_guard.as_mut() {
            Some(s) => s,
            None => return,
        };

        if state.buffer_ptrs.is_empty() {
            return;
        }

        let channels = state.channels;
        let buffer_size = state.buffer_size;
        let samples_needed = channels * buffer_size as usize;

        // Read from ring buffer
        let available = state.consumer.occupied_len();
        let to_read = samples_needed.min(available);

        for i in 0..to_read {
            state.temp_buffer[i] = state.consumer.try_pop().unwrap_or(0.0);
        }

        // Zero-fill if underrun
        if to_read < samples_needed {
            state.callback_underruns.fetch_add(1, Ordering::Relaxed);
            for i in to_read..samples_needed {
                state.temp_buffer[i] = 0.0;
            }
        }

        // Write to output buffers (deinterleave)
        for ch in 0..channels {
            let buf_ptr = state.buffer_ptrs[ch][buffer_index].0;

            match state.sample_format {
                SampleFormat::Int32 => {
                    let out_buf = unsafe {
                        std::slice::from_raw_parts_mut(buf_ptr as *mut i32, buffer_size as usize)
                    };
                    for frame in 0..buffer_size as usize {
                        let sample_idx = frame * channels + ch;
                        let f = state.temp_buffer[sample_idx].clamp(-1.0, 1.0);
                        out_buf[frame] = (f * 2147483647.0) as i32;
                    }
                }
                SampleFormat::Int32Lsb24 => {
                    let out_buf = unsafe {
                        std::slice::from_raw_parts_mut(buf_ptr as *mut i32, buffer_size as usize)
                    };
                    for frame in 0..buffer_size as usize {
                        let sample_idx = frame * channels + ch;
                        let f = state.temp_buffer[sample_idx].clamp(-1.0, 1.0);
                        let i24 = (f * 8388607.0) as i32;
                        out_buf[frame] = i24 << 8;
                    }
                }
                SampleFormat::Float32 => {
                    let out_buf = unsafe {
                        std::slice::from_raw_parts_mut(buf_ptr as *mut f32, buffer_size as usize)
                    };
                    for frame in 0..buffer_size as usize {
                        let sample_idx = frame * channels + ch;
                        out_buf[frame] = state.temp_buffer[sample_idx].clamp(-1.0, 1.0);
                    }
                }
            }
        }
    }

    /// Load the ASIO driver and prepare streams.
    /// Returns (buffer_size, sample_format, actual_sample_rate).
    pub fn load(
        &self,
        driver_name: &str,
        sample_rate: u32,
        channels: u16,
    ) -> Result<(i32, SampleFormat, u32), OutputError> {
        self.cmd_tx
            .send(AsioCommand::Load {
                driver_name: driver_name.to_string(),
                sample_rate,
                channels,
            })
            .map_err(|_| OutputError::Asio("Worker thread disconnected".to_string()))?;

        match self.resp_rx.recv() {
            Ok(AsioResponse::Loaded {
                buffer_size,
                sample_format,
                actual_sample_rate,
            }) => Ok((buffer_size, sample_format, actual_sample_rate)),
            Ok(AsioResponse::Error(e)) => Err(OutputError::Asio(e)),
            Ok(other) => Err(OutputError::Asio(format!(
                "Unexpected response: {:?}",
                other
            ))),
            Err(_) => Err(OutputError::Asio("Worker thread disconnected".to_string())),
        }
    }

    /// Start ASIO playback
    pub fn start(&self) -> Result<(), OutputError> {
        self.cmd_tx
            .send(AsioCommand::Start)
            .map_err(|_| OutputError::Asio("Worker thread disconnected".to_string()))?;

        match self.resp_rx.recv() {
            Ok(AsioResponse::Started) => Ok(()),
            Ok(AsioResponse::Error(e)) => Err(OutputError::Asio(e)),
            Ok(other) => Err(OutputError::Asio(format!(
                "Unexpected response: {:?}",
                other
            ))),
            Err(_) => Err(OutputError::Asio("Worker thread disconnected".to_string())),
        }
    }

    /// Stop ASIO playback
    pub fn stop(&self) -> Result<(), OutputError> {
        self.cmd_tx
            .send(AsioCommand::Stop)
            .map_err(|_| OutputError::Asio("Worker thread disconnected".to_string()))?;

        match self.resp_rx.recv() {
            Ok(AsioResponse::Stopped) => Ok(()),
            Ok(AsioResponse::Error(e)) => Err(OutputError::Asio(e)),
            Ok(other) => Err(OutputError::Asio(format!(
                "Unexpected response: {:?}",
                other
            ))),
            Err(_) => Err(OutputError::Asio("Worker thread disconnected".to_string())),
        }
    }

    pub fn open_control_panel(&self) -> Result<(), OutputError> {
        self.cmd_tx
            .send(AsioCommand::OpenControlPanel)
            .map_err(|_| OutputError::Asio("Worker thread disconnected".to_string()))?;

        match self.resp_rx.recv() {
            Ok(AsioResponse::ControlPanelOpened) => Ok(()),
            Ok(AsioResponse::Error(e)) => Err(OutputError::Asio(e)),
            Ok(other) => Err(OutputError::Asio(format!(
                "Unexpected response: {:?}",
                other
            ))),
            Err(_) => Err(OutputError::Asio("Worker thread disconnected".to_string())),
        }
    }

    pub fn available_space(&self) -> usize {
        if let Some(ref producer) = self.producer {
            producer.vacant_len()
        } else {
            0
        }
    }

    pub fn push_samples(&mut self, samples: &[f32], volume: f32) -> usize {
        let vol = if volume.is_finite() {
            volume.clamp(0.0, 1.0)
        } else {
            1.0
        };

        let mut written = 0;
        if let Some(ref mut producer) = self.producer {
            for &sample in samples {
                let adjusted = (sample * vol).clamp(-1.0, 1.0);
                if producer.try_push(adjusted).is_err() {
                    break;
                }
                written += 1;
            }
        }
        written
    }

    /// Push raw DoP samples to the ring buffer
    pub fn push_dop_samples(&mut self, dop_samples: &[u32]) -> usize {
        let mut written = 0;
        if let Some(ref mut producer) = self.producer {
            for &dop_sample in dop_samples {
                // Shift 24-bit DoP sample to top 24 bits of i32
                let i32_sample = (dop_sample << 8) as i32;
                // Normalize to [-1.0, 1.0) range
                let f32_sample = (i32_sample as f32) / 2147483648.0;

                if producer.try_push(f32_sample).is_err() {
                    self.dop_drops.fetch_add(1, Ordering::Relaxed);
                    warn!("ASIO ring buffer full, dropping DoP sample");
                    break;
                }
                written += 1;
            }
        }
        written
    }

    /// Clear all samples from the ring buffer.
    /// Used when switching formats to prevent stale audio playback.
    pub fn clear_ring_buffer(&self) {
        if let Some(ref mut state) = *self.callback_state.lock() {
            let occupied = state.consumer.occupied_len();
            if occupied > 0 {
                state.consumer.skip(occupied);
                debug!(cleared_samples = occupied, "Cleared ASIO ring buffer");
            }
        }
    }

    pub fn callback_underruns(&self) -> u64 {
        self.callback_underruns.load(Ordering::Relaxed)
    }

    pub fn dop_drops(&self) -> u64 {
        self.dop_drops.load(Ordering::Relaxed)
    }

    /// Shutdown the worker thread gracefully
    pub fn shutdown(&mut self) {
        let _ = self.cmd_tx.send(AsioCommand::Shutdown);
        if let Some(handle) = self.handle.take() {
            if let Err(e) = handle.join() {
                error!("ASIO worker thread panicked: {:?}", e);
            }
        }
    }
}

impl Drop for AsioWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_format_from_asio() {
        assert_eq!(
            SampleFormat::from_asio(&AsioSampleType::ASIOSTInt32LSB),
            Some(SampleFormat::Int32)
        );
        assert_eq!(
            SampleFormat::from_asio(&AsioSampleType::ASIOSTInt32LSB24),
            Some(SampleFormat::Int32Lsb24)
        );
        assert_eq!(
            SampleFormat::from_asio(&AsioSampleType::ASIOSTFloat32LSB),
            Some(SampleFormat::Float32)
        );
    }
}
