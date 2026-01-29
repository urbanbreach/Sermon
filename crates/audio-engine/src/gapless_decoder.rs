//! Gapless decoder with dual-decoder management for seamless track transitions.
//!
//! This module provides `GaplessDecoder`, which manages current and next track decoders
//! for gapless playback. It handles:
//!
//! - Loading tracks into memory via `MemoryAudioSource`
//! - Encoder delay trimming for lossy formats (MP3/AAC)
//! - Preloading the next track before current track ends
//! - Seamless transition from current to next track
//!
//! # Usage
//!
//! ```rust,ignore
//! use std::path::Path;
//! use audio_engine::gapless_decoder::GaplessDecoder;
//!
//! let mut decoder = GaplessDecoder::new(Path::new("track1.flac"))?;
//!
//! // Preload next track ~2 seconds before current ends
//! decoder.preload_next(Path::new("track2.flac"))?;
//!
//! // Decode samples - automatically transitions to next track
//! while let Some(samples) = decoder.decode_next()? {
//!     // Process samples
//! }
//! ```

use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;
use tracing::warn;

use crate::decode::DecodeError;
use crate::encoder_delay::EncoderDelay;
use crate::memory_source::MemoryAudioSource;

/// Audio format information for a decoded track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioFormat {
    /// Sample rate in Hz (e.g., 44100, 48000, 96000).
    pub sample_rate: u32,
    /// Number of audio channels (1 = mono, 2 = stereo).
    pub channels: u16,
}

impl AudioFormat {
    /// Check if two formats are compatible for gapless transition.
    ///
    /// Formats are compatible if they have the same sample rate and
    /// channel count. Bit depth is not considered as all audio is
    /// converted to f32 internally.
    pub fn is_gapless_compatible(&self, other: &AudioFormat) -> bool {
        self.sample_rate == other.sample_rate && self.channels == other.channels
    }
}

/// Type of transition between tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionType {
    /// Same format - seamless transition possible.
    Gapless,
    /// Different format - requires WASAPI reinit (gap expected).
    FormatChange,
    /// No next track in queue.
    EndOfQueue,
}

/// Internal state for a single track's decoder.
struct DecoderState {
    /// The memory-backed audio source (keeps file data alive).
    #[allow(dead_code)]
    source: MemoryAudioSource,
    /// File data for encoder delay detection.
    file_data: Vec<u8>,
    /// Symphonia format reader.
    format_reader: Box<dyn FormatReader>,
    /// Symphonia codec decoder.
    decoder: Box<dyn Decoder>,
    /// Encoder delay info for lossy formats.
    encoder_delay: Option<EncoderDelay>,
    /// Number of samples (per channel) decoded so far.
    samples_decoded: u64,
    /// Total samples (per channel) in the track, if known.
    total_samples: Option<u64>,
    /// Sample rate in Hz.
    sample_rate: u32,
    /// Number of channels.
    channels: usize,
    /// Track ID for packet filtering.
    track_id: u32,
    /// Reusable sample buffer.
    sample_buffer: Option<SampleBuffer<f32>>,
}

impl DecoderState {
    fn new(path: &Path) -> Result<Self, DecodeError> {
        let source = MemoryAudioSource::load(path)?;
        let file_data = std::fs::read(path)?;

        let source_for_probe = MemoryAudioSource::load(path)?;
        let mss = MediaSourceStream::new(Box::new(source_for_probe), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(map_probe_error)?;

        let format_reader = probed.format;

        let track = format_reader
            .default_track()
            .cloned()
            .ok_or(DecodeError::NoAudioTrack)?;

        if track.codec_params.codec == CODEC_TYPE_NULL {
            return Err(DecodeError::UnsupportedFormat);
        }

        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| DecodeError::DecoderError(e.to_string()))?;

        let sample_rate = track.codec_params.sample_rate.unwrap_or(0);
        let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(0);
        let total_samples = track.codec_params.n_frames;

        let encoder_delay = EncoderDelay::detect(&file_data);

        Ok(Self {
            source,
            file_data,
            format_reader,
            decoder,
            encoder_delay,
            samples_decoded: 0,
            total_samples,
            sample_rate,
            channels,
            track_id: track.id,
            sample_buffer: None,
        })
    }

    fn new_from_bytes(bytes: Vec<u8>) -> Result<Self, DecodeError> {
        let source = MemoryAudioSource::from_bytes(bytes.clone());
        let file_data = bytes;

        let source_for_probe = MemoryAudioSource::from_bytes(file_data.clone());
        let mss = MediaSourceStream::new(Box::new(source_for_probe), Default::default());

        let hint = Hint::new();

        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(map_probe_error)?;

        let format_reader = probed.format;

        let track = format_reader
            .default_track()
            .cloned()
            .ok_or(DecodeError::NoAudioTrack)?;

        if track.codec_params.codec == CODEC_TYPE_NULL {
            return Err(DecodeError::UnsupportedFormat);
        }

        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| DecodeError::DecoderError(e.to_string()))?;

        let sample_rate = track.codec_params.sample_rate.unwrap_or(0);
        let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(0);
        let total_samples = track.codec_params.n_frames;

        let encoder_delay = EncoderDelay::detect(&file_data);

        Ok(Self {
            source,
            file_data,
            format_reader,
            decoder,
            encoder_delay,
            samples_decoded: 0,
            total_samples,
            sample_rate,
            channels,
            track_id: track.id,
            sample_buffer: None,
        })
    }

    fn decode_raw(&mut self) -> Result<Option<&[f32]>, DecodeError> {
        loop {
            let packet = match self.format_reader.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(err))
                    if err.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    return Ok(None);
                }
                Err(SymphoniaError::IoError(err)) => return Err(DecodeError::Io(err)),
                Err(SymphoniaError::DecodeError(desc)) => {
                    warn!(%desc, "Skipping malformed packet from demuxer");
                    continue;
                }
                Err(SymphoniaError::ResetRequired) => {
                    self.decoder.reset();
                    continue;
                }
                Err(err) => return Err(DecodeError::DecoderError(err.to_string())),
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(SymphoniaError::DecodeError(desc)) => {
                    warn!(%desc, "Skipping undecodable packet");
                    continue;
                }
                Err(SymphoniaError::ResetRequired) => {
                    self.decoder.reset();
                    continue;
                }
                Err(SymphoniaError::IoError(err)) => return Err(DecodeError::Io(err)),
                Err(err) => return Err(DecodeError::DecoderError(err.to_string())),
            };

            let spec = *decoded.spec();
            self.sample_rate = spec.rate;
            self.channels = spec.channels.count();

            let required_frames = decoded.capacity();
            let required_samples = required_frames * self.channels;

            let needs_realloc = match self.sample_buffer.as_ref() {
                Some(buffer) => buffer.capacity() < required_samples,
                None => true,
            };

            if needs_realloc {
                self.sample_buffer = Some(SampleBuffer::<f32>::new(required_frames as u64, spec));
            }

            let buffer = self.sample_buffer.as_mut().expect("just set");
            buffer.copy_interleaved_ref(decoded);
            return Ok(Some(buffer.samples()));
        }
    }

    /// Seek to a specific position in the track.
    ///
    /// This resets the decoder state and re-aligns the encoder delay trimming
    /// to the new position.
    fn seek(&mut self, position_ms: u64) -> Result<(), DecodeError> {
        let time = Time::from(position_ms as f64 / 1000.0);

        self.format_reader
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time,
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|err| DecodeError::DecoderError(err.to_string()))?;

        // Reset codec state (matches AudioDecoder::seek behavior)
        self.decoder.reset();

        // Re-align trimming state (per-channel sample index)
        let seek_samples = (position_ms as u64 * self.sample_rate as u64) / 1000;
        self.samples_decoded = match self.total_samples {
            Some(total) => seek_samples.min(total),
            None => seek_samples,
        };

        Ok(())
    }
}

/// Gapless decoder with dual-decoder management for seamless track transitions.
///
/// `GaplessDecoder` manages a current track decoder and optionally a preloaded
/// next track decoder. It handles:
///
/// - Encoder delay trimming for lossy formats (MP3/AAC)
/// - Automatic transition from current to next track when current is exhausted
/// - Preloading the next track before the current track ends
///
/// # Track Transitions
///
/// The decoder automatically transitions from the current track to the next
/// when `decode_next()` returns `None` for the current track and a next track
/// has been preloaded. The transition is seamless - the next call to
/// `decode_next()` will return samples from the new track.
pub struct GaplessDecoder {
    /// Current track decoder state.
    current: DecoderState,
    /// Optional preloaded next track decoder state.
    next: Option<DecoderState>,
    /// Samples before end to trigger preload (approximately 2 seconds).
    #[allow(dead_code)]
    preload_trigger_samples: u64,
    /// Flag set when an internal gapless transition just occurred.
    just_transitioned: bool,
    /// Reusable decode buffer to avoid allocations.
    decode_buffer: Vec<f32>,
}

impl GaplessDecoder {
    /// Default preload trigger: approximately 2 seconds at 44.1kHz.
    const DEFAULT_PRELOAD_TRIGGER: u64 = 44100 * 2;

    /// Create a new gapless decoder for the given track.
    ///
    /// The track is loaded entirely into memory for fast seeking and
    /// to avoid I/O blocking during playback.
    ///
    /// # Arguments
    ///
    /// * `track_path` - Path to the audio file to decode
    ///
    /// # Errors
    ///
    /// Returns `DecodeError` if the file cannot be read, the format is
    /// unsupported, or no audio track is found.
    pub fn new(track_path: &Path) -> Result<Self, DecodeError> {
        let current = DecoderState::new(track_path)?;
        let preload_trigger = current.sample_rate as u64 * 2; // ~2 seconds

        Ok(Self {
            current,
            next: None,
            preload_trigger_samples: preload_trigger.max(Self::DEFAULT_PRELOAD_TRIGGER),
            just_transitioned: false,
            decode_buffer: Vec::with_capacity(4096),
        })
    }

    /// Decode the next chunk of samples from the current track.
    ///
    /// This method applies encoder delay trimming for lossy formats:
    /// - Skips `start_samples` at the beginning of the track
    /// - Stops `end_samples` before the end of the track
    ///
    /// When the current track is exhausted and a next track has been preloaded,
    /// the decoder automatically transitions to the next track. The next call
    /// to `decode_next()` will return samples from the new track.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(samples))` - Decoded samples (interleaved f32)
    /// - `Ok(None)` - Track exhausted (and no preloaded next track)
    /// - `Err(DecodeError)` - Decoding error
    pub fn decode_next(&mut self) -> Result<Option<Vec<f32>>, DecodeError> {
        loop {
            // Use a block to limit the lifetime of the borrow from self.current
            let raw_samples_len = {
                let raw_samples = self.current.decode_raw()?;
                match raw_samples {
                    Some(samples) => {
                        self.decode_buffer.clear();
                        self.decode_buffer.extend_from_slice(samples);
                        Some(self.decode_buffer.len())
                    }
                    None => None,
                }
            };

            match raw_samples_len {
                Some(_) => {
                    let trimmed = self.apply_encoder_delay_trimming();
                    if let Some(trimmed_samples) = trimmed {
                        return Ok(Some(trimmed_samples));
                    }
                    continue;
                }
                None => {
                    if self.next.is_some() {
                        self.current = self.next.take().unwrap();
                        self.just_transitioned = true;
                        continue;
                    }
                    return Ok(None);
                }
            }
        }
    }

    /// Apply encoder delay trimming to the samples in decode_buffer.
    ///
    /// Returns `None` if all samples were trimmed (start of track).
    fn apply_encoder_delay_trimming(&mut self) -> Option<Vec<f32>> {
        let channels = self.current.channels;
        if channels == 0 {
            return Some(self.decode_buffer.clone());
        }

        let frames = self.decode_buffer.len() / channels;
        let samples_before = self.current.samples_decoded;
        let samples_after = samples_before + frames as u64;

        self.current.samples_decoded = samples_after;

        let delay = match &self.current.encoder_delay {
            Some(d) if d.has_trimming() => d,
            _ => return Some(self.decode_buffer.clone()),
        };

        let start_trim = delay.start_samples;
        let end_trim = delay.end_samples;

        let end_boundary = self
            .current
            .total_samples
            .map(|total| total.saturating_sub(end_trim));

        let keep_start = if samples_before < start_trim {
            let skip_frames = (start_trim - samples_before).min(frames as u64) as usize;
            skip_frames
        } else {
            0
        };

        let keep_end = if let Some(end_bound) = end_boundary {
            if samples_after > end_bound {
                let total_keep = end_bound.saturating_sub(samples_before) as usize;
                total_keep.min(frames)
            } else {
                frames
            }
        } else {
            frames
        };

        if keep_start >= keep_end {
            return None;
        }

        let start_idx = keep_start * channels;
        let end_idx = keep_end * channels;

        Some(self.decode_buffer[start_idx..end_idx].to_vec())
    }

    /// Preload the next track for seamless transition.
    ///
    /// The next track is loaded into memory and its decoder is initialized.
    /// When the current track is exhausted, `decode_next()` will automatically
    /// transition to the preloaded track.
    ///
    /// # Arguments
    ///
    /// * `track_path` - Path to the next audio file to preload
    ///
    /// # Errors
    ///
    /// Returns `DecodeError` if the file cannot be read or decoded.
    /// The current playback is not affected if preloading fails.
    pub fn preload_next(&mut self, track_path: &Path) -> Result<(), DecodeError> {
        self.next = Some(DecoderState::new(track_path)?);
        Ok(())
    }

    /// Preload the next track from in-memory bytes.
    ///
    /// This avoids blocking file I/O on the calling thread.
    pub fn preload_next_from_bytes(&mut self, bytes: Vec<u8>) -> Result<(), DecodeError> {
        self.next = Some(DecoderState::new_from_bytes(bytes)?);
        Ok(())
    }

    /// Cancel any preloaded next track.
    ///
    /// This releases the memory used by the preloaded track's decoder.
    pub fn cancel_preload(&mut self) {
        self.next = None;
    }

    /// Check and clear the transition flag. Returns true if a gapless transition just occurred.
    pub fn take_just_transitioned(&mut self) -> bool {
        let v = self.just_transitioned;
        self.just_transitioned = false;
        v
    }

    /// Seek to a specific position in the current track.
    pub fn seek(&mut self, position_ms: u64) -> Result<(), DecodeError> {
        self.next = None;
        self.just_transitioned = false;
        self.current.seek(position_ms)
    }

    /// Get the audio format of the current track.
    ///
    /// Returns the sample rate and channel count of the currently
    /// playing track.
    pub fn format(&self) -> AudioFormat {
        AudioFormat {
            sample_rate: self.current.sample_rate,
            channels: self.current.channels as u16,
        }
    }

    /// Check if a next track is preloaded.
    pub fn has_preloaded_next(&self) -> bool {
        self.next.is_some()
    }

    /// Get the encoder delay info for the current track.
    ///
    /// Returns `None` if the track has no encoder delay (lossless formats).
    pub fn encoder_delay(&self) -> Option<&EncoderDelay> {
        self.current.encoder_delay.as_ref()
    }

    /// Get the total samples in the current track, if known.
    pub fn total_samples(&self) -> Option<u64> {
        self.current.total_samples
    }

    /// Get the number of samples decoded so far from the current track.
    pub fn samples_decoded(&self) -> u64 {
        self.current.samples_decoded
    }

    /// Check if gapless transition is possible to the preloaded next track.
    pub fn can_transition_gapless(&self) -> bool {
        self.next.as_ref().map_or(false, |next| {
            let next_format = AudioFormat {
                sample_rate: next.sample_rate,
                channels: next.channels as u16,
            };
            self.format().is_gapless_compatible(&next_format)
        })
    }

    /// Get the type of transition that will occur when current track ends.
    pub fn transition_type(&self) -> TransitionType {
        match &self.next {
            None => TransitionType::EndOfQueue,
            Some(next) => {
                let next_format = AudioFormat {
                    sample_rate: next.sample_rate,
                    channels: next.channels as u16,
                };
                if self.format().is_gapless_compatible(&next_format) {
                    TransitionType::Gapless
                } else {
                    TransitionType::FormatChange
                }
            }
        }
    }

    /// Get the format of the preloaded next track, if any.
    pub fn next_format(&self) -> Option<AudioFormat> {
        self.next.as_ref().map(|next| AudioFormat {
            sample_rate: next.sample_rate,
            channels: next.channels as u16,
        })
    }
}

/// Map Symphonia probe errors to DecodeError.
fn map_probe_error(err: SymphoniaError) -> DecodeError {
    match err {
        SymphoniaError::Unsupported(_) => DecodeError::UnsupportedFormat,
        SymphoniaError::IoError(err) => DecodeError::Io(err),
        other => DecodeError::DecoderError(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Create a minimal WAV file with the given samples.
    fn create_test_wav(samples: &[i16]) -> NamedTempFile {
        let mut file = NamedTempFile::with_suffix(".wav").expect("failed to create temp file");

        let channels: u16 = 1;
        let sample_rate: u32 = 44100;
        let bits_per_sample: u16 = 16;
        let byte_rate = sample_rate * (channels as u32) * (bits_per_sample as u32) / 8;
        let block_align = channels * bits_per_sample / 8;
        let data_size = (samples.len() * 2) as u32;
        let file_size = 36 + data_size;

        // RIFF header
        file.write_all(b"RIFF").unwrap();
        file.write_all(&file_size.to_le_bytes()).unwrap();
        file.write_all(b"WAVE").unwrap();

        // fmt chunk
        file.write_all(b"fmt ").unwrap();
        file.write_all(&16u32.to_le_bytes()).unwrap(); // chunk size
        file.write_all(&1u16.to_le_bytes()).unwrap(); // PCM format
        file.write_all(&channels.to_le_bytes()).unwrap();
        file.write_all(&sample_rate.to_le_bytes()).unwrap();
        file.write_all(&byte_rate.to_le_bytes()).unwrap();
        file.write_all(&block_align.to_le_bytes()).unwrap();
        file.write_all(&bits_per_sample.to_le_bytes()).unwrap();

        // data chunk
        file.write_all(b"data").unwrap();
        file.write_all(&data_size.to_le_bytes()).unwrap();
        for sample in samples {
            file.write_all(&sample.to_le_bytes()).unwrap();
        }

        file.flush().unwrap();
        file
    }

    #[test]
    fn test_gapless_decoder_single_track() {
        let samples: Vec<i16> = (0..1000).map(|i| (i % 1000) as i16).collect();
        let wav_file = create_test_wav(&samples);

        let mut decoder =
            GaplessDecoder::new(wav_file.path()).expect("failed to create gapless decoder");

        let format = decoder.format();
        assert_eq!(format.sample_rate, 44100);
        assert_eq!(format.channels, 1);

        let mut decoded_samples = Vec::new();
        while let Some(chunk) = decoder.decode_next().expect("decode failed") {
            decoded_samples.extend(chunk);
        }

        assert_eq!(decoded_samples.len(), samples.len());
    }

    #[test]
    fn test_gapless_decoder_transition() {
        let samples1: Vec<i16> = (0..100).map(|i| i as i16).collect();
        let samples2: Vec<i16> = (100..200).map(|i| i as i16).collect();

        let wav_file1 = create_test_wav(&samples1);
        let wav_file2 = create_test_wav(&samples2);

        let mut decoder =
            GaplessDecoder::new(wav_file1.path()).expect("failed to create gapless decoder");

        decoder
            .preload_next(wav_file2.path())
            .expect("failed to preload next track");
        assert!(decoder.has_preloaded_next());

        let mut decoded_samples = Vec::new();
        while let Some(chunk) = decoder.decode_next().expect("decode failed") {
            decoded_samples.extend(chunk);
        }

        assert_eq!(decoded_samples.len(), samples1.len() + samples2.len());
        assert!(!decoder.has_preloaded_next());
    }

    #[test]
    fn test_gapless_decoder_preload_cancel() {
        let samples: Vec<i16> = (0..100).collect();
        let wav_file = create_test_wav(&samples);

        let mut decoder =
            GaplessDecoder::new(wav_file.path()).expect("failed to create gapless decoder");

        decoder
            .preload_next(wav_file.path())
            .expect("failed to preload");
        assert!(decoder.has_preloaded_next());

        decoder.cancel_preload();
        assert!(!decoder.has_preloaded_next());
    }

    #[test]
    fn test_audio_format_equality() {
        let format1 = AudioFormat {
            sample_rate: 44100,
            channels: 2,
        };
        let format2 = AudioFormat {
            sample_rate: 44100,
            channels: 2,
        };
        let format3 = AudioFormat {
            sample_rate: 48000,
            channels: 2,
        };

        assert_eq!(format1, format2);
        assert_ne!(format1, format3);
    }

    #[test]
    fn test_gapless_decoder_no_encoder_delay_for_lossless() {
        let samples: Vec<i16> = (0..100).collect();
        let wav_file = create_test_wav(&samples);

        let decoder =
            GaplessDecoder::new(wav_file.path()).expect("failed to create gapless decoder");

        assert!(decoder.encoder_delay().is_none());
    }

    #[test]
    fn test_gapless_decoder_samples_decoded_tracking() {
        let samples: Vec<i16> = (0..1000).collect();
        let wav_file = create_test_wav(&samples);

        let mut decoder =
            GaplessDecoder::new(wav_file.path()).expect("failed to create gapless decoder");

        assert_eq!(decoder.samples_decoded(), 0);

        let _ = decoder.decode_next().expect("decode failed");
        assert!(decoder.samples_decoded() > 0);
    }

    #[test]
    fn test_audio_format_is_gapless_compatible_same_format() {
        let format1 = AudioFormat {
            sample_rate: 44100,
            channels: 2,
        };
        let format2 = AudioFormat {
            sample_rate: 44100,
            channels: 2,
        };
        assert!(format1.is_gapless_compatible(&format2));
    }

    #[test]
    fn test_audio_format_is_gapless_compatible_different_sample_rate() {
        let format1 = AudioFormat {
            sample_rate: 44100,
            channels: 2,
        };
        let format2 = AudioFormat {
            sample_rate: 48000,
            channels: 2,
        };
        assert!(!format1.is_gapless_compatible(&format2));
    }

    #[test]
    fn test_audio_format_is_gapless_compatible_different_channels() {
        let format1 = AudioFormat {
            sample_rate: 44100,
            channels: 2,
        };
        let format2 = AudioFormat {
            sample_rate: 44100,
            channels: 1,
        };
        assert!(!format1.is_gapless_compatible(&format2));
    }

    #[test]
    fn test_audio_format_is_gapless_compatible_different_both() {
        let format1 = AudioFormat {
            sample_rate: 44100,
            channels: 2,
        };
        let format2 = AudioFormat {
            sample_rate: 96000,
            channels: 6,
        };
        assert!(!format1.is_gapless_compatible(&format2));
    }

    #[test]
    fn test_transition_type_end_of_queue() {
        let samples: Vec<i16> = (0..100).collect();
        let wav_file = create_test_wav(&samples);

        let decoder =
            GaplessDecoder::new(wav_file.path()).expect("failed to create gapless decoder");

        assert_eq!(decoder.transition_type(), TransitionType::EndOfQueue);
        assert!(!decoder.can_transition_gapless());
        assert!(decoder.next_format().is_none());
    }

    #[test]
    fn test_transition_type_gapless_same_format() {
        let samples: Vec<i16> = (0..100).collect();
        let wav_file1 = create_test_wav(&samples);
        let wav_file2 = create_test_wav(&samples);

        let mut decoder =
            GaplessDecoder::new(wav_file1.path()).expect("failed to create gapless decoder");
        decoder
            .preload_next(wav_file2.path())
            .expect("failed to preload");

        assert_eq!(decoder.transition_type(), TransitionType::Gapless);
        assert!(decoder.can_transition_gapless());

        let next_fmt = decoder.next_format().expect("should have next format");
        assert_eq!(next_fmt.sample_rate, 44100);
        assert_eq!(next_fmt.channels, 1);
    }

    fn create_test_wav_with_format(
        samples: &[i16],
        sample_rate: u32,
        channels: u16,
    ) -> NamedTempFile {
        let mut file = NamedTempFile::with_suffix(".wav").expect("failed to create temp file");

        let bits_per_sample: u16 = 16;
        let byte_rate = sample_rate * (channels as u32) * (bits_per_sample as u32) / 8;
        let block_align = channels * bits_per_sample / 8;
        let data_size = (samples.len() * 2) as u32;
        let file_size = 36 + data_size;

        file.write_all(b"RIFF").unwrap();
        file.write_all(&file_size.to_le_bytes()).unwrap();
        file.write_all(b"WAVE").unwrap();

        file.write_all(b"fmt ").unwrap();
        file.write_all(&16u32.to_le_bytes()).unwrap();
        file.write_all(&1u16.to_le_bytes()).unwrap();
        file.write_all(&channels.to_le_bytes()).unwrap();
        file.write_all(&sample_rate.to_le_bytes()).unwrap();
        file.write_all(&byte_rate.to_le_bytes()).unwrap();
        file.write_all(&block_align.to_le_bytes()).unwrap();
        file.write_all(&bits_per_sample.to_le_bytes()).unwrap();

        file.write_all(b"data").unwrap();
        file.write_all(&data_size.to_le_bytes()).unwrap();
        for sample in samples {
            file.write_all(&sample.to_le_bytes()).unwrap();
        }

        file.flush().unwrap();
        file
    }

    #[test]
    fn test_transition_type_format_change_sample_rate() {
        let samples1: Vec<i16> = (0..100).collect();
        let samples2: Vec<i16> = (0..100).collect();
        let wav_file1 = create_test_wav_with_format(&samples1, 44100, 1);
        let wav_file2 = create_test_wav_with_format(&samples2, 48000, 1);

        let mut decoder =
            GaplessDecoder::new(wav_file1.path()).expect("failed to create gapless decoder");
        decoder
            .preload_next(wav_file2.path())
            .expect("failed to preload");

        assert_eq!(decoder.transition_type(), TransitionType::FormatChange);
        assert!(!decoder.can_transition_gapless());

        let next_fmt = decoder.next_format().expect("should have next format");
        assert_eq!(next_fmt.sample_rate, 48000);
    }

    #[test]
    fn test_transition_type_format_change_channels() {
        let samples1: Vec<i16> = (0..100).collect();
        let samples2: Vec<i16> = (0..200).collect();
        let wav_file1 = create_test_wav_with_format(&samples1, 44100, 1);
        let wav_file2 = create_test_wav_with_format(&samples2, 44100, 2);

        let mut decoder =
            GaplessDecoder::new(wav_file1.path()).expect("failed to create gapless decoder");
        decoder
            .preload_next(wav_file2.path())
            .expect("failed to preload");

        assert_eq!(decoder.transition_type(), TransitionType::FormatChange);
        assert!(!decoder.can_transition_gapless());

        let next_fmt = decoder.next_format().expect("should have next format");
        assert_eq!(next_fmt.channels, 2);
    }

    #[test]
    fn test_short_track_total_samples() {
        let samples: Vec<i16> = (0..1000).collect();
        let wav_file = create_test_wav(&samples);

        let decoder = GaplessDecoder::new(wav_file.path()).expect("failed to create decoder");

        assert!(decoder.total_samples().is_some());
        let total = decoder.total_samples().unwrap();
        assert!(total < 88200);
        assert_eq!(total, 1000);
    }

    #[test]
    fn test_short_track_gapless_transition() {
        let samples: Vec<i16> = (0..500).collect();
        let wav_file1 = create_test_wav(&samples);
        let wav_file2 = create_test_wav(&samples);

        let mut decoder = GaplessDecoder::new(wav_file1.path()).expect("failed");
        decoder
            .preload_next(wav_file2.path())
            .expect("failed to preload");

        let mut all_samples = Vec::new();
        while let Some(chunk) = decoder.decode_next().expect("decode failed") {
            all_samples.extend(chunk);
        }

        assert_eq!(all_samples.len(), 1000);
        assert_eq!(decoder.transition_type(), TransitionType::EndOfQueue);
    }

    #[test]
    fn test_preload_nonexistent_file() {
        let samples: Vec<i16> = (0..100).collect();
        let wav_file = create_test_wav(&samples);

        let mut decoder = GaplessDecoder::new(wav_file.path()).expect("failed");

        let result = decoder.preload_next(std::path::Path::new("/nonexistent/file.wav"));
        assert!(result.is_err());
        assert!(!decoder.has_preloaded_next());
        assert_eq!(decoder.transition_type(), TransitionType::EndOfQueue);
    }

    #[test]
    fn test_gapless_decoder_seek() {
        let sample_rate = 44100u32;
        let samples: Vec<i16> = (0..(sample_rate as usize * 2))
            .map(|i| (i % 1000) as i16)
            .collect();
        let wav_file = create_test_wav(&samples);

        let mut decoder = GaplessDecoder::new(wav_file.path()).expect("failed to create decoder");

        decoder.seek(1000).expect("seek failed");

        let chunk = decoder
            .decode_next()
            .expect("decode failed")
            .expect("expected samples");
        assert!(!chunk.is_empty());

        let expected_samples = (1000u64 * sample_rate as u64) / 1000;
        assert!(decoder.samples_decoded() >= expected_samples);
    }

    #[test]
    fn test_gapless_decoder_seek_cancels_preload() {
        let samples: Vec<i16> = (0..1000).map(|i| (i % 1000) as i16).collect();
        let wav1 = create_test_wav(&samples);
        let wav2 = create_test_wav(&samples);

        let mut decoder = GaplessDecoder::new(wav1.path()).expect("failed to create decoder");
        decoder.preload_next(wav2.path()).expect("preload failed");
        assert!(decoder.has_preloaded_next());

        decoder.seek(0).expect("seek failed");
        assert!(!decoder.has_preloaded_next());
    }

    #[test]
    fn test_gapless_decoder_transition_flag() {
        let samples1: Vec<i16> = (0..100).map(|i| i as i16).collect();
        let samples2: Vec<i16> = (100..200).map(|i| i as i16).collect();
        let wav1 = create_test_wav(&samples1);
        let wav2 = create_test_wav(&samples2);

        let mut decoder = GaplessDecoder::new(wav1.path()).expect("failed");
        decoder.preload_next(wav2.path()).expect("preload failed");

        assert!(!decoder.take_just_transitioned());

        let mut transitioned = false;
        while let Some(_chunk) = decoder.decode_next().expect("decode failed") {
            if decoder.take_just_transitioned() {
                transitioned = true;
                assert!(!decoder.take_just_transitioned());
                break;
            }
        }

        assert!(transitioned);
    }

    #[test]
    fn test_gapless_decoder_seek_resets_transition_flag() {
        // Create a longer file (~1 second at 44100 Hz) to allow seeking
        let samples: Vec<i16> = (0..44100).map(|i| (i % 1000) as i16).collect();
        let wav = create_test_wav(&samples);

        let mut decoder = GaplessDecoder::new(wav.path()).expect("failed");

        // Seek to 500ms (well within the ~1 second file)
        decoder.seek(500).expect("seek failed");
        assert!(!decoder.take_just_transitioned());
    }
}
