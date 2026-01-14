ultrawork
Generate the plan.

# Milestone 09 - asio-backend

## Goal
- Add **ASIO** as an advanced output option, including native DSD where drivers support it—while managing licensing/build/distribution risk.

## Scope (In)
- ASIO backend design:
    - Build-time feature flag: `asio`
    - When disabled, app ships without ASIO (default).
- ASIO integration strategy:
    - Prefer leveraging CPAL’s optional ASIO backend (only when `asio` feature enabled) to avoid implementing an ASIO host from scratch.
        - CPAL documents an optional `asio` feature on Windows and notes build requirements (drivers + LLVM/Clang for bindings generation).
- Licensing documentation and guardrails:
    - ASIO SDK is dual-licensed (GPLv3 or proprietary) according to publicly mirrored license text; proprietary terms historically restricted redistribution.
    - Clearly document what is required to legally build/distribute an ASIO-enabled build.
    - Provide CI/build defaults that do **not** enable ASIO.
- UI:
    - Preferences → Player → Output Mode includes ASIO only when feature enabled.
    - Device selection for ASIO drivers.
    - Buffer size control (ASIO panel).

## Non-scope (Out)
- Making ASIO the default path (WASAPI Exclusive remains default for Windows-first).
- Shipping an ASIO-enabled binary without clear licensing compliance.

## Prerequisites/Dependencies
- Milestone 08 — dsd-dop
- Milestone 03 — wasapi-exclusive-bit-perfect

## Key Decisions
- **ASIO as optional feature, not baseline**
    - Why: reduces legal and build complexity risk; keeps core app lightweight.
- **Use CPAL only for ASIO**
    - Why: CPAL’s WASAPI backend is shared-mode oriented; we keep our WASAPI implementation for Exclusive. (CPAL still helps for ASIO only.)
    - Alternative: implement ASIO host directly → more code + higher risk.

## Deliverables
- ASIO backend behind feature flag.
- `/docs/asio.md`:
    - build prerequisites
    - licensing notes (GPLv3 vs proprietary)
    - distribution guidance
- UI updates:
    - ASIO settings panel when enabled
- UI vision:
    - `/artifacts/ui/09-asio-backend/prefs-player-asio.png`
    - `/artifacts/ui/09-asio-backend/REVIEW.md`
- ADR:
    - `/docs/adr/0011-asio-strategy.md`
- Risk register update.

## Acceptance Criteria
- Default build (no ASIO feature) compiles and runs unchanged.
- ASIO-enabled build:
    - lists ASIO drivers (when installed)
    - can play PCM through selected ASIO device
- Clear documentation exists for licensing/build requirements.
- Snapshot pack produced.

## Commands (Labeled)
- `cargo tauri dev` (Pre-existing - Default)
- `cargo tauri dev --features asio` (Introduced - ASIO feature build)
- `pnpm ui:snapshots -- --milestone 09` (Introduced)

## Verification (Tiered)
- **Tier A — Hardware-agnostic**:
    - Build succeeds with and without `--features asio`.
    - Unit tests for ASIO configuration logic.
    - UI elements appear/hide correctly based on feature flag.
- **Tier B — Hardware-dependent**:
    - Playback via ASIO driver.
    - Native DSD playback (if driver supported).
    - "Needs real hardware confirmation".

## Risks & Mitigations
- **Licensing/distribution uncertainty** → keep ASIO off by default; document options; require explicit enable.
- **Toolchain friction (clang/bindgen)** → ASIO build path documented; CI not required to enable ASIO by default.

## Tweaks (MUST/SHOULD/MAY)
- **T8.5 (MUST)**: ASIO licensing checklist.
    - Explicitly document the "Legal/Compliance checklist" in `/docs/asio.md`.
    - Reference dual licensing (GPLv3 vs proprietary).
    - Ensure build instructions warn about redistribution restrictions.

## Suggestions
- Consider launching explore agents for codebase pattern discovery
- Verify all acceptance criteria with lsp_diagnostics before completion

## Deferred/Backlog
- None.

## Failure Recovery / Resume
- Checkpointing: note last completed deliverable after each major task group.
- Resume: if a session id is provided, use `sisyphus_task(resume="<session_id>", prompt="fix: <specific failure>")`.
- If resume fails: rerun the original task without category wrapper.

## Session Prompt Variables
- `milestone_id`: 09
- `milestone_title`: asio-backend
- `milestone_file`: C:\Obsidian Vaults\Sermon\prompts\Milestone 09 - asio-backend.md
- `special_emphasis`: two-tier hardware verification
- `dependencies`: Milestone 08 — dsd-dop; Milestone 03 — wasapi-exclusive-bit-perfect

## Milestone-specific Notes
### Source Additions
- Add a “Legal/Compliance checklist” section to `/docs/asio.md` referencing the dual licensing and restrictions.
