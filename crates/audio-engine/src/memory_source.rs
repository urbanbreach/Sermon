//! Memory-backed audio source for full-file RAM loading.
//!
//! This module provides `MemoryAudioSource`, which loads an entire audio file
//! into memory for gapless playback. By preloading the file into RAM, the decoder
//! can read data without blocking I/O operations, enabling seamless track transitions.
//!
//! # Usage
//!
//! ```rust,no_run
//! use std::path::Path;
//! use audio_engine::memory_source::MemoryAudioSource;
//! use symphonia::core::io::MediaSourceStream;
//!
//! let source = MemoryAudioSource::load(Path::new("track.flac")).unwrap();
//! let mss = MediaSourceStream::new(Box::new(source), Default::default());
//! // Use mss with Symphonia's probe and format reader
//! ```

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use symphonia::core::io::MediaSource;

/// A memory-backed audio source that loads the entire file into RAM.
///
/// This source is designed for gapless playback where the next track's data
/// needs to be immediately available without blocking I/O. The entire file
/// is loaded into memory on construction, and all subsequent read/seek
/// operations are performed against the in-memory buffer.
///
/// # Performance Considerations
///
/// - Memory usage equals the file size (no compression)
/// - Loading is blocking I/O (should be done on decode thread)
/// - Read/seek operations are very fast (memory access only)
pub struct MemoryAudioSource {
    data: Vec<u8>,
    position: u64,
}

impl MemoryAudioSource {
    /// Load an entire audio file into memory.
    pub fn load(path: &Path) -> Result<Self, io::Error> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(Self { data, position: 0 })
    }
}

impl Read for MemoryAudioSource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let pos = self.position as usize;
        if pos >= self.data.len() {
            return Ok(0);
        }
        let available = &self.data[pos..];
        let to_read = buf.len().min(available.len());
        buf[..to_read].copy_from_slice(&available[..to_read]);
        self.position += to_read as u64;
        Ok(to_read)
    }
}

impl Seek for MemoryAudioSource {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let len = self.data.len() as i64;
        let new_pos = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::End(offset) => len + offset,
            SeekFrom::Current(offset) => self.position as i64 + offset,
        };

        if new_pos < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "seek to negative position",
            ));
        }

        self.position = new_pos as u64;
        Ok(self.position)
    }
}

impl MediaSource for MemoryAudioSource {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.data.len() as u64)
    }
}

// SAFETY: The data is owned and position is only accessed via &mut self
unsafe impl Send for MemoryAudioSource {}
unsafe impl Sync for MemoryAudioSource {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_file(content: &[u8]) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write_all(content)
            .expect("failed to write to temp file");
        file.flush().expect("failed to flush temp file");
        file
    }

    #[test]
    fn test_memory_source_loads_file() {
        let content = b"Hello, gapless world!";
        let temp_file = create_temp_file(content);

        let source = MemoryAudioSource::load(temp_file.path()).expect("failed to load file");

        assert_eq!(source.data.len(), content.len());
        assert_eq!(&source.data, content);
        assert_eq!(source.position, 0);
        assert_eq!(source.byte_len(), Some(content.len() as u64));
        assert!(source.is_seekable());
    }

    #[test]
    fn test_memory_source_seek_read() {
        let content = b"0123456789ABCDEF";
        let temp_file = create_temp_file(content);

        let mut source = MemoryAudioSource::load(temp_file.path()).expect("failed to load file");

        let mut buf = [0u8; 4];
        let n = source.read(&mut buf).expect("read failed");
        assert_eq!(n, 4);
        assert_eq!(&buf, b"0123");
        assert_eq!(source.position, 4);

        let pos = source.seek(SeekFrom::Start(8)).expect("seek failed");
        assert_eq!(pos, 8);
        assert_eq!(source.position, 8);

        let n = source.read(&mut buf).expect("read failed");
        assert_eq!(n, 4);
        assert_eq!(&buf, b"89AB");

        let pos = source.seek(SeekFrom::Current(-4)).expect("seek failed");
        assert_eq!(pos, 8);

        let pos = source.seek(SeekFrom::End(-4)).expect("seek failed");
        assert_eq!(pos, 12);

        let n = source.read(&mut buf).expect("read failed");
        assert_eq!(n, 4);
        assert_eq!(&buf, b"CDEF");

        let n = source.read(&mut buf).expect("read at eof failed");
        assert_eq!(n, 0);
    }

    #[test]
    fn test_memory_source_seek_negative_error() {
        let content = b"test";
        let temp_file = create_temp_file(content);

        let mut source = MemoryAudioSource::load(temp_file.path()).expect("failed to load file");

        let result = source.seek(SeekFrom::Start(0));
        assert!(result.is_ok());

        let result = source.seek(SeekFrom::Current(-1));
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_source_load_nonexistent_file() {
        let result = MemoryAudioSource::load(Path::new("/nonexistent/path/to/file.flac"));
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_source_empty_file() {
        let temp_file = create_temp_file(b"");

        let mut source = MemoryAudioSource::load(temp_file.path()).expect("failed to load file");

        assert_eq!(source.byte_len(), Some(0));
        assert!(source.is_seekable());

        let mut buf = [0u8; 4];
        let n = source.read(&mut buf).expect("read failed");
        assert_eq!(n, 0);
    }

    #[test]
    fn test_memory_source_media_source_trait() {
        let content = b"media source test";
        let temp_file = create_temp_file(content);

        let source = MemoryAudioSource::load(temp_file.path()).expect("failed to load file");

        assert!(source.is_seekable());
        assert_eq!(source.byte_len(), Some(content.len() as u64));
    }
}
