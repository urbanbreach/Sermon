pub mod db;
pub mod error;
pub mod identity;
pub mod models;
pub mod safe_write;
pub mod scanner;
pub mod tag_edit;

pub use db::list_tracks;
pub use db::migrations::apply_migrations;
pub use db::open_db;
pub use db::set_missing;
pub use db::upsert_track;
pub use db::{
    get_audio_device_preference, get_audio_output_fade, get_audio_output_mode,
    get_audio_output_policy, get_audio_output_timing, get_audio_volume, get_setting,
    get_track_by_id, set_setting,
};
pub use models::{LibraryFolder, QuickScanSummary, TrackRow};
pub use scanner::{quick_scan, scan_folder};
