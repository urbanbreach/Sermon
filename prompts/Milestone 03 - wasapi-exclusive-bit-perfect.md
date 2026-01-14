ultrawork
Generate the plan.

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
            
- Seamless format switching:
    
    - When next track has different sample rate/bit depth, reinitialize exclusive stream automatically.
        
- Validation plan:
    
    - **Internal deterministic validation**: a “null sink” backend that captures output frames (pre-driver) to assert “no resample/no DSP/no unintended gain changes” for PCM.
        
    - **Practical external validation doc**:
        
        - DAC sample-rate indicator behavior
            
        - Optional DTS WAV passthrough method (user-provided file) where applicable
            
- UI:
    
    - Audio settings: output mode select, strict/compat policy
        
    - Diagnostics route: show track format vs output format; show “Exclusive active” state
        
- Logging:
    
    - Per track: requested format, actual format, and whether conversion occurred.

## Non-scope (Out)
- ASIO (milestone 09).
    
- DSD/DoP (milestone 08).
    
- DSP (must remain off by default).

## Prerequisites/Dependencies
- 03: Milestone 02 — playback-shared-now-playing

## Key Decisions
- **Use wasapi crate for Exclusive + event-driven buffering**
    
    - It explicitly supports shared and exclusive modes and both event-driven and polled buffering.
        
    - Alternative: raw `windows` API calls → more code + higher risk of subtle bugs.
        
- **Bit-perfect “strict” as default policy**
    
    - Why: matches requirement that default path must not resample or apply DSP; avoids silent “helpful” conversions.

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

## Acceptance Criteria
- On a system with an exclusive-capable DAC/device:
    
    - Exclusive mode starts successfully.
        
    - Playing a 44.1 kHz track then a 96 kHz track results in automatic format switch (verified in diagnostics + DAC indicator if available).
        
- In strict mode, if format unsupported → playback fails with clear UX and no silent conversion.
    
- Shared fallback still works.
    
- Snapshot pack produced.

## Commands (Labeled)
- `cargo tauri dev` (Pre-existing)
    
- `pnpm ui:snapshots -- --milestone 03` (Introduced)
    
- `cargo test -p audio-engine` (Pre-existing)

## Verification (Tiered)
- **Tier A — Hardware-agnostic**:
  - `cargo test` passes (logic tests, null sink).
  - UI diagnostics reflect "Exclusive" intent.
- **Tier B — Hardware-dependent**:
  - Real device check: Playback takes exclusive control (other apps muted).
  - DAC indicator confirmation (sample rate changes).

## Risks & Mitigations
- **Some devices reject “perfect match” formats** → Implement capability probing and good error messages; offer compatibility mode only as opt-in.
    
- **Exclusive mode blocks other apps** → UX should explain; easy toggle to Shared fallback.
    
- **Sample rate switch pops** → add small fade-out/in only if explicitly enabled (default off to preserve bit-perfect).

## Tweaks (MUST/SHOULD/MAY)
- [SHOULD] T2 — Gapless playback decision
  - Source: T2
  - Key points:
    - Revisit gapless support strategy in context of exclusive mode.
  - Rationale: High-end audio expectation.

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
- `milestone_id`: 03
- `milestone_title`: wasapi-exclusive-bit-perfect
- `milestone_file`: C:\Obsidian Vaults\Sermon\prompts\Milestone 03 - wasapi-exclusive-bit-perfect.md
- `special_emphasis`: two-tier hardware verification
- `dependencies`: Milestone 02 — playback-shared-now-playing

## Milestone-specific Notes
### Source Additions
- Add “Bit-perfect status” indicator in Now Playing:
    
    - `Bit-perfect: Yes/No (why)`
