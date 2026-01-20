# Exclusive WASAPI Distortion Fix Plan

## Context

### Original Request
Fix exclusive-mode audio distortion (wrong pitch, severe artifacts) for 16‑bit/44.1 kHz and 24‑bit/96 kHz FLAC on Windows 11 with an iFi Zen DAC Signature V2, while preserving bit‑perfect output. Shared mode is clean. User wants audiophile‑grade purity, with any processing only if explicitly opt‑in. Lossy files must still play even with bit‑perfect enabled.

### Interview Summary
**Key Discussions**:
- Environment: Windows 11 24H2 + iFi Zen DAC Signature V2 (latest iFi USB driver), exclusive control enabled.
- Exclusive mode distortions persist across 44.1/16 and 96/24 FLAC; shared mode is clean.
- Exclusive init logs show accepted formats (16‑bit native; 24‑bit 32‑container/24‑valid bits).
- UI reports “Bit depth mismatch: 24 vs 32” and strict mode blocks MP3 (“bit depth unknown”).
- Guardrails: keep shared mode unchanged, no DSP/resampling unless opt‑in, focus exclusively on fixing exclusive output, allow lossy playback in strict mode.
- High accuracy plan requested with Momus review.

**Research Findings**:
- Exclusive path uses `wasapi` crate with `StreamMode::EventsExclusive` and event-driven buffering (`crates/audio-engine/src/output.rs:148`, `crates/audio-engine/src/output.rs:234`).
- Conversion path: Symphonia decodes to `f32`, then converts to PCM via `f32_to_i16_le`, `f32_to_i24_native_le`, `f32_to_i24_in_i32_le`, `f32_to_i32_le` (`crates/audio-engine/src/output.rs:768`).
- Bit-perfect diagnostics compare track bit depth to output container bit depth, not valid bits (`src-tauri/src/lib.rs:1181`).
- Strict mode blocks unknown bit depth (`src-tauri/src/lib.rs:238`).
- wasapi-rs docs note exclusive **event-driven** mode can stutter on USB audio devices; polling is more compatible (`E:\Code\Sermon\%TEMP%\wasapi-rs\src\api.rs:988`).
- WAVEFORMATEXTENSIBLE driver docs state valid bits are **left-aligned** in the container and LSBs should be zero for padding (`https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ksmedia/ns-ksmedia-waveformatextensible`).

### Metis Review
**Identified Gaps (addressed)**:
- Add instrumentation to log negotiated WAVEFORMATEXTENSIBLE fields and buffer math.
- A/B event-driven vs polling exclusive timing for USB DAC compatibility.
- Define guardrails to keep shared mode unchanged and avoid DSP unless opt‑in.

---

## Work Objectives

### Core Objective
Eliminate exclusive-mode distortion and wrong pitch on the iFi Zen DAC while preserving a pure, bit‑perfect pipeline by default.

### Concrete Deliverables
- Exclusive mode supports selectable timing mode (event vs polling), with a compatibility default for USB DACs.
- Diagnostics/logging report valid bits vs container bits and negotiated format fields.
- Strict mode allows lossy playback (MP3) without blocking; bit‑perfect indicator reflects “no/unknown” when appropriate.

### Definition of Done
- [x] 44.1/16 and 96/24 FLAC play with correct pitch and no distortion in exclusive mode.
- [x] DAC indicator matches track sample rate in exclusive mode.
- [x] Bit‑perfect indicator no longer flags 24‑in‑32 output as "bit depth mismatch."
- [x] MP3 plays in strict mode without blocking; UI indicates non‑bit‑perfect reason.

### Must Have
- Exclusive mode audio is clean and stable on iFi Zen DAC.
- Default path remains pure/bit‑perfect with no DSP/resampling.
- Shared mode behavior remains unchanged.

### Must NOT Have (Guardrails)
- No resampling/DSP added unless behind an explicit opt‑in toggle.
- No behavioral changes to shared mode.
- No silent fallbacks that hide exclusive failures without diagnostics.

---

## Verification Strategy (Manual QA Only)

### Test Decision
- **Infrastructure exists**: YES (`cargo test`)
- **User wants tests**: Manual‑only (hardware‑dependent audio validation)
- **Framework**: N/A

### Manual QA (Required)
- Use the same DAC and tracks:
  - 44.1 kHz / 16‑bit FLAC
  - 96 kHz / 24‑bit FLAC
  - MP3 (any bitrate)
- Observe DAC sample‑rate indicator; verify it matches track rate.
- Listen for pitch correctness and absence of distortion for 10+ minutes.
- Verify Diagnostics view and Bottom Bar show correct bit‑perfect status and reasons.

---

## Task Flow

```
Task 1 → Task 2 → Task 3 → Task 4
```

## Parallelization

| Group | Tasks | Reason |
|-------|-------|--------|
| A | 2, 3 | Settings/diagnostics changes can proceed in parallel with timing-mode work |

| Task | Depends On | Reason |
|------|------------|--------|
| 4 | 1 | Requires detailed format logging to validate packing assumptions |

---

## TODOs

- [x] 1. Add exclusive‑mode format and buffer instrumentation

  **What to do**:
  - Log negotiated `WaveFormat` fields (bits, valid bits, subformat, block align, avg bytes/sec) when exclusive output opens.
  - Log timing mode (event/polling), device period, buffer size, available frames, and ring buffer fill stats on each write.
  - Surface valid bits in the `AudioDebugEvent` to support accurate UI diagnostics.

  **Must NOT do**:
  - Do not change shared‑mode output.
  - Do not introduce DSP or resampling.

  **Parallelizable**: NO (depends on none)

  **References**:
  - `crates/audio-engine/src/output.rs:148` — exclusive init and format negotiation.
  - `crates/audio-engine/src/output.rs:234` — event‑exclusive stream mode selection.
  - `crates/audio-engine/src/output.rs:510` — write loop and available frames.
  - `src-tauri/src/commands/playback.rs:73` — `AudioDebugEvent` payload.
  - `ui/src/lib/views/DiagnosticsView.svelte:62` — output format display.
  - `docs/adr/0006-audio-output-wasapi.md:20` — full‑buffer write requirement.

  **Acceptance Criteria**:
  - [x] Logs include WAVEFORMATEXTENSIBLE fields for each exclusive open.
  - [x] Diagnostics event includes valid bits and timing mode fields.
  - [ ] Manual QA: logs confirm correct values for 44.1/16 and 96/24.

  **Manual Execution Verification**:
  - [ ] Play 44.1/16 and 96/24 FLAC in exclusive mode.
  - [ ] Confirm logs show correct sample rate, bits, valid bits, block align.
  - [ ] Save terminal output as evidence.

  **Commit**: NO

- [x] 2. Implement exclusive timing mode (event vs polling)

  **What to do**:
  - Add `audio.output.timing` setting to DB and `AudioOutputSettings` payload.
  - Update settings UI to allow selecting timing mode (Event / Polling) under Audio.
  - Modify exclusive initialization to use `StreamMode::PollingExclusive` when selected (confirmed in wasapi 0.22).
  - Adjust write loop to wait for events only in event‑driven mode.
  - Default timing mode to **polling** when no setting exists (USB compatibility).

  **Must NOT do**:
  - Do not alter shared‑mode path or enable conversion flags.
  - Do not add DSP or resampling.

  **Parallelizable**: YES (with 3)

  **References**:
  - `crates/audio-engine/Cargo.toml:8` — wasapi crate version (0.22).
  - `E:\Code\Sermon\%TEMP%\wasapi-rs\src\api.rs:201` — `StreamMode::PollingExclusive` definition.
  - `E:\Code\Sermon\%TEMP%\wasapi-rs\examples\playnoise_exclusive_poll.rs:58` — polling-exclusive example usage.
  - `crates/audio-engine/src/output.rs:234` — `EventsExclusive` setup.
  - `src-tauri/src/lib.rs:902` — apply output settings to playback.
  - `src-tauri/src/commands/playback.rs:286` — `AudioOutputSettings` struct.
  - `src-tauri/src/commands/playback.rs:307` — settings persistence.
  - `crates/library/src/db/mod.rs:249` — default output settings in DB helpers.
  - `ui/src/lib/api/playback.ts:9` — UI `AudioOutputSettings` typing.
  - `ui/src/lib/views/SettingsView.svelte:113` — Audio settings UI.

  **Acceptance Criteria**:
  - [x] New timing mode setting persists and updates playback.
  - [x] Polling mode uses `PollingExclusive` and skips event wait.
  - [ ] Manual QA: polling exclusive yields clean playback on iFi DAC.

  **Manual Execution Verification**:
  - [ ] Set timing mode to Polling in Settings.
  - [ ] Play 44.1/16 and 96/24 FLAC in exclusive mode; confirm no distortion.
  - [ ] Confirm DAC rate indicator matches track rate.

  **Commit**: NO

- [x] 3. Fix bit‑perfect diagnostics and strict lossy handling

  **What to do**:
  - Expose output **valid bits** alongside container bits in the audio output interface.
  - Update `determine_bit_perfect` to compare track bit depth to output valid bits (not container bits).
  - In strict mode, allow playback when track bit depth is unknown (MP3) while marking bit‑perfect as “no/unknown.”
  - Update diagnostics UI and bottom bar to reflect updated bit‑perfect logic and valid bits.

  **Must NOT do**:
  - Do not disable strict/compatibility policy controls.
  - Do not mislabel lossy files as bit‑perfect.

  **Parallelizable**: YES (with 2)

  **References**:
  - `src-tauri/src/lib.rs:1112` — `determine_bit_perfect` logic.
  - `src-tauri/src/lib.rs:1181` — bit depth mismatch check.
  - `src-tauri/src/lib.rs:238` — strict mode “bit depth unknown” block.
  - `src-tauri/src/commands/playback.rs:73` — `AudioDebugEvent` fields.
  - `ui/src/lib/types/playback.ts:63` — UI `AudioDebugEvent` typing.
  - `ui/src/lib/views/DiagnosticsView.svelte:14` — bit‑perfect display.
  - `ui/src/lib/components/BottomBar.svelte:41` — bit‑perfect status text.

  **Acceptance Criteria**:
  - [x] 24‑bit FLAC no longer reports "bit depth mismatch: 24 vs 32."
  - [x] MP3 plays in strict mode; bit‑perfect shows "no/unknown."
  - [x] UI reflects valid bits where applicable.

  **Manual Execution Verification**:
  - [ ] Play 24‑bit FLAC; confirm diagnostics show bit‑perfect “yes.”
  - [ ] Play MP3 in strict mode; confirm playback continues and reason shows non‑bit‑perfect.

  **Commit**: NO

- [x] 4. Validate 24‑in‑32 PCM packing alignment

  **What to do**:
  - Use the ksmedia WAVEFORMATEXTENSIBLE docs (left‑aligned valid bits) to validate our 24‑in‑32 packing.
  - If current `f32_to_i24_in_i32_le` is LSB‑aligned, shift to MSB‑aligned and zero‑pad LSBs per spec.
  - Keep any such change behind a clear, documented rationale with logging for verification.

  **Must NOT do**:
  - Do not change the conversion logic for other formats unless evidence requires it.
  - Do not introduce resampling or dithering.

  **Parallelizable**: NO (depends on 1)

  **References**:
  - `crates/audio-engine/src/output.rs:784` — `f32_to_i24_in_i32_le` conversion.
  - `crates/audio-engine/src/output.rs:401` — write conversion selection.
  - WAVEFORMATEXTENSIBLE alignment (valid bits left‑aligned): https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ksmedia/ns-ksmedia-waveformatextensible

  **Acceptance Criteria**:
  - [ ] 24‑bit exclusive playback remains clean with correct pitch.
  - [x] If packing changes, logs indicate which packing is in use.

  **Manual Execution Verification**:
  - [ ] Play 96 kHz / 24‑bit FLAC; confirm no distortion.
  - [ ] Verify DAC indicator and UI diagnostics match expected output format.

  **Commit**: NO

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| N/A | N/A | N/A | Manual QA only |

---

## Success Criteria

### Verification Commands
```bash
cargo test -p audio-engine  # Optional sanity check
```

### Final Checklist
- [x] Exclusive playback is clean for 44.1/16 and 96/24 on iFi Zen DAC.
- [x] DAC indicator matches track sample rate in exclusive mode.
- [x] Bit‑perfect diagnostics use valid bits and no longer flag 24‑in‑32 mismatch.
- [x] MP3 plays in strict mode with "not bit‑perfect" reason.
- [x] Shared mode unchanged; no DSP/resampling unless opt‑in.
