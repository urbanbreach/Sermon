# Milestone 02 - playback-shared-now-playing

## Goal

- Deliver the first end-to-end experience: **scan → click track → hear audio**, using **WASAPI Shared** as the initial output backend (non-bit-perfect fallback path).
- Implement a minimal audio engine architecture that can later swap in WASAPI Exclusive and ASIO.

## Scope (In)

- Audio decode:
  - Use Symphonia for demux/decode of common formats (lossless + lossy). Symphonia supports multiple codecs/containers including FLAC/WAV/MP3/AAC/ALAC/Vorbis and common containers.
- Audio output:
  - WASAPI Shared mode output via `wasapi` crate (simple path first).
  - Device selection: default output device + ability to choose another.
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

## Non-scope (Out)

- WASAPI Exclusive (milestone 03).
- Bit-perfect validation (milestone 03).
- DSD (milestone 08).
- DSP features (off-scope unless explicitly opt-in later).

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

## Acceptance Criteria

- From the Tracks list, user can start playback and hear audio.
- Pause/resume and seek work.
- Output device can be switched (requires restart of stream).
- App remains responsive during playback (no UI freezes).
- UI snapshots generated.

## Commands (Labeled)

- `cargo tauri dev` (Pre-existing)
- `pnpm ui:snapshots -- --milestone 02` (Introduced)
- `cargo test -p audio-engine` (Introduced)

## Verification (Tiered)

- **Tier A — Hardware-agnostic**:
  - `cargo test -p audio-engine` passes.
  - UI snapshots generated correctly.
  - Playback state machine logic verified via tests.
- **Tier B — Hardware-dependent**: N/A — not hardware dependent

## Risks & Mitigations

- **Shared mode resampling by Windows audio engine** → UI must clearly label Shared as fallback; Exclusive will be default later.
- **Glitches/dropouts** → basic buffering controls introduced in milestone 07.

## Tweaks (MUST/SHOULD/MAY)

- [MUST] T1 — Play session + time accounting

  - Source: T1
  - Key points:
    - Implement play_id and played_ms rules.
    - Foundation for scrobbling and history.
  - Rationale: Essential for tracking playback usage.

- [SHOULD] T2 — Gapless playback decision

  - Source: T2
  - Key points:
    - Decide on gapless support strategy (support or defer).
  - Rationale: Important for seamless listening experience.

- [MAY] T8.3 — Bit-perfect indicator
  - Source: T8.3
  - Key points:
    - Add UI indicator for bit-perfect status.
  - Rationale: User feedback on audio quality path.

## Suggestions

- Consider launching explore agents for codebase pattern discovery
- Verify all acceptance criteria with lsp_diagnostics before completion

## Deferred/Backlog

[List MAY items with recommended defaults]

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
