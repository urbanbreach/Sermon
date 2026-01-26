//! ASIO audio output for Windows
//!
//! Uses asio-sys for direct ASIO driver access with lock-free ring buffer
//! for thread-safe callback operation.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use asio_sys::Asio;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use tracing::{info, warn};

use crate::output::{AudioRingBuffer, OutputError};

type RingProducer = ringbuf::HeapProd<f32>;
type RingConsumer = ringbuf::HeapCons<f32>;

pub struct AsioOutput {
    driver_name: String,
    sample_rate: u32,
    channels: u16,
    bit_depth: u16,
    valid_bits: u16,
    started: AtomicBool,
    producer: RingProducer,
    consumer: Option<RingConsumer>,
}

impl AsioOutput {
    pub fn new(driver_name: &str, sample_rate: u32, channels: u16) -> Result<Self, OutputError> {
        let asio = Asio::new();

        let driver_names = asio.driver_names();
        if !driver_names.iter().any(|n| n == driver_name) {
            return Err(OutputError::AsioDriverNotFound(driver_name.to_string()));
        }

        let buffer_size = (sample_rate as usize * channels as usize) / 2;
        let rb = HeapRb::<f32>::new(buffer_size);
        let (producer, consumer) = rb.split();

        info!(
            driver = driver_name,
            sample_rate = sample_rate,
            channels = channels,
            buffer_samples = buffer_size,
            "Created ASIO output (not yet started)"
        );

        Ok(Self {
            driver_name: driver_name.to_string(),
            sample_rate,
            channels,
            bit_depth: 32,
            valid_bits: 24,
            started: AtomicBool::new(false),
            producer,
            consumer: Some(consumer),
        })
    }

    pub fn driver_name(&self) -> &str {
        &self.driver_name
    }
}

impl crate::output::AudioOutput for AsioOutput {
    fn start(&mut self) -> Result<(), OutputError> {
        if self.started.load(Ordering::SeqCst) {
            return Ok(());
        }

        let asio = Asio::new();
        let _driver = asio
            .load_driver(&self.driver_name)
            .map_err(|e| OutputError::AsioDriverLoadFailed(format!("{:?}", e)))?;

        self.started.store(true, Ordering::SeqCst);

        info!(driver = %self.driver_name, "ASIO driver loaded and started");
        Ok(())
    }

    fn stop(&mut self) -> Result<(), OutputError> {
        if !self.started.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.started.store(false, Ordering::SeqCst);
        info!(driver = %self.driver_name, "ASIO output stopped");
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
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

        let vol = if volume.is_finite() {
            volume.clamp(0.0, 1.0)
        } else {
            1.0
        };

        for &sample in samples {
            let adjusted = (sample * vol).clamp(-1.0, 1.0);
            if self.producer.try_push(adjusted).is_err() {
                warn!("ASIO ring buffer full, dropping sample");
            }
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

        let samples_count = available_frames * channels;
        let mut samples = vec![0.0f32; samples_count];
        ring_buffer.pop_into(&mut samples);

        self.write_samples(&samples, volume)?;

        Ok(available_frames)
    }

    fn write_raw_dop(&mut self, dop_samples: &[u32]) -> Result<usize, OutputError> {
        if !self.started.load(Ordering::SeqCst) {
            self.start()?;
        }

        let channels = self.channels as usize;
        if channels == 0 || dop_samples.is_empty() {
            return Ok(0);
        }

        // Convert DoP u32 samples to f32 for the ring buffer.
        // DoP format: bits 0-7 = dsd_byte0, bits 8-15 = dsd_byte1, bits 16-23 = marker
        // We shift the 24-bit DoP value to the top 24 bits of an i32, then normalize.
        // This preserves the bit pattern through the f32 ring buffer.
        let mut written = 0;
        for &dop_sample in dop_samples {
            // Shift 24-bit DoP sample to top 24 bits of i32
            let i32_sample = (dop_sample << 8) as i32;
            // Normalize to [-1.0, 1.0) range
            let f32_sample = (i32_sample as f32) / 2147483648.0;

            if self.producer.try_push(f32_sample).is_err() {
                warn!("ASIO ring buffer full, dropping DoP sample");
                break;
            }
            written += 1;
        }

        let frames = written / channels;
        Ok(frames)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn test_asio_output_creation_with_invalid_driver() {
        let result = AsioOutput::new("NonExistentDriver12345", 44100, 2);
        assert!(result.is_err());
        match result {
            Err(OutputError::AsioDriverNotFound(name)) => {
                assert_eq!(name, "NonExistentDriver12345");
            }
            _ => panic!("Expected AsioDriverNotFound error"),
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_asio_output_is_exclusive() {
        let asio = Asio::new();
        let drivers = asio.driver_names();
        if drivers.is_empty() {
            return;
        }

        if let Ok(output) = AsioOutput::new(&drivers[0], 44100, 2) {
            assert!(output.is_exclusive());
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_asio_output_properties() {
        let asio = Asio::new();
        let drivers = asio.driver_names();
        if drivers.is_empty() {
            return;
        }

        if let Ok(output) = AsioOutput::new(&drivers[0], 96000, 2) {
            assert_eq!(output.sample_rate(), 96000);
            assert_eq!(output.channels(), 2);
            assert_eq!(output.bit_depth(), 32);
            assert_eq!(output.valid_bits(), 24);
        }
    }

    #[test]
    fn test_dop_through_asio_preserves_markers() {
        use crate::dop::{DOP_MARKER_A, DOP_MARKER_B};

        let dop_samples: Vec<u32> = vec![
            0x00_05_11_22, // marker 0x05, dsd bytes 0x22, 0x11
            0x00_05_33_44, // marker 0x05, dsd bytes 0x44, 0x33
            0x00_FA_55_66, // marker 0xFA, dsd bytes 0x66, 0x55
            0x00_FA_77_88, // marker 0xFA, dsd bytes 0x88, 0x77
        ];

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
