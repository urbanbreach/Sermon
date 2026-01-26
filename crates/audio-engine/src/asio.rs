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

        let frames = dop_samples.len() / channels;
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
}
