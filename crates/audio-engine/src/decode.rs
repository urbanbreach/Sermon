use std::fs::File;
use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, Decoder, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
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
    sample_buffer: Option<SampleBuffer<f32>>,
}

impl AudioDecoder {
    pub fn open(path: &Path) -> Result<Self, DecodeError> {
        let file = File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

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

        Ok(Self {
            format_reader,
            decoder,
            track_id: track.id,
            sample_rate,
            channels,
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

            let buffer = match self.sample_buffer.as_mut() {
                Some(buffer) if buffer.capacity() >= required_samples => buffer,
                _ => {
                    self.sample_buffer =
                        Some(SampleBuffer::<f32>::new(required_frames as u64, spec));
                    self.sample_buffer.as_mut().expect("just set")
                }
            };

            buffer.copy_interleaved_ref(decoded);
            return Ok(Some(buffer.samples().to_vec()));
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
}

fn map_open_error(err: SymphoniaError) -> DecodeError {
    match err {
        SymphoniaError::Unsupported(_) => DecodeError::UnsupportedFormat,
        SymphoniaError::IoError(err) => DecodeError::Io(err),
        other => DecodeError::DecoderError(other.to_string()),
    }
}
