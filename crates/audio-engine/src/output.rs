use std::collections::VecDeque;

use thiserror::Error;
use tracing::{info, warn};
use wasapi::{
    AudioClient, AudioRenderClient, Device, DeviceEnumerator, Direction, Handle, SampleType,
    StreamMode, WasapiError, WaveFormat, initialize_mta,
};

const AUDCLNT_E_DEVICE_INVALIDATED: u32 = 0x8889_0004;

#[derive(Debug, Error)]
pub enum OutputError {
    #[error("WASAPI error: {0}")]
    Wasapi(String),

    #[error("Device invalidated")]
    DeviceInvalidated,

    #[error("Buffer underrun")]
    BufferUnderrun,
}

/// WASAPI output configured for a specific source format.
/// Uses WASAPI Shared mode with AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM to let
/// Windows handle sample rate and format conversion.
pub struct WasapiOutput {
    client: AudioClient,
    render_client: AudioRenderClient,
    event_handle: Handle,
    // Source format (what we're feeding in)
    sample_rate: u32,
    channels: u16,
    buffer_frames: u32,
    started: bool,
}

impl WasapiOutput {
    /// Open output for a specific source format. Windows will handle conversion
    /// to the device's native format via AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM.
    pub fn open_with_format(sample_rate: u32, channels: u16) -> Result<Self, OutputError> {
        init_com()?;

        let enumerator = DeviceEnumerator::new().map_err(map_wasapi_error)?;
        let device = enumerator
            .get_default_device(&Direction::Render)
            .map_err(map_wasapi_error)?;

        Self::open_device_with_format(&device, sample_rate, channels)
    }

    /// Open a specific device with a specific source format.
    pub fn open_device_with_format(
        device: &Device,
        sample_rate: u32,
        channels: u16,
    ) -> Result<Self, OutputError> {
        init_com()?;

        let mut client = device.get_iaudioclient().map_err(map_wasapi_error)?;

        // Create a wave format for our source audio (32-bit float, interleaved)
        // WaveFormat::new(bits_per_sample, valid_bits, sample_type, sample_rate, channels, channel_mask)
        let channel_mask = if channels == 2 {
            Some(0x3) // SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT
        } else if channels == 1 {
            Some(0x4) // SPEAKER_FRONT_CENTER
        } else {
            None
        };

        let wave_format = WaveFormat::new(
            32, // bits_per_sample
            32, // valid_bits (same as bits for float)
            &SampleType::Float,
            sample_rate as usize,
            channels as usize,
            channel_mask,
        );

        let (def_time, _min_time) = client.get_device_period().map_err(map_wasapi_error)?;

        // Use EventsShared with autoconvert: true
        // This enables AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY
        let mode = StreamMode::EventsShared {
            autoconvert: true,
            buffer_duration_hns: def_time,
        };

        client
            .initialize_client(&wave_format, &Direction::Render, &mode)
            .map_err(map_wasapi_error)?;

        let event_handle = client.set_get_eventhandle().map_err(map_wasapi_error)?;
        let render_client = client.get_audiorenderclient().map_err(map_wasapi_error)?;
        let buffer_frames = client.get_buffer_size().map_err(map_wasapi_error)?;

        info!(
            sample_rate = sample_rate,
            channels = channels,
            buffer_frames = buffer_frames,
            "Opened WASAPI output with autoconvert"
        );

        Ok(Self {
            client,
            render_client,
            event_handle,
            sample_rate,
            channels,
            buffer_frames,
            started: false,
        })
    }

    /// Open with the device's default/mix format (for pre-initialization).
    pub fn open_default() -> Result<Self, OutputError> {
        init_com()?;

        let enumerator = DeviceEnumerator::new().map_err(map_wasapi_error)?;
        let device = enumerator
            .get_default_device(&Direction::Render)
            .map_err(map_wasapi_error)?;

        let client = device.get_iaudioclient().map_err(map_wasapi_error)?;
        let mix_format = client.get_mixformat().map_err(map_wasapi_error)?;

        let sample_rate = mix_format.get_samplespersec();
        let channels = mix_format.get_nchannels();

        Self::open_device_with_format(&device, sample_rate, channels)
    }

    pub fn write_samples(&mut self, samples: &[f32], volume: f32) -> Result<(), OutputError> {
        if !self.started {
            self.start()?;
        }

        let channels = self.channels as usize;
        if channels == 0 || samples.is_empty() {
            return Ok(());
        }

        let available_frames = match self.client.get_available_space_in_frames() {
            Ok(frames) => frames,
            Err(err) if is_device_invalidated(&err) => {
                let _ = self.stop();
                return Err(OutputError::DeviceInvalidated);
            }
            Err(err) => return Err(map_wasapi_error(err)),
        };

        if available_frames == 0 {
            return Ok(());
        }

        // Calculate how many frames we can actually write
        let input_frames = samples.len() / channels;
        let frames_to_write = input_frames.min(available_frames as usize);
        let samples_to_write = frames_to_write * channels;

        if frames_to_write == 0 {
            return Ok(());
        }

        let vol = if volume.is_finite() {
            volume.clamp(0.0, 1.0)
        } else {
            1.0
        };

        // Convert f32 samples to bytes (32-bit float LE) with volume applied
        let mut data = Vec::with_capacity(samples_to_write * 4);
        for &s in samples.iter().take(samples_to_write) {
            let adjusted = (s * vol).clamp(-1.0, 1.0);
            data.extend_from_slice(&adjusted.to_le_bytes());
        }

        match self
            .render_client
            .write_to_device(frames_to_write, &data, None)
        {
            Ok(()) => Ok(()),
            Err(err) if is_device_invalidated(&err) => {
                let _ = self.stop();
                Err(OutputError::DeviceInvalidated)
            }
            Err(err) => Err(map_wasapi_error(err)),
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    pub fn buffer_frames(&self) -> u32 {
        self.buffer_frames
    }

    pub fn start(&mut self) -> Result<(), OutputError> {
        if self.started {
            return Ok(());
        }
        self.client.start_stream().map_err(|err| {
            if is_device_invalidated(&err) {
                OutputError::DeviceInvalidated
            } else {
                map_wasapi_error(err)
            }
        })?;
        self.started = true;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), OutputError> {
        if !self.started {
            return Ok(());
        }
        self.client.stop_stream().map_err(|err| {
            if is_device_invalidated(&err) {
                OutputError::DeviceInvalidated
            } else {
                map_wasapi_error(err)
            }
        })?;
        self.started = false;
        Ok(())
    }

    pub fn event_handle(&self) -> &Handle {
        &self.event_handle
    }
}

fn init_com() -> Result<(), OutputError> {
    initialize_mta()
        .ok()
        .map_err(|err| OutputError::Wasapi(err.to_string()))?;
    Ok(())
}

fn map_wasapi_error(err: WasapiError) -> OutputError {
    if is_device_invalidated(&err) {
        return OutputError::DeviceInvalidated;
    }
    OutputError::Wasapi(err.to_string())
}

fn is_device_invalidated(err: &WasapiError) -> bool {
    match err {
        WasapiError::Windows(win_err) => win_err.code().0 as u32 == AUDCLNT_E_DEVICE_INVALIDATED,
        _ => false,
    }
}

pub fn convert_channels_interleaved_f32(
    samples: &[f32],
    input_channels: usize,
    output_channels: usize,
) -> Vec<f32> {
    if input_channels == 0 || output_channels == 0 {
        return Vec::new();
    }

    let frames = samples.len() / input_channels;
    if frames == 0 {
        return Vec::new();
    }

    match (input_channels, output_channels) {
        (1, 2) => {
            let mut out = Vec::with_capacity(frames * 2);
            for &s in samples.iter().take(frames) {
                out.push(s);
                out.push(s);
            }
            out
        }
        _ => {
            let mut out = Vec::with_capacity(frames * output_channels);
            for frame in 0..frames {
                let base = frame * input_channels;
                for ch in 0..output_channels {
                    let sample = if ch < input_channels {
                        samples[base + ch]
                    } else {
                        0.0
                    };
                    out.push(sample);
                }
            }
            out
        }
    }
}

pub struct AudioRingBuffer {
    channels: usize,
    capacity_samples: usize,
    buf: VecDeque<f32>,
}

impl AudioRingBuffer {
    /// Create a bounded ring buffer sized for ~`capacity_ms` of audio.
    pub fn new(sample_rate: u32, channels: usize, capacity_ms: u32) -> Self {
        let channels = channels.max(1);
        let capacity_frames = ((sample_rate as u64 * capacity_ms as u64) / 1000).max(1) as usize;
        let capacity_samples = capacity_frames.saturating_mul(channels).max(channels);

        Self {
            channels,
            capacity_samples,
            buf: VecDeque::with_capacity(capacity_samples),
        }
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn capacity_frames(&self) -> usize {
        self.capacity_samples / self.channels
    }

    pub fn available_frames(&self) -> usize {
        self.buf.len() / self.channels
    }

    pub fn push(&mut self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }

        let frame_aligned_len = samples.len() / self.channels * self.channels;
        if frame_aligned_len == 0 {
            return;
        }
        let samples = &samples[..frame_aligned_len];

        if samples.len() >= self.capacity_samples {
            warn!(
                capacity_samples = self.capacity_samples,
                incoming_samples = samples.len(),
                "Ring buffer overflow: dropping old samples"
            );
            self.buf.clear();
            self.buf.extend(
                samples[samples.len() - self.capacity_samples..]
                    .iter()
                    .copied(),
            );
            return;
        }

        let overflow = self
            .buf
            .len()
            .saturating_add(samples.len())
            .saturating_sub(self.capacity_samples);

        if overflow > 0 {
            warn!(
                overflow_samples = overflow,
                capacity_samples = self.capacity_samples,
                "Ring buffer overflow: dropping oldest samples"
            );
            for _ in 0..overflow {
                self.buf.pop_front();
            }
        }

        self.buf.extend(samples.iter().copied());
    }

    /// Pop into the provided output buffer. If insufficient data is available, the remainder is
    /// filled with silence and a warning is emitted.
    pub fn pop_into(&mut self, out: &mut [f32]) {
        if out.is_empty() {
            return;
        }

        let needed = out.len();
        let available = self.buf.len();
        let take = needed.min(available);

        for dst in out.iter_mut().take(take) {
            *dst = self.buf.pop_front().unwrap_or(0.0);
        }

        if take < needed {
            out[take..].fill(0.0);
            warn!(
                requested_samples = needed,
                available_samples = take,
                "Ring buffer underrun: writing silence"
            );
        }
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }
}
