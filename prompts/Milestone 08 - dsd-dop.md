# Milestone 08 - dsd-dop

## Goal

- Support DSD playback on Windows via **DoP over WASAPI Exclusive** (and clearly document limitations).

## Scope (In)

- Library:
  - Identify DSF/DFF files and store DSD technical metadata (DSD rate, channels).
- Playback:
  - DoP packing:
    - Implement DoP framing with marker bytes 0x05 / 0xFA (per DoP framing spec).
  - Output as PCM stream in Exclusive mode at the appropriate rate (e.g., DSD64 → 176.4k DoP).
  - Device capability detection:
    - If endpoint doesn’t support needed DoP PCM format, follow policy:
      - default: refuse with message (strict)
      - optional: “convert DSD to PCM” (explicit opt-in, not default; may be deferred if too big)
- UI:
  - Show DSD badges in track details and Now Playing.
  - Preferences: DoP enable/disable; strict behavior.

## Non-scope (Out)

- Native DSD via ASIO (milestone 09).
- DSP and resampling (must remain explicit opt-in only).

## Prerequisites/Dependencies

- Milestone 03 — wasapi-exclusive-bit-perfect

## Key Decisions

- **DSD strategy (Windows-first): DoP via WASAPI Exclusive**
  - Why: avoids reliance on ASIO; many DACs support DoP in exclusive mode.
- **No automatic DSD→PCM conversion by default**
  - Why: violates “no processing unless explicit.”

## Deliverables

- DSD metadata in DB.
- DoP playback path integrated into audio engine.
- `/docs/dsd.md`:
  - DoP explanation
  - device requirements
  - limitations and expected indicators
- Tests:
  - DoP packer correctness (markers, framing, byte ordering)
- UI vision:
  - `/artifacts/ui/08-dsd-dop/now-playing-dsd.png`
  - `/artifacts/ui/08-dsd-dop/prefs-devices-dsd.png`
  - `/artifacts/ui/08-dsd-dop/REVIEW.md`
- ADR:
  - `/docs/adr/0010-dsd-strategy.md`
- Risk register update.

## Acceptance Criteria

- On a DoP-capable DAC:
  - DSD file plays
  - Diagnostics indicates DoP output format
  - DAC indicates DSD (where supported)
- On non-capable device, app refuses playback in strict mode with clear guidance.
- Snapshot pack produced.

## Commands (Labeled)

- `cargo tauri dev` (Pre-existing)
- `pnpm ui:snapshots -- --milestone 08` (Introduced)
- `cargo test -p audio-engine` (Pre-existing)

## Verification (Tiered)

- **Tier A — Hardware-agnostic**:
  - Unit tests for DoP framing/packing logic (verify 0x05/0xFA markers).
  - Metadata extraction tests for DSF/DFF files.
  - Integration tests for "refuse playback" logic when DoP unsupported.
- **Tier B — Hardware-dependent**:
  - Playback on actual DSD-capable DAC.
  - Verify DAC display shows "DSD" (not PCM).
  - "Needs real hardware confirmation".

## Risks & Mitigations

- **Device compatibility fragmentation** → capability probing + good UX; recommend Shared fallback is not appropriate for DoP.
- **DoP framing bugs** → robust unit tests + known-good reference vectors.

## Tweaks (MUST/SHOULD/MAY)

- None explicitly mapped for this milestone.

## Suggestions

- Consider launching explore agents for codebase pattern discovery
- Verify all acceptance criteria with lsp_diagnostics before completion

## Deferred/Backlog

- Automatic DSD->PCM conversion (optional/deferred).

## Failure Recovery / Resume

- Checkpointing: note last completed deliverable after each major task group.
- Resume: if a session id is provided, use `sisyphus_task(resume="<session_id>", prompt="fix: <specific failure>")`.
- If resume fails: rerun the original task without category wrapper.

## Session Prompt Variables

- `milestone_id`: 08
- `milestone_title`: dsd-dop
- `milestone_file`: C:\Obsidian Vaults\Sermon\prompts\Milestone 08 - dsd-dop.md
- `special_emphasis`: two-tier hardware verification
- `dependencies`: Milestone 03 — wasapi-exclusive-bit-perfect

## Milestone-specific Notes

### Source Additions

- Add “DSD test track” checklist section to `/docs/dsd.md` (user-provided files; no redistribution).
