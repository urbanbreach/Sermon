
### 2026-01-27 Implementation Details
- Added `ui.bottombar.waveform_seekbar` setting to `effects.ts` and `AppearancePrefs.svelte`.
- Followed existing patterns for boolean toggles:
  - Store: `writable<boolean>(false)`
  - Setter: `store.set(val); await setSetting(KEY, val ? 'on' : 'off');`
  - Loader: `parseBool(val, false)`
  - Sync: `store.set(settings[KEY] === 'on')`
- Documented in `docs/settings.md` under new "Appearance & Layout" section.
## Backend Waveform Implementation - Tue Jan 27 14:55:47 EET 2026

### Key Patterns Used
- `WaveformCacheState` follows exact pattern from `ArtworkCacheState`: cache_dir + ParkingMutex
- Cache key uses blake3 hash of `waveform::track_id::size_bytes::mtime_ms`
- Binary cache format: `[version:1][duration_ms:8][bin_ms:4][peaks...]`
- Used `tauri::async_runtime::spawn_blocking` for heavy decode work

### Algorithm Details
- Bin size: 25ms (samples_per_bin = sample_rate * 25 / 1000)
- Max bins: 50000 (prevents unbounded output for very long tracks)
- Peaks normalized to u8 (0-255) using global max: `(peak / global_max) * 255`
- Mono amplitude computed as max(abs(sample)) across channels per frame

### Cache Format (version 1)
| Offset | Size | Field |
|--------|------|-------|
| 0 | 1 | format_version |
| 1 | 8 | duration_ms (u64 le) |
| 9 | 4 | bin_ms (u32 le) |
| 13 | N | peaks (u8 array) |

### Response Structure
```typescript
interface WaveformPeaksResponse {
  durationMs: number;  // Track duration in ms
  binMs: number;       // Time per bin (25ms)
  peaksBase64: string; // Base64-encoded u8 peaks array
  formatVersion: number; // Cache format version (1)
}
```

### Dependencies
- audio-engine::decode::AudioDecoder for Symphonia decoding
- library crate for DB access (track path resolution)
- blake3 for cache key hashing
- base64 for transport encoding

## Waveform Seekbar Color Fix
- Canvas 2D context cannot parse CSS `var()` functions.
- Always convert colors to explicit rgba/hex strings when drawing to canvas.
- Integrated waveform color setting in effects store for better customization.
