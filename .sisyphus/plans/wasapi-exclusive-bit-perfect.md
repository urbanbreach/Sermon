# Milestone 03 - wasapi-exclusive-bit-perfect

## Goal

- Implement **WASAPI Exclusive** as the primary playback path, with **automatic sample rate/bit depth switching per track** and a practical bit-perfect validation workflow.
- Make “bit-perfect by default” real for PCM content.

## Scope (In)

- Output modes:
  - WASAPI Exclusive (default)
  - WASAPI Shared (fallback)
- Format negotiation:
  - For each track, configure the exclusive stream to match the track’s PCM format when device supports it.
  - If the device doesn’t support exact format:
    - “Strict bit-perfect” mode: fail playback with actionable message
    - “Compatibility” mode: allow explicit conversion (must be opt-in; default remains strict)
  - Compatibility conversion is defined as: attempt 16‑bit zero‑pad into 24/32‑bit integer if the device accepts exclusive; otherwise fallback to WASAPI Shared (Windows SRC). No software resampler in M03.
  - Strict mode requires exact match (no zero-padding 16‑bit → 24‑bit).
- Seamless format switching:
  - When next track has different sample rate/bit depth, reinitialize exclusive stream automatically.
- Validation plan:
  - **Internal deterministic validation**: a “null sink” backend that captures output frames (pre-driver) to assert “no resample/no DSP/no unintended gain changes” for PCM.
  - **Practical external validation doc**:
    - DAC sample-rate indicator behavior
    - Optional DTS WAV passthrough method (user-provided file) where applicable
- UI:
  - Audio settings: output mode select, strict/compat policy, advanced fade toggle (default off)
  - Diagnostics route: show track format vs output format; show “Exclusive active” state; show conversion status
  - Now Playing: “Bit-perfect: Yes/No (why)” indicator
- Logging:
  - Per track: requested format, actual format, and whether conversion occurred (including Shared fallback reason)

## Non-scope (Out)

- ASIO (milestone 09).
- DSD/DoP (milestone 08).
- DSP (must remain off by default).
- Software resampling/SRC and proactive capability scanning beyond try/fail.
- CLI settings surface.

## Prerequisites/Dependencies

- 03: Milestone 02 — playback-shared-now-playing

## Key Decisions

- **Use wasapi crate for Exclusive + event-driven buffering**
  - It explicitly supports shared and exclusive modes and both event-driven and polled buffering.
  - Alternative: raw `windows` API calls → more code + higher risk of subtle bugs.
- **Bit-perfect “strict” as default policy**
  - Why: matches requirement that default path must not resample or apply DSP; avoids silent “helpful” conversions.
- **Compatibility conversion = zero‑pad 16‑bit when supported, otherwise Shared fallback**
  - Why: no resampler in M03; Shared uses Windows SRC and must be explicit; padding is allowed only in Compatibility and must be logged.
- **Exclusive+Strict enforces unity gain**
  - Why: preserve bit-perfect semantics; disable volume UI in strict path.
- **Exclusive+Strict output uses integer PCM matching track bit depth**
  - Exact match for 24‑bit allows 24‑in‑32 container with `valid_bits=24`.
  - Why: float output is not presumed bit-perfect; allow float only in Compatibility/Shared and label as non-bit-perfect.
- **Settings UI is the sole user-facing control**
  - Optional developer overrides (`SERMON_AUDIO_MODE`, `SERMON_AUDIO_POLICY`) allowed but not primary UX.
- **Gapless decision (SHOULD, T2)**
  - Defer true gapless guarantees in M03; no guaranteed gapless across format changes.

## Deliverables

- Exclusive output implementation + mode switching.
- “Audio Diagnostics” route in UI.
- `/docs/bit-perfect-validation.md` (step-by-step validation methods + limitations).
- Automated tests for:
  - format switching logic
  - null-sink PCM invariants
- UI vision:
  - `/artifacts/ui/03-wasapi-exclusive-bit-perfect/settings-audio.png`
  - `/artifacts/ui/03-wasapi-exclusive-bit-perfect/diagnostics-playing-44k.png`
  - `/artifacts/ui/03-wasapi-exclusive-bit-perfect/diagnostics-playing-96k.png`
  - `/artifacts/ui/03-wasapi-exclusive-bit-perfect/REVIEW.md`
- Risk register update.

### Work Plan (Tasks + Acceptance Criteria)

- [x] 1. Define audio output settings schema + API flow

  **What to do**:
  - Add settings keys in `crates/library/src/db/mod.rs`:
    - `audio.output.mode` = `exclusive|shared` (default `exclusive`)
    - `audio.output.policy` = `strict|compatibility` (default `strict`)
    - `audio.output.fade` = `off|on` (default `off`)
  - On app startup, read settings; if any are missing, persist defaults to the settings table in `spawn_audio_thread` before entering the loop (use `open_db/apply_migrations` like `resolve_track` in `src-tauri/src/lib.rs:699`).
  - Add Tauri payload struct `AudioOutputSettings` and commands:
    - `cmd_output_get_settings` (read from settings table)
    - `cmd_output_set_settings` (persist + emit to audio thread)
  - Extend `PlaybackCommand` with `SetOutputSettings { mode, policy, fade }` and wire to `handle_playback_command` to update playback state and reinit output if playing.
  - Add UI API wrappers in `ui/src/lib/api/playback.ts` and a new store in `ui/src/lib/state/playback.ts` to load/save settings.

  **Must NOT do**:
  - No CLI settings surface.

  **Parallelizable**: YES (with 2)

  **References**:
  - `crates/library/src/db/mod.rs:216` - `get_setting/set_setting` helpers.
  - `crates/library/tests/settings.rs:8` - settings CRUD test pattern.
  - `src-tauri/src/state.rs:26` - `PlaybackCommand` enum.
  - `src-tauri/src/lib.rs:366` - `spawn_audio_thread` startup context.
  - `src-tauri/src/commands/playback.rs:238` - existing output/device commands.
  - `ui/src/lib/api/playback.ts:1` - Tauri invoke pattern.

  **Acceptance Criteria**:
  - [ ] Settings persist and reload correctly.
  - [ ] Defaults are Exclusive + Strict + FadeOff.
  - [ ] Changing settings updates audio thread without crash.

- [x] 2. Implement Exclusive output pipeline + format negotiation

  **What to do**:
  - Introduce Exclusive stream creation in `crates/audio-engine/src/output.rs` using `StreamMode::EventsExclusive` with explicit `period_hns` selection:
    - Call `AudioClient::get_device_period()` (shared path already uses `def_time` at `crates/audio-engine/src/output.rs:81`).
    - Use `calculate_aligned_period_near(def_time, Some(128), &wave_format)` when available; fallback to `def_time` if alignment fails.
  - Probe exact formats via `is_supported_exclusive_with_quirks` (wasapi docs) and handle unsupported format errors.
  - Negotiation rules:
    - **Exact match definition**: sample_rate + channels must equal track; bit depth must match track. For 24‑bit tracks, allow 24‑in‑32 container with `valid_bits=24` as exact; any other alternate format returned by `is_supported_exclusive_with_quirks` is treated as unsupported in Strict.
    - Strict: require exact match; if unsupported or exclusive unavailable → fail with actionable message.
    - Compatibility: try exact → try 16‑bit zero‑pad into 24/32‑bit integer container (exclusive) → if still unsupported or exclusive unavailable, fallback to Shared.
  - Detect and map exclusive errors:
    - `AUDCLNT_E_EXCLUSIVE_MODE_NOT_ALLOWED` or `AUDCLNT_E_DEVICE_IN_USE` → `exclusive_unavailable`.
    - `AUDCLNT_E_UNSUPPORTED_FORMAT` → `exclusive_unsupported_format`.
  - Track output mode/format in `AudioPlayback` (`src-tauri/src/lib.rs:142`) and reinitialize on format changes.
  - Fade toggle behavior (advanced, default off): when reinitializing for a format change, apply linear fade‑out over 10ms before stop and linear fade‑in over 10ms after start; mark bit‑perfect false during fade window.

  **Must NOT do**:
  - No software resampler.
  - No proactive capability scanning beyond try/fail.

  **Parallelizable**: YES (with 1)

  **References**:
  - `crates/audio-engine/src/output.rs:24` - current shared-mode output implementation.
  - `src-tauri/src/lib.rs:202` - `open_output_for_format` reinit path.
  - `docs/adr/0006-audio-output-wasapi.md:36` - existing WASAPI ring buffer model.
  - `https://docs.rs/wasapi/latest/wasapi/` - `StreamMode::EventsExclusive`, `is_supported_exclusive_with_quirks`.

  **Acceptance Criteria**:
  - [ ] Exclusive stream opens for exact PCM format when supported.
  - [ ] Strict mode emits `PlaybackErrorEvent` with `code="exclusive_unavailable"` or `code="exclusive_unsupported_format"` and message: "Exclusive mode unavailable (device in use). Close other audio apps or switch to Shared/Compatibility." or "Exclusive format unsupported for this track. Switch to Compatibility/Shared.".
  - [ ] Compatibility uses `conversion="pad_16_to_24"` when zero‑padding succeeds; otherwise auto‑fallbacks to Shared with `conversion="shared_fallback"`.
  - [ ] Fade toggle applies 10ms fade‑out/in around format reinit when enabled.

- [x] 3. Add f32 → PCM integer conversion + NullSink backend

  **What to do**:
  - Add f32→PCM integer conversion for 16/24/32‑bit output (no dithering) at final write stage.
  - Enforce unity gain in Exclusive+Strict by overriding volume to 1.0 in `process_audio` (ignore `SetVolume` for strict; preserve stored volume for Shared/Compatibility).
  - Introduce an `OutputBackend` enum or `AudioOutput` trait in `crates/audio-engine/src/output.rs` with implementations for `WasapiOutput` and `NullSinkOutput`.
  - Wire `AudioPlayback` to hold `OutputBackend` instead of hardwired `WasapiOutput` (`src-tauri/src/lib.rs:144`) and add a test‑only constructor or feature flag to select `NullSinkOutput` in unit tests.

  **Must NOT do**:
  - No DSP or software resampling.

  **Parallelizable**: YES (with 2)

  **References**:
  - `crates/audio-engine/src/output.rs:134` - current f32 write path (shared).
  - `src-tauri/src/lib.rs:144` - output hardwired to `WasapiOutput`.
  - `crates/audio-engine/src/engine.rs:169` - tests pattern.

  **Acceptance Criteria**:
  - [ ] Unit tests validate f32→PCM conversion clamps correctly.
  - [ ] NullSink tests confirm no gain change in strict (unity) and capture frames for assertions.
  - [ ] Exclusive+Strict ignores `SetVolume` and uses unity gain at output.
  - [ ] `cargo test -p audio-engine` passes.

- [x] 4. Define bit‑depth source + diagnostics payload + bit‑perfect rules

  **What to do**:
  - Use `TrackInfo.bit_depth` from library row as authoritative bit‑depth source (`src-tauri/src/lib.rs:703`, `crates/library/src/models.rs:38`).
  - If bit depth is unknown, Strict fails with “Unknown bit depth for strict mode” and Compatibility falls back to Shared.
  - Expand `AudioDebugEvent` with fields:
    - `output_mode` (`exclusive|shared`)
    - `policy` (`strict|compatibility`)
    - `conversion` (`none|shared_fallback|pad_16_to_24`)
    - `gain_mode` (`unity|software`)
    - `fade_enabled` (bool)
    - `exclusive_active` (bool)
    - `bit_perfect` (`yes|no`) + `bit_perfect_reason` string
  - Emit new `PlaybackErrorEvent` codes for strict failures:
    - `exclusive_unavailable` (device in use / exclusive not allowed)
    - `exclusive_unsupported_format`
    - `bit_depth_unknown`
  - Bit‑perfect rules (exact “why” strings):
    - YES only if: `exclusive + strict + conversion=none + fade_enabled=false + gain_mode=unity + output format matches track sample_rate/bit_depth/channels + bit_depth known`.
    - NO reasons (single string) with **precedence order** (first matching wins):
      1. `Output mode: Shared fallback (Windows SRC)`
      2. `Policy: Compatibility`
      3. `Conversion: Zero-pad 16→24 (compat)`
      4. `Gain: Software volume`
      5. `Fade enabled (not bit-perfect during fade window)`
      6. `Format mismatch (track vs output)`
      7. `Bit depth unknown`
  - Log requested vs actual formats per track with conversion info.

  **Parallelizable**: NO (depends on 2, 3)

  **References**:
  - `src-tauri/src/commands/playback.rs:71` - `AudioDebugEvent` payload and `PlaybackErrorEvent`.
  - `src-tauri/src/lib.rs:832` - `emit_audio_debug`.
  - `ui/src/lib/types/playback.ts:63` - UI debug types.
  - `ui/src/lib/components/BottomBar.svelte:19` - error banner surface.
  - `crates/library/src/models.rs:38` - `TrackRow.bit_depth`.
  - `src-tauri/src/lib.rs:703` - `resolve_track` populates `TrackInfo`.

  **Acceptance Criteria**:
  - [ ] Diagnostics payload includes conversion, gain, fade, and bit‑perfect reason fields.
  - [ ] Error banner displays the exact strict‑mode messages from Task 2.
  - [ ] Logs show requested vs actual output format and conversion reason.

- [x] 5. Implement UI settings + diagnostics route + bit‑perfect indicator

  **What to do**:
  - Add `diagnostics` route and view; update `route.ts`, `App.svelte`, and `LeftNav.svelte`.
  - Create `DiagnosticsView.svelte` showing track/output format, exclusive active, conversion, gain mode, bit‑perfect status + reason.
  - Update Settings view to include Output Mode, Policy, Fade toggle; wire to `outputSettings` store.
  - Disable volume slider in `BottomBar.svelte` when Exclusive+Strict; show “Unity” label.
  - Add “Bit‑perfect: Yes/No (why)” indicator in Now Playing (prefer BottomBar or NowPlayingView).

  **Parallelizable**: NO (depends on 1, 4)

  **References**:
  - `ui/src/lib/state/route.ts:3` - route union.
  - `ui/src/App.svelte:43` - route rendering.
  - `ui/src/lib/components/LeftNav.svelte:4` - nav items.
  - `ui/src/lib/views/SettingsView.svelte:95` - audio section.
  - `ui/src/lib/views/NowPlayingView.svelte:95` - debug overlay pattern.
  - `ui/src/lib/components/BottomBar.svelte:54` - volume slider.

  **Acceptance Criteria**:
  - [ ] Settings UI can switch output mode/policy/fade.
  - [ ] Diagnostics route shows exclusive active, conversion status, gain mode.
  - [ ] Now Playing shows bit‑perfect indicator with reason.

- [x] 6. Write `/docs/bit-perfect-validation.md`

  **What to do**:
  - Document internal null-sink validation steps and limitations.
  - Document external validation (DAC indicator + DTS WAV passthrough).
  - Include explicit references:
    - https://msbtechnology.com/support/bit-perfect-testing/
    - https://matthewvaneerde.wordpress.com/2017/10/17/how-to-negotiate-an-audio-format-for-a-windows-audio-session-api-wasapi-client/
    - https://github.com/jwhitham/spdif-bit-exactness-tools

  **Parallelizable**: YES (with 2–5)

  **Acceptance Criteria**:
  - [ ] Doc includes step‑by‑step validation + limitations.

- [x] 7. Update UI snapshot tooling + review pack

  **What to do**:
  - Add milestone 03 mapping to `scripts/snapshot.ps1` and required screenshots list.
  - Capture screenshots and write `artifacts/ui/03-wasapi-exclusive-bit-perfect/REVIEW.md`.

  **Parallelizable**: NO (depends on 5)

  **References**:
  - `scripts/snapshot.ps1:8` - milestone mapping.
  - `artifacts/ui/02-playback-shared-now-playing/REVIEW.md:1` - review pack template.
  - `artifacts/README.md:1` - snapshot process.

  **Acceptance Criteria**:
  - [ ] `pnpm ui:snapshots -- --milestone 03` produces artifacts.
  - [ ] Review checklist includes the three required screenshots.

- [x] 8. Update risk register

  **What to do**:
  - Add/update risks for Exclusive device‑in‑use and compatibility (non‑bit‑perfect) fallback.

  **Parallelizable**: YES (with 6)

  **References**:
  - `docs/risk-register.md:3` - risk table format.

  **Acceptance Criteria**:
  - [ ] New risk entries added with mitigation + status.

## Acceptance Criteria

- On a system with an exclusive-capable DAC/device:
  - Exclusive mode starts successfully.
  - Playing a 44.1 kHz track then a 96 kHz track results in automatic format switch (verified in diagnostics + DAC indicator).
- In strict mode, if format unsupported → playback fails with clear UX and no silent conversion.
- Shared fallback still works.
- Snapshot pack produced.
- Exclusive+Strict enforces unity gain and disables volume control.
- Compatibility auto‑fallbacks to Shared for exclusive‑in‑use errors with clear UX.

## Commands (Labeled)

- `cargo tauri dev` (Pre-existing)
- `pnpm ui:snapshots -- --milestone 03` (Introduced)
- `cargo test -p audio-engine` (Pre-existing)

## Verification (Tiered)

### Verification Strategy (MANDATORY)
- **Infrastructure exists**: YES (Rust tests in `crates/audio-engine/src/engine.rs:169` and `crates/library/tests/*.rs`).
- **User wants tests**: YES (tests-after).
- **Required automated tests**:
  - Format negotiation decision logic (strict fail vs compatibility fallback).
  - f32→PCM conversion correctness (clamp, no dithering).
  - Null-sink invariants (no gain change, no resample).
  - Mode switching behavior (exclusive → shared fallback) without crash.
- **Manual QA**:
  - Play 44.1k then 96k track; verify diagnostics and DAC indicator change.
  - Verify Shared fallback and conversion reason strings (`shared_fallback` or `pad_16_to_24`).
  - Toggle fade on; confirm diagnostics show "Fade enabled (not bit-perfect during fade window)".
  - Verify volume disabled in Exclusive+Strict.

### Tier A — Hardware-agnostic
- `cargo test` passes (logic tests, null sink).
- UI diagnostics reflect "Exclusive" intent.

### Tier B — Hardware-dependent
- Real device check: Playback takes exclusive control (other apps muted).
- DAC indicator confirmation (sample rate changes).

## Risks & Mitigations

- **Some devices reject “perfect match” formats** → Implement capability probing and good error messages; offer compatibility mode only as opt-in.
- **Exclusive mode blocks other apps** → UX should explain; easy toggle to Shared fallback.
- **Sample rate switch pops** → add small fade-out/in only if explicitly enabled (default off to preserve bit-perfect).
- **Exclusive device in use** → strict fails with actionable message; compatibility auto-fallback to Shared.

## Tweaks (MUST/SHOULD/MAY)

- [SHOULD] T2 — Gapless playback decision
  - Source: T2
  - Key points:
    - Revisit gapless support strategy in context of exclusive mode.
    - No guaranteed gapless across format changes in M03.
    - Maintain architecture compatibility for future same-format gapless.
  - Rationale: High-end audio expectation.

## Suggestions

- Consider launching explore agents for codebase pattern discovery
- Verify all acceptance criteria with lsp_diagnostics before completion

## Deferred/Backlog

- MAY items with recommended defaults:
  - Software resampler (defer; compatibility remains zero‑pad or Shared fallback only).
  - True gapless across format changes (defer).
  - ASIO backend (Milestone 09).
  - DSD/DoP (Milestone 08).

## Failure Recovery / Resume

- Checkpointing: note last completed deliverable after each major task group.
- Resume: if a session id is provided, use `sisyphus_task(resume="<session_id>", prompt="fix: <specific failure>")`.
- If resume fails: rerun the original task without category wrapper.

## Session Prompt Variables

- `milestone_id`: 03
- `milestone_title`: wasapi-exclusive-bit-perfect
- `milestone_file`: C:\Obsidian Vaults\Sermon\prompts\Milestone 03 - wasapi-exclusive-bit-perfect.md
- `special_emphasis`: two-tier hardware verification
- `dependencies`: Milestone 02 — playback-shared-now-playing

## Milestone-specific Notes

### Source Additions

- Add “Bit-perfect status” indicator in Now Playing:
  - `Bit-perfect: Yes/No (why)`
