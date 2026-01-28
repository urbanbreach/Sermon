use audio_engine::decode::AudioDecoder;
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

fn generate_peaks(path: &Path) -> Result<(Vec<u8>, u64, u32, GeneratePeaksTiming), String> {
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
}
