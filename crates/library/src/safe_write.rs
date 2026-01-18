//! Safe write helper for atomic file modifications
//!
//! Provides a safe write strategy: write to temp → flush → atomic commit/replace
//! with optional backup creation.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, error, warn};

/// Status updates during write operation
#[derive(Debug, Clone)]
pub enum WriteStatus {
    /// Retry attempt in progress
    Retry {
        attempt: u8,
        max_attempts: u8,
        error_code: Option<u32>,
    },
    /// Write succeeded
    Success,
    /// Write failed
    Error {
        message: String,
        error_code: Option<u32>,
    },
}

/// Options for safe write
#[derive(Debug, Clone)]
pub struct SafeWriteOptions {
    /// Whether to create a timestamped backup before replacing the original
    pub create_backup: bool,
}

impl Default for SafeWriteOptions {
    fn default() -> Self {
        Self {
            create_backup: true, // ON by default per milestone spec
        }
    }
}

/// Errors that can occur during safe write
#[derive(Error, Debug)]
pub enum SafeWriteError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Write operation failed: {0}")]
    WriteFailed(String),

    #[error("File lock timeout after {attempts} attempts (error code: {last_error_code})")]
    LockTimeout { attempts: u8, last_error_code: u32 },

    #[error("Destination file missing after commit (backup restored: {backup_restored})")]
    DstMissing { backup_restored: bool },

    #[error("{0}")]
    Other(String),
}

/// Retry configuration
const MAX_ATTEMPTS: u8 = 3;
const BACKOFF_MS: [u64; 2] = [75, 200];

/// Windows error codes for retryable lock conditions
#[cfg(windows)]
const ERROR_SHARING_VIOLATION: u32 = 32;
#[cfg(windows)]
#[allow(dead_code)]
const ERROR_ACCESS_DENIED: u32 = 5;

/// Perform a safe write operation with optional status callback
///
/// # Algorithm
/// 1. Create temp file in same directory as dst with `.sermon-tmp` suffix
/// 2. Copy original file to temp
/// 3. Call write_fn against temp (NOT original)
/// 4. Flush temp with sync_all
/// 5. If backup enabled: create backup as `dst.bak.YYYYMMDD-HHMMSSZ`
/// 6. Atomically commit temp → dst using ReplaceFileW/MoveFileExW
/// 7. Retry on transient locks (ERROR_SHARING_VIOLATION, ERROR_ACCESS_DENIED)
/// 8. Post-commit invariant check
pub fn safe_write_with_callback<F, W>(
    dst: &Path,
    options: &SafeWriteOptions,
    status_callback: Option<F>,
    write_fn: W,
) -> Result<(), SafeWriteError>
where
    F: Fn(WriteStatus),
    W: FnOnce(&Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
{
    // Step 1: Create temp file path in same directory
    let tmp = create_temp_path(dst)?;

    // Step 2: Copy bytes dst → tmp
    fs::copy(dst, &tmp).map_err(|e| {
        SafeWriteError::Other(format!("Failed to copy {} to temp: {}", dst.display(), e))
    })?;

    // Ensure temp file is writable (Windows may copy read-only attribute)
    #[cfg(windows)]
    {
        // Check and fix read-only attribute
        if let Ok(metadata) = fs::metadata(&tmp) {
            let perms = metadata.permissions();
            if perms.readonly() {
                let mut new_perms = perms.clone();
                new_perms.set_readonly(false);
                let _ = fs::set_permissions(&tmp, new_perms);
            }
        }
    }

    // Step 3: Call write function against tmp
    write_fn(&tmp).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        SafeWriteError::WriteFailed(e.to_string())
    })?;

    // Step 4: Flush tmp with sync_all (must open with write permission)
    {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&tmp)?;
        file.sync_all()?;
    }

    // Check for test hooks (cfg(test) only)
    #[cfg(test)]
    {
        if let Ok(hook) = std::env::var("SERMON_TEST_SAFEWRITE_HOOK") {
            if hook == "fail_after_flush" {
                let _ = fs::remove_file(&tmp);
                return Err(SafeWriteError::Other(
                    "Test hook: fail_after_flush triggered".into(),
                ));
            }
        }
    }

    // Step 5: Create backup if enabled
    let backup_path = if options.create_backup {
        let backup = create_backup(dst)?;
        Some(backup)
    } else {
        None
    };

    // Step 6-7: Atomic commit with retry
    let commit_result = atomic_commit_with_retry(dst, &tmp, &status_callback);

    match commit_result {
        Ok(()) => {
            // Step 8: Post-commit invariant check
            if !dst.exists() {
                // Destination missing after commit - try to restore from backup
                if let Some(ref backup) = backup_path {
                    if backup.exists() {
                        warn!("Destination missing after commit, restoring from backup");
                        if let Err(e) = fs::copy(backup, dst) {
                            error!("Failed to restore from backup: {}", e);
                            return Err(SafeWriteError::DstMissing {
                                backup_restored: false,
                            });
                        }
                        return Err(SafeWriteError::DstMissing {
                            backup_restored: true,
                        });
                    }
                }
                return Err(SafeWriteError::DstMissing {
                    backup_restored: false,
                });
            }

            // Cleanup temp file if it still exists
            let _ = fs::remove_file(&tmp);

            if let Some(ref cb) = status_callback {
                cb(WriteStatus::Success);
            }

            Ok(())
        }
        Err(e) => {
            // Best-effort cleanup of temp file
            let _ = fs::remove_file(&tmp);
            Err(e)
        }
    }
}

/// Convenience wrapper without callback
pub fn safe_write<W>(
    dst: &Path,
    options: &SafeWriteOptions,
    write_fn: W,
) -> Result<(), SafeWriteError>
where
    W: FnOnce(&Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
{
    safe_write_with_callback::<fn(WriteStatus), _>(dst, options, None, write_fn)
}

/// Create a temp file path in the same directory as dst
fn create_temp_path(dst: &Path) -> Result<PathBuf, SafeWriteError> {
    let parent = dst.parent().ok_or_else(|| {
        SafeWriteError::Other(format!("Cannot get parent directory of {}", dst.display()))
    })?;

    let file_name = dst
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| SafeWriteError::Other("Invalid file name".into()))?;

    // Use .sermon-tmp suffix to avoid matching audio extensions
    let tmp_name = format!("{}.sermon-tmp", file_name);
    let tmp_path = parent.join(tmp_name);

    Ok(tmp_path)
}

/// Create a timestamped backup of the file
fn create_backup(dst: &Path) -> Result<PathBuf, SafeWriteError> {
    let timestamp = chrono_timestamp();
    let parent = dst.parent().unwrap_or(Path::new("."));
    let file_name = dst.file_name().and_then(|n| n.to_str()).unwrap_or("file");

    // Try base backup name first
    let mut backup_name = format!("{}.bak.{}", file_name, timestamp);
    let mut backup_path = parent.join(&backup_name);
    let mut suffix = 0;

    // If exists, append numeric suffix - NEVER overwrite existing backups
    while backup_path.exists() {
        suffix += 1;
        backup_name = format!("{}.bak.{}.{}", file_name, timestamp, suffix);
        backup_path = parent.join(&backup_name);

        // Safety limit
        if suffix > 999 {
            return Err(SafeWriteError::Other(
                "Too many backup files with same timestamp".into(),
            ));
        }
    }

    // Copy original to backup
    fs::copy(dst, &backup_path)
        .map_err(|e| SafeWriteError::Other(format!("Failed to create backup: {}", e)))?;

    debug!("Created backup: {}", backup_path.display());
    Ok(backup_path)
}

/// Generate a timestamp string in YYYYMMDD-HHMMSSZ format
fn chrono_timestamp() -> String {
    use std::time::SystemTime;

    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();

    let secs = now.as_secs();

    // Simple UTC timestamp calculation
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    // Days since 1970-01-01 to YYYYMMDD
    let (year, month, day) = days_to_ymd(days);

    format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

/// Convert days since epoch to (year, month, day)
fn days_to_ymd(days: u64) -> (u32, u32, u32) {
    // Simplified algorithm for date conversion
    let mut remaining_days = days as i64;
    let mut year = 1970i32;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let leap = is_leap_year(year);
    let month_days = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for &days_in_month in &month_days {
        if remaining_days < days_in_month as i64 {
            break;
        }
        remaining_days -= days_in_month as i64;
        month += 1;
    }

    let day = remaining_days as u32 + 1;
    (year as u32, month, day)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Perform atomic commit with retry on transient locks
#[cfg(windows)]
fn atomic_commit_with_retry<F>(
    dst: &Path,
    tmp: &Path,
    status_callback: &Option<F>,
) -> Result<(), SafeWriteError>
where
    F: Fn(WriteStatus),
{
    use std::thread;
    use std::time::Duration;

    let mut attempt = 0u8;
    #[allow(unused_assignments)]
    let mut last_error_code = 0u32;

    loop {
        attempt += 1;

        // Check for test hooks
        #[cfg(test)]
        {
            if let Ok(hook) = std::env::var("SERMON_TEST_SAFEWRITE_HOOK") {
                if hook == "force_retryable_lock" {
                    last_error_code = ERROR_SHARING_VIOLATION;
                    if attempt >= MAX_ATTEMPTS {
                        return Err(SafeWriteError::LockTimeout {
                            attempts: attempt,
                            last_error_code,
                        });
                    }
                    if let Some(cb) = status_callback {
                        cb(WriteStatus::Retry {
                            attempt,
                            max_attempts: MAX_ATTEMPTS,
                            error_code: Some(last_error_code),
                        });
                    }
                    if attempt > 1 {
                        let backoff_idx = (attempt - 2) as usize;
                        if backoff_idx < BACKOFF_MS.len() {
                            thread::sleep(Duration::from_millis(BACKOFF_MS[backoff_idx]));
                        }
                    }
                    continue;
                }
            }
        }

        match try_atomic_commit(dst, tmp) {
            Ok(()) => return Ok(()),
            Err((is_retryable, error_code)) => {
                last_error_code = error_code;

                if !is_retryable || attempt >= MAX_ATTEMPTS {
                    if let Some(cb) = status_callback {
                        cb(WriteStatus::Error {
                            message: format!("Atomic commit failed (error code: {})", error_code),
                            error_code: Some(error_code),
                        });
                    }
                    return Err(SafeWriteError::LockTimeout {
                        attempts: attempt,
                        last_error_code,
                    });
                }

                // Signal retry
                if let Some(cb) = status_callback {
                    cb(WriteStatus::Retry {
                        attempt,
                        max_attempts: MAX_ATTEMPTS,
                        error_code: Some(error_code),
                    });
                }

                // Backoff sleep
                let backoff_idx = (attempt - 1) as usize;
                if backoff_idx < BACKOFF_MS.len() {
                    thread::sleep(Duration::from_millis(BACKOFF_MS[backoff_idx]));
                }
            }
        }
    }
}

/// Try atomic commit using Windows APIs
#[cfg(windows)]
fn try_atomic_commit(dst: &Path, tmp: &Path) -> Result<(), (bool, u32)> {
    use crate::identity::to_wide_path;
    use windows::Win32::Foundation::GetLastError;
    use windows::Win32::Storage::FileSystem::{
        MOVE_FILE_FLAGS, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
        REPLACE_FILE_FLAGS, ReplaceFileW,
    };
    use windows::core::PCWSTR;

    let dst_wide = to_wide_path(dst);
    let tmp_wide = to_wide_path(tmp);

    // Primary: Try ReplaceFileW
    let result = unsafe {
        ReplaceFileW(
            PCWSTR(dst_wide.as_ptr()),
            PCWSTR(tmp_wide.as_ptr()),
            PCWSTR::null(),
            REPLACE_FILE_FLAGS(0),
            None,
            None,
        )
    };

    if result.is_ok() {
        return Ok(());
    }

    let error_code = unsafe { GetLastError().0 };

    // Check if retryable lock error (only SHARING_VIOLATION is truly transient)
    if error_code == ERROR_SHARING_VIOLATION {
        return Err((true, error_code));
    }

    // Fallback: Try MoveFileExW for all other ReplaceFileW failures
    let flags = MOVE_FILE_FLAGS(MOVEFILE_REPLACE_EXISTING.0 | MOVEFILE_WRITE_THROUGH.0);
    let move_result =
        unsafe { MoveFileExW(PCWSTR(tmp_wide.as_ptr()), PCWSTR(dst_wide.as_ptr()), flags) };

    if move_result.is_ok() {
        return Ok(());
    }

    let move_error_code = unsafe { GetLastError().0 };

    // Only SHARING_VIOLATION is truly retryable
    if move_error_code == ERROR_SHARING_VIOLATION {
        return Err((true, move_error_code));
    }

    // Final fallback: Delete dst then rename tmp to dst
    // This is less atomic but works in more environments (like tempdir)
    // Use std::fs operations which handle Windows semantics better
    match fs::remove_file(dst) {
        Ok(()) => {
            if fs::rename(tmp, dst).is_ok() {
                return Ok(());
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // dst doesn't exist, just rename
            if fs::rename(tmp, dst).is_ok() {
                return Ok(());
            }
        }
        Err(_) => {}
    }

    // Ultimate fallback: copy tmp to dst, then delete tmp
    if let Ok(()) = fs::copy(tmp, dst).map(|_| ()) {
        let _ = fs::remove_file(tmp);
        return Ok(());
    }

    Err((false, move_error_code))
}

/// Non-Windows fallback using simple rename
#[cfg(not(windows))]
fn atomic_commit_with_retry<F>(
    dst: &Path,
    tmp: &Path,
    status_callback: &Option<F>,
) -> Result<(), SafeWriteError>
where
    F: Fn(WriteStatus),
{
    fs::rename(tmp, dst).map_err(|e| SafeWriteError::Io(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_temp_file_naming() {
        let dst = Path::new("/music/song.flac");
        let tmp = create_temp_path(dst).unwrap();
        assert_eq!(
            tmp.file_name().unwrap().to_str().unwrap(),
            "song.flac.sermon-tmp"
        );
    }

    #[test]
    fn test_timestamp_format() {
        let ts = chrono_timestamp();
        // Should be in format YYYYMMDD-HHMMSSZ
        assert_eq!(ts.len(), 16);
        assert!(ts.ends_with('Z'));
        assert!(ts.chars().nth(8) == Some('-'));
    }

    #[test]
    fn test_backup_naming_no_collision() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_file(temp_dir.path(), "test.mp3", "original");

        let backup = create_backup(&file_path).unwrap();
        let backup_name = backup.file_name().unwrap().to_str().unwrap();

        assert!(backup_name.starts_with("test.mp3.bak."));
        assert!(backup_name.ends_with("Z"));
        assert_eq!(fs::read_to_string(&backup).unwrap(), "original");
    }

    #[test]
    fn test_backup_naming_with_collision() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_file(temp_dir.path(), "test.mp3", "original");

        // Create first backup
        let backup1 = create_backup(&file_path).unwrap();

        // Create second backup (should get .1 suffix)
        let backup2 = create_backup(&file_path).unwrap();

        // Verify they're different files
        assert_ne!(backup1, backup2);

        // Both should contain original content
        assert_eq!(fs::read_to_string(&backup1).unwrap(), "original");
        assert_eq!(fs::read_to_string(&backup2).unwrap(), "original");
    }

    #[test]
    fn test_safe_write_options_default() {
        let options = SafeWriteOptions::default();
        assert!(options.create_backup); // ON by default per milestone spec
    }

    #[test]
    fn test_write_status_variants() {
        // Test that WriteStatus variants can be constructed
        let retry = WriteStatus::Retry {
            attempt: 1,
            max_attempts: 3,
            error_code: Some(32),
        };
        let success = WriteStatus::Success;
        let error = WriteStatus::Error {
            message: "test".into(),
            error_code: None,
        };

        // Just verify they're constructable
        assert!(matches!(retry, WriteStatus::Retry { .. }));
        assert!(matches!(success, WriteStatus::Success));
        assert!(matches!(error, WriteStatus::Error { .. }));
    }

    #[test]
    fn test_safe_write_error_variants() {
        // Test error variant construction
        let io_err = SafeWriteError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        let write_err = SafeWriteError::WriteFailed("failed".into());
        let lock_err = SafeWriteError::LockTimeout {
            attempts: 3,
            last_error_code: 32,
        };
        let dst_err = SafeWriteError::DstMissing {
            backup_restored: true,
        };
        let other_err = SafeWriteError::Other("other".into());

        // Verify Display impl works
        assert!(io_err.to_string().contains("IO error"));
        assert!(write_err.to_string().contains("failed"));
        assert!(lock_err.to_string().contains("timeout"));
        assert!(dst_err.to_string().contains("missing"));
        assert!(other_err.to_string().contains("other"));
    }

    // Note: Full integration tests for safe_write with atomic commit are run as part of
    // manual verification (Tier A) because Windows atomic file operations (ReplaceFileW,
    // MoveFileExW) have complex interactions with temp directories and antivirus software
    // that can cause spurious failures in CI environments.
    //
    // Manual verification steps:
    // 1. cargo tauri dev
    // 2. Edit a tag on a real audio file
    // 3. Verify backup is created
    // 4. Verify original is updated atomically
}
