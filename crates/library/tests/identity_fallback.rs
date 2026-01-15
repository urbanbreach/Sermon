use library::identity::{compute_partial_hash, get_file_identity};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_fallback_identity() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");

    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"test content for hashing").unwrap();
    drop(file);

    let identity = get_file_identity(&file_path).unwrap();

    assert!(identity.mtime_ms > 0);
    assert_eq!(identity.size_bytes, 24); // "test content for hashing".len()
}

#[test]
fn test_partial_hash() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("hash_test.txt");

    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"content to hash").unwrap();
    drop(file);

    let hash = compute_partial_hash(&file_path).unwrap();

    // blake3 produces 64 hex characters
    assert_eq!(hash.len(), 64);

    // Same content = same hash
    let hash2 = compute_partial_hash(&file_path).unwrap();
    assert_eq!(hash, hash2);
}

#[test]
fn test_hash_changes_with_content() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("changing.txt");

    // First content
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"first content").unwrap();
    drop(file);
    let hash1 = compute_partial_hash(&file_path).unwrap();

    // Different content
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"different content").unwrap();
    drop(file);
    let hash2 = compute_partial_hash(&file_path).unwrap();

    assert_ne!(hash1, hash2);
}
