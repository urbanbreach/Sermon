# Milestone 07 - preferences-audiophile-settings

## Goal

- Build the **Preferences** experience (MusicBee-inspired categories) and wire up “audiophile-grade” settings—while keeping all DSP off by default. Leverage frontend UI/UX engineer subagent and tauri MCP server for design tasks and UI polish.

## Scope (In)

- Settings persistence:
  - Decide storage: SQLite `settings` table vs simple JSON store.
  - Must support schema evolution/versioning.
- Preferences UI categories (minimum):
  - General (startup behavior, language later)
  - Player (output mode, device, buffer size, preload)
  - Now Playing (double-click behavior, queue rules, shuffle modes)
  - Library (folders, scan on startup, continuous monitoring toggle)
  - Tags (backup policy, write behavior)
  - Internet (artwork providers, Last.fm placeholder)
  - Devices (DSD/DoP toggles placeholder if not yet)
- Wire settings to actual behavior:
  - Output mode selection (Shared/Exclusive, ASIO driver)
  - Buffer size (where applicable)
  - Library scan-on-startup
- UX:
  - Defaults must satisfy **bit-perfect requirements**: Exclusive on, DSP off.

## Non-scope (Out)

- Full hotkey customization UI (optional later).
- Smart playlists.

## Prerequisites/Dependencies

- Milestone 03 — wasapi-exclusive-bit-perfect

## Key Decisions

- **Settings storage: SQLite** (preferred)
  - Why: already present; transactional; easy migration with schema versioning.
  - Alternative: JSON store → simpler, but needs careful migration strategy.
- **Settings model versioning**
  - Must have forward migrations; no silent resets.

## Deliverables

- Preferences UI window with left-nav categories (like MusicBee).
- Settings persistence + migrations.
- `/docs/settings.md` (schema, defaults, migration policy)
- UI vision via available tauri MCP server:
  - `/artifacts/ui/07-preferences-audiophile-settings/prefs-general.png`
  - `/artifacts/ui/07-preferences-audiophile-settings/prefs-player.png`
  - `/artifacts/ui/07-preferences-audiophile-settings/prefs-library.png`
  - `/artifacts/ui/07-preferences-audiophile-settings/prefs-tags.png`
  - `/artifacts/ui/07-preferences-audiophile-settings/REVIEW.md`
- ADR:
  - `/docs/adr/0009-settings-storage.md`
- Risk register update.

## Acceptance Criteria

- Settings persist across app restart.
- Changing output mode/device affects playback (with clear UX if restart required).
- Changing library scan settings affects scanning behavior.
- Snapshot pack produced.

## Commands (Labeled)

- `cargo tauri dev` (Pre-existing)
- `pnpm ui:snapshots -- --milestone 07` (Introduced)
- `cargo test` (Pre-existing)

## Verification (Tiered)

- **Tier A — Hardware-agnostic**:
  1. Open Preferences, change a value (e.g., Output Mode).
  2. Restart application.
  3. Verify value persists.
- **Tier B — Hardware-dependent**: N/A — not hardware dependent

## Risks & Mitigations

- **Settings sprawl** → keep non-functional placeholders clearly labeled and gated.
- **Users accidentally disabling bit-perfect** → explain tradeoffs inline; defaults remain strict.

## Tweaks (MUST/SHOULD/MAY)

- None specific.

## Suggestions

- Consider launching explore agents for codebase pattern discovery
- Verify all acceptance criteria with lsp_diagnostics before completion

## Deferred/Backlog

- Full hotkey customization (MAY).

## Failure Recovery / Resume

- Checkpointing: note last completed deliverable after each major task group.
- Resume: if a session id is provided, use `sisyphus_task(resume="<session_id>", prompt="fix: <specific failure>")`.
- If resume fails: rerun the original task without category wrapper.

## Session Prompt Variables

- `milestone_id`: 07
- `milestone_title`: preferences-audiophile-settings
- `milestone_file`: C:\Obsidian Vaults\Sermon\prompts\Milestone 07 - preferences-audiophile-settings.md
- `special_emphasis`: none
- `dependencies`: Milestone 03 — wasapi-exclusive-bit-perfect

## Milestone-specific Notes

### Source Additions

- Add “Reset to Defaults” (per category) and “Export Diagnostics” placeholder.
