# Waveform Seekbar Bottom Bar (MusicBee-Like)

## Context

### Original Request
Add an option to use a MusicBee-like waveform/spectrogram-style progress bar in Sermon’s bottom bar. It should retain Sermon’s liquid-glass visual identity, be draggable, and provide hover feedback showing where you’d seek (visual indicator + exact timestamp).

### Interview Summary (Decisions)
- Visualization: waveform/peaks (not a true frequency spectrogram).
- Layout (when enabled): single-row, MusicBee-like.
- Drag behavior: preview while dragging; seek happens on release.
- Scope: bottom bar only (do not change `ui/src/lib/views/NowPlayingView.svelte`).
- Testing: use Tauri MCP for UI interaction verification; Vitest optional if it meaningfully improves confidence; add Rust tests where appropriate.

### Existing Implementation References (Key Files)
- Seekbar + drag scaffolding: `ui/src/lib/components/BottomBar.svelte`
- Playback seek action + stores: `ui/src/lib/state/playback.ts`
- IPC seek command: `ui/src/lib/api/playback.ts` -> `src-tauri/src/commands/playback.rs:cmd_playback_seek`
- Track path availability: `ui/src/lib/state/playback.ts` (`currentTrackFull`) + `ui/src/lib/types/library.ts:TrackRow.path`
- UI settings pattern (unvalidated key/value): `ui/src/lib/state/effects.ts` (`cmd_settings_get`/`cmd_settings_set`) + CSS variables in `ui/src/app.css`
- Cache pattern precedent: `src-tauri/src/commands/artwork.rs` (cache dir, lock, base64 transport)
- Tauri MCP is available in debug builds: `src-tauri/src/lib.rs` (registers `tauri_plugin_mcp_bridge::init()`)

### Metis Review (Gaps Addressed)
- Define peaks data source and cache strategy: backend computes lazily on-demand and caches to disk (artwork-style).
- Define resolution and bounds: peaks are computed at a time-binned resolution with a max bins cap (prevents unbounded memory for long tracks).
- Guardrails: do not modify NowPlayingView; do not block UI while generating peaks; keep seek pipeline unchanged.

---

## Work Objectives

### Core Objective
Implement an optional waveform/peaks seekbar in the bottom bar with MusicBee-like single-row layout, hover timestamp preview, and drag-to-preview + seek-on-release, while preserving Sermon’s liquid-glass styling.

### Concrete Deliverables
- A persistent setting (toggle) to enable/disable waveform seekbar mode.
- Bottom bar layout switches to a single-row MusicBee-like layout when enabled.
- Waveform seekbar component renders peaks, current position, hover indicator, and tooltip timestamp.
- Backend command to generate/load cached peaks for the current track.
- Robust fallback: if peaks are unavailable/loading/fail, show the existing simple progress track.

### Definition of Done
- [x] Toggle persists and survives restart.
- [x] Hovering the waveform shows a vertical indicator + timestamp preview.
- [x] Dragging previews position; releasing seeks to the previewed time.
- [x] No regression in existing seek behavior when waveform mode is disabled.
- [x] Peaks generation is async (does not block UI) and cached per track.

### Must NOT Have (Guardrails)
- Must NOT change `ui/src/lib/views/NowPlayingView.svelte`.
- Must NOT change seek plumbing (`seek(ms)` -> `cmd_playback_seek`) beyond calling it.
- Must NOT require decoding audio in the UI thread.
- Must NOT introduce new backend preference categories (avoid `cmd_settings_*_category` for this feature).

---

## Verification Strategy

### Test Decision
- UI automated testing framework: not currently present in `ui/`.
- Primary verification: Tauri MCP (debug build) + manual checks.
- Secondary: Rust tests via `cargo test` for peaks generation/caching logic.
- Optional: Vitest for any pure TS logic added (only if it materially helps).

### Manual + Tauri MCP Verification (Required)

Use debug build so MCP bridge is available:
- Start app: `cargo tauri dev`
- Connect: use `tauri_driver_session` (host/port default unless configured).

Evidence to capture (for each key interaction):
- Tauri console logs: `tauri_read_logs(source="console")` filtered by relevant markers.
- Screenshots: `tauri_webview_screenshot(filePath=".sisyphus/evidence/<id>.jpeg")`.

---

## Task Flow

Backend peaks API + caching -> UI settings toggle -> Waveform seekbar component -> Bottom bar layout swap -> Interaction polish + edge cases -> Verification

---

## TODOs

### 0. Confirm UI Setting + Add Toggle (effects.ts Pattern)

**What to do**:
- Add a new unvalidated UI setting key using the existing `effects.ts` pattern:
  - Key: `ui.bottombar.waveform_seekbar`
  - Values: `'on'` / `'off'`
- Extend `ui/src/lib/state/effects.ts`:
  - Add KEY constant
  - Add writable store `bottomBarWaveformSeekbarEnabled`
  - Load it in `loadEffectsSettings()` (via `cmd_settings_get`)
  - Add setter `setBottomBarWaveformSeekbarEnabled(bool)` (via `cmd_settings_set`)
- Add the toggle to the Preferences UI:
  - Recommended location: `ui/src/lib/components/preferences/AppearancePrefs.svelte` under Layout.
  - Disable in mock mode (`SERMON_MOCK=1`).
- Document the new setting key in `docs/settings.md`.

**Must NOT do**:
- Do not use `saveCategorySetting('appearance', ...)` for this feature (backend category validation does not include appearance).

**Parallelizable**: YES

**References**:
- Effects settings pattern: `ui/src/lib/state/effects.ts`
- Preferences layout grouping: `ui/src/lib/components/preferences/AppearancePrefs.svelte`
- Settings storage doc: `docs/settings.md`

**Acceptance Criteria**:
- [x] Toggle exists in Preferences.
- [x] Toggling updates `cmd_settings_set` key `ui.bottombar.waveform_seekbar`.
- [x] Restart app and `loadEffectsSettings()` restores the toggle state.
- [x] `docs/settings.md` includes `ui.bottombar.waveform_seekbar` (type, default, description).

### 1. Add Backend Waveform Peaks Command (Lazy + Cached)

**Goal**: Provide compact peaks data for a track without blocking UI, and cache results.

**What to do**:
- Add a new cache directory (artwork-style): `waveform-cache` under app data dir.
  - Pattern: `src-tauri/src/lib.rs` creates dir + manages state.
- Add a new state holder similar to artwork cache:
  - Add `WaveformCacheState` in `src-tauri/src/state.rs` with `cache_dir` + `lock`.
  - Manage it in `src-tauri/src/lib.rs` setup.
- Add a new backend command that takes `track_id` and returns peaks:
  - Recommended signature (avoid passing filesystem paths from UI):
    - `cmd_waveform_get_peaks(track_id: i64) -> { durationMs, binMs, peaksBase64, formatVersion }`
- Resolve track path + duration from DB (`tracks` table) in the backend.
  - If file missing: return a structured error (UI falls back to normal progress bar).
- Compute peaks using Symphonia decode loop:
  - Use `crates/audio-engine/src/decode.rs:AudioDecoder` as the decode primitive.
  - Convert to mono amplitude (max of abs across channels per frame).
  - Bin by time (default bin size: 25ms) with max bins cap (e.g., 50k) to bound memory.
  - Normalize peaks to `u8` 0..255 for compact transfer.
- Caching:
  - Cache key includes: track_id + file size + file mtime + binMs (or derived binMs) + formatVersion.
  - Cache file path: `<app_data>/waveform-cache/<cache_key>`.
  - Store raw bytes (u8 peaks array) to cache file.
  - Return base64 from cache (pattern used in `src-tauri/src/commands/artwork.rs:cmd_artwork_get_bytes`).
- Failure modes:
  - On decode error: log + return error; UI uses fallback.
  - For DSD/DoP: either (a) skip peaks and return error, or (b) return a deterministic placeholder. Default: skip and fall back.

**Must NOT do**:
- Do not block the main thread; use `tauri::async_runtime::spawn_blocking` for heavy work.
- Do not emit peaks continuously from the audio thread; this is a static seekbar visualization.

**Parallelizable**: YES

**References**:
- Command registration + setup patterns: `src-tauri/src/lib.rs`
- Existing playback commands: `src-tauri/src/commands/playback.rs`
- Command registration (invoke handler): `src-tauri/src/lib.rs`
- Artwork cache pattern + base64 transport: `src-tauri/src/commands/artwork.rs`
- Symphonia decoding loop: `crates/audio-engine/src/decode.rs`
- Track validation + missing marking pattern: `src-tauri/src/commands/playback.rs:cmd_playback_start`

**Acceptance Criteria**:
- [x] New command returns peaks for a normal PCM track (MP3/WAV/FLAC).
- [x] Second call for same track returns from cache (no re-decode; log indicates cache hit).
- [x] Long track does not produce unbounded output (max bins enforced).
- [x] `cargo test` includes a new test for peaks binning/normalization.

### 2. Add UI API Wrapper + Peaks State

**What to do**:
- Add a UI API function: `ui/src/lib/api/playback.ts` (or a new `ui/src/lib/api/waveform.ts`) to invoke `cmd_waveform_get_peaks`.
- Add a UI store for peaks state keyed by track id:
  - Suggested location: `ui/src/lib/state/playback.ts` or new `ui/src/lib/state/waveform.ts`.
  - Store: `{ status: 'idle'|'loading'|'ready'|'error', trackId, durationMs, binMs, peaksU8 }`.
- On `currentTrack` change in bottom bar, request peaks async.
- In snapshot/mock mode (`SERMON_MOCK=1`), generate deterministic fake peaks locally (so snapshot runs do not depend on backend waveform computation).

**Parallelizable**: YES

**References**:
- Existing Tauri invoke wrapper patterns: `ui/src/lib/api/playback.ts`
- Playback stores + mock/snapshot mode: `ui/src/lib/state/playback.ts`
- Snapshot docs: `artifacts/README.md`

**Acceptance Criteria**:
- [x] In real mode, peaks load asynchronously and update store.
- [x] In snapshot mode, peaks appear deterministically without backend calls.

### 3. Build Waveform Seekbar Component (Canvas)

**What to do**:
- Create a dedicated component (suggested): `ui/src/lib/components/WaveformSeekbar.svelte`.
- Render waveform using `<canvas>` with liquid-glass-friendly styling:
  - Base waveform color: `var(--text-tertiary)` / `var(--surface-1)`.
  - Played portion overlay: use Sermon accent (`--theme-accent-*`) similar to `BottomBar.svelte` gradient.
  - Hover indicator: 1px vertical line + small timestamp label.
- Interactions:
  - Pointer move: compute hover progress from element bounds and show timestamp.
  - Pointer down: enter dragging mode; preview position updates visually.
  - Pointer up: call `seek(previewMs)` and exit dragging.
  - Cancel drag if track changes.
  - Debounce redraw on resize.
- Accessibility:
  - Provide `aria-label` for seekbar.
  - Provide keyboard support as a follow-up (optional). Minimum: keep click/drag.

**Must NOT do**:
- Do not issue seek repeatedly on move (seek-on-release only).

**Parallelizable**: YES

**References**:
- Existing time formatting: `ui/src/lib/components/BottomBar.svelte:formatTime()`
- Existing drag seek pattern: `ui/src/lib/components/BottomBar.svelte` (mouse down/move/up)
- CSS tokens for bar sizing: `ui/src/app.css` (`--layout-player-height`, `--layout-bottom-bar-progress-height`)

**Acceptance Criteria**:
- [x] Hover shows a clear indicator and timestamp preview.
- [x] Drag previews position; on release, audio seeks.
- [x] If peaks are missing/loading/error, component shows fallback (existing track bar style).

### 4. Update Bottom Bar Layout (Single-Row Mode)

**What to do**:
- Update `ui/src/lib/components/BottomBar.svelte` to conditionally render:
  - Default mode: current 2-row layout unchanged.
  - Waveform mode: single-row layout similar to MusicBee:
    - Left cluster: previous / play-pause / next (remove shuffle/repeat in this mode).
    - Center: `WaveformSeekbar`.
    - Right cluster: time labels + volume.
    - Keep a compact now-playing identity element (artwork + title/artist) as fits the row.
- Ensure the bar remains fixed, same height token unless proven insufficient; allow slight height bump only when waveform mode is enabled.

**Must NOT do**:
- Do not remove shuffle/repeat globally unless requested; remove only in waveform layout.

**Parallelizable**: NO (depends on 0 and 3)

**References**:
- Existing bottom bar structure + styling: `ui/src/lib/components/BottomBar.svelte`

**Acceptance Criteria**:
- [x] Toggle switches between layouts without breaking playback controls.
- [x] Waveform layout matches MusicBee-like single-row intent while retaining Sermon glass styling.

### 5. Performance + Edge Case Polish

**What to do**:
- Ensure peaks generation does not block UI:
  - Show fallback while loading.
  - Consider caching in memory for the current track for instant redraw.
- Handle edge cases:
  - Very short tracks.
  - Very long tracks (bins cap).
  - Track changes mid-hover/drag.
  - File missing/corrupt.
  - Window resize.

**Parallelizable**: NO (follows 1-4)

**Acceptance Criteria**:
- [x] No crashes on edge cases; fallback always available.
- [x] Waveform stays responsive during playback.

### 6. Verification Pass (Tauri MCP + Rust)

**What to do**:
- Rust:
  - Run `cargo test`.
  - Add/extend tests for binning + normalization + cache key stability.
- UI via Tauri MCP:
  - Connect to running app (debug).
  - Toggle waveform mode.
  - Hover waveform (verify indicator + timestamp).
  - Drag and release (verify seek).
  - Capture screenshots/logs as evidence.

**Parallelizable**: NO

**Acceptance Criteria**:
- [x] `cargo test` passes.
- [x] MCP-driven verification steps succeed and evidence saved to `.sisyphus/evidence/`.

---

## Commit Strategy

- Commit 1: Backend waveform command + cache state + tests
- Commit 2: UI effects setting + Preferences toggle
- Commit 3: WaveformSeekbar component
- Commit 4: BottomBar layout switch + polish

---

## Success Criteria

- Waveform seekbar mode looks MusicBee-like, but clearly Sermon (liquid glass tokens, accent usage).
- Hover timestamp + indicator is accurate.
- Drag previews and seek-on-release works reliably.
- Peaks generation is async and cached.
