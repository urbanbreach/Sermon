use audio_engine::EngineState;
use crossbeam_channel::Sender;
use parking_lot::Mutex;
use parking_lot::Mutex as ParkingMutex;
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

pub struct ArtworkCacheState {
    pub cache_dir: PathBuf,
    pub lock: ParkingMutex<()>,
}

impl ArtworkCacheState {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            lock: ParkingMutex::new(()),
        }
    }
}

pub struct WaveformCacheState {
    pub cache_dir: PathBuf,
    pub lock: ParkingMutex<()>,
}

impl WaveformCacheState {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            lock: ParkingMutex::new(()),
        }
    }
}

pub enum PlaybackCommand {
    PlayNow {
        track_id: i64,
    },
    PlayNowWithQueue {
        track_ids: Vec<i64>,
        start_index: usize,
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
        timing: String,
        asio_driver: Option<String>,
    },
}
