use library::{
    apply_migrations, backfill_loudness_metadata_once, list_tracks, open_db, scan_folder,
};
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

fn create_test_wav(path: &std::path::Path) {
    let sample_rate: u32 = 44100;
    let channels: u16 = 2;
    let bits_per_sample: u16 = 16;
    let data_size: u32 = 4410 * 4;
    let file_size: u32 = 36 + data_size;

    let mut file = File::create(path).unwrap();
    file.write_all(b"RIFF").unwrap();
    file.write_all(&file_size.to_le_bytes()).unwrap();
    file.write_all(b"WAVE").unwrap();
    file.write_all(b"fmt ").unwrap();
    file.write_all(&16u32.to_le_bytes()).unwrap();
    file.write_all(&1u16.to_le_bytes()).unwrap();
    file.write_all(&channels.to_le_bytes()).unwrap();
    file.write_all(&sample_rate.to_le_bytes()).unwrap();
    file.write_all(&(sample_rate * channels as u32 * bits_per_sample as u32 / 8).to_le_bytes())
        .unwrap();
    file.write_all(&(channels * bits_per_sample / 8).to_le_bytes())
        .unwrap();
    file.write_all(&bits_per_sample.to_le_bytes()).unwrap();
    file.write_all(b"data").unwrap();
    file.write_all(&data_size.to_le_bytes()).unwrap();
    file.write_all(&vec![0u8; data_size as usize]).unwrap();
}

#[test]
fn test_scan_folder_basic() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");
    let music_dir = temp.path().join("music");
    std::fs::create_dir(&music_dir).unwrap();

    // Create test files
    create_test_wav(&music_dir.join("track1.wav"));
    create_test_wav(&music_dir.join("track2.wav"));

    // Initialize DB
    let conn = open_db(&db_path).unwrap();
    apply_migrations(&conn).unwrap();
    drop(conn);

    // Scan
    let progress = Arc::new(Mutex::new(Vec::new()));
    let progress_clone = progress.clone();
    let summary = scan_folder(&db_path, music_dir.to_str().unwrap(), move |p| {
        progress_clone.lock().unwrap().push(p.scanned);
    })
    .unwrap();

    assert_eq!(summary.scanned, 2);
    assert_eq!(summary.total, 2);
    assert_eq!(summary.errors, 0);

    // Verify tracks in DB
    let conn = open_db(&db_path).unwrap();
    let tracks = list_tracks(&conn, "id", "asc").unwrap();
    assert_eq!(tracks.len(), 2);
}

#[test]
fn test_scan_skip_unchanged() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");
    let music_dir = temp.path().join("music");
    std::fs::create_dir(&music_dir).unwrap();

    create_test_wav(&music_dir.join("track.wav"));

    let conn = open_db(&db_path).unwrap();
    apply_migrations(&conn).unwrap();
    drop(conn);

    // First scan
    let summary1 = scan_folder(&db_path, music_dir.to_str().unwrap(), |_| {}).unwrap();
    assert_eq!(summary1.scanned, 1);
    assert_eq!(summary1.skipped, 0);

    // Second scan - should skip
    let summary2 = scan_folder(&db_path, music_dir.to_str().unwrap(), |_| {}).unwrap();
    assert_eq!(summary2.scanned, 0);
    assert_eq!(summary2.skipped, 1);
}

#[test]
fn test_scan_no_duplicates() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");
    let music_dir = temp.path().join("music");
    std::fs::create_dir(&music_dir).unwrap();

    create_test_wav(&music_dir.join("track.wav"));

    let conn = open_db(&db_path).unwrap();
    apply_migrations(&conn).unwrap();
    drop(conn);

    // Scan twice
    scan_folder(&db_path, music_dir.to_str().unwrap(), |_| {}).unwrap();
    scan_folder(&db_path, music_dir.to_str().unwrap(), |_| {}).unwrap();

    // Should still be 1 track
    let conn = open_db(&db_path).unwrap();
    let tracks = list_tracks(&conn, "id", "asc").unwrap();
    assert_eq!(tracks.len(), 1);
}

#[test]
fn test_scan_marks_missing() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");
    let music_dir = temp.path().join("music");
    std::fs::create_dir(&music_dir).unwrap();

    let track_path = music_dir.join("track.wav");
    create_test_wav(&track_path);

    let conn = open_db(&db_path).unwrap();
    apply_migrations(&conn).unwrap();
    drop(conn);

    // First scan
    scan_folder(&db_path, music_dir.to_str().unwrap(), |_| {}).unwrap();

    // Delete the file
    std::fs::remove_file(&track_path).unwrap();

    // Second scan
    scan_folder(&db_path, music_dir.to_str().unwrap(), |_| {}).unwrap();

    // Track should be marked missing
    let conn = open_db(&db_path).unwrap();
    let tracks = list_tracks(&conn, "id", "asc").unwrap();
    assert_eq!(tracks.len(), 1);
    assert!(tracks[0].is_missing);
}

#[test]
fn test_scan_idempotent_results() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");
    let music_dir = temp.path().join("music");
    std::fs::create_dir(&music_dir).unwrap();

    create_test_wav(&music_dir.join("track1.wav"));
    create_test_wav(&music_dir.join("track2.wav"));

    let conn = open_db(&db_path).unwrap();
    apply_migrations(&conn).unwrap();
    drop(conn);

    let first = scan_folder(&db_path, music_dir.to_str().unwrap(), |_| {}).unwrap();
    let second = scan_folder(&db_path, music_dir.to_str().unwrap(), |_| {}).unwrap();

    assert_eq!(first.total, 2);
    assert_eq!(first.scanned, 2);
    assert_eq!(first.skipped, 0);
    assert_eq!(first.errors, 0);

    assert_eq!(second.total, 2);
    assert_eq!(second.scanned, 0);
    assert_eq!(second.skipped, 2);
    assert_eq!(second.errors, 0);

    let conn = open_db(&db_path).unwrap();
    let tracks = list_tracks(&conn, "id", "asc").unwrap();
    assert_eq!(tracks.len(), 2);

    let mut paths: Vec<_> = tracks.into_iter().map(|t| t.path).collect();
    paths.sort();
    assert_eq!(
        paths[0],
        music_dir.join("track1.wav").to_string_lossy().to_string()
    );
    assert_eq!(
        paths[1],
        music_dir.join("track2.wav").to_string_lossy().to_string()
    );
}

#[test]
fn test_scan_empty_folder() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");
    let music_dir = temp.path().join("music");
    std::fs::create_dir(&music_dir).unwrap();

    let conn = open_db(&db_path).unwrap();
    apply_migrations(&conn).unwrap();
    drop(conn);

    let summary = scan_folder(&db_path, music_dir.to_str().unwrap(), |_| {}).unwrap();

    assert_eq!(summary.total, 0);
    assert_eq!(summary.scanned, 0);
    assert_eq!(summary.skipped, 0);
    assert_eq!(summary.errors, 0);

    let conn = open_db(&db_path).unwrap();
    let tracks = list_tracks(&conn, "id", "asc").unwrap();
    assert!(tracks.is_empty());
}

#[test]
fn test_loudness_backfill_sets_completion_marker() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("test.db");

    let conn = open_db(&db_path).unwrap();
    apply_migrations(&conn).unwrap();
    drop(conn);

    let first = backfill_loudness_metadata_once(&db_path).unwrap();
    assert!(!first.skipped);
    assert_eq!(first.candidates, 0);
    assert_eq!(first.checked, 0);
    assert_eq!(first.updated, 0);

    let conn = open_db(&db_path).unwrap();
    let marker = library::db::get_setting(&conn, "library.loudness_backfill_v1_done").unwrap();
    assert_eq!(marker.as_deref(), Some("on"));
    drop(conn);

    let second = backfill_loudness_metadata_once(&db_path).unwrap();
    assert!(second.skipped);
}
