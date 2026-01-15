#![cfg(windows)]

use library::identity::{IdentitySource, get_file_identity};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_ntfs_identity_extraction() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");

    // Create a file
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"test content").unwrap();
    drop(file);

    // Get identity
    let identity = get_file_identity(&file_path).unwrap();

    // On NTFS, should use NTFS identity
    // (May be fallback on some CI environments)
    assert!(identity.mtime_ms > 0);
    assert!(identity.size_bytes > 0);

    if identity.source == IdentitySource::Ntfs {
        assert!(identity.volume_serial.is_some());
        assert!(identity.file_id.is_some());
    }
}

#[test]
fn test_rename_preserves_identity() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("original.txt");
    let renamed_path = dir.path().join("renamed.txt");

    // Create a file
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"test content").unwrap();
    drop(file);

    // Get identity before rename
    let identity_before = get_file_identity(&file_path).unwrap();

    // Rename the file
    std::fs::rename(&file_path, &renamed_path).unwrap();

    // Get identity after rename
    let identity_after = get_file_identity(&renamed_path).unwrap();

    // On NTFS, file ID should be preserved
    if identity_before.source == IdentitySource::Ntfs {
        assert_eq!(identity_before.volume_serial, identity_after.volume_serial);
        assert_eq!(identity_before.file_id, identity_after.file_id);
    }
}
