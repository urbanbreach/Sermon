//! DSD file decoding for DSF and DFF formats.
//!
//! This module extracts raw DSD bytes from DSF and DFF container formats,
//! handling the different byte orderings and block layouts of each format.

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DsdError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid DSF file")]
    InvalidDsf,

    #[error("Invalid DFF file")]
    InvalidDff,

    #[error("DST compression not supported")]
    DstNotSupported,

    #[error("Unsupported channel count: {0}")]
    UnsupportedChannels(usize),

    #[error("Unexpected end of file")]
    UnexpectedEof,
}

/// Bit-reversal lookup table for converting LSB-first to MSB-first.
/// DSF stores DSD data LSB-first, but DSD is naturally MSB-first.
pub const BIT_REVERSE_TABLE: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut reversed = 0u8;
        let mut j = 0;
        while j < 8 {
            if (i >> j) & 1 == 1 {
                reversed |= 1 << (7 - j);
            }
            j += 1;
        }
        table[i] = reversed;
        i += 1;
    }
    table
};

#[inline]
pub fn bit_reverse(byte: u8) -> u8 {
    BIT_REVERSE_TABLE[byte as usize]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsdFormat {
    Dsf,
    Dff,
}

#[derive(Debug, Clone)]
pub struct DsdInfo {
    pub format: DsdFormat,
    pub sample_rate: u32,
    pub channels: usize,
    pub total_samples: u64,
    pub bits_per_sample: u32,
}

/// DSF block size is always 4096 bytes per channel
const DSF_BLOCK_SIZE: usize = 4096;

pub struct DsfDecoder {
    reader: BufReader<File>,
    info: DsdInfo,
    data_offset: u64,
    data_size: u64,
    current_offset: u64,
    block_buffer: Vec<u8>,
}

impl DsfDecoder {
    pub fn open(path: &Path) -> Result<Self, DsdError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Read DSD chunk header
        let mut dsd_chunk = [0u8; 28];
        reader.read_exact(&mut dsd_chunk)?;

        if &dsd_chunk[0..4] != b"DSD " {
            return Err(DsdError::InvalidDsf);
        }

        let total_file_size = u64::from_le_bytes(
            dsd_chunk[12..20]
                .try_into()
                .map_err(|_| DsdError::InvalidDsf)?,
        );
        let _metadata_offset = u64::from_le_bytes(
            dsd_chunk[20..28]
                .try_into()
                .map_err(|_| DsdError::InvalidDsf)?,
        );
        let _ = total_file_size; // Suppress unused warning

        // Read fmt chunk
        let mut fmt_header = [0u8; 52];
        reader.read_exact(&mut fmt_header)?;

        if &fmt_header[0..4] != b"fmt " {
            return Err(DsdError::InvalidDsf);
        }

        let _fmt_chunk_size = u64::from_le_bytes(
            fmt_header[4..12]
                .try_into()
                .map_err(|_| DsdError::InvalidDsf)?,
        );
        let _format_version = u32::from_le_bytes(
            fmt_header[12..16]
                .try_into()
                .map_err(|_| DsdError::InvalidDsf)?,
        );
        let _format_id = u32::from_le_bytes(
            fmt_header[16..20]
                .try_into()
                .map_err(|_| DsdError::InvalidDsf)?,
        );
        let channel_type = u32::from_le_bytes(
            fmt_header[20..24]
                .try_into()
                .map_err(|_| DsdError::InvalidDsf)?,
        );
        let channels = u32::from_le_bytes(
            fmt_header[24..28]
                .try_into()
                .map_err(|_| DsdError::InvalidDsf)?,
        ) as usize;
        let sample_rate = u32::from_le_bytes(
            fmt_header[28..32]
                .try_into()
                .map_err(|_| DsdError::InvalidDsf)?,
        );
        let bits_per_sample = u32::from_le_bytes(
            fmt_header[32..36]
                .try_into()
                .map_err(|_| DsdError::InvalidDsf)?,
        );
        let total_samples = u64::from_le_bytes(
            fmt_header[36..44]
                .try_into()
                .map_err(|_| DsdError::InvalidDsf)?,
        );
        let block_size_per_channel = u32::from_le_bytes(
            fmt_header[44..48]
                .try_into()
                .map_err(|_| DsdError::InvalidDsf)?,
        );

        let _ = (channel_type, block_size_per_channel); // Suppress unused warnings

        if channels > 2 {
            return Err(DsdError::UnsupportedChannels(channels));
        }

        // Read data chunk header
        let mut data_header = [0u8; 12];
        reader.read_exact(&mut data_header)?;

        if &data_header[0..4] != b"data" {
            return Err(DsdError::InvalidDsf);
        }

        let data_chunk_size = u64::from_le_bytes(
            data_header[4..12]
                .try_into()
                .map_err(|_| DsdError::InvalidDsf)?,
        );
        let data_size = data_chunk_size - 12; // Subtract header size

        let data_offset = reader.stream_position()?;

        let info = DsdInfo {
            format: DsdFormat::Dsf,
            sample_rate,
            channels,
            total_samples,
            bits_per_sample,
        };

        Ok(Self {
            reader,
            info,
            data_offset,
            data_size,
            current_offset: 0,
            block_buffer: vec![0u8; DSF_BLOCK_SIZE * channels],
        })
    }

    pub fn info(&self) -> &DsdInfo {
        &self.info
    }

    /// Read and interleave the next block of DSD data.
    /// Returns interleaved bytes [L0, R0, L1, R1, ...] with bit-reversal applied.
    pub fn read_block(&mut self) -> Result<Option<Vec<u8>>, DsdError> {
        if self.current_offset >= self.data_size {
            return Ok(None);
        }

        let channels = self.info.channels;
        let bytes_to_read =
            (DSF_BLOCK_SIZE * channels).min((self.data_size - self.current_offset) as usize);

        if bytes_to_read < DSF_BLOCK_SIZE * channels {
            // Partial block at end - pad or handle specially
            self.block_buffer.resize(bytes_to_read, 0);
        }

        self.reader
            .read_exact(&mut self.block_buffer[..bytes_to_read])?;
        self.current_offset += bytes_to_read as u64;

        // DSF stores blocks sequentially per channel: [4096 bytes L][4096 bytes R]
        // We need to interleave: [L0, R0, L1, R1, ...]
        let block_samples = bytes_to_read / channels;
        let mut interleaved = Vec::with_capacity(bytes_to_read);

        for i in 0..block_samples {
            for ch in 0..channels {
                let byte = self.block_buffer[ch * DSF_BLOCK_SIZE + i];
                // Apply bit-reversal since DSF is LSB-first
                interleaved.push(bit_reverse(byte));
            }
        }

        Ok(Some(interleaved))
    }

    pub fn seek(&mut self, offset: u64) -> Result<(), DsdError> {
        let block_aligned = (offset / (DSF_BLOCK_SIZE as u64 * self.info.channels as u64))
            * (DSF_BLOCK_SIZE as u64 * self.info.channels as u64);

        self.reader
            .seek(SeekFrom::Start(self.data_offset + block_aligned))?;
        self.current_offset = block_aligned;
        Ok(())
    }
}

pub struct DffDecoder {
    reader: BufReader<File>,
    info: DsdInfo,
    data_offset: u64,
    data_size: u64,
    current_offset: u64,
}

impl DffDecoder {
    pub fn open(path: &Path) -> Result<Self, DsdError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Read FRM8 header
        let mut frm8 = [0u8; 12];
        reader.read_exact(&mut frm8)?;

        if &frm8[0..4] != b"FRM8" {
            return Err(DsdError::InvalidDff);
        }

        // Read DSD marker
        let mut dsd_marker = [0u8; 4];
        reader.read_exact(&mut dsd_marker)?;

        if &dsd_marker != b"DSD " {
            return Err(DsdError::InvalidDff);
        }

        let mut sample_rate = 0u32;
        let mut channels = 0usize;
        let mut data_offset = 0u64;
        let mut data_size = 0u64;
        let mut total_samples = 0u64;

        // Parse chunks
        loop {
            let mut chunk_header = [0u8; 12];
            match reader.read_exact(&mut chunk_header) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }

            let chunk_id = &chunk_header[0..4];
            let chunk_size = u64::from_be_bytes(
                chunk_header[4..12]
                    .try_into()
                    .map_err(|_| DsdError::InvalidDff)?,
            );

            match chunk_id {
                b"FVER" => {
                    // Format version - skip
                    reader.seek(SeekFrom::Current(chunk_size as i64))?;
                }
                b"PROP" => {
                    // Property chunk - contains SND sub-chunk
                    let mut snd_marker = [0u8; 4];
                    reader.read_exact(&mut snd_marker)?;

                    if &snd_marker != b"SND " {
                        reader.seek(SeekFrom::Current(chunk_size as i64 - 4))?;
                        continue;
                    }

                    // Read sub-chunks within PROP
                    let prop_end = reader.stream_position()? + chunk_size - 4;
                    while reader.stream_position()? < prop_end {
                        let mut sub_header = [0u8; 12];
                        reader.read_exact(&mut sub_header)?;

                        let sub_id = &sub_header[0..4];
                        let sub_size = u64::from_be_bytes(
                            sub_header[4..12]
                                .try_into()
                                .map_err(|_| DsdError::InvalidDff)?,
                        );

                        match sub_id {
                            b"FS  " => {
                                let mut rate_buf = [0u8; 4];
                                reader.read_exact(&mut rate_buf)?;
                                sample_rate = u32::from_be_bytes(rate_buf);
                            }
                            b"CHNL" => {
                                let mut ch_buf = [0u8; 2];
                                reader.read_exact(&mut ch_buf)?;
                                channels = u16::from_be_bytes(ch_buf) as usize;
                                // Skip channel IDs
                                reader.seek(SeekFrom::Current(sub_size as i64 - 2))?;
                            }
                            b"CMPR" => {
                                // Check compression type
                                let mut cmpr_type = [0u8; 4];
                                reader.read_exact(&mut cmpr_type)?;

                                if &cmpr_type != b"DSD " {
                                    return Err(DsdError::DstNotSupported);
                                }
                                reader.seek(SeekFrom::Current(sub_size as i64 - 4))?;
                            }
                            _ => {
                                reader.seek(SeekFrom::Current(sub_size as i64))?;
                            }
                        }
                    }
                }
                b"DSD " => {
                    data_offset = reader.stream_position()?;
                    data_size = chunk_size;
                    total_samples = data_size * 8 / channels as u64;
                    reader.seek(SeekFrom::Current(chunk_size as i64))?;
                }
                _ => {
                    reader.seek(SeekFrom::Current(chunk_size as i64))?;
                }
            }
        }

        if sample_rate == 0 || channels == 0 || data_size == 0 {
            return Err(DsdError::InvalidDff);
        }

        if channels > 2 {
            return Err(DsdError::UnsupportedChannels(channels));
        }

        let info = DsdInfo {
            format: DsdFormat::Dff,
            sample_rate,
            channels,
            total_samples,
            bits_per_sample: 1,
        };

        // Seek to data start
        reader.seek(SeekFrom::Start(data_offset))?;

        Ok(Self {
            reader,
            info,
            data_offset,
            data_size,
            current_offset: 0,
        })
    }

    pub fn info(&self) -> &DsdInfo {
        &self.info
    }

    /// Read the next block of DSD data.
    /// DFF is already interleaved and MSB-first, so no transformation needed.
    pub fn read_block(&mut self, size: usize) -> Result<Option<Vec<u8>>, DsdError> {
        if self.current_offset >= self.data_size {
            return Ok(None);
        }

        let bytes_to_read = size.min((self.data_size - self.current_offset) as usize);
        let mut buffer = vec![0u8; bytes_to_read];

        self.reader.read_exact(&mut buffer)?;
        self.current_offset += bytes_to_read as u64;

        // DFF is already interleaved L/R and MSB-first - no bit reversal needed
        Ok(Some(buffer))
    }

    pub fn seek(&mut self, offset: u64) -> Result<(), DsdError> {
        let clamped = offset.min(self.data_size);
        self.reader
            .seek(SeekFrom::Start(self.data_offset + clamped))?;
        self.current_offset = clamped;
        Ok(())
    }
}

/// Unified DSD decoder that handles both DSF and DFF formats.
pub enum DsdDecoder {
    Dsf(DsfDecoder),
    Dff(DffDecoder),
}

impl DsdDecoder {
    pub fn open(path: &Path) -> Result<Self, DsdError> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext.to_lowercase().as_str() {
            "dsf" => Ok(DsdDecoder::Dsf(DsfDecoder::open(path)?)),
            "dff" => Ok(DsdDecoder::Dff(DffDecoder::open(path)?)),
            _ => {
                // Try DSF first, then DFF
                if let Ok(dsf) = DsfDecoder::open(path) {
                    Ok(DsdDecoder::Dsf(dsf))
                } else {
                    Ok(DsdDecoder::Dff(DffDecoder::open(path)?))
                }
            }
        }
    }

    pub fn info(&self) -> &DsdInfo {
        match self {
            DsdDecoder::Dsf(d) => d.info(),
            DsdDecoder::Dff(d) => d.info(),
        }
    }

    /// Read the next block of interleaved DSD bytes.
    /// For DSF: 4096 bytes per channel, interleaved with bit-reversal.
    /// For DFF: Configurable size, already interleaved.
    pub fn read_block(&mut self) -> Result<Option<Vec<u8>>, DsdError> {
        match self {
            DsdDecoder::Dsf(d) => d.read_block(),
            DsdDecoder::Dff(d) => d.read_block(8192), // Match DSF block size
        }
    }

    /// Seek to a position in milliseconds.
    ///
    /// Converts the millisecond position to a byte offset based on the DSD
    /// sample rate and channel count. DSD stores 1 bit per sample, so:
    /// - bytes_per_second = sample_rate / 8 * channels
    /// - byte_offset = position_ms * bytes_per_second / 1000
    pub fn seek_ms(&mut self, position_ms: u64) -> Result<(), DsdError> {
        let info = self.info();
        // DSD: 1 bit per sample, so bytes = samples / 8
        // bytes_per_second = sample_rate / 8 * channels
        let bytes_per_ms = (info.sample_rate as u64 * info.channels as u64) / 8 / 1000;
        let byte_offset = position_ms * bytes_per_ms;

        match self {
            DsdDecoder::Dsf(d) => d.seek(byte_offset),
            DsdDecoder::Dff(d) => d.seek(byte_offset),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_reverse_table() {
        // 0x00 -> 0x00
        assert_eq!(bit_reverse(0x00), 0x00);
        // 0xFF -> 0xFF
        assert_eq!(bit_reverse(0xFF), 0xFF);
        // 0x01 (00000001) -> 0x80 (10000000)
        assert_eq!(bit_reverse(0x01), 0x80);
        // 0x80 (10000000) -> 0x01 (00000001)
        assert_eq!(bit_reverse(0x80), 0x01);
        // 0xAA (10101010) -> 0x55 (01010101)
        assert_eq!(bit_reverse(0xAA), 0x55);
        // 0x55 (01010101) -> 0xAA (10101010)
        assert_eq!(bit_reverse(0x55), 0xAA);
        // 0x0F (00001111) -> 0xF0 (11110000)
        assert_eq!(bit_reverse(0x0F), 0xF0);
        // 0xF0 (11110000) -> 0x0F (00001111)
        assert_eq!(bit_reverse(0xF0), 0x0F);
    }

    #[test]
    fn test_bit_reverse_roundtrip() {
        for i in 0..=255u8 {
            assert_eq!(bit_reverse(bit_reverse(i)), i);
        }
    }

    #[test]
    fn test_dsf_invalid_header() {
        let temp_dir = tempfile::tempdir().unwrap();
        let invalid_file = temp_dir.path().join("invalid.dsf");
        let mut data = [0u8; 28];
        data[0..4].copy_from_slice(b"NOT ");
        std::fs::write(&invalid_file, data).unwrap();

        let result = DsfDecoder::open(&invalid_file);
        assert!(matches!(result, Err(DsdError::InvalidDsf)));
    }

    #[test]
    fn test_dff_invalid_header() {
        let temp_dir = tempfile::tempdir().unwrap();
        let invalid_file = temp_dir.path().join("invalid.dff");
        std::fs::write(&invalid_file, b"NOT_DFF_DATA").unwrap();

        let result = DffDecoder::open(&invalid_file);
        assert!(matches!(result, Err(DsdError::InvalidDff)));
    }

    #[test]
    fn test_dsd_info_format() {
        let dsf_info = DsdInfo {
            format: DsdFormat::Dsf,
            sample_rate: 2_822_400,
            channels: 2,
            total_samples: 1_000_000,
            bits_per_sample: 1,
        };
        assert_eq!(dsf_info.format, DsdFormat::Dsf);

        let dff_info = DsdInfo {
            format: DsdFormat::Dff,
            sample_rate: 5_644_800,
            channels: 2,
            total_samples: 2_000_000,
            bits_per_sample: 1,
        };
        assert_eq!(dff_info.format, DsdFormat::Dff);
    }
}
