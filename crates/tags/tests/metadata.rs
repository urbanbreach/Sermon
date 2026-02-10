use std::fs::File;
use std::io::Write;
use std::path::Path;
use tags::{read_metadata, read_metadata_result};
use tags::{write_tags, NumberPatch, TagPatch, TagPatches, TagWriteOptions};
use tempfile::tempdir;

fn create_test_wav(path: &Path) {
    let sample_rate: u32 = 44100;
    let channels: u16 = 2;
    let bits_per_sample: u16 = 16;
    let data_size: u32 = 4410 * 4; // 0.1 seconds stereo 16-bit
    let file_size: u32 = 36 + data_size;

    let mut file = File::create(path).unwrap();

    // RIFF header
    file.write_all(b"RIFF").unwrap();
    file.write_all(&file_size.to_le_bytes()).unwrap();
    file.write_all(b"WAVE").unwrap();

    // fmt chunk
    file.write_all(b"fmt ").unwrap();
    file.write_all(&16u32.to_le_bytes()).unwrap(); // chunk size
    file.write_all(&1u16.to_le_bytes()).unwrap(); // PCM
    file.write_all(&channels.to_le_bytes()).unwrap();
    file.write_all(&sample_rate.to_le_bytes()).unwrap();
    file.write_all(&(sample_rate * channels as u32 * bits_per_sample as u32 / 8).to_le_bytes())
        .unwrap();
    file.write_all(&(channels * bits_per_sample / 8).to_le_bytes())
        .unwrap();
    file.write_all(&bits_per_sample.to_le_bytes()).unwrap();

    // data chunk
    file.write_all(b"data").unwrap();
    file.write_all(&data_size.to_le_bytes()).unwrap();
    file.write_all(&vec![0u8; data_size as usize]).unwrap();
}

#[test]
fn test_read_wav_metadata() {
    let dir = tempdir().unwrap();
    let wav_path = dir.path().join("test.wav");
    create_test_wav(&wav_path);

    let metadata = read_metadata(&wav_path);

    // Technical info should be extracted
    assert_eq!(metadata.sample_rate, Some(44100));
    assert_eq!(metadata.channels, Some(2));
    assert_eq!(metadata.bit_depth, Some(16));
    assert!(metadata.duration_ms.is_some());
}

#[test]
fn test_read_wav_metadata_without_tags() {
    let dir = tempdir().unwrap();
    let wav_path = dir.path().join("no_tags.wav");
    create_test_wav(&wav_path);

    let metadata = read_metadata(&wav_path);

    // Untagged file should still expose technical metadata.
    assert_eq!(metadata.sample_rate, Some(44100));
    assert_eq!(metadata.channels, Some(2));
    assert_eq!(metadata.bit_depth, Some(16));
    assert!(metadata.duration_ms.is_some());

    // But editable text/number tag fields should be empty.
    assert!(metadata.title.is_none());
    assert!(metadata.artist.is_none());
    assert!(metadata.album.is_none());
    assert!(metadata.album_artist.is_none());
    assert!(metadata.genre.is_none());
    assert!(metadata.lyricist.is_none());
    assert!(metadata.track_no.is_none());
    assert!(metadata.disc_no.is_none());
    assert!(metadata.year.is_none());
}

#[test]
fn test_read_wav_metadata_with_minimal_tags() {
    let dir = tempdir().unwrap();
    let wav_path = dir.path().join("minimal_tags.wav");
    create_test_wav(&wav_path);

    let patches = TagPatches {
        title: TagPatch::Set("Only Title".to_string()),
        ..Default::default()
    };
    write_tags(&wav_path, &patches, &TagWriteOptions::new()).unwrap();

    let metadata = read_metadata(&wav_path);

    assert_eq!(metadata.title, Some("Only Title".to_string()));
    assert!(metadata.artist.is_none());
    assert!(metadata.album.is_none());
    assert!(metadata.album_artist.is_none());
    assert!(metadata.genre.is_none());
    assert!(metadata.track_no.is_none());
    assert!(metadata.disc_no.is_none());
    assert!(metadata.year.is_none());

    // Technical metadata should still be intact after write.
    assert_eq!(metadata.sample_rate, Some(44100));
    assert_eq!(metadata.channels, Some(2));
    assert_eq!(metadata.bit_depth, Some(16));
}

#[test]
fn test_read_nonexistent_file() {
    let metadata = read_metadata(Path::new("/nonexistent/file.mp3"));

    // Should return default (empty) metadata, not panic
    assert!(metadata.title.is_none());
    assert!(metadata.sample_rate.is_none());
}

#[test]
fn test_read_invalid_file() {
    let dir = tempdir().unwrap();
    let invalid_path = dir.path().join("not_audio.txt");
    std::fs::write(&invalid_path, "not audio content").unwrap();

    let metadata = read_metadata(&invalid_path);

    // Should return default metadata, not panic
    assert!(metadata.title.is_none());
}

// ============================================================================
// Tag Write Tests (Milestone 05)
// ============================================================================

#[test]
fn test_write_tags_set_title() {
    let dir = tempdir().unwrap();
    let wav_path = dir.path().join("test_write.wav");
    create_test_wav(&wav_path);

    // Write a title
    let patches = TagPatches {
        title: TagPatch::Set("Test Title".to_string()),
        ..Default::default()
    };
    write_tags(&wav_path, &patches, &TagWriteOptions::new()).unwrap();

    // Read back and verify
    let metadata = read_metadata(&wav_path);
    assert_eq!(metadata.title, Some("Test Title".to_string()));

    // Technical info should be preserved
    assert_eq!(metadata.sample_rate, Some(44100));
    assert_eq!(metadata.channels, Some(2));
    assert_eq!(metadata.bit_depth, Some(16));
}

#[test]
fn test_write_tags_read_write_read_invariant() {
    let dir = tempdir().unwrap();
    let wav_path = dir.path().join("test_invariant.wav");
    create_test_wav(&wav_path);

    // Initial write to set up tags
    let initial_patches = TagPatches {
        title: TagPatch::Set("Original Title".to_string()),
        artist: TagPatch::Set("Original Artist".to_string()),
        album: TagPatch::Set("Original Album".to_string()),
        track_no: NumberPatch::Set(5),
        year: NumberPatch::Set(2024),
        ..Default::default()
    };
    write_tags(&wav_path, &initial_patches, &TagWriteOptions::new()).unwrap();

    // Read to verify initial state
    let before = read_metadata(&wav_path);
    assert_eq!(before.title, Some("Original Title".to_string()));
    assert_eq!(before.artist, Some("Original Artist".to_string()));
    assert_eq!(before.album, Some("Original Album".to_string()));
    assert_eq!(before.track_no, Some(5));
    assert_eq!(before.year, Some(2024));

    // Change only title (read -> write one field -> read)
    let update_patches = TagPatches {
        title: TagPatch::Set("Updated Title".to_string()),
        // Leave all others unchanged
        ..Default::default()
    };
    write_tags(&wav_path, &update_patches, &TagWriteOptions::new()).unwrap();

    // Read and verify invariant: only title changed, others preserved
    let after = read_metadata(&wav_path);
    assert_eq!(after.title, Some("Updated Title".to_string())); // Changed
    assert_eq!(after.artist, Some("Original Artist".to_string())); // Preserved
    assert_eq!(after.album, Some("Original Album".to_string())); // Preserved
    assert_eq!(after.track_no, Some(5)); // Preserved
    assert_eq!(after.year, Some(2024)); // Preserved
}

#[test]
fn test_write_tags_leave_does_not_change() {
    let dir = tempdir().unwrap();
    let wav_path = dir.path().join("test_leave.wav");
    create_test_wav(&wav_path);

    // Set initial title
    let init_patches = TagPatches {
        title: TagPatch::Set("Initial Title".to_string()),
        ..Default::default()
    };
    write_tags(&wav_path, &init_patches, &TagWriteOptions::new()).unwrap();

    // Apply Leave patch - should not change anything
    let leave_patches = TagPatches {
        title: TagPatch::Leave,
        artist: TagPatch::Leave,
        ..Default::default()
    };
    write_tags(&wav_path, &leave_patches, &TagWriteOptions::new()).unwrap();

    // Verify title is unchanged
    let metadata = read_metadata(&wav_path);
    assert_eq!(metadata.title, Some("Initial Title".to_string()));
}

#[test]
fn test_write_tags_clear_removes_field() {
    let dir = tempdir().unwrap();
    let wav_path = dir.path().join("test_clear.wav");
    create_test_wav(&wav_path);

    // Set initial tags
    let init_patches = TagPatches {
        title: TagPatch::Set("Title To Clear".to_string()),
        artist: TagPatch::Set("Artist To Keep".to_string()),
        track_no: NumberPatch::Set(10),
        ..Default::default()
    };
    write_tags(&wav_path, &init_patches, &TagWriteOptions::new()).unwrap();

    // Clear title and track_no
    let clear_patches = TagPatches {
        title: TagPatch::Clear,
        track_no: NumberPatch::Clear,
        ..Default::default()
    };
    write_tags(&wav_path, &clear_patches, &TagWriteOptions::new()).unwrap();

    // Verify cleared fields are gone, others preserved
    let metadata = read_metadata(&wav_path);
    assert_eq!(metadata.title, None); // Cleared
    assert_eq!(metadata.artist, Some("Artist To Keep".to_string())); // Preserved
    assert_eq!(metadata.track_no, None); // Cleared
}

#[test]
fn test_write_tags_set_changes_value() {
    let dir = tempdir().unwrap();
    let wav_path = dir.path().join("test_set.wav");
    create_test_wav(&wav_path);

    // Set initial value
    let init_patches = TagPatches {
        year: NumberPatch::Set(2020),
        ..Default::default()
    };
    write_tags(&wav_path, &init_patches, &TagWriteOptions::new()).unwrap();

    // Verify initial
    let before = read_metadata(&wav_path);
    assert_eq!(before.year, Some(2020));

    // Change value
    let update_patches = TagPatches {
        year: NumberPatch::Set(2025),
        ..Default::default()
    };
    write_tags(&wav_path, &update_patches, &TagWriteOptions::new()).unwrap();

    // Verify changed
    let after = read_metadata(&wav_path);
    assert_eq!(after.year, Some(2025));
}

#[test]
fn test_write_tags_all_fields() {
    let dir = tempdir().unwrap();
    let wav_path = dir.path().join("test_all_fields.wav");
    create_test_wav(&wav_path);

    // Set all 8 editable fields
    let patches = TagPatches {
        title: TagPatch::Set("Full Title".to_string()),
        artist: TagPatch::Set("Full Artist".to_string()),
        album: TagPatch::Set("Full Album".to_string()),
        album_artist: TagPatch::Set("Full Album Artist".to_string()),
        genre: TagPatch::Set("Rock".to_string()),
        track_no: NumberPatch::Set(7),
        disc_no: NumberPatch::Set(2),
        year: NumberPatch::Set(2023),
        ..Default::default()
    };
    write_tags(&wav_path, &patches, &TagWriteOptions::new()).unwrap();

    // Verify all fields
    let metadata = read_metadata(&wav_path);
    assert_eq!(metadata.title, Some("Full Title".to_string()));
    assert_eq!(metadata.artist, Some("Full Artist".to_string()));
    assert_eq!(metadata.album, Some("Full Album".to_string()));
    assert_eq!(metadata.album_artist, Some("Full Album Artist".to_string()));
    assert_eq!(metadata.genre, Some("Rock".to_string()));
    assert_eq!(metadata.track_no, Some(7));
    assert_eq!(metadata.disc_no, Some(2));
    assert_eq!(metadata.year, Some(2023));
}

#[test]
fn test_read_metadata_result_success() {
    let dir = tempdir().unwrap();
    let wav_path = dir.path().join("test_result.wav");
    create_test_wav(&wav_path);

    // read_metadata_result should succeed for valid file
    let result = read_metadata_result(&wav_path);
    assert!(result.is_ok());

    let metadata = result.unwrap();
    assert_eq!(metadata.sample_rate, Some(44100));
}

#[test]
fn test_read_metadata_result_error() {
    // read_metadata_result should return error for nonexistent file
    let result = read_metadata_result(Path::new("/nonexistent/file.wav"));
    assert!(result.is_err());
}

#[test]
fn test_write_to_nonexistent_file_fails() {
    let patches = TagPatches {
        title: TagPatch::Set("Test".to_string()),
        ..Default::default()
    };

    // Should fail gracefully, not panic
    let result = write_tags(
        Path::new("/nonexistent/path/file.wav"),
        &patches,
        &TagWriteOptions::new(),
    );
    assert!(result.is_err());
}

// ============================================================================
// DSD Metadata Tests (Milestone 08)
// ============================================================================

const DFF_BASE64: &str = include_str!("fixtures/1kHz.dff.base64");

fn decode_base64_fixture(base64_data: &str) -> Vec<u8> {
    use base64::Engine;
    let cleaned: String = base64_data.chars().filter(|c| !c.is_whitespace()).collect();
    base64::engine::general_purpose::STANDARD
        .decode(&cleaned)
        .expect("Invalid base64 fixture")
}

#[test]
fn test_read_dff_metadata() {
    let dir = tempdir().unwrap();
    let dff_path = dir.path().join("test.dff");

    let dff_bytes = decode_base64_fixture(DFF_BASE64);
    std::fs::write(&dff_path, dff_bytes).unwrap();

    let metadata = read_metadata(&dff_path);

    assert_eq!(metadata.codec, Some("DFF".to_string()));
    assert_eq!(metadata.container, Some("DFF".to_string()));
    assert_eq!(metadata.dsd_rate_hz, Some(2_822_400));
    assert_eq!(metadata.dsd_channels, Some(2));
    assert_eq!(metadata.bit_depth, Some(1));
    assert_eq!(metadata.channels, Some(2));
    assert_eq!(metadata.sample_rate, Some(2_822_400));
}
