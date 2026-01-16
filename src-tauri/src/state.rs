use audio_engine::EngineState;
use crossbeam_channel::Sender;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};

pub struct LibraryState {
    pub db_path: PathBuf,
    pub scan_lock: StdMutex<Option<u64>>, // Some(scan_id) if scan is running
}

impl LibraryState {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            scan_lock: StdMutex::new(None),
        }
    }
}

pub struct AudioState {
    pub engine: Arc<Mutex<EngineState>>,
    pub command_tx: Sender<PlaybackCommand>,
}

pub enum PlaybackCommand {
    PlayNow {
        track_id: i64,
    },
    AddToQueue {
        track_id: i64,
    },
    Pause,
    Resume,
    Stop,
    Seek {
        position_ms: u64,
    },
    Next,
    Previous,
    SetVolume {
        volume: f32,
    },
    SetDevice {
        device_id: String,
    },
    SetOutputSettings {
        mode: String,
        policy: String,
        fade: bool,
    },
}
