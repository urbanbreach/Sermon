//! ASIO audio output for Windows
//!
//! Uses asio-sys for direct ASIO driver access with lock-free ring buffer
//! for thread-safe callback operation. All ASIO operations happen on a
//! dedicated worker thread with STA COM initialization to avoid conflicts
//! with WASAPI's MTA initialization.

use std::sync::atomic::{AtomicBool, Ordering};

use tracing::{debug, error, info};

use crate::asio_worker::AsioWorker;
use crate::output::{AudioOutput, AudioRingBuffer, OutputError};

pub struct AsioOutput {
    driver_name: String,
    requested_sample_rate: u32,
    actual_sample_rate: u32,
    channels: u16,
    bit_depth: u16,
    valid_bits: u16,
    worker: Option<AsioWorker>,
    started: AtomicBool,
    buffer_size: i32,
}

impl AsioOutput {
    /// Create a new ASIO output instance.
    ///
    /// This validates the driver exists but does NOT load it yet.
    /// The driver is loaded lazily on first `start()` call.
    pub fn new(driver_name: &str, sample_rate: u32, channels: u16) -> Result<Self, OutputError> {
        info!(
            driver = driver_name,
            sample_rate = sample_rate,
            channels = channels,
            "Creating ASIO output (deferred initialization)"
        );

        Ok(Self {
            driver_name: driver_name.to_string(),
            requested_sample_rate: sample_rate,
            actual_sample_rate: sample_rate,
            channels,
            bit_depth: 32,
            valid_bits: 24,
            worker: None,
            started: AtomicBool::new(false),
            buffer_size: 0,
        })
    }

    pub fn driver_name(&self) -> &str {
        &self.driver_name
    }

    pub fn is_exclusive(&self) -> bool {
        true
    }

    pub fn requested_sample_rate(&self) -> u32 {
        self.requested_sample_rate
    }

    pub fn needs_resampling(&self) -> bool {
        self.actual_sample_rate != self.requested_sample_rate
    }

    fn start_internal(&mut self) -> Result<(), OutputError> {
        if self.started.load(Ordering::SeqCst) {
            return Ok(());
        }

        info!(driver = %self.driver_name, "Starting ASIO with dedicated worker thread");

        let worker = AsioWorker::new(
            self.driver_name.clone(),
            self.requested_sample_rate,
            self.channels,
        )?;

        let (buffer_size, sample_format, actual_sample_rate) =
            worker.load(&self.driver_name, self.requested_sample_rate, self.channels)?;

        self.actual_sample_rate = actual_sample_rate;

        debug!(
            buffer_size = buffer_size,
            sample_format = ?sample_format,
            requested_rate = self.requested_sample_rate,
            actual_rate = actual_sample_rate,
            "ASIO driver loaded"
        );

        if actual_sample_rate != self.requested_sample_rate {
            info!(
                requested = self.requested_sample_rate,
                actual = actual_sample_rate,
                "ASIO sample rate mismatch - resampling will be needed"
            );
        }

        worker.start()?;

        self.buffer_size = buffer_size;
        self.worker = Some(worker);
        self.started.store(true, Ordering::SeqCst);

        info!(driver = %self.driver_name, actual_rate = actual_sample_rate, "ASIO driver started via worker thread");
        Ok(())
    }

    fn stop_internal(&mut self) -> Result<(), OutputError> {
        if !self.started.load(Ordering::SeqCst) {
            return Ok(());
        }

        info!(driver = %self.driver_name, "Stopping ASIO driver");

        if let Some(ref worker) = self.worker {
            worker.stop()?;
        }

        self.worker = None;
        self.buffer_size = 0;
        self.started.store(false, Ordering::SeqCst);

        info!(driver = %self.driver_name, "ASIO output stopped");
        Ok(())
    }
}

impl AudioOutput for AsioOutput {
    fn start(&mut self) -> Result<(), OutputError> {
        self.start_internal()
    }

    fn stop(&mut self) -> Result<(), OutputError> {
        self.stop_internal()
    }

    fn sample_rate(&self) -> u32 {
        self.actual_sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn bit_depth(&self) -> u16 {
        self.bit_depth
    }

    fn valid_bits(&self) -> u16 {
        self.valid_bits
    }

    fn is_exclusive(&self) -> bool {
        true
    }

    fn write_samples(&mut self, samples: &[f32], volume: f32) -> Result<(), OutputError> {
        if !self.started.load(Ordering::SeqCst) {
            self.start()?;
        }

        if let Some(ref mut worker) = self.worker {
            worker.push_samples(samples, volume);
        }

        Ok(())
    }

    fn write_from_buffer(
        &mut self,
        ring_buffer: &mut AudioRingBuffer,
        volume: f32,
    ) -> Result<usize, OutputError> {
        if !self.started.load(Ordering::SeqCst) {
            self.start()?;
        }

        let channels = self.channels as usize;
        if channels == 0 {
            return Ok(0);
        }

        let available_frames = ring_buffer.available_frames();
        if available_frames == 0 {
            return Ok(0);
        }

        let worker = match self.worker.as_mut() {
            Some(w) => w,
            None => return Ok(0),
        };

        let space_samples = worker.available_space();
        let space_frames = space_samples / channels;

        if space_frames == 0 {
            return Ok(0);
        }

        let frames_to_transfer = available_frames.min(space_frames);
        let samples_count = frames_to_transfer * channels;
        let mut samples = vec![0.0f32; samples_count];
        ring_buffer.pop_into(&mut samples);

        let written_samples = worker.push_samples(&samples, volume);
        let written_frames = written_samples / channels;

        Ok(written_frames)
    }

    fn write_raw_dop(&mut self, dop_samples: &[u32]) -> Result<usize, OutputError> {
        if !self.started.load(Ordering::SeqCst) {
            self.start()?;
        }

        let channels = self.channels as usize;
        if channels == 0 || dop_samples.is_empty() {
            return Ok(0);
        }

        let worker = match self.worker.as_mut() {
            Some(w) => w,
            None => return Ok(0),
        };

        let space_samples = worker.available_space();
        if space_samples == 0 {
            return Ok(0);
        }

        let samples_to_write = dop_samples.len().min(space_samples);
        let frame_aligned = (samples_to_write / channels) * channels;
        if frame_aligned == 0 {
            return Ok(0);
        }

        let written = worker.push_dop_samples(&dop_samples[..frame_aligned]);
        let frames = written / channels;
        Ok(frames)
    }

    fn available_dop_space(&mut self) -> usize {
        if let Some(ref worker) = self.worker {
            worker.available_space()
        } else {
            0
        }
    }
}

impl AsioOutput {
    pub fn clear_ring_buffer(&self) {
        if let Some(ref worker) = self.worker {
            worker.clear_ring_buffer();
        }
    }

    pub fn callback_underruns(&self) -> u64 {
        self.worker
            .as_ref()
            .map(|worker| worker.callback_underruns())
            .unwrap_or(0)
    }

    pub fn dop_drops(&self) -> u64 {
        self.worker
            .as_ref()
            .map(|worker| worker.dop_drops())
            .unwrap_or(0)
    }

    pub fn open_control_panel(&self) -> Result<(), crate::output::OutputError> {
        if let Some(ref worker) = self.worker {
            worker.open_control_panel()
        } else {
            Err(crate::output::OutputError::Asio(
                "ASIO worker not running".to_string(),
            ))
        }
    }

    pub fn buffer_size_frames(&self) -> u32 {
        self.buffer_size.max(0) as u32
    }

    pub fn sample_format_name(&self) -> String {
        match (self.bit_depth, self.valid_bits) {
            (32, 24) => "int32lsb24".to_string(),
            (32, 32) => "float32".to_string(),
            (24, 24) => "int24".to_string(),
            (16, 16) => "int16".to_string(),
            _ => format!("int{}v{}", self.bit_depth, self.valid_bits),
        }
    }
}

impl Drop for AsioOutput {
    fn drop(&mut self) {
        if self.started.load(Ordering::SeqCst) {
            if let Err(e) = self.stop_internal() {
                error!("Error stopping ASIO on drop: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asio_sys::Asio;

    #[test]
    #[cfg(windows)]
    fn test_asio_output_creation_with_invalid_driver() {
        let result = AsioOutput::new("NonExistentDriver12345", 44100, 2);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(windows)]
    fn test_asio_output_is_exclusive() {
        if let Ok(output) = AsioOutput::new("FlexASIO", 44100, 2) {
            assert!(output.is_exclusive());
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_asio_output_properties() {
        if let Ok(output) = AsioOutput::new("FlexASIO", 96000, 2) {
            assert_eq!(output.sample_rate(), 96000);
            assert_eq!(output.channels(), 2);
            assert_eq!(output.bit_depth(), 32);
            assert_eq!(output.valid_bits(), 24);
        }
    }

    #[test]
    fn test_dop_through_asio_preserves_markers() {
        use crate::dop::{DOP_MARKER_A, DOP_MARKER_B};

        let dop_samples: Vec<u32> =
            vec![0x00_05_11_22, 0x00_05_33_44, 0x00_FA_55_66, 0x00_FA_77_88];

        for &dop_sample in &dop_samples {
            let marker = ((dop_sample >> 16) & 0xFF) as u8;
            assert!(
                marker == DOP_MARKER_A || marker == DOP_MARKER_B,
                "Input marker {:#04x} should be 0x05 or 0xFA",
                marker
            );

            let i32_sample = (dop_sample << 8) as i32;
            let f32_sample = (i32_sample as f32) / 2147483648.0;

            let recovered_i32 = (f32_sample * 2147483648.0) as i32;
            let recovered_u32 = (recovered_i32 as u32) >> 8;

            let recovered_marker = ((recovered_u32 >> 16) & 0xFF) as u8;
            assert_eq!(
                recovered_marker, marker,
                "Marker mismatch: expected {:#04x}, got {:#04x}",
                marker, recovered_marker
            );

            let original_dsd = dop_sample & 0xFFFF;
            let recovered_dsd = recovered_u32 & 0xFFFF;
            assert_eq!(
                recovered_dsd, original_dsd,
                "DSD bytes mismatch: expected {:#06x}, got {:#06x}",
                original_dsd, recovered_dsd
            );
        }
    }
}
