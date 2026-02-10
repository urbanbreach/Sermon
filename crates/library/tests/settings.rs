use library::db::{
    get_audio_device_preference, get_audio_volume, get_setting, get_track_by_id, open_db,
    set_setting, upsert_track,
};
use library::{apply_migrations, TrackRow};
use tempfile::tempdir;

#[test]
fn test_settings_crud() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_settings.db");
    let conn = open_db(&db_path).unwrap();
    apply_migrations(&conn).unwrap();

    // Test defaults
    assert_eq!(get_audio_volume(&conn), 1.0);
    assert_eq!(get_audio_device_preference(&conn), "default");

    // Test specific setting
    assert!(get_setting(&conn, "some.key").unwrap().is_none());
    set_setting(&conn, "some.key", "some_value").unwrap();
    assert_eq!(
        get_setting(&conn, "some.key").unwrap().unwrap(),
        "some_value"
    );

    // Test overwriting
    set_setting(&conn, "some.key", "new_value").unwrap();
    assert_eq!(
        get_setting(&conn, "some.key").unwrap().unwrap(),
        "new_value"
    );

    // Test audio volume helper
    set_setting(&conn, "audio.volume", "0.5").unwrap();
    assert_eq!(get_audio_volume(&conn), 0.5);

    // Test bad volume
    set_setting(&conn, "audio.volume", "not_a_number").unwrap();
    assert_eq!(get_audio_volume(&conn), 1.0); // Should return default on parse error

    // Test audio device helper
    set_setting(&conn, "audio.device.preference", "device_guid_123").unwrap();
    assert_eq!(get_audio_device_preference(&conn), "device_guid_123");

    // Test new output settings
    use library::db::{
        get_audio_output_fade, get_audio_output_mode, get_audio_output_policy,
        get_audio_output_timing,
    };

    // Defaults
    assert_eq!(get_audio_output_mode(&conn), "exclusive");
    assert_eq!(get_audio_output_policy(&conn), "strict");
    assert_eq!(get_audio_output_fade(&conn), false);

    // Set values
    set_setting(&conn, "audio.output.mode", "shared").unwrap();
    set_setting(&conn, "audio.output.policy", "compatibility").unwrap();
    set_setting(&conn, "audio.output.fade", "on").unwrap();

    assert_eq!(get_audio_output_mode(&conn), "shared");
    assert_eq!(get_audio_output_policy(&conn), "compatibility");
    assert_eq!(get_audio_output_fade(&conn), true);

    set_setting(&conn, "audio.output.fade", "off").unwrap();
    assert_eq!(get_audio_output_fade(&conn), false);

    set_setting(&conn, "audio.output.mode", "  ExClUsIvE ").unwrap();
    set_setting(&conn, "audio.output.policy", " STRICT ").unwrap();
    set_setting(&conn, "audio.output.timing", " EVENT ").unwrap();

    assert_eq!(get_audio_output_mode(&conn), "exclusive");
    assert_eq!(get_audio_output_policy(&conn), "strict");
    assert_eq!(get_audio_output_timing(&conn), "event");

    set_setting(&conn, "audio.output.mode", "bad-mode").unwrap();
    set_setting(&conn, "audio.output.policy", "bad-policy").unwrap();
    set_setting(&conn, "audio.output.timing", "bad-timing").unwrap();

    assert_eq!(get_audio_output_mode(&conn), "exclusive");
    assert_eq!(get_audio_output_policy(&conn), "strict");
    assert_eq!(get_audio_output_timing(&conn), "polling");
}

#[test]
fn test_get_track_by_id() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_tracks.db");
    let conn = open_db(&db_path).unwrap();
    apply_migrations(&conn).unwrap();

    // Insert a dummy folder first
    conn.execute(
        "INSERT INTO library_folders (path) VALUES (?)",
        ["/tmp/music"],
    )
    .unwrap();
    let folder_id = conn.last_insert_rowid();

    let track = TrackRow {
        library_folder_id: folder_id,
        path: "/tmp/music/track.mp3".to_string(),
        title: Some("Test Track".to_string()),
        mtime_ms: 1000,
        size_bytes: 1024,
        identity_source: "fallback".to_string(),
        ..Default::default()
    };

    let id = upsert_track(&conn, &track).unwrap();

    let fetched = get_track_by_id(&conn, id).unwrap();
    assert_eq!(fetched.id, Some(id));
    assert_eq!(fetched.title.as_deref(), Some("Test Track"));
    assert_eq!(fetched.path, "/tmp/music/track.mp3");
}
