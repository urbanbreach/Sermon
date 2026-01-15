use std::path::PathBuf;
use std::sync::Mutex;

pub struct LibraryState {
    pub db_path: PathBuf,
    pub scan_lock: Mutex<Option<u64>>, // Some(scan_id) if scan is running
}

impl LibraryState {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            scan_lock: Mutex::new(None),
        }
    }
}
