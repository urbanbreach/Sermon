# Backend Optimization Learnings

## 2026-01-29T20:12:55 Session Init
- Plan: backend-optimization (58 tasks across 9 major task groups)
- Scope: Backend only (src-tauri/, crates/), no UI changes
- TDD approach: tests first for code changes
- Frequent commits following repo style
- Windows tests via WSL invocation

## Codebase Structure
- `src-tauri/` - Tauri backend (commands/, state.rs, lib.rs audio loop)
- `crates/audio-engine/` - Audio decode, WASAPI output, playback state
- `crates/library/` - SQLite database, folder/track management, settings
- `crates/tags/` - Metadata extraction

## Key References from Plan
- Audio loop: src-tauri/src/lib.rs ~109-195, ~1116-1289
- Gapless decoder: crates/audio-engine/src/gapless_decoder.rs ~102-107, ~160-217
- Output/ring buffers: crates/audio-engine/src/output.rs ~626-699, ~804-929
- ASIO worker: crates/audio-engine/src/asio_worker.rs ~511-585, ~675-714
- DB: crates/library/src/db/mod.rs ~11-99, ~440-600
## ADR 0007: Compatibility and Performance Budgets
- Documented supported codecs and containers (Symphonia + DSD).
- Defined numerical performance budgets for latency, CPU, memory, and startup.
- Formalized realtime guardrails for audio callback safety.
- Established bit-perfect requirement for audiophile backends.
# Learnings - Backend Optimization & Hardening

## Error Handling Patterns
- Replaced `.expect()` and `.unwrap()` with `Result` propagation (`?`) in critical startup paths (directory resolution, database initialization).
- Replaced brittle `try_into().unwrap()` in DSD (DSF/DFF) parsing with `.map_err(|_| DsdError::InvalidDsf)?` to ensure recoverable errors when encountering malformed files.
- Used `thiserror` to define structured error types for the audio engine, improving error clarity and allowing better propagation to the frontend.

## Startup Robustness
- Improved Tauri `setup` hook to handle errors gracefully instead of panicking, allowing the application to report startup failures if critical resources (like the app data directory or database) are unavailable.
- Implemented an `ensure_migrations` flag in `LibraryState` to prevent redundant migration attempts during the app lifecycle, while still ensuring they run once at startup.

## Error Propagation & Diagnostics
- Introduced `DiagnosticsState` to track startup and playback timing metrics via atomic counters.
- Added a detailed telemetry system that emits `evt_audio_debug` and `evt_audio_telemetry` events, providing deep visibility into the signal path, bit-perfect status, and buffer stability.
- Standardized error codes (e.g., `REASON_BIT_DEPTH_UNKNOWN`, `REASON_SAMPLE_RATE_MISMATCH`) to allow the frontend to provide specific feedback to the user.

## 2026-01-29 Baseline Metrics Instrumentation

### Metrics Implementation
- `DiagnosticsState` already existed with atomic counters for startup, playback, and seek timing.
- Added `sysinfo` crate (already present in Cargo.toml) for process CPU% and RSS memory tracking.
- Extended `cmd_settings_export_diagnostics` to include new JSON fields:
  - `metrics.startup_ms`, `metrics.playback_start_ms`, `metrics.seek_ms`
  - `audio.underruns`
  - `process.cpu_pct`, `process.rss_bytes`

### Timing Hook Locations
- Startup: `run()` records `startup_start_ms` before tracing init; `sermon://first-interactive` listener records `startup_complete_ms`.
- Playback: `handle_playback_command` for `PlayNow`/`PlayNowWithQueue` records timestamps before `start_playback()` and after success.
- Seek: `handle_playback_command` for `Seek` records timestamps around the seek operation.

### Architecture Notes
- `DiagnosticsState` is managed as `Arc<DiagnosticsState>` in Tauri state for thread-safe access.
- Timing uses `std::time::SystemTime::now()` for milliseconds since epoch.
- The audio thread receives diagnostics Arc through `spawn_audio_thread` and `handle_playback_command`.
- Process metrics use `sysinfo::System::refresh_processes()` with specific PID for low overhead.

### Testing
- Existing tests in `state.rs` cover `DiagnosticsState` functionality (duration calculations, underrun counting, thread safety).
- All cargo tests pass (85 audio-engine, 7 library, 19 sermon_lib tests).

## Database Migration Optimization (2026-01-29)

### Problem
- `apply_migrations()` was called in every single Tauri command handler (39 call sites across 6 files)
- Caused unnecessary overhead on every API call
- `LibraryState` already had `migrations_applied: AtomicBool` and `ensure_migrations()` method but commands bypassed it

### Solution
1. **Removed all redundant `apply_migrations()` calls** from command handlers:
   - `library.rs` - Removed import and ~20 calls
   - `artwork.rs` - Removed import and 7 calls
   - `playback.rs` - Removed import and 4 calls
   - `waveform.rs` - Removed import and 1 call

2. **Migrations run once at startup** in `src-tauri/src/lib.rs` (lines 136-141)

3. **Added performance indexes** via migration `0007_browse_indexes.sql`:
   - `idx_tracks_is_missing` - For is_missing filtering (used in all browse queries)
   - `idx_tracks_album_browse` - Covering index for album browsing
   - `idx_tracks_artist_browse` - Covering index for artist browsing
   - `idx_tracks_title_sort`, `idx_tracks_artist_sort`, `idx_tracks_album_sort` - For tracks page sorting

### Key Insight
The existing `LibraryState` infrastructure was designed to gate migrations via `ensure_migrations()` but commands were calling `library::apply_migrations()` directly instead. The fix was simply removing all direct calls since migrations already run at startup.

### Committed
- Commit `4e56531` - "fix(backend): harden startup + DSD parsing errors" includes all these changes.

## Release Profile Optimization (2026-01-29)

### Binary Size Reduction
- Added `[profile.release]` to root `Cargo.toml`
- Settings: `lto = "thin"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`, `opt-level = 3`
- **BEFORE**: 23,933,440 bytes (22.83 MB)
- **AFTER**: 14,889,984 bytes (14.20 MB)
- **SAVINGS**: 9,043,456 bytes (8.63 MB, **37.8% reduction**)

### Build Time Impact
- Release build time increased from ~1m47s to ~2m46s (+55%)
- Trade-off acceptable: LTO and single codegen-unit improve binary but slow compilation

### Feature Trimming Decisions
- **Symphonia**: Kept `features = ["all"]` - ADR 0007 requires full codec support (FLAC, WAV, MP3, AAC, ALAC, Vorbis, AIFF, MKV/WebM)
- **rusqlite**: Kept `features = ["bundled"]` - Windows reliability + FTS5 availability

### Pre-existing Test Issue Found
- `test_in_memory_migration` expects `user_version = 7` but there are 8 migrations
- Migration 0008 (waveform_cache) was added without updating the test
- Not caused by release profile changes

## Cache Eviction and Memory Policy (2026-01-29)

### Implementation Summary
Implemented cache eviction policies and memory caps per ADR 0007 budget specifications.

### Waveform Cache (1GB cap)
- Added `waveform_cache_map` table via migration 0008 with: `cache_key`, `track_id`, `size_bytes`, `mtime_ms`, `last_access_ms`
- On cache read hit: updates `last_access_ms` for LRU tracking
- On cache write: checks size, evicts oldest entries if over cap, records entry in DB
- Skip caching entries > 256MB (25% of cap)

### Artwork Cache (256MB cap)
- Uses existing `artwork_cache_map_album` and `artwork_cache_map_track` tables with `selected_at` column
- On cache read hit: updates `selected_at` in appropriate mapping table
- On cache write: evicts oldest entries (sorted by `selected_at`) until under cap
- Skip caching entries > 64MB (25% of cap)
- Artwork eviction requires scanning filesystem for file sizes (not stored in DB)

### Gapless Preload Memory Policy (512MB file cap)
- Added file size check in `AudioPlayback::start_playback`
- Files > 512MB fallback to `AudioDecoder::open` (streaming mode) instead of `GaplessDecoder::new` (RAM load)
- Setting-aware: respects `player.load_to_memory` user preference

### Key Design Decisions
1. **Waveform uses DB tracking**: Stores size_bytes in table for efficient eviction without filesystem scans
2. **Artwork uses filesystem scan**: Existing schema didn't include size; scanning on eviction is acceptable for smaller cache
3. **Eviction is synchronous**: Runs before write to ensure cap is respected
4. **25% entry cap**: Prevents single large entries from dominating cache

### Files Modified
- `crates/library/migrations/0008_waveform_cache.sql` - New migration
- `crates/library/src/db/migrations.rs` - Register migration
- `src-tauri/src/commands/waveform.rs` - LRU eviction on read/write
- `src-tauri/src/commands/artwork.rs` - LRU eviction using selected_at
- `src-tauri/src/lib.rs` - File size check for gapless preload
- `crates/library/tests/migrations.rs` - Updated expected version to 8

### Test Results
All cargo tests pass (85 audio-engine, 24 library, 19 sermon_lib, 13 tags tests)

## 2026-01-29 Player Settings Propagation
- Propagated player settings (buffer size, load-to-memory, preload-next) over the playback command channel to keep the audio thread from opening the DB for those values.
- Cached settings in `AudioPlayback` and gated preload behavior on the `preload_next` flag.

## 2026-01-29 Gapless Preload I/O Off Audio Tick
- Added a preload worker thread with request/result channels so disk reads happen off the audio tick.
- Audio tick now sends preload requests non-blocking and applies bytes via `GaplessDecoder::preload_next_from_bytes`.
- Introduced `MemoryAudioSource::from_bytes` to build decoders from pre-read file bytes.

## 2026-01-30 ASIO ring buffer batching
- `ringbuf::traits::Producer::push_iter`/`push_slice` provide batch writes and reduce per-sample `try_push` overhead.
- `push_iter` returns the count written and stops when the buffer is full, preserving sample order.
