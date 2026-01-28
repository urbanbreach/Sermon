use audio_engine::decode::AudioDecoder;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use library::{apply_migrations, open_db};
use rusqlite::OptionalExtension;
use serde::Serialize;
use std::fs;
use std::path::Path;
use tauri::State;
use tracing::info;

use crate::state::{LibraryState, WaveformCacheState};

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

/// Generate waveform peaks from audio file
fn generate_peaks(path: &Path) -> Result<(Vec<u8>, u64, u32), String> {
    let mut decoder = AudioDecoder::open(path).map_err(|e| format!("Failed to open audio: {}", e))?;
    
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
    
    loop {
        let samples = match decoder.decode_next() {
            Ok(Some(samples)) => samples,
            Ok(None) => break,
            Err(e) => return Err(format!("Decode error: {}", e)),
        };
        
        if samples.is_empty() {
            continue;
        }
        
        for frame in samples.chunks_exact(channels) {
            let frame_max: f32 = frame.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
            
            current_bin_max = current_bin_max.max(frame_max);
            samples_in_current_bin += 1;
            total_samples += 1;
            
            if samples_in_current_bin >= samples_per_bin {
                peaks.push(current_bin_max);
                current_bin_max = 0.0;
                samples_in_current_bin = 0;
                
                if peaks.len() >= MAX_BINS {
                    loop {
                        match decoder.decode_next() {
                            Ok(Some(s)) => {
                                total_samples += (s.len() / channels) as u64;
                            }
                            Ok(None) => break,
                            Err(_) => break,
                        }
                    }
                    break;
                }
            }
        }
    }
    
    if samples_in_current_bin > 0 && peaks.len() < MAX_BINS {
        peaks.push(current_bin_max);
    }
    
    if peaks.is_empty() {
        return Err("No peaks computed - audio file may be empty".to_string());
    }
    
    let global_max = peaks.iter().fold(0.0f32, |a, &b| a.max(b));
    let normalized: Vec<u8> = if global_max > 0.0 {
        peaks.iter().map(|&p| ((p / global_max) * 255.0) as u8).collect()
    } else {
        vec![0u8; peaks.len()]
    };
    
    let duration_ms = (total_samples as f64 / sample_rate as f64 * 1000.0) as u64;
    
    Ok((normalized, duration_ms, BIN_MS))
}

/// Get or generate waveform peaks for a track
#[tauri::command]
pub async fn cmd_waveform_get_peaks(
    track_id: i64,
    library_state: State<'_, LibraryState>,
    waveform_state: State<'_, WaveformCacheState>,
) -> Result<WaveformPeaksResponse, String> {
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
    
    {
        let _lock = waveform_state.lock.lock();
        let cache_path = waveform_state.cache_dir.join(&cache_key);
        
        if cache_path.exists() {
            let cached_data = fs::read(&cache_path)
                .map_err(|e| format!("Failed to read cache file: {}", e))?;
            
            if cached_data.len() >= 13 {
                // Parse cached format: [version:1][duration_ms:8][bin_ms:4][peaks...]
                let format_version = cached_data[0];
                let duration_ms = u64::from_le_bytes(cached_data[1..9].try_into().unwrap());
                let bin_ms = u32::from_le_bytes(cached_data[9..13].try_into().unwrap());
                let peaks = &cached_data[13..];
                
                info!(
                    track_id = track_id,
                    cache_key = %cache_key,
                    peaks_count = peaks.len(),
                    "waveform_cache_hit"
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
    
    info!(track_id = track_id, cache_key = %cache_key, "waveform_cache_miss");
    
    let path = track_path.clone();
    let (peaks, duration_ms, bin_ms) = tauri::async_runtime::spawn_blocking(move || {
        generate_peaks(Path::new(&path))
    })
    .await
    .map_err(|e| e.to_string())??;
    
    let final_duration_ms = db_duration_ms.map(|d| d as u64).unwrap_or(duration_ms);
    
    let format_version: u8 = 1;
    {
        let _lock = waveform_state.lock.lock();
        let cache_path = waveform_state.cache_dir.join(&cache_key);
        
        // Build cache format: [version:1][duration_ms:8][bin_ms:4][peaks...]
        let mut cache_data = Vec::with_capacity(13 + peaks.len());
        cache_data.push(format_version);
        cache_data.extend_from_slice(&final_duration_ms.to_le_bytes());
        cache_data.extend_from_slice(&bin_ms.to_le_bytes());
        cache_data.extend_from_slice(&peaks);
        
        fs::write(&cache_path, &cache_data)
            .map_err(|e| format!("Failed to write cache file: {}", e))?;
        
        info!(
            track_id = track_id,
            cache_key = %cache_key,
            peaks_count = peaks.len(),
            duration_ms = final_duration_ms,
            "waveform_cache_write"
        );
    }
    
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
