use audio_engine::EngineState;
use crossbeam_channel::Sender;
use parking_lot::Mutex;
use parking_lot::Mutex as ParkingMutex;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};

pub struct LibraryState {
    pub db_path: PathBuf,
    pub scan_lock: StdMutex<Option<u64>>, // Some(scan_id) if scan is running
    pub migrations_applied: AtomicBool,
}

impl LibraryState {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            scan_lock: StdMutex::new(None),
            migrations_applied: AtomicBool::new(false),
        }
    }

    /// Ensures migrations are applied. Only runs migrations once per app lifecycle.
    /// Returns Ok(()) if migrations already applied or successfully applied now.
    pub fn ensure_migrations(&self, conn: &rusqlite::Connection) -> Result<(), String> {
        if self.migrations_applied.load(Ordering::SeqCst) {
            return Ok(());
        }
        library::apply_migrations(conn).map_err(|e| e.to_string())?;
        self.migrations_applied.store(true, Ordering::SeqCst);
        Ok(())
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

/// Diagnostics state for performance metrics tracking.
/// All timing values are in milliseconds since UNIX epoch.
/// Thread-safe via atomic operations for lock-free updates from multiple threads.
pub struct DiagnosticsState {
    /// When app startup began (ms since epoch)
    pub startup_start_ms: AtomicU64,
    /// When app startup completed (first-interactive, ms since epoch)
    pub startup_complete_ms: AtomicU64,
    /// When current playback started (ms since epoch)
    pub playback_start_ms: AtomicU64,
    /// When current playback became ready (ms since epoch)
    pub playback_complete_ms: AtomicU64,
    /// When current seek operation started (ms since epoch)
    pub seek_start_ms: AtomicU64,
    /// When current seek operation completed (ms since epoch)
    pub seek_complete_ms: AtomicU64,
    /// Total count of audio underrun events
    pub underrun_count: AtomicU64,
}

impl Default for DiagnosticsState {
    fn default() -> Self {
        Self {
            startup_start_ms: AtomicU64::new(0),
            startup_complete_ms: AtomicU64::new(0),
            playback_start_ms: AtomicU64::new(0),
            playback_complete_ms: AtomicU64::new(0),
            seek_start_ms: AtomicU64::new(0),
            seek_complete_ms: AtomicU64::new(0),
            underrun_count: AtomicU64::new(0),
        }
    }
}

impl DiagnosticsState {
    /// Create a new DiagnosticsState with all counters zeroed
    pub fn new() -> Self {
        Self::default()
    }

    /// Get startup duration in ms, or None if startup not complete
    pub fn startup_duration_ms(&self) -> Option<u64> {
        let start = self.startup_start_ms.load(Ordering::SeqCst);
        let complete = self.startup_complete_ms.load(Ordering::SeqCst);
        if start > 0 && complete >= start {
            Some(complete - start)
        } else {
            None
        }
    }

    /// Get playback start duration in ms (time from request to audio playing)
    pub fn playback_start_duration_ms(&self) -> Option<u64> {
        let start = self.playback_start_ms.load(Ordering::SeqCst);
        let complete = self.playback_complete_ms.load(Ordering::SeqCst);
        if start > 0 && complete >= start {
            Some(complete - start)
        } else {
            None
        }
    }

    /// Get seek duration in ms
    pub fn seek_duration_ms(&self) -> Option<u64> {
        let start = self.seek_start_ms.load(Ordering::SeqCst);
        let complete = self.seek_complete_ms.load(Ordering::SeqCst);
        if start > 0 && complete >= start {
            Some(complete - start)
        } else {
            None
        }
    }

    /// Increment underrun counter
    pub fn increment_underruns(&self) {
        self.underrun_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current underrun count
    pub fn underruns(&self) -> u64 {
        self.underrun_count.load(Ordering::Relaxed)
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
    OpenAsioControlPanel {
        driver_name: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_state_default() {
        let state = DiagnosticsState::default();
        assert_eq!(state.startup_start_ms.load(Ordering::SeqCst), 0);
        assert_eq!(state.startup_complete_ms.load(Ordering::SeqCst), 0);
        assert_eq!(state.playback_start_ms.load(Ordering::SeqCst), 0);
        assert_eq!(state.playback_complete_ms.load(Ordering::SeqCst), 0);
        assert_eq!(state.seek_start_ms.load(Ordering::SeqCst), 0);
        assert_eq!(state.seek_complete_ms.load(Ordering::SeqCst), 0);
        assert_eq!(state.underrun_count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_diagnostics_state_new() {
        let state = DiagnosticsState::new();
        assert_eq!(state.startup_start_ms.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_startup_duration_not_complete() {
        let state = DiagnosticsState::new();
        state.startup_start_ms.store(1000, Ordering::SeqCst);
        assert_eq!(state.startup_duration_ms(), None);
    }

    #[test]
    fn test_startup_duration_complete() {
        let state = DiagnosticsState::new();
        state.startup_start_ms.store(1000, Ordering::SeqCst);
        state.startup_complete_ms.store(1500, Ordering::SeqCst);
        assert_eq!(state.startup_duration_ms(), Some(500));
    }

    #[test]
    fn test_playback_start_duration() {
        let state = DiagnosticsState::new();
        state.playback_start_ms.store(5000, Ordering::SeqCst);
        state.playback_complete_ms.store(5200, Ordering::SeqCst);
        assert_eq!(state.playback_start_duration_ms(), Some(200));
    }

    #[test]
    fn test_seek_duration() {
        let state = DiagnosticsState::new();
        state.seek_start_ms.store(10000, Ordering::SeqCst);
        state.seek_complete_ms.store(10050, Ordering::SeqCst);
        assert_eq!(state.seek_duration_ms(), Some(50));
    }

    #[test]
    fn test_increment_underruns() {
        let state = DiagnosticsState::new();
        assert_eq!(state.underruns(), 0);
        state.increment_underruns();
        assert_eq!(state.underruns(), 1);
        state.increment_underruns();
        state.increment_underruns();
        assert_eq!(state.underruns(), 3);
    }

    #[test]
    fn test_underruns_thread_safe() {
        use std::thread;

        let state = Arc::new(DiagnosticsState::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let s = Arc::clone(&state);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    s.increment_underruns();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(state.underruns(), 1000);
    }
}
