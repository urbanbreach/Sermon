# Milestone 05 - tagging-safe-write-editor

## Goal

- Implement safe **read/write tagging** with a basic in-app tag editor.
- Guarantee lossless behavior for unknown metadata fields (as feasible per container) and safe writes.

## Scope (In)

- Tag I/O library choice and integration:
  - Use **Lofty** for tag read/write across common audio formats.
- Safe write strategy (non-negotiable):
  - Write to temp file → fsync/flush → atomic replace
  - Optional “create backup before write”
- Supported container policy (must document):
  - FLAC, MP3, MP4/M4A, OGG/Vorbis/Opus, WAV, AIFF (best-effort)
  - Document known limitations (e.g., WAV tagging interoperability)
- UI:
  - Track Info panel: “Edit Tags”
  - Editable fields: Title, Artist, Album, Album Artist, Track #, Disc #, Year, Genre
  - Apply/Cancel, and “Write backup” toggle
- DB sync:
  - After tag write, update DB metadata.

## Non-scope (Out)

- Bulk tag editing across thousands of tracks (can be later).
- Advanced tag fields (composer, conductor, classical, etc.) unless trivial.
- Artwork embedding (milestone 06).

## Prerequisites/Dependencies

- Milestone 01 — library-db-scan

## Key Decisions

- **Tags: Lofty**
  - Why: Rust-native tag read/write across multiple formats; avoids per-format bespoke writers.
  - Alternative: format-specific tag libs → higher maintenance, inconsistent behavior.
- **Atomic write helper**
  - Consider using a small “safe write” helper crate (or implement internally) that enforces temp + atomic replace. (Example: `safe-write` exists as a crate concept.)
  - Decision: prefer minimal internal implementation unless crate clearly reduces risk.

## Deliverables

- Tag read/write pipeline.
- `/docs/tagging.md`:
  - supported formats
  - limitations
  - unknown-field preservation policy
  - backup + atomic replace behavior
- Test corpus + acceptance tests:
  - “read → write (change one field) → read” invariants
  - Unknown-field preservation tests (where feasible)
- UI vision:
  - `/artifacts/ui/05-tagging-safe-write-editor/tag-editor.png`
  - `/artifacts/ui/05-tagging-safe-write-editor/tag-editor-dirty.png`
  - `/artifacts/ui/05-tagging-safe-write-editor/REVIEW.md`
- ADR:
  - `/docs/adr/0007-tagging-lofty-safe-write.md`
- Risk register update.

## Acceptance Criteria

- Editing a tag updates the file on disk and updates UI/DB.
- Safe-write behavior confirmed:
  - backup created when enabled
  - no partial file writes (simulate interruption via test harness where possible)
- Unknown metadata not explicitly edited is preserved per documented policy.
- Snapshot pack produced.

## Commands (Labeled)

- `cargo test -p tags` (Pre-existing/Focused)
- `cargo tauri dev` (Pre-existing)
- `pnpm ui:snapshots -- --milestone 05` (Introduced)

## Verification (Tiered)

- **Tier A — Hardware-agnostic**:
  1. Edit metadata for a supported file (e.g., change Title).
  2. Verify file modification time updates and content changes on disk.
  3. Verify previous metadata is preserved (no data loss).
- **Tier B — Hardware-dependent**: N/A — not hardware dependent

## Risks & Mitigations

- **WAV tagging interoperability** → clearly message constraints; default to conservative behavior.
- **User data loss risk** → backups ON by default until proven stable; strong tests.

## Tweaks (MUST/SHOULD/MAY)

- **T3** (MUST): Safe-write tagging vs stable file identity. Ensure that the atomic replacement strategy (write temp -> rename) does not break file identity expectations (like inode tracking if used, though less critical on Windows/standard desktop use) and properly handles file locks.

## Suggestions

- Consider launching explore agents for codebase pattern discovery
- Verify all acceptance criteria with lsp_diagnostics before completion

## Deferred/Backlog

- Bulk editing (MAY).

## Failure Recovery / Resume

- Checkpointing: note last completed deliverable after each major task group.
- Resume: if a session id is provided, use `sisyphus_task(resume="<session_id>", prompt="fix: <specific failure>")`.
- If resume fails: rerun the original task without category wrapper.

## Session Prompt Variables

- `milestone_id`: 05
- `milestone_title`: tagging-safe-write-editor
- `milestone_file`: C:\Obsidian Vaults\Sermon\prompts\Milestone 05 - tagging-safe-write-editor.md
- `special_emphasis`: none
- `dependencies`: Milestone 01 — library-db-scan

## Milestone-specific Notes

### Source Additions

- Add “Show raw tags” (read-only) for debugging (optional but helpful for audiophiles).
