# Milestone 04 - library-browse-search-polish

## Goal

- Upgrade library UX to a polished audiophile-grade browser:
  - Albums grid (Roon/Audirvana-inspired)
  - Artist view
  - Album detail view
  - Global fast search

## Scope (In)

- SQLite schema evolution:
  - Add **FTS5** indexes for fast search over track/album/artist text fields.
  - Add supporting indexes (album sort keys, artist sort keys).
- UI routes:
  - Albums (grid with fast scrolling and optional alphabet jump)
  - Album detail (track list, play/add actions)
  - Artists (list/grid, artist detail)
  - Tracks (table)
- Search UX:
  - Global search bar with fast results and keyboard navigation.
- Performance:
  - Backend queries must be paginated; avoid sending entire library to frontend.
  - UI virtualization for large grids/lists (either minimal dependency or lightweight implementation).

## Non-scope (Out)

- Smart playlists (explicitly out of scope).
- Streaming services (out of scope).
- Tag editing (milestone 05).
- Artwork fetching (milestone 06; until then use placeholders/embedded art only).

## Prerequisites/Dependencies

- Milestone 01 — library-db-scan
- Milestone 02 — playback-shared-now-playing

## Key Decisions

- **Search: SQLite FTS5**
  - Why: high performance, embedded, no server; supports MATCH queries and ranking.
  - Alternative: external search engine → too heavy for “lightweight desktop app.”
- **UI virtualization**
  - Prefer a lightweight virtualization approach to keep UI smooth for 10k+ albums.

## Deliverables

- DB migration for FTS tables and triggers (described, not coded here).
- New UI views + navigation:
  - `Albums`, `Album Detail`, `Artists`, `Tracks`, `Search Results`
- UI vision:
  - `/artifacts/ui/04-library-browse-search-polish/albums-grid.png`
  - `/artifacts/ui/04-library-browse-search-polish/album-detail.png`
  - `/artifacts/ui/04-library-browse-search-polish/artists.png`
  - `/artifacts/ui/04-library-browse-search-polish/search-results.png`
  - `/artifacts/ui/04-library-browse-search-polish/REVIEW.md`
- Perf notes:
  - `/docs/perf-baseline.md` updated with:
    - cold start time
    - typical search latency (sample library)
    - memory (idle with library loaded)

## Acceptance Criteria

- Albums grid loads and scrolls smoothly on a library with at least several thousand tracks.
- Search returns results quickly and does not freeze UI.
- Album detail page can “Play Now” and “Add to Queue” reliably.
- Snapshot pack produced.

## Commands (Labeled)

- `cargo tauri dev` (Pre-existing)
- `pnpm ui:snapshots -- --milestone 04` (Introduced)
- `cargo test` (Pre-existing)

## Verification (Tiered)

- **Tier A — Hardware-agnostic**:
  1. Populate library with dummy data (min 1000 tracks).
  2. Verify Album Grid loads and scrolls without visible lag.
  3. Perform global search for unique string, verify result appears instantly.
- **Tier B — Hardware-dependent**: N/A — not hardware dependent

## Risks & Mitigations

- **FTS index bloat** → index only needed columns; keep normalization sane.
- **UI stutter from huge DOM** → virtualization + pagination.

## Tweaks (MUST/SHOULD/MAY)

- **T8.2** (MAY): DB pragmas (WAL, synchronous). Consider tuning SQLite pragmas if search performance is suboptimal on lower-end hardware.

## Suggestions

- Consider launching explore agents for codebase pattern discovery
- Verify all acceptance criteria with lsp_diagnostics before completion

## Deferred/Backlog

- Advanced filtering or smart playlists (MAY).

## Failure Recovery / Resume

- Checkpointing: note last completed deliverable after each major task group.
- Resume: if a session id is provided, use `sisyphus_task(resume="<session_id>", prompt="fix: <specific failure>")`.
- If resume fails: rerun the original task without category wrapper.

## Session Prompt Variables

- `milestone_id`: 04
- `milestone_title`: library-browse-search-polish
- `milestone_file`: C:\Obsidian Vaults\Sermon\prompts\Milestone 04 - library-browse-search-polish.md
- `special_emphasis`: none
- `dependencies`: Milestone 01 — library-db-scan; Milestone 02 — playback-shared-now-playing

## Milestone-specific Notes

- Use frontend UI/UX engineer subagent for design tasks.

### Source Additions

- Add a “Library Stats” debug panel (track count, album count, scan last run, DB size).
