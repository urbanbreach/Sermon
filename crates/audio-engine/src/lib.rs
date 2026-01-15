//! Audio playback engine for Sermon
//!
//! This crate provides the core audio playback functionality using
//! Symphonia for decoding and WASAPI for output.

pub mod decode;
pub mod device;
pub mod engine;
pub mod events;
pub mod output;
pub mod queue;
pub mod types;

pub use crate::engine::EngineState;
pub use crate::queue::{PlaybackQueue, PreviousAction, QueueItem};
pub use crate::types::{AudioSample, PlaySession, PlaybackState, TrackInfo};
