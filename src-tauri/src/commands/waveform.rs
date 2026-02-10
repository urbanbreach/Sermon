use audio_engine::decode::{AudioDecoder, DecodePacketRef};
use audio_engine::{DsdDecoder, DsdError};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use library::open_db;
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::State;
use tracing::{info, warn};

use crate::state::{LibraryState, WaveformCacheState};

const WAVEFORM_CACHE_CAP_BYTES: i64 = 1024 * 1024 * 1024;
const WAVEFORM_MAX_ENTRY_SIZE: i64 = WAVEFORM_CACHE_CAP_BYTES / 4;
const WAVEFORM_FORMAT_VERSION: u8 = 6;
const MIN_BIN_MS: u32 = 12;
const MAX_BIN_MS: u32 = 250;
const TARGET_BINS: u64 = 2400;
const MAX_BINS: usize = 50000;
const TARGET_FRAME_SAMPLES_PER_BIN: usize = 1024;

struct GeneratePeaksTiming {
    decode_open_ms: u64,
    decode_loop_ms: u64,
    normalize_ms: u64,
    analysis_stride: u32,
    decode_packet_stride: u16,
}

fn drain_remaining(decoder: &mut AudioDecoder, channels: usize, total_samples: &mut u64) {
    loop {
        match decoder.decode_next_ref() {
            Ok(Some(interleaved)) => {
                *total_samples += (interleaved.len() / channels) as u64;
            }
            Ok(None) | Err(_) => break,
        }
    }
}

#[inline]
fn select_bin_ms(duration_hint_ms: Option<u64>) -> u32 {
    match duration_hint_ms.filter(|d| *d > 0) {
        Some(duration_ms) => {
            let adaptive = duration_ms / TARGET_BINS;
            adaptive.clamp(MIN_BIN_MS as u64, MAX_BIN_MS as u64) as u32
        }
        None => MIN_BIN_MS,
    }
}

#[inline]
fn push_binned_peak(
    peaks: &mut Vec<f32>,
    pending_empty_bins: &mut usize,
    has_decoded_samples: bool,
    peak: f32,
) -> bool {
    if has_decoded_samples && *pending_empty_bins > 0 {
        for _ in 0..*pending_empty_bins {
            peaks.push(0.0);
            if peaks.len() >= MAX_BINS {
                *pending_empty_bins = 0;
                return true;
            }
        }

        *pending_empty_bins = 0;
    }

    if has_decoded_samples {
        peaks.push(peak);
    } else {
        *pending_empty_bins += 1;
    }

    peaks.len() + *pending_empty_bins >= MAX_BINS
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
    let input = format!(
        "waveform::v{}::{}::{}::{}",
        WAVEFORM_FORMAT_VERSION, track_id, size_bytes, mtime_ms
    );
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn update_waveform_cache_access(conn: &Connection, cache_key: &str) {
    let now = now_ms();
    let _ = conn.execute(
        "UPDATE waveform_cache_map SET last_access_ms = ?1 WHERE cache_key = ?2",
        rusqlite::params![now, cache_key],
    );
}

fn record_waveform_cache_entry(
    conn: &Connection,
    cache_key: &str,
    track_id: i64,
    size_bytes: i64,
    mtime_ms: i64,
) {
    let now = now_ms();
    let _ = conn.execute(
        "INSERT OR REPLACE INTO waveform_cache_map (cache_key, track_id, size_bytes, mtime_ms, last_access_ms) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![cache_key, track_id, size_bytes, mtime_ms, now],
    );
}

fn evict_waveform_cache_lru(conn: &Connection, cache_dir: &Path, target_free: i64) {
    let total_size: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(size_bytes), 0) FROM waveform_cache_map",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if total_size <= WAVEFORM_CACHE_CAP_BYTES - target_free {
        return;
    }

    let to_free = total_size - (WAVEFORM_CACHE_CAP_BYTES - target_free);
    let mut freed: i64 = 0;

    let mut stmt = conn
        .prepare("SELECT cache_key, size_bytes FROM waveform_cache_map ORDER BY last_access_ms ASC")
        .ok();

    if let Some(ref mut stmt) = stmt {
        let iter = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        });
        if let Ok(rows) = iter {
            for row in rows.flatten() {
                let (key, size) = row;
                let cache_path = cache_dir.join(&key);
                if fs::remove_file(&cache_path).is_ok() {
                    let _ = conn.execute(
                        "DELETE FROM waveform_cache_map WHERE cache_key = ?1",
                        [&key],
                    );
                    freed += size;
                    info!(evicted_key = %key, evicted_size = size, "waveform_cache_evict");
                    if freed >= to_free {
                        break;
                    }
                }
            }
        }
    }
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
fn generate_peaks(
    path: &Path,
    duration_hint_ms: Option<u64>,
) -> Result<(Vec<u8>, u64, u32, GeneratePeaksTiming), String> {
    if is_dsd_file(path) {
        generate_peaks_dsd(path, duration_hint_ms)
    } else {
        generate_peaks_pcm(path, duration_hint_ms)
    }
}

/// Generate peaks for DSD files (DSF/DFF) using bit-density amplitude calculation
fn generate_peaks_dsd(
    path: &Path,
    duration_hint_ms: Option<u64>,
) -> Result<(Vec<u8>, u64, u32, GeneratePeaksTiming), String> {
    let open_start = Instant::now();
    let mut decoder =
        DsdDecoder::open(path).map_err(|e: DsdError| format!("Failed to open DSD: {}", e))?;
    let decode_open_ms = open_start.elapsed().as_millis() as u64;

    let info = decoder.info();
    let sample_rate = info.sample_rate;
    let channels = info.channels;
    let total_dsd_samples = info.total_samples;

    if sample_rate == 0 || channels == 0 {
        return Err("Invalid DSD format: zero sample rate or channels".to_string());
    }

    // Keep a bounded number of analysis windows per bin to avoid excessive
    // per-byte work on high-rate DSD streams.
    const TARGET_WINDOWS_PER_BIN: usize = 96;

    let computed_duration_ms = (total_dsd_samples as f64 / sample_rate as f64 * 1000.0) as u64;
    let bin_ms = select_bin_ms(duration_hint_ms.or(Some(computed_duration_ms)));

    // DSD sample rate is bits per second per channel
    // Calculate bytes per bin per channel
    let bits_per_bin = (sample_rate as u64 * bin_ms as u64) / 1000;
    let bytes_per_bin_per_ch = ((bits_per_bin + 7) / 8) as usize; // Round up
    let bytes_per_bin_total = bytes_per_bin_per_ch * channels;

    let mut window_bytes_total = (bytes_per_bin_total / TARGET_WINDOWS_PER_BIN).max(channels);
    window_bytes_total = ((window_bytes_total / channels).max(1)) * channels;

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

            // Use bounded windows for peak detection
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
        peaks
            .iter()
            .map(|&p| ((p / global_max) * 255.0) as u8)
            .collect()
    } else {
        vec![0u8; peaks.len()]
    };
    let normalize_ms = normalize_start.elapsed().as_millis() as u64;

    let timing = GeneratePeaksTiming {
        decode_open_ms,
        decode_loop_ms,
        normalize_ms,
        analysis_stride: window_bytes_total.min(u32::MAX as usize) as u32,
        decode_packet_stride: 1,
    };

    Ok((normalized, computed_duration_ms, bin_ms, timing))
}

/// Generate peaks for PCM files (FLAC, WAV, MP3, etc.) using Symphonia
fn generate_peaks_pcm(
    path: &Path,
    duration_hint_ms: Option<u64>,
) -> Result<(Vec<u8>, u64, u32, GeneratePeaksTiming), String> {
    let open_start = Instant::now();
    let mut decoder =
        AudioDecoder::open(path).map_err(|e| format!("Failed to open audio: {}", e))?;
    let decode_open_ms = open_start.elapsed().as_millis() as u64;

    let sample_rate = decoder.sample_rate();
    let channels = decoder.channels();

    if sample_rate == 0 || channels == 0 {
        return Err("Invalid audio format: zero sample rate or channels".to_string());
    }

    let bin_ms = select_bin_ms(duration_hint_ms);

    let samples_per_bin = (sample_rate as f64 * bin_ms as f64 / 1000.0) as usize;

    if samples_per_bin == 0 {
        return Err("Invalid samples per bin calculation".to_string());
    }

    let frame_stride = (samples_per_bin / TARGET_FRAME_SAMPLES_PER_BIN).max(1);
    // Always decode every packet to preserve waveform shape consistency
    // across short and long tracks. Frame-level stride already controls cost.
    let packet_stride = 1usize;

    let mut peaks: Vec<f32> = Vec::with_capacity(MAX_BINS);
    let mut current_bin_max: f32 = 0.0;
    let mut samples_in_current_bin: usize = 0;
    let mut total_samples: u64 = 0;
    let mut reached_max_bins = false;
    let mut current_bin_has_decoded = false;
    let mut pending_empty_bins: usize = 0;
    let mut packet_index: usize = 0;

    let loop_start = Instant::now();
    loop {
        let decode_this_packet = packet_stride == 1 || packet_index % packet_stride == 0;
        packet_index += 1;

        let decoded_packet = match decoder.decode_next_ref_maybe(decode_this_packet) {
            Ok(Some(packet_ref)) => packet_ref,
            Ok(None) => break,
            Err(e) => return Err(format!("Decode error: {}", e)),
        };

        match decoded_packet {
            DecodePacketRef::Skipped { frame_count } => {
                if frame_count == 0 {
                    continue;
                }
                total_samples += frame_count as u64;

                samples_in_current_bin += frame_count;
                while samples_in_current_bin >= samples_per_bin {
                    if push_binned_peak(
                        &mut peaks,
                        &mut pending_empty_bins,
                        current_bin_has_decoded,
                        current_bin_max,
                    ) {
                        reached_max_bins = true;
                        break;
                    }

                    current_bin_max = 0.0;
                    current_bin_has_decoded = false;
                    samples_in_current_bin -= samples_per_bin;
                }

                if reached_max_bins {
                    break;
                }
            }
            DecodePacketRef::Decoded {
                frame_count,
                interleaved,
            } => {
                if frame_count == 0 || interleaved.is_empty() {
                    continue;
                }
                total_samples += frame_count as u64;

                match channels {
                    1 => {
                        let mut frame_idx = 0usize;
                        while frame_idx < frame_count {
                            let frame_max = interleaved[frame_idx].abs();
                            current_bin_max = current_bin_max.max(frame_max);
                            current_bin_has_decoded = true;
                            let advanced = frame_stride.min(frame_count - frame_idx);
                            samples_in_current_bin += advanced;

                            if samples_in_current_bin >= samples_per_bin {
                                if push_binned_peak(
                                    &mut peaks,
                                    &mut pending_empty_bins,
                                    current_bin_has_decoded,
                                    current_bin_max,
                                ) {
                                    reached_max_bins = true;
                                    break;
                                }

                                current_bin_max = 0.0;
                                current_bin_has_decoded = false;
                                samples_in_current_bin -= samples_per_bin;
                            }

                            frame_idx += advanced;
                        }
                    }
                    2 => {
                        let mut frame_idx = 0usize;
                        while frame_idx < frame_count {
                            let sample_idx = frame_idx * 2;
                            let frame_max = interleaved[sample_idx]
                                .abs()
                                .max(interleaved[sample_idx + 1].abs());
                            current_bin_max = current_bin_max.max(frame_max);
                            current_bin_has_decoded = true;
                            let advanced = frame_stride.min(frame_count - frame_idx);
                            samples_in_current_bin += advanced;

                            if samples_in_current_bin >= samples_per_bin {
                                if push_binned_peak(
                                    &mut peaks,
                                    &mut pending_empty_bins,
                                    current_bin_has_decoded,
                                    current_bin_max,
                                ) {
                                    reached_max_bins = true;
                                    break;
                                }

                                current_bin_max = 0.0;
                                current_bin_has_decoded = false;
                                samples_in_current_bin -= samples_per_bin;
                            }

                            frame_idx += advanced;
                        }
                    }
                    _ => {
                        let mut frame_idx = 0usize;
                        while frame_idx < frame_count {
                            let sample_idx = frame_idx * channels;
                            let frame = &interleaved[sample_idx..sample_idx + channels];
                            let frame_max: f32 =
                                frame.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
                            current_bin_max = current_bin_max.max(frame_max);
                            current_bin_has_decoded = true;
                            let advanced = frame_stride.min(frame_count - frame_idx);
                            samples_in_current_bin += advanced;

                            if samples_in_current_bin >= samples_per_bin {
                                if push_binned_peak(
                                    &mut peaks,
                                    &mut pending_empty_bins,
                                    current_bin_has_decoded,
                                    current_bin_max,
                                ) {
                                    reached_max_bins = true;
                                    break;
                                }

                                current_bin_max = 0.0;
                                current_bin_has_decoded = false;
                                samples_in_current_bin -= samples_per_bin;
                            }

                            frame_idx += advanced;
                        }
                    }
                }

                if reached_max_bins {
                    break;
                }
            }
        }

        if reached_max_bins {
            break;
        }
    }

    if reached_max_bins && duration_hint_ms.is_none() {
        drain_remaining(&mut decoder, channels, &mut total_samples);
    }

    if samples_in_current_bin > 0 && peaks.len() < MAX_BINS {
        let _ = push_binned_peak(
            &mut peaks,
            &mut pending_empty_bins,
            current_bin_has_decoded,
            current_bin_max,
        );
    }

    if pending_empty_bins > 0 && peaks.len() < MAX_BINS {
        for _ in 0..pending_empty_bins {
            peaks.push(0.0);
            if peaks.len() >= MAX_BINS {
                break;
            }
        }
    }
    let decode_loop_ms = loop_start.elapsed().as_millis() as u64;

    if peaks.is_empty() {
        return Err("No peaks computed - audio file may be empty".to_string());
    }

    let normalize_start = Instant::now();
    let global_max = peaks.iter().fold(0.0f32, |a, &b| a.max(b));
    let normalized: Vec<u8> = if global_max > 0.0 {
        peaks
            .iter()
            .map(|&p| ((p / global_max) * 255.0) as u8)
            .collect()
    } else {
        vec![0u8; peaks.len()]
    };
    let normalize_ms = normalize_start.elapsed().as_millis() as u64;

    let duration_ms = duration_hint_ms
        .unwrap_or_else(|| (total_samples as f64 / sample_rate as f64 * 1000.0) as u64);

    let timing = GeneratePeaksTiming {
        decode_open_ms,
        decode_loop_ms,
        normalize_ms,
        analysis_stride: frame_stride.min(u32::MAX as usize) as u32,
        decode_packet_stride: packet_stride.min(u16::MAX as usize) as u16,
    };

    Ok((normalized, duration_ms, bin_ms, timing))
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

    let track_info: Option<(String, i64, i64, Option<i64>)> = conn
        .query_row(
            "SELECT path, size_bytes, mtime_ms, duration_ms FROM tracks WHERE id = ?1",
            rusqlite::params![track_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let (track_path, size_bytes, mtime_ms, db_duration_ms) =
        track_info.ok_or_else(|| format!("Track not found: {}", track_id))?;

    let cache_key = compute_waveform_cache_key(track_id, size_bytes, mtime_ms);

    let cache_read_start = Instant::now();
    if !should_bypass {
        let _lock = waveform_state.lock.lock();
        let cache_path = waveform_state.cache_dir.join(&cache_key);

        if let Ok(cached_data) = fs::read(&cache_path) {
            if cached_data.len() >= 13 {
                let format_version = cached_data[0];
                if format_version == WAVEFORM_FORMAT_VERSION {
                    let duration_ms = u64::from_le_bytes(cached_data[1..9].try_into().unwrap());
                    let bin_ms = u32::from_le_bytes(cached_data[9..13].try_into().unwrap());
                    let peaks = &cached_data[13..];
                    let cache_read_ms = cache_read_start.elapsed().as_millis() as u64;
                    let total_ms = total_start.elapsed().as_millis() as u64;

                    update_waveform_cache_access(&conn, &cache_key);

                    info!(
                        track_id = track_id,
                        cache_read_ms = cache_read_ms,
                        total_ms = total_ms,
                        format_version = format_version,
                        bin_ms = bin_ms,
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
    }
    let cache_read_ms = cache_read_start.elapsed().as_millis() as u64;

    let duration_hint_ms = db_duration_ms.map(|d| d as u64);
    let path = track_path.clone();
    let (peaks, duration_ms, bin_ms, timing) = tauri::async_runtime::spawn_blocking(move || {
        generate_peaks(Path::new(&path), duration_hint_ms)
    })
    .await
    .map_err(|e| e.to_string())??;

    let final_duration_ms = db_duration_ms.map(|d| d as u64).unwrap_or(duration_ms);

    let cache_write_start = Instant::now();
    let format_version = WAVEFORM_FORMAT_VERSION;

    let mut cache_data = Vec::with_capacity(13 + peaks.len());
    cache_data.push(format_version);
    cache_data.extend_from_slice(&final_duration_ms.to_le_bytes());
    cache_data.extend_from_slice(&bin_ms.to_le_bytes());
    cache_data.extend_from_slice(&peaks);

    let entry_size = cache_data.len() as i64;
    let should_cache = entry_size <= WAVEFORM_MAX_ENTRY_SIZE;

    if should_cache {
        let _lock = waveform_state.lock.lock();

        evict_waveform_cache_lru(&conn, &waveform_state.cache_dir, entry_size);

        let cache_path = waveform_state.cache_dir.join(&cache_key);
        if let Err(e) = fs::write(&cache_path, &cache_data) {
            warn!(error = %e, "waveform_cache_write_failed");
        } else {
            record_waveform_cache_entry(&conn, &cache_key, track_id, entry_size, mtime_ms);
        }
    } else {
        warn!(
            entry_size = entry_size,
            max_size = WAVEFORM_MAX_ENTRY_SIZE,
            "waveform_cache_skip_large_entry"
        );
    }

    let cache_write_ms = cache_write_start.elapsed().as_millis() as u64;
    let total_ms = total_start.elapsed().as_millis() as u64;

    info!(
        track_id = track_id,
        cache_read_ms = cache_read_ms,
        decode_open_ms = timing.decode_open_ms,
        decode_loop_ms = timing.decode_loop_ms,
        normalize_ms = timing.normalize_ms,
        analysis_stride = timing.analysis_stride,
        decode_packet_stride = timing.decode_packet_stride,
        bin_ms = bin_ms,
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
    fn test_push_binned_peak_empty_bins_are_zero_filled() {
        let mut peaks = Vec::new();
        let mut pending_empty_bins = 0usize;

        assert!(!push_binned_peak(
            &mut peaks,
            &mut pending_empty_bins,
            false,
            0.75,
        ));
        assert!(peaks.is_empty());
        assert_eq!(pending_empty_bins, 1);

        assert!(!push_binned_peak(
            &mut peaks,
            &mut pending_empty_bins,
            true,
            0.5,
        ));
        assert_eq!(pending_empty_bins, 0);
        assert_eq!(peaks.len(), 2);
        assert!((peaks[0] - 0.0).abs() < f32::EPSILON);
        assert!((peaks[1] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_push_binned_peak_accumulates_multiple_empty_bins() {
        let mut peaks = Vec::new();
        let mut pending_empty_bins = 0usize;

        assert!(!push_binned_peak(
            &mut peaks,
            &mut pending_empty_bins,
            false,
            0.75,
        ));
        assert!(!push_binned_peak(
            &mut peaks,
            &mut pending_empty_bins,
            false,
            0.75,
        ));
        assert_eq!(pending_empty_bins, 2);

        assert!(!push_binned_peak(
            &mut peaks,
            &mut pending_empty_bins,
            true,
            0.25,
        ));
        assert_eq!(pending_empty_bins, 0);
        assert_eq!(peaks.len(), 3);
        assert!((peaks[0] - 0.0).abs() < f32::EPSILON);
        assert!((peaks[1] - 0.0).abs() < f32::EPSILON);
        assert!((peaks[2] - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_select_bin_ms_defaults_to_min() {
        assert_eq!(select_bin_ms(None), MIN_BIN_MS);
    }

    #[test]
    fn test_select_bin_ms_zero_hint_falls_back_to_min() {
        let selected = select_bin_ms(Some(0));
        assert_eq!(
            selected, MIN_BIN_MS,
            "zero duration hints must fall back to MIN_BIN_MS to avoid invalid bin widths"
        );
    }

    #[test]
    fn test_select_bin_ms_short_duration_clamps_to_min() {
        let selected = select_bin_ms(Some(1_000));
        assert_eq!(
            selected, MIN_BIN_MS,
            "short tracks should clamp to MIN_BIN_MS when adaptive bin size underflows"
        );
    }

    #[test]
    fn test_select_bin_ms_adapts_for_medium_tracks() {
        assert_eq!(select_bin_ms(Some(300_000)), 125);
    }

    #[test]
    fn test_select_bin_ms_caps_for_long_tracks() {
        assert_eq!(select_bin_ms(Some(7_200_000)), MAX_BIN_MS);
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
        assert!(
            amp_large < 0.01,
            "50% density (512/1024) should be near zero, got {}",
            amp_large
        );
    }

    #[test]
    fn test_dsd_amp_fullscale_is_one() {
        // 0xFF = all ones = 100% density -> amp = |2*1 - 1| = 1
        let amp_all_ones = amp_from_ones(8, 8);
        assert!(
            (amp_all_ones - 1.0).abs() < 0.001,
            "100% density should be 1.0, got {}",
            amp_all_ones
        );

        // 0x00 = all zeros = 0% density -> amp = |2*0 - 1| = 1
        let amp_all_zeros = amp_from_ones(0, 8);
        assert!(
            (amp_all_zeros - 1.0).abs() < 0.001,
            "0% density should be 1.0, got {}",
            amp_all_zeros
        );
    }

    #[test]
    fn test_dsd_amp_intermediate_values() {
        // 75% ones -> amp = |2*0.75 - 1| = |0.5| = 0.5
        let amp_75 = amp_from_ones(6, 8);
        assert!(
            (amp_75 - 0.5).abs() < 0.01,
            "75% density should be 0.5, got {}",
            amp_75
        );

        // 25% ones -> amp = |2*0.25 - 1| = |-0.5| = 0.5
        let amp_25 = amp_from_ones(2, 8);
        assert!(
            (amp_25 - 0.5).abs() < 0.01,
            "25% density should be 0.5, got {}",
            amp_25
        );
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
    fn test_popcount_interleaved_single_frame_counts_both_channels() {
        let (left, right) = popcount_interleaved_2ch(&[0b0000_0001, 0b1000_0000]);
        assert_eq!(
            left, 1,
            "single interleaved stereo frame should count left-channel ones"
        );
        assert_eq!(
            right, 1,
            "single interleaved stereo frame should count right-channel ones"
        );
    }

    #[test]
    fn test_popcount_interleaved_single_byte_is_ignored() {
        let (left, right) = popcount_interleaved_2ch(&[0xFF]);
        assert_eq!(
            left, 0,
            "incomplete stereo frame should not contribute to left-channel ones"
        );
        assert_eq!(
            right, 0,
            "incomplete stereo frame should not contribute to right-channel ones"
        );
    }

    #[test]
    fn test_push_binned_peak_reaches_capacity_with_decoded_peak() {
        let mut peaks = vec![0.25; MAX_BINS - 1];
        let mut pending_empty_bins = 0usize;

        let reached_limit = push_binned_peak(&mut peaks, &mut pending_empty_bins, true, 0.75);

        assert!(
            reached_limit,
            "reaching MAX_BINS with a decoded peak should signal capacity"
        );
        assert_eq!(
            peaks.len(), MAX_BINS,
            "decoded peak should be appended before reporting MAX_BINS"
        );
        assert_eq!(
            pending_empty_bins, 0,
            "pending empty bins should remain cleared for decoded samples"
        );
        assert!(
            (peaks[MAX_BINS - 1] - 0.75).abs() < f32::EPSILON,
            "last peak should preserve the decoded amplitude when capacity is reached"
        );
    }

    #[test]
    fn test_push_binned_peak_pending_only_can_trigger_capacity() {
        let mut peaks = vec![0.25; MAX_BINS - 1];
        let mut pending_empty_bins = 0usize;

        let reached_limit = push_binned_peak(&mut peaks, &mut pending_empty_bins, false, 0.9);

        assert!(
            reached_limit,
            "pending empty bins should still trigger MAX_BINS guard when no samples decode"
        );
        assert_eq!(
            peaks.len(),
            MAX_BINS - 1,
            "no decoded sample means no peak should be appended immediately"
        );
        assert_eq!(
            pending_empty_bins, 1,
            "single skipped bin should be tracked for later zero-fill"
        );
    }

    #[test]
    fn test_push_binned_peak_flush_stops_at_capacity_before_new_peak() {
        let mut peaks = vec![0.25; MAX_BINS - 2];
        let mut pending_empty_bins = 2usize;

        let reached_limit = push_binned_peak(&mut peaks, &mut pending_empty_bins, true, 0.9);

        assert!(
            reached_limit,
            "flushing pending empty bins to MAX_BINS should stop before adding a new decoded peak"
        );
        assert_eq!(
            peaks.len(), MAX_BINS,
            "pending zero bins should fill the remaining capacity"
        );
        assert_eq!(
            pending_empty_bins, 0,
            "pending bins should be cleared after flush even when capacity is reached"
        );
        assert!(
            (peaks[MAX_BINS - 1] - 0.0).abs() < f32::EPSILON,
            "last bin should be a zero-filled pending bin, not the decoded peak"
        );
    }

    #[test]
    fn test_popcount_interleaved_odd_length() {
        // With odd length, last byte should be ignored (chunks_exact behavior)
        let (left, right) = popcount_interleaved_2ch(&[0xFF, 0x00, 0xFF]);
        assert_eq!(left, 8);
        assert_eq!(right, 0);
    }
}
