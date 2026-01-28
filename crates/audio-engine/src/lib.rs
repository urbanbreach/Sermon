//! Audio playback engine for Sermon
//!
//! This crate provides the core audio playback functionality using
//! Symphonia for decoding and WASAPI for output.

#[cfg(windows)]
pub mod asio;
#[cfg(windows)]
pub mod asio_device;
#[cfg(windows)]
pub mod asio_worker;
pub mod decode;
pub mod device;
pub mod dop;
pub mod dsd_decode;
pub mod encoder_delay;
pub mod engine;
pub mod events;
pub mod gapless;
pub mod gapless_decoder;
pub mod memory_source;
pub mod output;
pub mod queue;
pub mod resample;
pub mod types;

pub use crate::engine::EngineState;
pub use crate::queue::{PlaybackQueue, PreviousAction, QueueItem};
pub use crate::types::{AudioSample, PlaySession, PlaybackState, TrackInfo};

pub use crate::dop::{
    dop_sample_rate, marker_for_frame, pack_dop_sample, DopPacker, DOP_MARKER_A, DOP_MARKER_B,
};
pub use crate::dsd_decode::{
    bit_reverse, DffDecoder, DsdDecoder, DsdError, DsdFormat, DsdInfo, DsfDecoder,
    BIT_REVERSE_TABLE,
};
pub use crate::resample::{Resampler, ResamplerQuality};
