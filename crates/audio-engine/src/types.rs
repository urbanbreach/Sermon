use std::time::Instant;

use ulid::Ulid;

pub type AudioSample = f32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub id: i64,
    pub path: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_ms: Option<u64>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u16>,
    pub channels: Option<u16>,
    pub codec: Option<String>,
    pub container: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PlaySession {
    pub play_id: Ulid,
    pub track_id: i64,
    pub track: TrackInfo,
    pub played_ms: u64,
    pub position_ms: u64,
    pub started_at: Instant,
    pub last_play_start: Option<Instant>,
}

impl PlaySession {
    pub fn new(track: TrackInfo, started_at: Instant) -> Self {
        Self {
            play_id: Ulid::new(),
            track_id: track.id,
            track,
            played_ms: 0,
            position_ms: 0,
            started_at,
            last_play_start: Some(started_at),
        }
    }

    pub fn is_playing(&self) -> bool {
        self.last_play_start.is_some()
    }
}
