use std::collections::VecDeque;

use thiserror::Error;
use tracing::{info, warn};
use wasapi::{
    AudioClient, AudioRenderClient, Device, DeviceEnumerator, Direction, Handle, SampleType,
    StreamMode, WasapiError, WaveFormat, initialize_mta,
};

const AUDCLNT_E_DEVICE_INVALIDATED: u32 = 0x8889_0004;
const AUDCLNT_E_EXCLUSIVE_MODE_NOT_ALLOWED: u32 = 0x8889_000E;
const AUDCLNT_E_DEVICE_IN_USE: u32 = 0x8889_000A;
const AUDCLNT_E_UNSUPPORTED_FORMAT: u32 = 0x8889_0018;

#[derive(Debug, Error)]
pub enum OutputError {
    #[error("WASAPI error: {0}")]
    Wasapi(String),

    #[error("Device invalidated")]
    DeviceInvalidated,

    #[error("Buffer underrun")]
    BufferUnderrun,

    #[error("Exclusive mode not allowed or device in use")]
    ExclusiveUnavailable,

    #[error("Format not supported in exclusive mode")]
    ExclusiveUnsupportedFormat,
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
    bit_depth: u16,  // Container size (e.g., 32 for 24-in-32)
    valid_bits: u16, // Actual valid bits (e.g., 24 for 24-in-32)
    sample_type: SampleType,
    buffer_frames: u32,
    started: bool,
    is_exclusive: bool,
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
            bit_depth: 32,
            valid_bits: 32,
            sample_type: SampleType::Float,
            buffer_frames,
            started: false,
            is_exclusive: false,
        })
    }

    pub fn open_exclusive_with_format(
        sample_rate: u32,
        channels: u16,
        bit_depth: u16,
    ) -> Result<Self, OutputError> {
        init_com()?;

        let enumerator = DeviceEnumerator::new().map_err(map_wasapi_error)?;
        let device = enumerator
            .get_default_device(&Direction::Render)
            .map_err(map_wasapi_error)?;

        Self::open_device_exclusive_with_format(&device, sample_rate, channels, bit_depth)
    }

    pub fn open_device_exclusive_with_format(
        device: &Device,
        sample_rate: u32,
        channels: u16,
        bit_depth: u16,
    ) -> Result<Self, OutputError> {
        init_com()?;

        let mut client = device.get_iaudioclient().map_err(map_wasapi_error)?;

        let channel_mask = if channels == 2 {
            Some(0x3) // SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT
        } else if channels == 1 {
            Some(0x4) // SPEAKER_FRONT_CENTER
        } else {
            None
        };

        // Try different format configurations in order of preference
        // Many DACs prefer 32-bit containers for 24-bit audio, or 32-bit float
        let format_attempts: Vec<(usize, usize, SampleType)> = match bit_depth {
            24 => vec![
                (32, 24, SampleType::Int),   // 24-bit in 32-bit container (most compatible)
                (24, 24, SampleType::Int),   // Native 24-bit
                (32, 32, SampleType::Float), // 32-bit float fallback
            ],
            16 => vec![
                (16, 16, SampleType::Int),   // Native 16-bit
                (32, 32, SampleType::Float), // 32-bit float fallback
            ],
            32 => vec![
                (32, 32, SampleType::Float), // 32-bit float
                (32, 32, SampleType::Int),   // 32-bit int
            ],
            _ => vec![(bit_depth as usize, bit_depth as usize, SampleType::Int)],
        };

        let mut last_error = None;
        for (store_bits, valid_bits, sample_type) in &format_attempts {
            let desired_format = WaveFormat::new(
                *store_bits,
                *valid_bits,
                sample_type,
                sample_rate as usize,
                channels as usize,
                channel_mask,
            );

            info!(
                store_bits = store_bits,
                valid_bits = valid_bits,
                sample_type = ?sample_type,
                sample_rate = sample_rate,
                "Trying exclusive format"
            );

            match client.is_supported_exclusive_with_quirks(&desired_format) {
                Ok(wave_format) => {
                    // Format is supported, try to initialize
                    let actual_bits = wave_format.get_bitspersample();
                    let actual_valid_bits = wave_format.get_validbitspersample();
                    let actual_sample_type = wave_format.get_subformat().unwrap_or(*sample_type);

                    info!(
                        requested_bits = bit_depth,
                        store_bits = store_bits,
                        actual_bits = actual_bits,
                        actual_valid_bits = actual_valid_bits,
                        "Format accepted for exclusive mode"
                    );

                    let (def_time, min_time) =
                        client.get_device_period().map_err(map_wasapi_error)?;

                    // Calculate aligned period for better compatibility
                    let desired_period = client
                        .calculate_aligned_period_near(def_time, Some(128), &wave_format)
                        .unwrap_or(def_time);

                    info!(
                        def_period = def_time,
                        min_period = min_time,
                        desired_period = desired_period,
                        "Exclusive mode period calculation"
                    );

                    let mode = StreamMode::EventsExclusive {
                        period_hns: desired_period,
                    };

                    if let Err(e) =
                        client.initialize_client(&wave_format, &Direction::Render, &mode)
                    {
                        if let WasapiError::Windows(werr) = &e {
                            let code = werr.code().0 as u32;
                            if code == AUDCLNT_E_EXCLUSIVE_MODE_NOT_ALLOWED
                                || code == AUDCLNT_E_DEVICE_IN_USE
                            {
                                return Err(OutputError::ExclusiveUnavailable);
                            }
                        }
                        // Try next format
                        last_error = Some(e);
                        // Need a new audio client for the next attempt
                        client = device.get_iaudioclient().map_err(map_wasapi_error)?;
                        continue;
                    }

                    let event_handle = client.set_get_eventhandle().map_err(map_wasapi_error)?;
                    let render_client = client.get_audiorenderclient().map_err(map_wasapi_error)?;
                    let buffer_frames = client.get_buffer_size().map_err(map_wasapi_error)?;
                    let block_align = wave_format.get_blockalign();

                    info!(
                        sample_rate = sample_rate,
                        channels = channels,
                        bit_depth = actual_bits,
                        valid_bits = actual_valid_bits,
                        buffer_frames = buffer_frames,
                        block_align = block_align,
                        "Opened WASAPI output (Exclusive)"
                    );

                    return Ok(Self {
                        client,
                        render_client,
                        event_handle,
                        sample_rate,
                        channels,
                        bit_depth: actual_bits,
                        valid_bits: actual_valid_bits,
                        sample_type: actual_sample_type,
                        buffer_frames,
                        started: false,
                        is_exclusive: true,
                    });
                }
                Err(e) => {
                    info!(
                        store_bits = store_bits,
                        valid_bits = valid_bits,
                        error = %e,
                        "Format not supported, trying next"
                    );
                    last_error = Some(e);
                    // Need a new audio client for the next attempt
                    client = device.get_iaudioclient().map_err(map_wasapi_error)?;
                }
            }
        }

        // All formats failed
        if let Some(e) = last_error {
            warn!("All exclusive formats failed, last error: {}", e);
        }
        Err(OutputError::ExclusiveUnsupportedFormat)
    }

    pub fn negotiate_exclusive_format(
        device: &Device,
        sample_rate: u32,
        channels: u16,
        bit_depth: u16,
        policy: &str, // "strict" or "compatibility"
    ) -> Result<(WasapiOutput, Option<String>), OutputError> {
        // Try exact match
        match Self::open_device_exclusive_with_format(device, sample_rate, channels, bit_depth) {
            Ok(output) => return Ok((output, None)),
            Err(OutputError::ExclusiveUnsupportedFormat) => {
                if policy == "strict" {
                    return Err(OutputError::ExclusiveUnsupportedFormat);
                }
                // Compatibility: try zero-pad 16->24 if original was 16
                if bit_depth == 16 {
                    match Self::open_device_exclusive_with_format(device, sample_rate, channels, 24)
                    {
                        Ok(output) => return Ok((output, Some("pad_16_to_24".to_string()))),
                        Err(_) => {} // Fall through
                    }
                    // Try 32?
                    match Self::open_device_exclusive_with_format(device, sample_rate, channels, 32)
                    {
                        Ok(output) => return Ok((output, Some("pad_16_to_32".to_string()))),
                        Err(_) => {} // Fall through
                    }
                }
                Err(OutputError::ExclusiveUnsupportedFormat)
            }
            Err(e) => Err(e),
        }
    }

    pub fn is_exclusive(&self) -> bool {
        self.is_exclusive
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

        // Convert f32 samples to bytes based on format
        // Use valid_bits to determine actual audio precision, bit_depth for container size
        let samples_subset = &samples[..samples_to_write];
        let data = match (self.sample_type, self.bit_depth, self.valid_bits) {
            (SampleType::Float, 32, _) => {
                let mut d = Vec::with_capacity(samples_to_write * 4);
                for &s in samples_subset {
                    let adjusted = (s * vol).clamp(-1.0, 1.0);
                    d.extend_from_slice(&adjusted.to_le_bytes());
                }
                d
            }
            (SampleType::Int, 16, 16) => f32_to_i16_le(samples_subset, vol),
            (SampleType::Int, 24, 24) => {
                // Native 24-bit: 3 bytes per sample
                f32_to_i24_native_le(samples_subset, vol)
            }
            (SampleType::Int, 32, 24) => {
                // 24-bit in 32-bit container: 4 bytes per sample, 24 valid bits
                f32_to_i24_in_i32_le(samples_subset, vol)
            }
            (SampleType::Int, 32, 32) => f32_to_i32_le(samples_subset, vol),
            _ => {
                // Fallback to float
                let mut d = Vec::with_capacity(samples_to_write * 4);
                for &s in samples_subset {
                    let adjusted = (s * vol).clamp(-1.0, 1.0);
                    d.extend_from_slice(&adjusted.to_le_bytes());
                }
                d
            }
        };

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

    pub fn bit_depth(&self) -> u16 {
        self.bit_depth
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

    /// Wait for WASAPI to signal it needs more data. Returns false on timeout/error.
    pub fn wait_for_buffer_request(&self, timeout_ms: u32) -> bool {
        self.event_handle.wait_for_event(timeout_ms).is_ok()
    }

    /// Get the number of frames available in the buffer for writing.
    pub fn available_frames(&self) -> Result<u32, OutputError> {
        match self.client.get_available_space_in_frames() {
            Ok(frames) => Ok(frames),
            Err(err) if is_device_invalidated(&err) => Err(OutputError::DeviceInvalidated),
            Err(err) => Err(map_wasapi_error(err)),
        }
    }

    /// Write samples from a ring buffer, filling as much of the available space as possible.
    /// For exclusive mode with event-driven timing, this waits for the event handle first.
    pub fn write_from_buffer(
        &mut self,
        ring_buffer: &mut AudioRingBuffer,
        volume: f32,
    ) -> Result<usize, OutputError> {
        if !self.started {
            self.start()?;
        }

        let channels = self.channels as usize;
        if channels == 0 {
            return Ok(0);
        }

        // In exclusive mode, wait for WASAPI to signal it needs data
        if self.is_exclusive {
            if !self.wait_for_buffer_request(100) {
                // Timeout - no data needed yet, or error
                return Ok(0);
            }
        }

        let available_frames = self.available_frames()?;
        if available_frames == 0 {
            return Ok(0);
        }

        let frames_to_write = available_frames as usize;
        let samples_to_write = frames_to_write * channels;

        // Get samples from ring buffer
        let mut samples = vec![0.0f32; samples_to_write];
        ring_buffer.pop_into(&mut samples);

        let vol = if volume.is_finite() {
            volume.clamp(0.0, 1.0)
        } else {
            1.0
        };

        // Convert f32 samples to bytes based on format
        // Use valid_bits to determine actual audio precision, bit_depth for container size
        let data = match (self.sample_type, self.bit_depth, self.valid_bits) {
            (SampleType::Float, 32, _) => {
                let mut d = Vec::with_capacity(samples_to_write * 4);
                for &s in &samples {
                    let adjusted = (s * vol).clamp(-1.0, 1.0);
                    d.extend_from_slice(&adjusted.to_le_bytes());
                }
                d
            }
            (SampleType::Int, 16, 16) => f32_to_i16_le(&samples, vol),
            (SampleType::Int, 24, 24) => {
                // Native 24-bit: 3 bytes per sample
                f32_to_i24_native_le(&samples, vol)
            }
            (SampleType::Int, 32, 24) => {
                // 24-bit in 32-bit container: 4 bytes per sample, 24 valid bits
                f32_to_i24_in_i32_le(&samples, vol)
            }
            (SampleType::Int, 32, 32) => f32_to_i32_le(&samples, vol),
            _ => {
                // Fallback to float
                let mut d = Vec::with_capacity(samples_to_write * 4);
                for &s in &samples {
                    let adjusted = (s * vol).clamp(-1.0, 1.0);
                    d.extend_from_slice(&adjusted.to_le_bytes());
                }
                d
            }
        };

        match self
            .render_client
            .write_to_device(frames_to_write, &data, None)
        {
            Ok(()) => Ok(frames_to_write),
            Err(err) if is_device_invalidated(&err) => {
                let _ = self.stop();
                Err(OutputError::DeviceInvalidated)
            }
            Err(err) => Err(map_wasapi_error(err)),
        }
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

/// Convert f32 samples to 16-bit PCM (little-endian bytes)
pub fn f32_to_i16_le(samples: &[f32], volume: f32) -> Vec<u8> {
    let vol = if volume.is_finite() {
        volume.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let mut out = Vec::with_capacity(samples.len() * 2);
    for &s in samples {
        let sample = (s * vol).clamp(-1.0, 1.0);
        let i16_sample = (sample * 32767.0) as i16;
        out.extend_from_slice(&i16_sample.to_le_bytes());
    }
    out
}

/// Convert f32 samples to 24-bit PCM in 32-bit container (little-endian bytes).
/// Uses 24 valid bits right-justified in the 32-bit container (lower 24 bits).
/// This matches WAVEFORMATEXTENSIBLE with wBitsPerSample=32 and wValidBitsPerSample=24.
pub fn f32_to_i24_in_i32_le(samples: &[f32], volume: f32) -> Vec<u8> {
    let vol = if volume.is_finite() {
        volume.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let mut out = Vec::with_capacity(samples.len() * 4);
    // 24-bit signed integer max is 2^23 - 1 = 8,388,607
    const MAX_24BIT: f32 = 8_388_607.0;

    for &s in samples {
        let sample = (s * vol).clamp(-1.0, 1.0);
        let i24_val = (sample * MAX_24BIT) as i32;
        // Pack into 32-bit container, right-justified (lower 24 bits)
        // The upper 8 bits will be sign-extended naturally by the i32 cast
        // For WASAPI, valid bits are in the LSB position
        out.extend_from_slice(&i24_val.to_le_bytes());
    }
    out
}

/// Convert f32 samples to native 24-bit PCM (3 bytes per sample, little-endian).
/// This is used when the device accepts 24/24 format (not 24-in-32 container).
pub fn f32_to_i24_native_le(samples: &[f32], volume: f32) -> Vec<u8> {
    let vol = if volume.is_finite() {
        volume.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let mut out = Vec::with_capacity(samples.len() * 3);
    // 24-bit signed integer max is 2^23 - 1 = 8,388,607
    const MAX_24BIT: f32 = 8_388_607.0;

    for &s in samples {
        let sample = (s * vol).clamp(-1.0, 1.0);
        let i24_val = (sample * MAX_24BIT) as i32;
        // Write only the lower 3 bytes (little-endian)
        let bytes = i24_val.to_le_bytes();
        out.push(bytes[0]);
        out.push(bytes[1]);
        out.push(bytes[2]);
    }
    out
}

/// Convert f32 samples to 32-bit PCM (little-endian bytes)
pub fn f32_to_i32_le(samples: &[f32], volume: f32) -> Vec<u8> {
    let vol = if volume.is_finite() {
        volume.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let mut out = Vec::with_capacity(samples.len() * 4);
    for &s in samples {
        let sample = (s * vol).clamp(-1.0, 1.0);
        let i32_sample = (sample * 2147483647.0) as i32;
        out.extend_from_slice(&i32_sample.to_le_bytes());
    }
    out
}

pub trait AudioOutput {
    fn start(&mut self) -> Result<(), OutputError>;
    fn stop(&mut self) -> Result<(), OutputError>;
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u16;
    fn bit_depth(&self) -> u16;
    fn is_exclusive(&self) -> bool;
    fn write_samples(&mut self, samples: &[f32], volume: f32) -> Result<(), OutputError>;
    fn write_from_buffer(
        &mut self,
        ring_buffer: &mut AudioRingBuffer,
        volume: f32,
    ) -> Result<usize, OutputError>;
}

impl AudioOutput for WasapiOutput {
    fn start(&mut self) -> Result<(), OutputError> {
        self.start()
    }

    fn stop(&mut self) -> Result<(), OutputError> {
        self.stop()
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate()
    }

    fn channels(&self) -> u16 {
        self.channels()
    }

    fn bit_depth(&self) -> u16 {
        self.bit_depth()
    }

    fn is_exclusive(&self) -> bool {
        self.is_exclusive()
    }

    fn write_samples(&mut self, samples: &[f32], volume: f32) -> Result<(), OutputError> {
        self.write_samples(samples, volume)
    }

    fn write_from_buffer(
        &mut self,
        ring_buffer: &mut AudioRingBuffer,
        volume: f32,
    ) -> Result<usize, OutputError> {
        self.write_from_buffer(ring_buffer, volume)
    }
}

pub struct NullSinkOutput {
    sample_rate: u32,
    channels: u16,
    bit_depth: u16,
    is_exclusive: bool,
    captured_samples: Vec<f32>,
    started: bool,
}

impl NullSinkOutput {
    pub fn new(sample_rate: u32, channels: u16, bit_depth: u16, is_exclusive: bool) -> Self {
        Self {
            sample_rate,
            channels,
            bit_depth,
            is_exclusive,
            captured_samples: Vec::new(),
            started: false,
        }
    }

    pub fn captured_samples(&self) -> &[f32] {
        &self.captured_samples
    }

    pub fn clear_captured(&mut self) {
        self.captured_samples.clear();
    }
}

impl AudioOutput for NullSinkOutput {
    fn start(&mut self) -> Result<(), OutputError> {
        self.started = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), OutputError> {
        self.started = false;
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

    fn is_exclusive(&self) -> bool {
        self.is_exclusive
    }

    fn write_samples(&mut self, samples: &[f32], volume: f32) -> Result<(), OutputError> {
        if !self.started {
            self.start()?;
        }

        let vol = if volume.is_finite() {
            volume.clamp(0.0, 1.0)
        } else {
            1.0
        };

        for &s in samples {
            self.captured_samples.push(s * vol);
        }
        Ok(())
    }

    fn write_from_buffer(
        &mut self,
        ring_buffer: &mut AudioRingBuffer,
        volume: f32,
    ) -> Result<usize, OutputError> {
        if !self.started {
            self.start()?;
        }
        let available = ring_buffer.available_frames();
        let channels = self.channels as usize;
        let samples_count = available * channels;

        let mut samples = vec![0.0; samples_count];
        ring_buffer.pop_into(&mut samples);

        self.write_samples(&samples, volume)?;

        Ok(available)
    }
}

pub enum OutputBackend {
    Wasapi(WasapiOutput),
    NullSink(NullSinkOutput),
}

impl AudioOutput for OutputBackend {
    fn start(&mut self) -> Result<(), OutputError> {
        match self {
            OutputBackend::Wasapi(o) => o.start(),
            OutputBackend::NullSink(o) => o.start(),
        }
    }

    fn stop(&mut self) -> Result<(), OutputError> {
        match self {
            OutputBackend::Wasapi(o) => o.stop(),
            OutputBackend::NullSink(o) => o.stop(),
        }
    }

    fn sample_rate(&self) -> u32 {
        match self {
            OutputBackend::Wasapi(o) => o.sample_rate(),
            OutputBackend::NullSink(o) => o.sample_rate(),
        }
    }

    fn channels(&self) -> u16 {
        match self {
            OutputBackend::Wasapi(o) => o.channels(),
            OutputBackend::NullSink(o) => o.channels(),
        }
    }

    fn bit_depth(&self) -> u16 {
        match self {
            OutputBackend::Wasapi(o) => o.bit_depth(),
            OutputBackend::NullSink(o) => o.bit_depth(),
        }
    }

    fn is_exclusive(&self) -> bool {
        match self {
            OutputBackend::Wasapi(o) => o.is_exclusive(),
            OutputBackend::NullSink(o) => o.is_exclusive(),
        }
    }

    fn write_samples(&mut self, samples: &[f32], volume: f32) -> Result<(), OutputError> {
        match self {
            OutputBackend::Wasapi(o) => o.write_samples(samples, volume),
            OutputBackend::NullSink(o) => o.write_samples(samples, volume),
        }
    }

    fn write_from_buffer(
        &mut self,
        ring_buffer: &mut AudioRingBuffer,
        volume: f32,
    ) -> Result<usize, OutputError> {
        match self {
            OutputBackend::Wasapi(o) => o.write_from_buffer(ring_buffer, volume),
            OutputBackend::NullSink(o) => o.write_from_buffer(ring_buffer, volume),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_to_i16_clamps_correctly() {
        let input = [1.0, 0.5, 0.0, -0.5, -1.0, 1.5, -1.5];
        let volume = 1.0;
        let bytes = f32_to_i16_le(&input, volume);

        // i16 max is 32767
        // 1.0 -> 32767 (0x7FFF) -> LE: FF 7F
        // 0.5 -> 16383 (0x3FFF) -> LE: FF 3F
        // 0.0 -> 0 -> LE: 00 00
        // -0.5 -> -16383 (0xC001) -> LE: 01 C0
        // -1.0 -> -32767 (0x8001) -> LE: 01 80

        assert_eq!(bytes.len(), input.len() * 2);

        let mut i = 0;
        // 1.0
        assert_eq!(bytes[i], 0xFF);
        assert_eq!(bytes[i + 1], 0x7F);
        i += 2;
        // 0.5
        assert_eq!(bytes[i], 0xFF);
        assert_eq!(bytes[i + 1], 0x3F);
        i += 2;
        // 0.0
        assert_eq!(bytes[i], 0x00);
        assert_eq!(bytes[i + 1], 0x00);
        i += 2;
        // -0.5
        assert_eq!(bytes[i], 0x01);
        assert_eq!(bytes[i + 1], 0xC0);
        i += 2;
        // -1.0
        assert_eq!(bytes[i], 0x01);
        assert_eq!(bytes[i + 1], 0x80);
        i += 2;
        // 1.5 -> clamped to 1.0
        assert_eq!(bytes[i], 0xFF);
        assert_eq!(bytes[i + 1], 0x7F);
        i += 2;
        // -1.5 -> clamped to -1.0
        assert_eq!(bytes[i], 0x01);
        assert_eq!(bytes[i + 1], 0x80);
        i += 2;
    }

    #[test]
    fn test_f32_to_i24_in_i32_le() {
        let input = [1.0, 0.0, -1.0];
        let volume = 1.0;
        let bytes = f32_to_i24_in_i32_le(&input, volume);

        // 24-bit values are right-justified in 32-bit container (lower 24 bits)
        // 1.0 * 8388607 = 8388607 (0x007FFFFF)
        // LE: FF FF 7F 00

        // -1.0 * 8388607 = -8388607
        // -8388607 in 32-bit two's complement = 0xFF800001
        // LE: 01 00 80 FF

        assert_eq!(bytes.len(), input.len() * 4);

        let mut i = 0;
        // 1.0 -> 8388607 = 0x007FFFFF -> LE: FF FF 7F 00
        assert_eq!(bytes[i], 0xFF);
        assert_eq!(bytes[i + 1], 0xFF);
        assert_eq!(bytes[i + 2], 0x7F);
        assert_eq!(bytes[i + 3], 0x00);
        i += 4;
        // 0.0 -> 0 = 0x00000000 -> LE: 00 00 00 00
        assert_eq!(bytes[i], 0x00);
        assert_eq!(bytes[i + 1], 0x00);
        assert_eq!(bytes[i + 2], 0x00);
        assert_eq!(bytes[i + 3], 0x00);
        i += 4;
        // -1.0 -> -8388607 = 0xFF800001 -> LE: 01 00 80 FF
        assert_eq!(bytes[i], 0x01);
        assert_eq!(bytes[i + 1], 0x00);
        assert_eq!(bytes[i + 2], 0x80);
        assert_eq!(bytes[i + 3], 0xFF);
    }

    #[test]
    fn test_null_sink_captures_samples() {
        let mut sink = NullSinkOutput::new(44100, 2, 16, false);
        sink.start().unwrap();
        let samples = [0.5, -0.5, 0.25, -0.25];
        sink.write_samples(&samples, 1.0).unwrap();
        assert_eq!(sink.captured_samples(), &samples);
    }

    #[test]
    fn test_f32_to_i24_native_le() {
        let input = [1.0, 0.0, -1.0];
        let volume = 1.0;
        let bytes = f32_to_i24_native_le(&input, volume);

        // Native 24-bit: 3 bytes per sample
        // 1.0 * 8388607 = 8388607 (0x7FFFFF)
        // LE: FF FF 7F

        // -1.0 * 8388607 = -8388607
        // -8388607 in 24-bit two's complement = 0x800001
        // LE: 01 00 80

        assert_eq!(bytes.len(), input.len() * 3);

        let mut i = 0;
        // 1.0 -> 8388607 = 0x7FFFFF -> LE: FF FF 7F
        assert_eq!(bytes[i], 0xFF);
        assert_eq!(bytes[i + 1], 0xFF);
        assert_eq!(bytes[i + 2], 0x7F);
        i += 3;
        // 0.0 -> 0 = 0x000000 -> LE: 00 00 00
        assert_eq!(bytes[i], 0x00);
        assert_eq!(bytes[i + 1], 0x00);
        assert_eq!(bytes[i + 2], 0x00);
        i += 3;
        // -1.0 -> -8388607 = 0x800001 -> LE: 01 00 80
        assert_eq!(bytes[i], 0x01);
        assert_eq!(bytes[i + 1], 0x00);
        assert_eq!(bytes[i + 2], 0x80);
    }

    #[test]
    fn test_null_sink_volume() {
        let mut sink = NullSinkOutput::new(44100, 2, 16, false);
        sink.write_samples(&[1.0, -1.0], 0.5).unwrap();
        assert_eq!(sink.captured_samples(), &[0.5, -0.5]);
    }
}
