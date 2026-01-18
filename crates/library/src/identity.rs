use crate::error::LibraryError;
use std::path::Path;

/// Result of file identity extraction
#[derive(Debug, Clone)]
pub struct FileIdentity {
    pub source: IdentitySource,
    pub volume_serial: Option<u64>,
    pub file_id: Option<u64>,
    pub mtime_ms: i64,
    pub size_bytes: i64,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentitySource {
    Ntfs,
    Fallback,
}

impl IdentitySource {
    pub fn as_str(&self) -> &'static str {
        match self {
            IdentitySource::Ntfs => "ntfs",
            IdentitySource::Fallback => "fallback",
        }
    }
}

/// Get file identity - tries NTFS first on Windows, falls back to path+mtime+size
pub fn get_file_identity(path: &Path) -> Result<FileIdentity, LibraryError> {
    let metadata = std::fs::metadata(path)?;
    let mtime_ms = metadata
        .modified()
        .map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64
        })
        .unwrap_or(0);
    let size_bytes = metadata.len() as i64;

    #[cfg(windows)]
    {
        if let Some((vol_serial, file_id)) = get_ntfs_identity(path) {
            return Ok(FileIdentity {
                source: IdentitySource::Ntfs,
                volume_serial: Some(vol_serial),
                file_id: Some(file_id),
                mtime_ms,
                size_bytes,
                hash: None,
            });
        }
    }

    // Fallback: path + mtime + size (hash computed separately when needed)
    Ok(FileIdentity {
        source: IdentitySource::Fallback,
        volume_serial: None,
        file_id: None,
        mtime_ms,
        size_bytes,
        hash: None,
    })
}

/// Windows-specific: Get Volume Serial Number + File ID using Win32 API
#[cfg(windows)]
fn get_ntfs_identity(path: &Path) -> Option<(u64, u64)> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_BACKUP_SEMANTICS,
        FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, GetFileInformationByHandle,
        OPEN_EXISTING,
    };
    use windows::core::PCWSTR;

    // Convert path to wide string with \\?\ prefix for long path support
    let wide_path = to_wide_path(path);
    let path_ptr = PCWSTR(wide_path.as_ptr());

    unsafe {
        let handle = CreateFileW(
            path_ptr,
            0, // FILE_READ_ATTRIBUTES = 0x80, but 0 works for getting info
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_ATTRIBUTE_NORMAL,
            None,
        );

        match handle {
            Ok(h) => {
                let mut info = BY_HANDLE_FILE_INFORMATION::default();
                let result = GetFileInformationByHandle(h, &mut info);
                let _ = CloseHandle(h);

                if result.is_ok() {
                    let volume_serial = info.dwVolumeSerialNumber as u64;
                    let file_id =
                        ((info.nFileIndexHigh as u64) << 32) | (info.nFileIndexLow as u64);
                    Some((volume_serial, file_id))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

/// Convert Path to wide string with \\?\\ prefix for long path support
#[cfg(windows)]
pub fn to_wide_path(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    let path_str = path.as_os_str();
    let needs_prefix = !path_str.to_string_lossy().starts_with("\\\\?\\");

    let mut wide: Vec<u16> = if needs_prefix {
        "\\\\?\\".encode_utf16().collect()
    } else {
        Vec::new()
    };

    wide.extend(path_str.encode_wide());
    wide.push(0); // null terminator
    wide
}

/// Compute blake3 hash of first 256KB of file (for fallback identity)
pub fn compute_partial_hash(path: &Path) -> Result<String, LibraryError> {
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut buffer = vec![0u8; 256 * 1024]; // 256KB
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    let hash = blake3::hash(&buffer);
    Ok(hash.to_hex().to_string())
}

/// Check if file identity has changed (for incremental scanning)
pub fn identity_changed(old: &FileIdentity, new: &FileIdentity) -> bool {
    match (&old.source, &new.source) {
        (IdentitySource::Ntfs, IdentitySource::Ntfs) => {
            // NTFS: same file ID means same file, just check mtime for content change
            old.volume_serial != new.volume_serial
                || old.file_id != new.file_id
                || old.mtime_ms != new.mtime_ms
                || old.size_bytes != new.size_bytes
        }
        _ => {
            // Fallback: path+mtime+size
            old.mtime_ms != new.mtime_ms || old.size_bytes != new.size_bytes
        }
    }
}
