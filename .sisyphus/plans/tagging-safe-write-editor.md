# Milestone 05 - tagging-safe-write-editor

## Goal

- Implement safe **read/write tagging** with a basic in-app tag editor.
- Guarantee lossless behavior for unknown metadata fields (as feasible per container) and safe writes.

## Scope (In)

- Tag I/O library choice and integration:
  - Use **Lofty** for tag read/write across common audio formats.
- Safe write strategy (non-negotiable):
  - Write to temp file → fsync/flush → atomic commit/replace
  - Optional "create backup before write"
- Supported container policy (must document):
  - FLAC, MP3, MP4/M4A, OGG/Vorbis/Opus, WAV, AIFF (best-effort)
  - Document known limitations (e.g., WAV tagging interoperability)
- UI:
  - Track Info panel: "Edit Tags"
  - Editable fields: Title, Artist, Album, Album Artist, Track #, Disc #, Year, Genre
  - Apply/Cancel, and "Write backup" toggle
- DB sync:
  - After tag write, update DB metadata.

## Non-scope (Out)

- Bulk tag editing across thousands of tracks (can be later).
- Advanced tag fields (composer, conductor, classical, etc.) unless trivial.
- Artwork embedding (milestone 06).

## Prerequisites/Dependencies

- Milestone 01 — library-db-scan

## TODOs

- [x] 1. Add tag write API surface in `tags` crate (MUST)
- [x] 2. Implement safe-write helper (temp → flush → atomic commit) + optional backup (MUST)
- [x] 3. Define and document supported container + unknown-field preservation policy (MUST)
- [x] 4. Update library scanning "supported containers" discoverability (SHOULD)
- [x] 5. Add DB update pathway after tag write (MUST)
- [x] 6. Implement UI tag editor flow (MUST)
- [x] 7. Add "Show raw tags" read-only debug view (MAY)
- [x] 8. Add tests + corpus for tag write invariants (MUST)
- [x] 9. Add ADR `0007-tagging-lofty-safe-write` (MUST)
- [x] 10. Update risk register for milestone learnings (MUST)
- [x] 11. Produce UI artifacts + snapshot pack (MUST)

## Final Checklist

- [x] Tag editor edits persist to disk and DB.
- [x] Backup behavior works when enabled.
- [x] Safe write shows no partial writes and fails safely.
- [x] Unknown metadata is preserved per `/docs/tagging.md` policy.
- [x] Snapshot pack produced under `/artifacts/ui/05-tagging-safe-write-editor/`.

## Verification Commands

- `cargo test -p tags`
- `cargo tauri dev`
- `pnpm ui:snapshots -- --milestone 05`

## Status: COMPLETE ✅
