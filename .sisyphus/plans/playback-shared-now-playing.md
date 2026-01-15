# Milestone 02 - playback-shared-now-playing

## Goal

- Deliver the first end-to-end experience: **scan → click track → hear audio**, using **WASAPI Shared** as the initial output backend (non-bit-perfect fallback path).
- Implement a minimal audio engine architecture that can later swap in WASAPI Exclusive and ASIO.

## Scope (In)

- Audio decode:
  - Use Symphonia for demux/decode of common formats (lossless + lossy). Symphonia supports multiple codecs/containers including FLAC/WAV/MP3/AAC/ALAC/Vorbis and common containers.
  - Verified formats for this milestone: FLAC, WAV, MP3. Best-effort decode for AAC/M4A, ALAC, Vorbis (errors surface, no crash).
- Audio output:
  - WASAPI Shared mode output via `wasapi` crate (simple path first).
  - Device selection: default output device + ability to choose another.
  - Device preference persisted in SQLite settings table; fallback to default if saved device missing.
- Playback controls:
  - Play/pause/stop
  - Seek (basic)
  - Next/previous (queue-based)
  - Queue: “Play Now” and “Add to Queue”
- UI:
  - Now Playing bar becomes functional
  - Minimal Now Playing screen:
    - album art placeholder (real art later)
    - track details
    - timeline scrubber
    - queue panel (simple)
  - Audio Debug overlay (Shared mode, device name/ID, stream format, decode format)
  - Volume control (0.0–1.0) with persistence
- Playback state updates:
  - Backend → UI events for playback state, now playing, position_ms (~250ms), queue changes, device changes, errors

## Non-scope (Out)

- WASAPI Exclusive (milestone 03).
- Bit-perfect validation (milestone 03).
- DSD (milestone 08).
- DSP features (off-scope unless explicitly opt-in later).
- Gapless playback guarantees (explicitly deferred).
- Resampler/ReplayGain/normalization beyond minimal volume scalar.

## Prerequisites/Dependencies

- 02: Milestone 01 — library-db-scan

## Key Decisions

- **Decode: Symphonia (pure Rust)**
  - Why: broad common codec/container support, avoids bundling ffmpeg/libav; keeps runtime dependencies minimal.
  - Alternative: ffmpeg → heavy binary distribution, larger surface area.
- **Output (for this milestone): WASAPI Shared**
  - Why: easiest functional baseline and broad compatibility; explicitly labeled as non-bit-perfect fallback.
- **Output API wrapper: wasapi crate**
  - Why: provides safe Rust wrappers and supports shared/exclusive, event-driven/polling, etc.
- **Canonical crate path**: `crates/audio-engine` (package name `audio-engine`); exclude `crates/audio` from workspace (via `exclude`) and leave a deprecation stub only.
- **Queue scope**: read-only “Up Next” UI; only Play Now (clear + set) and Add to Queue (append) mutations.
- **Queue persistence**: in-memory only for M02.
- **Gapless decision (T2)**: explicitly defer in M02; design interfaces to allow prebuffering later.
- **Bit-perfect indicator (T8.3)**: defer; Audio Debug overlay only (no “bit-perfect” claim).
- **Settings persistence**: SQLite `settings` table with `audio.volume` and `audio.device.preference` keys.
- **Device identifier persistence**: store the WASAPI `Device::get_id()` string; use `default` sentinel when following system default.
- **Device change handling**:
  - Default device change/unplug → attempt one controlled recovery; resume if possible, otherwise pause + surface error.
  - Specific device removal → stop + surface error; offer “Switch to Default”.
- **Event-driven UI**: backend emits `evt_*` events; avoid UI polling.

## Deliverables

- `crates/audio-engine/` (core playback state machine + decode + output abstraction).
- Tauri commands to:
  - start playback (track id)
  - pause/resume/seek
  - queue management
  - list/select output device
- UI:
  - Now Playing view + queue panel (functional)
- UI vision:
  - `/artifacts/ui/02-playback-shared-now-playing/now-playing-idle.png`
  - `/artifacts/ui/02-playback-shared-now-playing/now-playing-playing.png`
  - `/artifacts/ui/02-playback-shared-now-playing/queue.png`
  - `/artifacts/ui/02-playback-shared-now-playing/REVIEW.md`
- ADR:
  - `/docs/adr/0005-audio-decode-symphonia.md`
  - `/docs/adr/0006-audio-output-wasapi.md`
- Risk register update.
- SQLite settings table for audio volume/device preference.

### Audio Engine API Contract

- **Module boundaries (suggested)**: `engine.rs` (public API), `queue.rs`, `decode.rs`, `output.rs`, `device.rs`, `events.rs`, `types.rs`.
- **Public API surface** (Rust): `play_now(track)`, `add_to_queue(track)`, `pause()`, `resume()`, `stop()`, `seek(ms)`, `next()`, `previous()`, `set_volume(f32)`, `set_device(device_id|default)`, `list_devices()`, `current_state()`.
- **Threading model**: single audio thread owns decode/output loop; Tauri commands send `PlaybackCommand` via channel; engine emits `PlaybackEvent` via callback to Tauri emitter; queue stored inside engine (e.g., `VecDeque`).

### Work Plan Tasks

1. **Create `crates/audio-engine` and deprecate `crates/audio`**
   - What to do:
     - Add new crate `crates/audio-engine` with module layout for engine, queue, device, and decode.
     - Update root `Cargo.toml` workspace to `exclude = ["crates/audio"]` and leave `crates/audio` as deprecation-only stub.
   - References:
     - `Cargo.toml:1` — workspace member pattern to update.
     - `crates/audio/Cargo.toml:1` — placeholder crate to deprecate.
     - `crates/audio/src/lib.rs:1` — placeholder content to replace with deprecation notice.
   - Acceptance Criteria:
     - [ ] `crates/audio-engine/Cargo.toml` exists with required deps (symphonia, wasapi, tracing, uuid/ulid).
     - [ ] Root `Cargo.toml` excludes `crates/audio` from workspace members.
     - [ ] Workspace builds `cargo test -p audio-engine` without referencing `crates/audio`.
     - [ ] `crates/audio/src/lib.rs` contains a deprecation notice and no functional exports.

2. **Add SQLite settings table + accessors**
   - What to do:
     - Create migration `crates/library/migrations/0002_settings.sql` with `settings(key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at INTEGER)`.
     - Update `crates/library/src/db/migrations.rs` to apply the new migration and bump `user_version`.
     - Add get/set helpers in `crates/library/src/db/mod.rs` (or new module) and export via `crates/library/src/lib.rs`.
     - Add `get_track_by_id` helper to resolve track id → path/metadata for playback start.
     - Use keys `audio.volume` (float 0.0–1.0) and `audio.device.preference` (string: `default` or device id).
   - References:
     - `crates/library/migrations/0001_init.sql:4` — schema style + user_version pattern.
     - `crates/library/src/db/migrations.rs:5` — migration application flow.
     - `crates/library/src/db/mod.rs:8` — DB helper patterns (`open_db`, queries).
     - `crates/library/src/db/mod.rs:52` — query mapping pattern for track retrieval.
     - `crates/library/src/models.rs:14` — `TrackRow` fields for playback metadata.
     - `crates/library/tests/migrations.rs:4` — migration test conventions.
   - Acceptance Criteria:
     - [ ] Migration idempotency test updated to assert `settings` table exists and `user_version = 2`.
     - [ ] Get/set helpers return defaults when missing (`audio.volume=1.0`, `audio.device.preference=default`).
     - [ ] `get_track_by_id` returns `TrackRow` with `path` and metadata; missing id yields a clear error.

3. **Implement audio-engine state machine + queue + time accounting**
   - What to do:
     - Define core state: `PlaybackState` (Stopped/Playing/Paused), `play_id`, `played_ms`, `position_ms`.
     - Apply time rules: `played_ms` increments only while Playing; seek does not increment by itself; new play_id on Play Now or track change.
     - Implement queue operations: Play Now (clear + set + start), Add to Queue (append), Next/Previous.
     - Define Next/Previous rules: Next advances to next track or stops at end; Previous restarts current track if position_ms > 3s, otherwise moves to previous; Next/Previous while paused should start playback.
     - Implement software volume scalar (0.0–1.0) applied before output.
   - References:
     - `crates/library/src/models.rs:14` — `TrackRow` metadata (duration/sample_rate/bit_depth).
     - `prompts/Milestone 02 - playback-shared-now-playing.md:96` — T1 play session/time accounting requirement.
   - Acceptance Criteria:
     - [ ] Unit tests in `crates/audio-engine` cover play/pause/seek, queue ops, play_id/played_ms rules.
     - [ ] Tests cover Next/Previous edge cases (start/end of queue, paused behavior).
     - [ ] `cargo test -p audio-engine` passes with time-accounting tests.

4. **Implement Symphonia decode + WASAPI Shared output + device handling**
   - What to do:
     - Decode FLAC/WAV/MP3 using Symphonia; best-effort AAC/ALAC/Vorbis with error reporting.
     - Convert decoded audio to interleaved `f32` via `SampleBuffer` for consistent processing.
     - Maintain a bounded ring buffer (~200–500ms) between decode and render; on underrun, write silence and emit warning.
     - Use `wasapi` Shared mode (`StreamMode::EventsShared` preferred) and device enumeration.
     - Use WASAPI mix format; apply sample format conversion to match output format (no manual resampling; rely on shared-mode autoconvert).
     - Channel policy: if output has 2 channels and decode is mono, duplicate; if decode has more channels, drop extras.
     - Persist `Device::get_id()` string to `audio.device.preference`; compare against enumerated IDs on startup.
     - Handle device invalidation (`AUDCLNT_E_DEVICE_INVALIDATED`) by stopping/releasing and re-opening.
     - Apply volume scalar to samples prior to rendering.
   - References:
     - Symphonia docs: https://docs.rs/symphonia — probe/decoder pattern.
     - Wasapi docs: https://docs.rs/wasapi — shared mode init and device collection.
     - MS device recovery: https://learn.microsoft.com/en-us/windows/win32/coreaudio/recovering-from-an-invalid-device-error
   - Acceptance Criteria:
     - [ ] Playback succeeds for FLAC/WAV/MP3 files from library paths.
     - [ ] Decoder errors stop playback and emit error event without crashing.
     - [ ] Switching default device restarts stream once; specific device removal stops with error.
     - [ ] Stored `audio.device.preference` matches a `Device::get_id()` value or falls back to `default`.

5. **Wire Tauri commands + event emission**
   - What to do:
     - Create `src-tauri/src/commands/playback.rs` with commands: start, pause, resume, stop, seek, queue play-now/add, list devices, set device, get/set volume.
     - Register playback commands in `src-tauri/src/commands/mod.rs` and `src-tauri/src/lib.rs` (`tauri::generate_handler!`).
     - Add `AudioState` to `src-tauri/src/state.rs` with explicit ownership (e.g., `AudioState { engine: Arc<Mutex<AudioEngine>>, command_tx: mpsc::Sender<PlaybackCommand>, settings_cache: AudioSettings }`) and manage in `src-tauri/src/lib.rs`.
     - `cmd_playback_start` resolves track id → path/metadata using `library::db::get_track_by_id` before starting decode.
     - Define explicit event payload contracts (Rust structs + TS interfaces):
       - `evt_playback_state`: `{ state: "playing"|"paused"|"stopped", play_id: string|null, track_id: number|null }`
       - `evt_now_playing`: `{ play_id: string, track: { id, title, artist, album, duration_ms, sample_rate, bit_depth, channels, codec, container }, position_ms: number }`
       - `evt_playback_position`: `{ play_id: string, position_ms: number, played_ms: number, duration_ms: number }`
       - `evt_queue_changed`: `{ play_id: string|null, current_index: number|null, queue: [{ track_id, title, artist, album, duration_ms }] }`
       - `evt_device_changed`: `{ device_id: string, device_name: string, is_default: boolean }`
       - `evt_audio_debug`: `{ output_mode: "shared", device_id, device_name, output_format: { sample_rate, bit_depth, channels }, decode_format: { sample_rate, bit_depth, channels, codec, container } }`
       - `evt_playback_error`: `{ code: string, message: string, track_id?: number, recoverable: boolean, action?: "switch_to_default" }`
     - Emit events on state changes; throttle `evt_playback_position` to ~250ms.
     - Support recovery action by allowing UI to call `cmd_output_set_device("default")` when `evt_playback_error.action == "switch_to_default"`.
   - References:
     - `src-tauri/src/commands/library.rs:47` — `#[tauri::command]` patterns.
     - `src-tauri/src/commands/mod.rs:1` — command module export pattern.
     - `src-tauri/src/lib.rs:63` — state management and command registration.
     - `src-tauri/src/state.rs:4` — state struct pattern.
     - `docs/adr/0002-repo-architecture.md:18` — command/event naming convention.
   - Acceptance Criteria:
     - [ ] All commands registered in `src-tauri/src/commands/mod.rs` and `src-tauri/src/lib.rs`.
     - [ ] `cmd_playback_start` resolves track id via DB; missing track emits error event.
     - [ ] Event payload structs + TS interfaces match the contracts above.
     - [ ] Events emitted on state changes and received by UI store.

6. **Update UI playback store + views**
   - What to do:
     - Add `ui/src/lib/api/playback.ts` (invoke pattern from `ui/src/lib/api/library.ts`).
     - Add `ui/src/lib/types/playback.ts` mirroring event payloads (pattern from `ui/src/lib/types/library.ts`).
     - Add `ui/src/lib/state/playback.ts` store with event listeners (`@tauri-apps/api/event`).
     - Initialize playback store on app start (`App.svelte` onMount) so listeners are active regardless of current view.
     - Update `TracksView` to expose Play Now / Add to Queue actions (row action buttons; double-click row triggers Play Now).
     - Update `BottomBar` to show track details, play/pause toggle, prev/next buttons, volume slider.
     - Update `NowPlayingView` with timeline scrubber (drag → seek), queue panel (highlight current), Audio Debug overlay.
     - Add a lightweight error banner/toast (BottomBar or NowPlayingView) with “Switch to Default” action when `evt_playback_error.action == "switch_to_default"`.
     - Update `SettingsView` to list output devices and persist selection.
   - References:
     - `ui/src/lib/api/library.ts:1` — invoke pattern.
     - `ui/src/lib/types/library.ts:1` — TS interface style.
     - `ui/src/lib/state/library.ts:22` — event listener pattern.
     - `ui/src/App.svelte:2` — global onMount location.
     - `ui/src/lib/views/TracksView.svelte:2` — current `initLibrary` call site.
     - `ui/src/lib/views/TracksView.svelte:31` — track list layout.
     - `ui/src/lib/components/BottomBar.svelte:9` — playback bar layout.
     - `ui/src/lib/views/NowPlayingView.svelte:9` — Now Playing view.
     - `ui/src/lib/views/SettingsView.svelte:88` — audio settings section placeholder.
   - Acceptance Criteria:
     - [ ] Playback store initializes on app start and subscribes to events.
     - [ ] Play Now / Add to Queue UI actions invoke commands and update queue state.
     - [ ] UI can start playback from Tracks list and reflects state changes from events.
     - [ ] Volume slider updates `audio.volume` and persists across restart.
     - [ ] Device dropdown reflects backend list and updates preference.
     - [ ] Audio Debug overlay shows Shared mode + device + formats.
     - [ ] “Switch to Default” action triggers `cmd_output_set_device("default")` and resumes if possible.

7. **Update snapshot tooling + review pack**
   - What to do:
     - Extend `scripts/snapshot.ps1` milestone map and screenshot list for milestone 02.
     - Create `artifacts/ui/02-playback-shared-now-playing/REVIEW.md` mirroring Milestone 01 review format.
     - Capture required screenshots with `pnpm ui:snapshots -- --milestone 02`.
   - References:
     - `scripts/snapshot.ps1:7` — milestone mapping pattern.
     - `artifacts/ui/01-library-db-scan/REVIEW.md:1` — review format.
     - `docs/adr/0001-ui-vision-loop.md:18` — snapshot process rules.
     - `artifacts/README.md:9` — snapshot instructions and directory layout.
   - Acceptance Criteria:
     - [ ] Snapshot script lists new milestone screenshots and correct artifacts dir.
     - [ ] `REVIEW.md` checklist includes three screenshots and change summary.
     - [ ] Screenshots exist in `artifacts/ui/02-playback-shared-now-playing/`.

8. **Write ADRs 0005 + 0006**
   - What to do:
     - `0005-audio-decode-symphonia.md`: Context, Decision, Consequences for Symphonia.
     - `0006-audio-output-wasapi.md`: Context, Decision, Shared mode rationale, device handling.
   - References:
     - `docs/adr/0001-ui-vision-loop.md:1` — ADR format.
     - `docs/adr/0002-repo-architecture.md:1` — ADR format and repo context.
   - Acceptance Criteria:
     - [ ] Both ADRs exist and include Context/Decision/Consequences sections.
     - [ ] ADRs reflect Shared-mode fallback and verified formats decision.

9. **Update risk register**
   - What to do:
     - Add risks for device invalidation, playback latency/position drift, and device preference missing.
   - References:
     - `docs/risk-register.md:1` — risk register format.
   - Acceptance Criteria:
     - [ ] New risks added with owner/mitigation/status fields.

## Acceptance Criteria

- From the Tracks list, user can start playback and hear audio.
- Pause/resume and seek work.
- Output device can be switched (requires restart of stream).
- App remains responsive during playback (no UI freezes).
- UI snapshots generated.
- Device unplug/default change handled per decision (recover once or stop + error).
- Decoder errors surface a user-visible error and stop safely.
- “Switch to Default” action available when a selected device disappears.
- Volume persists across restart; missing device preference falls back to default with warning.

## Commands (Labeled)

- `cargo tauri dev` (Pre-existing)
- `pnpm ui:snapshots -- --milestone 02` (Introduced)
- `cargo test -p audio-engine` (Introduced)

## Verification (Tiered)

### Verification Strategy
- Automated: `cargo test -p audio-engine` for state machine/time accounting/queue behavior.
- Manual app QA (required): use `cargo tauri dev`, start playback from Tracks list, use Play/Pause/Seek, Next/Previous, and observe event-driven UI updates.
- Manual device verification (required):
  - Start playback on Default device, change Windows default output → expect one restart and audio resumes or error with “Switch to Default”.
  - Select a specific device in Settings, unplug it → expect stop + error + “Switch to Default” action.
  - Click “Switch to Default” → expect playback resumes on default device.
- UI snapshots: run `pnpm ui:snapshots -- --milestone 02`, capture required images, update review pack.
- Note: Tier B remains “N/A” for automated tests, but manual device verification is required locally.

- **Tier A — Hardware-agnostic**:
  - `cargo test -p audio-engine` passes.
  - UI snapshots generated correctly.
  - Playback state machine logic verified via tests.
- **Tier B — Hardware-dependent**: N/A — not hardware dependent

## Risks & Mitigations

- **Shared mode resampling by Windows audio engine** → UI must clearly label Shared as fallback; Exclusive will be default later.
- **Glitches/dropouts** → basic buffering controls introduced in milestone 07.
- **Device invalidation during playback** → detect errors, restart on default once, surface error otherwise.
- **Playback position drift** → drive UI position from engine ticks; throttle events to avoid UI overload.

## Tweaks (MUST/SHOULD/MAY)

- [MUST] T1 — Play session + time accounting

  - Source: T1
  - Key points:
    - Implement play_id and played_ms rules.
    - Foundation for scrobbling and history.
    - played_ms increments only while Playing; seek does not increment.
  - Rationale: Essential for tracking playback usage.

- [SHOULD] T2 — Gapless playback decision

  - Source: T2
  - Key points:
    - Decide on gapless support strategy (support or defer).
  - Rationale: Important for seamless listening experience.
  - Decision: Defer true gapless for M02; design interfaces to allow same-format prebuffering later.

- [MAY] T8.3 — Bit-perfect indicator

  - Source: T8.3
  - Key points:
    - Add UI indicator for bit-perfect status.
  - Rationale: User feedback on audio quality path.
  - Decision: Defer; keep Hidden.

## Suggestions

- Consider launching explore agents for codebase pattern discovery
- Verify all acceptance criteria with lsp_diagnostics before completion

## Deferred/Backlog

- T8.3 — Bit-perfect indicator (Default: Hidden). Already tracked in `prompts/BACKLOG.md:7`.

## Failure Recovery / Resume

- Checkpointing: note last completed deliverable after each major task group.
- Resume: if a session id is provided, use `sisyphus_task(resume="<session_id>", prompt="fix: <specific failure>")`.
- If resume fails: rerun the original task without category wrapper.

## Session Prompt Variables

- `milestone_id`: 02
- `milestone_title`: playback-shared-now-playing
- `milestone_file`: C:\Obsidian Vaults\Sermon\prompts\Milestone 02 - playback-shared-now-playing.md
- `special_emphasis`: none
- `dependencies`: Milestone 01 — library-db-scan

## Milestone-specific Notes

### Source Additions

- Add a minimal “Audio Debug” overlay (UI) that shows:
  - output mode: Shared
  - device name
  - negotiated format (sample rate/bit depth/channels)
