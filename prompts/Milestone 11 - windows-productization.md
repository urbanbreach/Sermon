# Milestone 11 - windows-productization

## Goal

- Ship a minimal Windows product: installer build, basic update strategy, and key Windows integrations (media keys + device UX polish), while hitting performance budgets.

## Scope (In)

- Packaging:
  - Choose Windows installer target(s): MSI (WiX) and/or NSIS via Tauri.
  - Tauri docs note Windows distribution is via MSI (WiX Toolset v3) or NSIS setup executables.
- Update strategy (explicit):
  - Decide: implement Tauri updater plumbing now (even if hosted update server is deferred).
  - Document signing requirements and a “local update feed” demo for acceptance.
- Windows integrations:
  - Media keys (System Media Transport Controls or equivalent)
  - Audio device selection UX polish (clear mode + format display)
  - Optional: tray/miniplayer (only if small; otherwise defer explicitly)
- Performance checkpoints:
  - Cold start → interactive: measure and record (target ≤ 2s)
  - Idle memory with library loaded: measure (target 250–400MB)
  - Scan throughput: record baseline tracks/min
- Release hygiene:
  - “Export diagnostics bundle” (logs + DB stats + config snapshot, no private data unless user opts in)

## Non-scope (Out)

- MSIX Store distribution (optional later; if not done, document why).
- Crash reporting service (unless trivial/local).

## Prerequisites/Dependencies

- All prior milestones (00-10)

## Key Decisions

- **Installer: MSI (WiX) first**
  - Why: standard enterprise-friendly Windows packaging; Tauri supports MSI via WiX on Windows.
  - Alternative: NSIS (also supported) for simpler setup-exe distribution.
- **Updater: implement config hooks now, hosting later**
  - Why: keeps milestone small while ensuring the architecture is ready.

## Deliverables

- Windows installer build configured (MSI and/or NSIS).
- `/docs/release.md`:
  - build steps
  - signing notes
  - update feed strategy (even if “defer hosting”)
- Media key support.
- Diagnostics export.
- UI vision:
  - `/artifacts/ui/11-windows-productization/about-diagnostics.png`
  - `/artifacts/ui/11-windows-productization/REVIEW.md`
- ADR:
  - `/docs/adr/0012-windows-packaging-updates.md`
- Risk register final update.

## Acceptance Criteria

- `tauri build` produces a Windows installer artifact (MSI and/or NSIS) on Windows.
- Installed app launches and plays audio in WASAPI Exclusive (on compatible device).
- Media keys control playback.
- Diagnostics export creates a bundle file with logs + metadata.
- Performance baseline documented and compared to targets.
- Snapshot pack produced.

## Commands (Labeled)

- `cargo tauri build` (Introduced)
- `cargo tauri dev` (Pre-existing)
- `pnpm ui:snapshots -- --milestone 11` (Introduced)

## Verification (Tiered)

- **Tier A — Hardware-agnostic**:
  - `tauri build` succeeds.
  - Installer artifact created.
  - Diagnostics bundle contains expected files.
- **Tier B — Hardware-dependent**:
  - N/A (assuming standard Windows VM availability for general testing).

## Risks & Mitigations

- **Signing / SmartScreen friction** → document required signing steps; allow unsigned dev builds.
- **Update hosting complexity** → defer hosting; provide local feed demo.
- **WebView2 deployment** → follow Tauri guidance on WebView2 installation options (bootstrapper vs fixed) (documented in release notes).

## Tweaks (MUST/SHOULD/MAY)

- **T6.1 (MUST)**: Windows Installer choice documentation.
  - Document that MSI build must run on Windows.
- **T6.2 (MUST)**: Updater key management.
  - Document where keys live (dev vs CI).
  - Document key rotation strategy (or lack thereof).
- **T6.3 (SHOULD)**: WebView2 install mode selection.
  - Decide: Download bootstrapper (small installer, needs internet) vs Offline installer.
  - Document decision in `/docs/release.md`.
- **T6.4 (SHOULD)**: Media keys (SMTC vs plugin).
  - Prefer native SMTC via `windows` crate (consistent with audio stack).
- **T6.5 (MAY)**: Single instance enforcement.
  - Use Tauri Single Instance plugin to prevent double-playback.

## Suggestions

- Consider launching explore agents for codebase pattern discovery
- Verify all acceptance criteria with lsp_diagnostics before completion

## Deferred/Backlog

- MSIX Store distribution.

## Failure Recovery / Resume

- Checkpointing: note last completed deliverable after each major task group.
- Resume: if a session id is provided, use `sisyphus_task(resume="<session_id>", prompt="fix: <specific failure>")`.
- If resume fails: rerun the original task without category wrapper.

## Session Prompt Variables

- `milestone_id`: 11
- `milestone_title`: windows-productization
- `milestone_file`: C:\Obsidian Vaults\Sermon\prompts\Milestone 11 - windows-productization.md
- `special_emphasis`: none
- `dependencies`: All prior milestones (00-10)

## Milestone-specific Notes

### Source Additions

- **Global Definition of Done checklist** (in `/docs/release.md`):
  - Build passes
  - WASAPI Exclusive validated
  - Library scan + search works
  - Tags safe write works
  - UI snapshot packs exist for all milestones
  - Minimal release artifact exists
