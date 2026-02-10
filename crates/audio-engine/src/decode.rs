use std::fs::File;
use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, Decoder, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported format")]
    UnsupportedFormat,

    #[error("No audio track found")]
    NoAudioTrack,

    #[error("Decoder error: {0}")]
    DecoderError(String),
}

pub struct AudioDecoder {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    sample_rate: u32,
    channels: usize,
    bit_depth: Option<u32>, // None for lossy formats like MP3
    sample_buffer: Option<SampleBuffer<f32>>,
}

#[derive(Debug, Clone, Copy)]
pub struct DecodePacketInfo {
    pub frame_count: usize,
    pub decoded: bool,
}

pub enum DecodePacketRef<'a> {
    Skipped {
        frame_count: usize,
    },
    Decoded {
        frame_count: usize,
        interleaved: &'a [f32],
    },
}

impl AudioDecoder {
    pub fn open(path: &Path) -> Result<Self, DecodeError> {
        let file = File::open(path)?;
        let mss = MediaSourceStream::new(
            Box::new(file),
            MediaSourceStreamOptions {
                // Larger read-back buffer improves demux throughput for
                // sequential peak extraction scans.
                buffer_len: 1024 * 1024,
            },
        );

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
            hint.with_extension(ext);
        }

        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(map_open_error)?;

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
        let channels = track
            .codec_params
            .channels
            .map(|channels| channels.count())
            .unwrap_or(0);
        let bit_depth = track.codec_params.bits_per_sample;

        Ok(Self {
            format_reader,
            decoder,
            track_id: track.id,
            sample_rate,
            channels,
            bit_depth,
            sample_buffer: None,
        })
    }

    pub fn decode_next(&mut self) -> Result<Option<Vec<f32>>, DecodeError> {
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

            let needs_resize = self
                .sample_buffer
                .as_ref()
                .map(|buffer| buffer.capacity() < required_samples)
                .unwrap_or(true);

            if needs_resize {
                self.sample_buffer = Some(SampleBuffer::<f32>::new(required_frames as u64, spec));
            }

            let buffer = self
                .sample_buffer
                .as_mut()
                .expect("sample buffer should be set");

            buffer.copy_interleaved_ref(decoded);
            return Ok(Some(buffer.samples().to_vec()));
        }
    }

    /// Decode next packet into a caller-provided buffer, avoiding allocation.
    ///
    /// Clears `out`, then copies decoded samples into it. Reuses internal SampleBuffer.
    /// Returns `Ok(Some(sample_count))` on success, `Ok(None)` at EOF.
    pub fn decode_next_into(&mut self, out: &mut Vec<f32>) -> Result<Option<usize>, DecodeError> {
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

            let needs_resize = self
                .sample_buffer
                .as_ref()
                .map(|buffer| buffer.capacity() < required_samples)
                .unwrap_or(true);

            if needs_resize {
                self.sample_buffer = Some(SampleBuffer::<f32>::new(required_frames as u64, spec));
            }

            let buffer = self
                .sample_buffer
                .as_mut()
                .expect("sample buffer should be set");

            buffer.copy_interleaved_ref(decoded);
            let samples = buffer.samples();
            out.clear();
            out.extend_from_slice(samples);
            return Ok(Some(samples.len()));
        }
    }

    /// Decode next packet into `out` only when `decode_packet` is true.
    ///
    /// When `decode_packet` is false, this may skip decoding and return
    /// frame timing from packet duration (`Packet::dur`) when available.
    /// If packet duration is unavailable, it falls back to decoding to keep
    /// timing accurate.
    pub fn decode_next_into_maybe(
        &mut self,
        out: &mut Vec<f32>,
        decode_packet: bool,
    ) -> Result<Option<DecodePacketInfo>, DecodeError> {
        match self.decode_next_ref_maybe(decode_packet)? {
            Some(DecodePacketRef::Skipped { frame_count }) => {
                out.clear();
                Ok(Some(DecodePacketInfo {
                    frame_count,
                    decoded: false,
                }))
            }
            Some(DecodePacketRef::Decoded {
                frame_count,
                interleaved,
            }) => {
                out.clear();
                out.extend_from_slice(interleaved);
                Ok(Some(DecodePacketInfo {
                    frame_count,
                    decoded: true,
                }))
            }
            None => Ok(None),
        }
    }

    /// Decode next packet and return a borrowed interleaved f32 slice from
    /// the internal decode buffer.
    pub fn decode_next_ref(&mut self) -> Result<Option<&[f32]>, DecodeError> {
        match self.decode_next_ref_maybe(true)? {
            Some(DecodePacketRef::Decoded { interleaved, .. }) => Ok(Some(interleaved)),
            Some(DecodePacketRef::Skipped { .. }) => Ok(None),
            None => Ok(None),
        }
    }

    /// Decode next packet into an internal reusable interleaved buffer when
    /// `decode_packet` is true. When false, skips decode and only returns
    /// frame timing when packet duration is known.
    pub fn decode_next_ref_maybe(
        &mut self,
        decode_packet: bool,
    ) -> Result<Option<DecodePacketRef<'_>>, DecodeError> {
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

            if !decode_packet {
                let packet_frames = packet.dur as usize;
                if packet_frames > 0 {
                    return Ok(Some(DecodePacketRef::Skipped {
                        frame_count: packet_frames,
                    }));
                }
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

            let needs_resize = self
                .sample_buffer
                .as_ref()
                .map(|buffer| buffer.capacity() < required_samples)
                .unwrap_or(true);

            if needs_resize {
                self.sample_buffer = Some(SampleBuffer::<f32>::new(required_frames as u64, spec));
            }

            let buffer = self
                .sample_buffer
                .as_mut()
                .expect("sample buffer should be set");

            buffer.copy_interleaved_ref(decoded);
            let samples = buffer.samples();
            return Ok(Some(DecodePacketRef::Decoded {
                frame_count: samples.len() / self.channels,
                interleaved: samples,
            }));
        }
    }

    pub fn seek(&mut self, position_ms: u64) -> Result<(), DecodeError> {
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

        self.decoder.reset();
        Ok(())
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn bit_depth(&self) -> Option<u32> {
        self.bit_depth
    }
}

fn map_open_error(err: SymphoniaError) -> DecodeError {
    match err {
        SymphoniaError::Unsupported(_) => DecodeError::UnsupportedFormat,
        SymphoniaError::IoError(err) => DecodeError::Io(err),
        other => DecodeError::DecoderError(other.to_string()),
    }
}
