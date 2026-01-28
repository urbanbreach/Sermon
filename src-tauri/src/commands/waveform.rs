use audio_engine::decode::AudioDecoder;
use audio_engine::{DsdDecoder, DsdError};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use library::{apply_migrations, open_db};
use rusqlite::OptionalExtension;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::time::Instant;
use tauri::State;
use tracing::info;

use crate::state::{LibraryState, WaveformCacheState};

struct GeneratePeaksTiming {
    decode_open_ms: u64,
    decode_loop_ms: u64,
    normalize_ms: u64,
}

fn drain_remaining(
    decoder: &mut AudioDecoder,
    buf: &mut Vec<f32>,
    channels: usize,
    total_samples: &mut u64,
) {
    loop {
        match decoder.decode_next_into(buf) {
            Ok(Some(len)) => {
                *total_samples += (len / channels) as u64;
            }
            Ok(None) | Err(_) => break,
        }
    }
}

/// Response structure for waveform peaks data
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WaveformPeaksResponse {
    pub duration_ms: u64,
    pub bin_ms: u32,
    pub peaks_base64: String,
    pub format_version: u8,
}

/// Compute cache key from track identity (track_id + file_size + mtime)
fn compute_waveform_cache_key(track_id: i64, size_bytes: i64, mtime_ms: i64) -> String {
    let input = format!("waveform::{}::{}::{}", track_id, size_bytes, mtime_ms);
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

/// Check if a file is a DSD format (DSF or DFF) by extension
fn is_dsd_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    matches!(ext.to_ascii_lowercase().as_str(), "dsf" | "dff")
}

/// Convert bit density (ones count) to amplitude using centered-absolute mapping.
/// Silence (~50% ones) maps to near zero; full scale (0% or 100% ones) maps to 1.0.
#[inline]
fn amp_from_ones(ones: u32, total_bits: u32) -> f32 {
    if total_bits == 0 {
        return 0.0;
    }
    let density = ones as f32 / total_bits as f32;
    // Centered-absolute: amp = |2*density - 1|
    // silence (50%) -> 0, full scale (0% or 100%) -> 1
    (2.0 * density - 1.0).abs()
}

/// Count ones in interleaved stereo DSD bytes, returning (left_ones, right_ones)
fn popcount_interleaved_2ch(bytes: &[u8]) -> (u32, u32) {
    let mut left_ones: u32 = 0;
    let mut right_ones: u32 = 0;
    for chunk in bytes.chunks_exact(2) {
        left_ones += chunk[0].count_ones();
        right_ones += chunk[1].count_ones();
    }
    (left_ones, right_ones)
}

/// Route to appropriate peak generator based on file type
fn generate_peaks(path: &Path) -> Result<(Vec<u8>, u64, u32, GeneratePeaksTiming), String> {
    if is_dsd_file(path) {
        generate_peaks_dsd(path)
    } else {
        generate_peaks_pcm(path)
    }
}

/// Generate peaks for DSD files (DSF/DFF) using bit-density amplitude calculation
fn generate_peaks_dsd(path: &Path) -> Result<(Vec<u8>, u64, u32, GeneratePeaksTiming), String> {
    let open_start = Instant::now();
    let mut decoder = DsdDecoder::open(path)
        .map_err(|e: DsdError| format!("Failed to open DSD: {}", e))?;
    let decode_open_ms = open_start.elapsed().as_millis() as u64;

    let info = decoder.info();
    let sample_rate = info.sample_rate;
    let channels = info.channels;
    let total_dsd_samples = info.total_samples;

    if sample_rate == 0 || channels == 0 {
        return Err("Invalid DSD format: zero sample rate or channels".to_string());
    }

    const BIN_MS: u32 = 25;
    const MAX_BINS: usize = 50000;
    // Micro-window for peak detection within each bin (1024 bits = 128 bytes per channel)
    const WINDOW_BITS: usize = 1024;
    const WINDOW_BYTES_PER_CH: usize = WINDOW_BITS / 8; // 128

    // DSD sample rate is bits per second per channel
    // Calculate bytes per bin per channel
    let bits_per_bin = (sample_rate as u64 * BIN_MS as u64) / 1000;
    let bytes_per_bin_per_ch = ((bits_per_bin + 7) / 8) as usize; // Round up
    let bytes_per_bin_total = bytes_per_bin_per_ch * channels;

    let mut peaks: Vec<f32> = Vec::with_capacity(MAX_BINS);
    let mut current_bin_max: f32 = 0.0;
    let mut bytes_in_current_bin: usize = 0;
    let mut block_buffer: Vec<u8> = Vec::new();
    let mut block_offset: usize = 0;

    let loop_start = Instant::now();

    'outer: loop {
        // Read next block if we've exhausted the current one
        if block_offset >= block_buffer.len() {
            match decoder.read_block() {
                Ok(Some(block)) => {
                    block_buffer = block;
                    block_offset = 0;
                }
                Ok(None) => break, // EOF
                Err(e) => return Err(format!("DSD decode error: {}", e)),
            }
        }

        if block_buffer.is_empty() {
            break;
        }

        // Process bytes from current block
        while block_offset < block_buffer.len() {
            // Calculate how many bytes to process in this iteration
            let remaining_in_bin = bytes_per_bin_total.saturating_sub(bytes_in_current_bin);
            let remaining_in_block = block_buffer.len() - block_offset;

            // Use micro-windows for peak detection
            let window_bytes_total = WINDOW_BYTES_PER_CH * channels;
            let bytes_to_process = remaining_in_bin
                .min(remaining_in_block)
                .min(window_bytes_total);

            if bytes_to_process == 0 {
                break;
            }

            // Ensure we process complete channel frames
            let bytes_to_process = (bytes_to_process / channels) * channels;
            if bytes_to_process == 0 {
                block_offset += 1; // Skip incomplete frame
                continue;
            }

            let window_slice = &block_buffer[block_offset..block_offset + bytes_to_process];
            block_offset += bytes_to_process;
            bytes_in_current_bin += bytes_to_process;

            // Calculate amplitude for this window
            let amp_window = match channels {
                1 => {
                    let ones: u32 = window_slice.iter().map(|b| b.count_ones()).sum();
                    let total_bits = (bytes_to_process * 8) as u32;
                    amp_from_ones(ones, total_bits)
                }
                2 => {
                    let (ones_l, ones_r) = popcount_interleaved_2ch(window_slice);
                    let bits_per_ch = ((bytes_to_process / 2) * 8) as u32;
                    let amp_l = amp_from_ones(ones_l, bits_per_ch);
                    let amp_r = amp_from_ones(ones_r, bits_per_ch);
                    amp_l.max(amp_r)
                }
                _ => {
                    // Generic multi-channel: sum all ones, average amplitude
                    let ones: u32 = window_slice.iter().map(|b| b.count_ones()).sum();
                    let total_bits = (bytes_to_process * 8) as u32;
                    amp_from_ones(ones, total_bits)
                }
            };

            current_bin_max = current_bin_max.max(amp_window);

            // Check if bin is complete
            if bytes_in_current_bin >= bytes_per_bin_total {
                peaks.push(current_bin_max);
                current_bin_max = 0.0;
                bytes_in_current_bin = 0;

                if peaks.len() >= MAX_BINS {
                    break 'outer;
                }
            }
        }
    }

    // Handle trailing partial bin
    if bytes_in_current_bin > 0 && peaks.len() < MAX_BINS {
        peaks.push(current_bin_max);
    }

    let decode_loop_ms = loop_start.elapsed().as_millis() as u64;

    if peaks.is_empty() {
        return Err("No peaks computed - DSD file may be empty".to_string());
    }

    // Normalize to u8 (same as PCM)
    let normalize_start = Instant::now();
    let global_max = peaks.iter().fold(0.0f32, |a, &b| a.max(b));
    let normalized: Vec<u8> = if global_max > 0.0 {
        peaks.iter().map(|&p| ((p / global_max) * 255.0) as u8).collect()
    } else {
        vec![0u8; peaks.len()]
    };
    let normalize_ms = normalize_start.elapsed().as_millis() as u64;

    // Calculate duration from DSD metadata
    let duration_ms = (total_dsd_samples as f64 / sample_rate as f64 * 1000.0) as u64;

    let timing = GeneratePeaksTiming {
        decode_open_ms,
        decode_loop_ms,
        normalize_ms,
    };

    Ok((normalized, duration_ms, BIN_MS, timing))
}

/// Generate peaks for PCM files (FLAC, WAV, MP3, etc.) using Symphonia
fn generate_peaks_pcm(path: &Path) -> Result<(Vec<u8>, u64, u32, GeneratePeaksTiming), String> {
    let open_start = Instant::now();
    let mut decoder = AudioDecoder::open(path).map_err(|e| format!("Failed to open audio: {}", e))?;
    let decode_open_ms = open_start.elapsed().as_millis() as u64;
    
    let sample_rate = decoder.sample_rate();
    let channels = decoder.channels();
    
    if sample_rate == 0 || channels == 0 {
        return Err("Invalid audio format: zero sample rate or channels".to_string());
    }
    
    const BIN_MS: u32 = 25;
    const MAX_BINS: usize = 50000;
    
    let samples_per_bin = (sample_rate as f64 * BIN_MS as f64 / 1000.0) as usize;
    
    if samples_per_bin == 0 {
        return Err("Invalid samples per bin calculation".to_string());
    }
    
    let mut peaks: Vec<f32> = Vec::with_capacity(MAX_BINS);
    let mut current_bin_max: f32 = 0.0;
    let mut samples_in_current_bin: usize = 0;
    let mut total_samples: u64 = 0;
    
    let mut samples_buf: Vec<f32> = Vec::with_capacity(8192);
    
    let loop_start = Instant::now();
    loop {
        match decoder.decode_next_into(&mut samples_buf) {
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(e) => return Err(format!("Decode error: {}", e)),
        };
        
        if samples_buf.is_empty() {
            continue;
        }
        
        match channels {
            1 => {
                for &sample in &samples_buf {
                    let frame_max = sample.abs();
                    current_bin_max = current_bin_max.max(frame_max);
                    samples_in_current_bin += 1;
                    total_samples += 1;
                    
                    if samples_in_current_bin >= samples_per_bin {
                        peaks.push(current_bin_max);
                        current_bin_max = 0.0;
                        samples_in_current_bin = 0;
                        
                        if peaks.len() >= MAX_BINS {
                            drain_remaining(&mut decoder, &mut samples_buf, channels, &mut total_samples);
                            break;
                        }
                    }
                }
            }
            2 => {
                for chunk in samples_buf.chunks_exact(2) {
                    let frame_max = chunk[0].abs().max(chunk[1].abs());
                    current_bin_max = current_bin_max.max(frame_max);
                    samples_in_current_bin += 1;
                    total_samples += 1;
                    
                    if samples_in_current_bin >= samples_per_bin {
                        peaks.push(current_bin_max);
                        current_bin_max = 0.0;
                        samples_in_current_bin = 0;
                        
                        if peaks.len() >= MAX_BINS {
                            drain_remaining(&mut decoder, &mut samples_buf, channels, &mut total_samples);
                            break;
                        }
                    }
                }
            }
            _ => {
                for frame in samples_buf.chunks_exact(channels) {
                    let frame_max: f32 = frame.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
                    current_bin_max = current_bin_max.max(frame_max);
                    samples_in_current_bin += 1;
                    total_samples += 1;
                    
                    if samples_in_current_bin >= samples_per_bin {
                        peaks.push(current_bin_max);
                        current_bin_max = 0.0;
                        samples_in_current_bin = 0;
                        
                        if peaks.len() >= MAX_BINS {
                            drain_remaining(&mut decoder, &mut samples_buf, channels, &mut total_samples);
                            break;
                        }
                    }
                }
            }
        }
        
        if peaks.len() >= MAX_BINS {
            break;
        }
    }
    
    if samples_in_current_bin > 0 && peaks.len() < MAX_BINS {
        peaks.push(current_bin_max);
    }
    let decode_loop_ms = loop_start.elapsed().as_millis() as u64;
    
    if peaks.is_empty() {
        return Err("No peaks computed - audio file may be empty".to_string());
    }
    
    let normalize_start = Instant::now();
    let global_max = peaks.iter().fold(0.0f32, |a, &b| a.max(b));
    let normalized: Vec<u8> = if global_max > 0.0 {
        peaks.iter().map(|&p| ((p / global_max) * 255.0) as u8).collect()
    } else {
        vec![0u8; peaks.len()]
    };
    let normalize_ms = normalize_start.elapsed().as_millis() as u64;
    
    let duration_ms = (total_samples as f64 / sample_rate as f64 * 1000.0) as u64;
    
    let timing = GeneratePeaksTiming {
        decode_open_ms,
        decode_loop_ms,
        normalize_ms,
    };
    
    Ok((normalized, duration_ms, BIN_MS, timing))
}

/// Get or generate waveform peaks for a track
#[tauri::command]
pub async fn cmd_waveform_get_peaks(
    track_id: i64,
    bypass_cache: Option<bool>,
    library_state: State<'_, LibraryState>,
    waveform_state: State<'_, WaveformCacheState>,
) -> Result<WaveformPeaksResponse, String> {
    let total_start = Instant::now();
    
    #[cfg(debug_assertions)]
    let should_bypass = bypass_cache.unwrap_or(false);
    #[cfg(not(debug_assertions))]
    let should_bypass = false;
    let _ = bypass_cache;
    
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;
    apply_migrations(&conn).map_err(|e| e.to_string())?;
    
    let track_info: Option<(String, i64, i64, Option<i64>)> = conn
        .query_row(
            "SELECT path, size_bytes, mtime_ms, duration_ms FROM tracks WHERE id = ?1",
            rusqlite::params![track_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    
    let (track_path, size_bytes, mtime_ms, db_duration_ms) = track_info
        .ok_or_else(|| format!("Track not found: {}", track_id))?;
    
    let cache_key = compute_waveform_cache_key(track_id, size_bytes, mtime_ms);
    
    let cache_read_start = Instant::now();
    if !should_bypass {
        let _lock = waveform_state.lock.lock();
        let cache_path = waveform_state.cache_dir.join(&cache_key);
        
        if cache_path.exists() {
            let cached_data = fs::read(&cache_path)
                .map_err(|e| format!("Failed to read cache file: {}", e))?;
            
            if cached_data.len() >= 13 {
                let format_version = cached_data[0];
                let duration_ms = u64::from_le_bytes(cached_data[1..9].try_into().unwrap());
                let bin_ms = u32::from_le_bytes(cached_data[9..13].try_into().unwrap());
                let peaks = &cached_data[13..];
                let cache_read_ms = cache_read_start.elapsed().as_millis() as u64;
                let total_ms = total_start.elapsed().as_millis() as u64;
                
                info!(
                    track_id = track_id,
                    cache_read_ms = cache_read_ms,
                    total_ms = total_ms,
                    peaks_count = peaks.len(),
                    cache_hit = true,
                    "waveform_perf"
                );
                
                return Ok(WaveformPeaksResponse {
                    duration_ms,
                    bin_ms,
                    peaks_base64: BASE64.encode(peaks),
                    format_version,
                });
            }
        }
    }
    let cache_read_ms = cache_read_start.elapsed().as_millis() as u64;
    
    let path = track_path.clone();
    let (peaks, duration_ms, bin_ms, timing) = tauri::async_runtime::spawn_blocking(move || {
        generate_peaks(Path::new(&path))
    })
    .await
    .map_err(|e| e.to_string())??;
    
    let final_duration_ms = db_duration_ms.map(|d| d as u64).unwrap_or(duration_ms);
    
    let cache_write_start = Instant::now();
    let format_version: u8 = 1;
    {
        let _lock = waveform_state.lock.lock();
        let cache_path = waveform_state.cache_dir.join(&cache_key);
        
        let mut cache_data = Vec::with_capacity(13 + peaks.len());
        cache_data.push(format_version);
        cache_data.extend_from_slice(&final_duration_ms.to_le_bytes());
        cache_data.extend_from_slice(&bin_ms.to_le_bytes());
        cache_data.extend_from_slice(&peaks);
        
        fs::write(&cache_path, &cache_data)
            .map_err(|e| format!("Failed to write cache file: {}", e))?;
    }
    let cache_write_ms = cache_write_start.elapsed().as_millis() as u64;
    let total_ms = total_start.elapsed().as_millis() as u64;
    
    info!(
        track_id = track_id,
        cache_read_ms = cache_read_ms,
        decode_open_ms = timing.decode_open_ms,
        decode_loop_ms = timing.decode_loop_ms,
        normalize_ms = timing.normalize_ms,
        cache_write_ms = cache_write_ms,
        total_ms = total_ms,
        peaks_count = peaks.len(),
        cache_hit = false,
        "waveform_perf"
    );
    
    Ok(WaveformPeaksResponse {
        duration_ms: final_duration_ms,
        bin_ms,
        peaks_base64: BASE64.encode(&peaks),
        format_version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_peaks_normalization() {
        let raw_peaks: Vec<f32> = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        let global_max = raw_peaks.iter().fold(0.0f32, |a, &b| a.max(b));
        
        let normalized: Vec<u8> = raw_peaks
            .iter()
            .map(|&p| ((p / global_max) * 255.0) as u8)
            .collect();
        
        assert_eq!(normalized, vec![0, 63, 127, 191, 255]);
    }
    
    #[test]
    fn test_cache_key_generation() {
        let key1 = compute_waveform_cache_key(1, 1000, 2000);
        let key2 = compute_waveform_cache_key(1, 1000, 2000);
        let key3 = compute_waveform_cache_key(2, 1000, 2000);
        
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
        assert_eq!(key1.len(), 64);
    }

    #[test]
    fn test_is_dsd_file() {
        assert!(is_dsd_file(Path::new("/music/track.dsf")));
        assert!(is_dsd_file(Path::new("/music/track.DSF")));
        assert!(is_dsd_file(Path::new("/music/track.dff")));
        assert!(is_dsd_file(Path::new("/music/track.DFF")));
        assert!(!is_dsd_file(Path::new("/music/track.flac")));
        assert!(!is_dsd_file(Path::new("/music/track.mp3")));
        assert!(!is_dsd_file(Path::new("/music/track.wav")));
    }

    #[test]
    fn test_dsd_amp_silence_is_near_zero() {
        // 0xAA = 10101010 = 4 ones out of 8 bits = 50% density
        // 0x55 = 01010101 = 4 ones out of 8 bits = 50% density
        // Silence in DSD is ~50% ones, which should map to near zero amplitude
        let amp = amp_from_ones(4, 8);
        assert!(amp < 0.01, "50% density should be near zero, got {}", amp);
        
        // Also test with larger sample
        let amp_large = amp_from_ones(512, 1024);
        assert!(amp_large < 0.01, "50% density (512/1024) should be near zero, got {}", amp_large);
    }

    #[test]
    fn test_dsd_amp_fullscale_is_one() {
        // 0xFF = all ones = 100% density -> amp = |2*1 - 1| = 1
        let amp_all_ones = amp_from_ones(8, 8);
        assert!((amp_all_ones - 1.0).abs() < 0.001, "100% density should be 1.0, got {}", amp_all_ones);
        
        // 0x00 = all zeros = 0% density -> amp = |2*0 - 1| = 1
        let amp_all_zeros = amp_from_ones(0, 8);
        assert!((amp_all_zeros - 1.0).abs() < 0.001, "0% density should be 1.0, got {}", amp_all_zeros);
    }

    #[test]
    fn test_dsd_amp_intermediate_values() {
        // 75% ones -> amp = |2*0.75 - 1| = |0.5| = 0.5
        let amp_75 = amp_from_ones(6, 8);
        assert!((amp_75 - 0.5).abs() < 0.01, "75% density should be 0.5, got {}", amp_75);
        
        // 25% ones -> amp = |2*0.25 - 1| = |-0.5| = 0.5
        let amp_25 = amp_from_ones(2, 8);
        assert!((amp_25 - 0.5).abs() < 0.01, "25% density should be 0.5, got {}", amp_25);
    }

    #[test]
    fn test_dsd_amp_zero_bits_handled() {
        let amp = amp_from_ones(0, 0);
        assert_eq!(amp, 0.0, "zero total bits should return 0");
    }

    #[test]
    fn test_popcount_interleaved_2ch() {
        // [L0=0xFF, R0=0x00, L1=0xFF, R1=0x00]
        // Left: 8+8=16 ones, Right: 0+0=0 ones
        let (left, right) = popcount_interleaved_2ch(&[0xFF, 0x00, 0xFF, 0x00]);
        assert_eq!(left, 16);
        assert_eq!(right, 0);
        
        // [L0=0xAA, R0=0x55, L1=0xAA, R1=0x55]
        // Left: 4+4=8 ones, Right: 4+4=8 ones
        let (left, right) = popcount_interleaved_2ch(&[0xAA, 0x55, 0xAA, 0x55]);
        assert_eq!(left, 8);
        assert_eq!(right, 8);
    }

    #[test]
    fn test_popcount_interleaved_empty() {
        let (left, right) = popcount_interleaved_2ch(&[]);
        assert_eq!(left, 0);
        assert_eq!(right, 0);
    }

    #[test]
    fn test_popcount_interleaved_odd_length() {
        // With odd length, last byte should be ignored (chunks_exact behavior)
        let (left, right) = popcount_interleaved_2ch(&[0xFF, 0x00, 0xFF]);
        assert_eq!(left, 8);
        assert_eq!(right, 0);
    }
}
