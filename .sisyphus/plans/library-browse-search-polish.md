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

### Planning Session Locks (Implementation choices within scope)

- Albums/Artists backing source: **derive from `tracks.album` / `tracks.artist`**.
  - Constraint: treat normalized `albums`/`artists` tables as reserved for later; do not rely on them in M04 queries.
- Search UX: always-visible TopBar input with lightweight dropdown results + keyboard navigation; Enter navigates to full Search Results route.
  - Dropdown guardrail: limit to top ~8–12 results.
- Virtualization approach: add a small dependency.
  - Prefer Virtua (`virtua/svelte`); TanStack Virtual acceptable.
- Pagination UX: infinite scroll / “Load more” (cursor-based preferred).
  - Backend: `LIMIT/OFFSET` is acceptable as a fallback for cases where cursor is impractical, but cursor is preferred.
- Library Stats panel location: add as a “Library Stats” card inside `DiagnosticsView` (debug-only).
- Tests expectation: YES, add/expand Rust tests for FTS + paginated queries.
- Guardrails confirmed: no smart playlists, no streaming services, no tag editing, no artwork fetching beyond placeholders/embedded only.

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

### Snapshot tooling alignment (required for snapshot pack)

Milestone 04 introduces `pnpm ui:snapshots -- --milestone 04`, but the repo currently only supports milestones 00–03.

- Current snapshot script mapping: `scripts/snapshot.ps1:8`
- Current snapshot script invocation: `package.json:11`
- Artifacts instructions: `artifacts/README.md:14`

The executor must update these to support milestone 04 and output to `artifacts/ui/04-library-browse-search-polish/`.

## Acceptance Criteria

- Albums grid loads and scrolls smoothly on a library with at least several thousand tracks.
- Search returns results quickly and does not freeze UI.
- Album detail page can “Play Now” and “Add to Queue” reliably.
- Snapshot pack produced.

### Acceptance clarifications (make verification executable)

- “Several thousand tracks” is validated via Tier A (>=1000 tracks) and, if available, a larger dataset (>=5000) for confidence.
- “Returns quickly” is recorded as a measured value in `docs/perf-baseline.md` (typical search latency) and must have no perceptible UI freeze during typing.

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

### Verification Strategy (MANDATORY)

**Automated (Rust)**

- Use `cargo test` and add/extend tests in `crates/library/tests/`.
  - Existing migration test pattern: `crates/library/tests/migrations.rs:4`

**Manual (UI / App)**

- Use `cargo tauri dev` for interactive verification.
- Use `pnpm ui:snapshots -- --milestone 04` to produce the manual snapshot pack.

### Tier A dummy data (make it reproducible)

Because `ui/fixtures/` is gitignored (`.gitignore:90`) and the current fixture loader hard-imports `../../../fixtures/library.json` (`ui/src/lib/data/fixtures.ts:31`), the executor must ensure Tier A data is reproducible in clean clones.

Choose one primary approach and document it:

- **Approach A (Preferred for UI performance + snapshots): deterministic UI mock generator**
  - Add `SERMON_MOCK_SIZE=<N>` support (default `N=60`, allow `N=1000` and `N=5000`).
  - In `SERMON_MOCK=1`, generate albums/artists/tracks in-memory deterministically (seeded), so:
    - Albums/Artists browse can be virtualized and stress-tested.
    - Search dropdown and Search Results can work in mock mode for snapshot capture.
  - Keep Play Now/Add actions visible but disabled in mock mode (since playback acceptance should be verified against real scanned tracks).

- **Approach B (Optional for backend perf confidence): seed the SQLite DB**
  - Add a debug-only seed mechanism to insert N synthetic `tracks` rows into the SQLite DB for backend pagination + FTS testing.
  - Guard behind `SERMON_DEBUG=1` to avoid “shipping” a seed endpoint.

### Evidence required

- Snapshot images saved to `artifacts/ui/04-library-browse-search-polish/` with exact filenames.
- `docs/perf-baseline.md` updated with recorded cold start time, typical search latency, and idle memory.

## Risks & Mitigations

- **FTS index bloat** → index only needed columns; keep normalization sane.
- **UI stutter from huge DOM** → virtualization + pagination.

### Additional risks (repo reality)

- **Albums/Artists tables exist but are unused by scanner** (`crates/library/migrations/0001_init.sql:51`, `crates/library/src/scanner.rs:200`) → mitigate by deriving Albums/Artists from tracks (locked decision).
  - Evidence: scanner upserts tracks only via `db::upsert_track` (`crates/library/src/scanner.rs:200`) and contains no insert/update paths for `albums`, `artists`, `track_album`, or `track_artist` (string-match search for those table names returns no hits).- **Unpaginated track list today** → mitigate by adding paginated track endpoint and updating UI store/view end-to-end.
  - DB query: `crates/library/src/db/mod.rs:70`
  - Tauri command: `src-tauri/src/commands/library.rs:68`
  - UI call chain: `ui/src/lib/api/library.ts:12` → `ui/src/lib/state/library.ts:45`
- **Snapshot tooling mismatch for milestone 04** → mitigate by updating `package.json`, `scripts/snapshot.ps1`, and `artifacts/README.md`.
- **TopBar drag region blocks input** (`ui/src/lib/components/TopBar.svelte:23`) → mitigate with `no-drag` wrapper around search input + dropdown.
- **FTS5 not available in runtime SQLite build** → mitigate with explicit runtime check + fail-fast message.

## Tweaks (MUST/SHOULD/MAY)

- **T8.2** (MAY): DB pragmas (WAL, synchronous). Consider tuning SQLite pragmas if search performance is suboptimal on lower-end hardware.

## Suggestions

- Consider launching explore agents for codebase pattern discovery
- Verify all acceptance criteria with lsp_diagnostics before completion
- Use frontend UI/UX agent for all design tasks

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

### Project-level numbering mismatch (resolve explicitly)

- Repo `README.md` milestone table currently labels Milestone 04 as “Gapless Playback”, but this plan and prompt define Milestone 04 as **Library Browse + Search + Polish**.
- For this milestone execution, treat `prompts/Milestone 04 - library-browse-search-polish.md` as authoritative.
- Update the README milestone table (or add a clear note) to avoid future snapshot/artifact collisions.

### Data Model Rules (MUST): Derived Albums/Artists (no normalized tables)

These rules prevent the executor from inventing grouping logic.

- **Artist identity**:
  - `artist_display` = `COALESCE(NULLIF(TRIM(tracks.artist), ''), 'Unknown Artist')`
  - `artist_sort` = `LOWER(artist_display)`
  - Artist key for routing/pagination = `artist_sort`.

- **Album identity**:
  - `album_title_display` = `COALESCE(NULLIF(TRIM(tracks.album), ''), 'Unknown Album')`
  - `album_artist_display` = `COALESCE(NULLIF(TRIM(tracks.album_artist), ''), NULLIF(TRIM(tracks.artist), ''), 'Unknown Artist')`
  - `album_title_sort` = `LOWER(album_title_display)`
  - `album_artist_sort` = `LOWER(album_artist_display)`
  - Album key for routing/pagination = tuple `(album_artist_sort, album_title_sort)`.

- **Album year derivation**:
  - `album_year` = `MIN(tracks.year)` ignoring NULLs; if all NULL, show blank/unknown.

- **Album detail track ordering**:
  - ORDER BY `disc_no` ASC (NULL→0), `track_no` ASC (NULL→0), `title_sort` ASC, `id` ASC.
  - Where `title_sort = LOWER(COALESCE(NULLIF(TRIM(title), ''), ''))`.

- **Album detail pagination cursor**:
  - Cursor tuple = `(discNo, trackNo, titleSort, id)`.
  - Keyset filter must compare the full tuple to avoid duplicates/gaps.

- **Missing tracks**:
  - Exclude `is_missing=1` from Play Now / Add to Queue actions by default; optionally render them disabled.

### Routing + Back Behavior (MUST)

Current routing is string-based (`ui/src/lib/state/route.ts:3`). Milestone 04 needs parameters (album key, artist key, search query) and consistent Back behavior.

Choose one approach and implement consistently:

- **Preferred**: convert `Route` to a discriminated union and keep a small back stack.
  - Example shape:
    - `{ name: 'albums' } | { name: 'album-detail', albumArtistSort, albumTitleSort } | ...`
  - Maintain a `routeStack` store (array of previous routes); `navigate()` pushes; `goBack()` pops.
  - Now Playing should return to the previous route, not hard-coded `tracks` (fix existing behavior: `ui/src/lib/views/NowPlayingView.svelte:10`).

### IPC / API Contracts (MUST)

To avoid guesswork, implement these Tauri commands with **explicit** request/response schemas.

**Conventions**

- Use camelCase in JSON payloads.
- Use request structs in Rust with `#[serde(rename_all = "camelCase")]`.
- All paginated responses use `{ items, nextCursor }`.

**Types (TS) (MUST: explicit DTOs, no guesswork)**

All new browse/search DTOs MUST live in `ui/src/lib/types/library.ts:1` (do not introduce `search.ts` in M04).

Add these interfaces/types:

- `export interface AlbumListItem`:
  - `albumTitleDisplay: string`
  - `albumArtistDisplay: string`
  - `albumTitleSort: string`
  - `albumArtistSort: string`
  - `year?: number`
  - `trackCount: number`

- `export interface ArtistListItem`:
  - `artistDisplay: string`
  - `artistSort: string`
  - `trackCount: number`
  - `albumCount: number`

- `export type SearchHit`:
  - Track:
    - `{ type: 'track'; trackId: number; title?: string; artist?: string; album?: string }`
  - Album:
    - `{ type: 'album'; albumArtistSort: string; albumTitleSort: string; albumArtistDisplay: string; albumTitleDisplay: string; year?: number }`
  - Artist:
    - `{ type: 'artist'; artistSort: string; artistDisplay: string }`

- `export interface CursorOffset`:
  - `{ offset: number }`

- `export interface Page<T, C>`:
  - `{ items: T[]; nextCursor?: C }`

- `export interface LibraryStats`:
  - `{ trackCount: number; albumCount: number; artistCount: number; dbSizeBytes: number; lastScanCompletedMs?: number }`

**Rust DTO mirror (MUST)**

Create Rust structs/enums mirroring the above shapes (serde camelCase):

- `AlbumListItem`
- `ArtistListItem`
- `SearchHit` enum (`track` | `album` | `artist`)
- `LibraryStats`

**Commands table**

- `cmd_library_list_tracks_page`
  - Request: `{ sortBy, direction, limit, cursor?: { offset?: number } }`
  - Response: `{ items: TrackRow[], nextCursor?: { offset: number } }`
  - Notes: Use `LIMIT/OFFSET` for Tracks pagination (explicitly allowed fallback) to keep sort-direction behavior simple.

**Cursor encoding**

- Tauri payloads support structured JSON, so cursors are passed as objects (not strings). This avoids fragile parsing/encoding.

**Snapshot CLI contract (MUST be consistent with existing PowerShell param parsing)**

Reality check: `scripts/snapshot.ps1` only accepts `-Milestone <id>` and calling it with `--milestone` fails.

To satisfy the milestone’s command text exactly (`pnpm ui:snapshots -- --milestone 04`), implement argument translation via a small Node wrapper:

- Create `scripts/ui-snapshots.mjs` (new file) that:
  - Parses `process.argv` for `--milestone <id>`.
  - Spawns PowerShell: `pwsh ./scripts/snapshot.ps1 -Milestone <id>`.
  - Exits with the same exit code.
- Update root `package.json:11` to:
  - `"ui:snapshots": "node ./scripts/ui-snapshots.mjs"`
- Keep `scripts/snapshot.ps1` parameter name as `Milestone` to avoid breaking existing usage.
- Update `artifacts/README.md:14` to keep using the stable interface `pnpm ui:snapshots -- --milestone XX`.

Acceptance for this wiring:

- `pnpm ui:snapshots -- --milestone 00` still works.
- `pnpm ui:snapshots -- --milestone 04` works and routes to `scripts/snapshot.ps1 -Milestone 04`.

- `cmd_library_list_albums_page`
  - Request: `{ limit, cursor?: { albumArtistSort, albumTitleSort } }`
  - Response: `{ items: AlbumListItem[], nextCursor?: { albumArtistSort, albumTitleSort } }`

- `cmd_library_list_artists_page`
  - Request: `{ limit, cursor?: { artistSort } }`
  - Response: `{ items: ArtistListItem[], nextCursor?: { artistSort } }`

- `cmd_library_list_album_tracks_page`
  - Request: `{ albumArtistSort, albumTitleSort, limit, cursor?: { discNo, trackNo, titleSort, id } }`
  - Response: `{ items: TrackRow[], nextCursor?: { discNo, trackNo, titleSort, id } }`
  - Notes: This cursor exactly matches the locked ORDER BY tuple for album detail (disc_no, track_no, titleSort, id), preventing duplicates/gaps.

- `cmd_library_list_artist_tracks_page`
  - Request: `{ artistSort, limit, cursor?: { albumTitleSort, discNo, trackNo, titleSort, id } }`
  - Response: `{ items: TrackRow[], nextCursor?: { albumTitleSort, discNo, trackNo, titleSort, id } }`
  - Deterministic ORDER BY (locked): `(album_title_sort, disc_no, track_no, title_sort, id)`.
  - Notes:
    - `album_title_sort = LOWER(COALESCE(NULLIF(TRIM(album), ''), 'Unknown Album'))`
    - `title_sort = LOWER(COALESCE(NULLIF(TRIM(title), ''), ''))`

- `cmd_library_search_suggest`
  - Request: `{ query, limit?: number }` (default 12)
  - Response: `{ results: SearchHit[] }` where `SearchHit` is a discriminated union with routing keys.
  - Deterministic behavior:
    - Apply Search Query Normalization rules.
    - Rank Track hits by `bm25()`.
    - Rank Album/Artist hits by `trackCount` desc then sort key asc.
    - Return at most `limit` results total.

- `cmd_library_search_tracks_page`
  - Request: `{ query, sortBy, direction, limit, cursor?: { offset: number } }`
  - Response: `{ items: TrackRow[], nextCursor?: { offset: number } }`
  - Deterministic ORDER BY (locked): same as Tracks view pagination: `(sortBy, id)`.
  - Note: Search relevance ordering (bm25) is only used for dropdown suggestions, not for paginated search pages.

- `cmd_library_search_albums_page`
  - Request: `{ query, limit, cursor?: { albumArtistSort, albumTitleSort } }`
  - Response: `{ items: AlbumListItem[], nextCursor?: { albumArtistSort, albumTitleSort } }`
  - Deterministic ORDER BY (locked): `(albumArtistSort, albumTitleSort)`.

- `cmd_library_search_artists_page`
  - Request: `{ query, limit, cursor?: { artistSort } }`
  - Response: `{ items: ArtistListItem[], nextCursor?: { artistSort } }`
  - Deterministic ORDER BY (locked): `artistSort`.

- `cmd_library_get_stats`
  - Request: `{}`
  - Response: `{ trackCount, albumCount, artistCount, dbSizeBytes, lastScanCompletedMs? }`
  - Notes:
    - `lastScanCompletedMs` is **MAX across all folders** from `scan_state.last_scan_completed_ms` (`crates/library/migrations/0001_init.sql:80`).

### Search Query Normalization (MUST)

FTS5 `MATCH` will throw errors for malformed query strings. Implement a safe “plain text → MATCH query” transformer.

- Input: user-typed string from TopBar.
- Output: a safe `MATCH` string or “no-op” when empty.

Rules:

1. `q = input.trim()`.
2. If `q` is empty: return no results.
3. Split into tokens by whitespace.
4. For each token:
   - Escape `"` by doubling (FTS phrase escaping): `token = token.replaceAll('"', '""')`.
   - Strip leading/trailing FTS operator characters that are likely user-typed punctuation: `() : * ^`.
   - If token length >= 2: use prefix query: `"{token}"*`.
   - Else: `"{token}"`.
5. Join tokens with `AND`.
6. If SQLite returns an FTS syntax error anyway, catch and return empty results (do not crash UI).

**Album/Artist search semantics (MUST, no guesswork)**

All search endpoints are backed by a tracks FTS table:

- Track hits come from direct FTS match rows.
- Album hits are derived by aggregating track hits to unique album keys.
- Artist hits are derived by aggregating track hits to unique artist keys.

Rules:

- A Track match qualifies for album/artist derivation if the track has non-empty derived `album_title_sort` / `artist_sort`.
- Album hits: group by `(album_artist_sort, album_title_sort)` and return `AlbumListItem` with:
  - `trackCount = COUNT(*)` of matching tracks in that album
  - `year = MIN(year)` ignoring NULLs
- Artist hits: group by `artist_sort` and return `ArtistListItem` with:
  - `trackCount = COUNT(*)` matching tracks
  - `albumCount = COUNT(DISTINCT json_array(album_artist_sort, album_title_sort))` (SQLite-valid distinct key via JSON1)
- Ranking:
  - For dropdown, rank Track hits by `bm25()`; rank Album/Artist hits by `trackCount` desc then sort key asc.
  - For Search Results pages, use deterministic ORDER BY per pagination contracts (not relevance ordering) to ensure stable cursors.

### FTS5 Availability (MUST): expectation + remediation

- Project uses bundled SQLite via `rusqlite` (`crates/library/Cargo.toml:7`). Bundled builds enable FTS5 via `-DSQLITE_ENABLE_FTS5` (libsqlite3-sys build script source for the repo’s lockfile version 0.30.1: https://docs.rs/crate/libsqlite3-sys/0.30.1/source/build.rs).
- Still implement a runtime check:
  - `PRAGMA module_list;` contains `fts5`, OR attempt `CREATE VIRTUAL TABLE temp_fts_check USING fts5(x);`.
- If missing: fail fast with a message that includes remediation (e.g., build with bundled SQLite + FTS5 flags such as `LIBSQLITE3_FLAGS="-DSQLITE_ENABLE_FTS5"`).

### Work Plan TODOs (task-level acceptance criteria)

> Implementation + tests are combined in each task.
> Apply MUST/SHOULD/MAY rubric as labeled.

- [x] 1. **[MUST]** Implement DB migration for FTS5 + supporting indexes

  **What to do**:
  - Add a new migration SQL file (e.g., `crates/library/migrations/0003_fts.sql`).
  - Update migrator `crates/library/src/db/migrations.rs:5` to apply it and advance `PRAGMA user_version`.
  - Create an external-content FTS5 table over `tracks` (`content='tracks'`, `content_rowid='id'`).
  - **FTS table name (locked)**: `tracks_fts`.
  - **FTS columns (locked)**: `title`, `artist`, `album`, `album_artist`, `genre`.
  - **Tokenizer (locked)**: `tokenize="unicode61 remove_diacritics 1"`.
  - **Prefix (locked)**: `prefix='2 3'`.
  - Add triggers for INSERT/UPDATE/DELETE.
  - Backfill/rebuild FTS index for existing libraries (MUST):
    - After creating the FTS virtual table, run the standard rebuild command:
      - `INSERT INTO tracks_fts(tracks_fts) VALUES('rebuild');`
    - Add a Rust test that:
      - inserts a track row into a v2 schema DB,
      - runs migrations to v3,
      - verifies the track is returned by a MATCH query.
  - Implement the milestone-required “supporting indexes” as expression indexes (no schema changes to `tracks`):
    - **Artist sort expression**: `LOWER(COALESCE(NULLIF(TRIM(artist), ''), 'Unknown Artist'))`
    - **Album title sort expression**: `LOWER(COALESCE(NULLIF(TRIM(album), ''), 'Unknown Album'))`
    - **Album artist sort expression**: `LOWER(COALESCE(NULLIF(TRIM(album_artist), ''), NULLIF(TRIM(artist), ''), 'Unknown Artist'))`
    - Create indexes matching browse ORDER BY:
      - Artists browse: `(artist_sort_expr)`
      - Albums browse: `(album_artist_sort_expr, album_title_sort_expr)`

  **Must NOT do**:
  - Do not rely on normalized `albums`/`artists` tables.

  **References**:
  - `crates/library/migrations/0001_init.sql:15` - tracks columns to index.
  - `crates/library/src/db/migrations.rs:5` - migration gating.

  **Acceptance Criteria**:
  - [ ] `cargo test -p library` → PASS.
  - [ ] New/updated test verifies `PRAGMA user_version` increments (pattern: `crates/library/tests/migrations.rs:36`).

- [x] 2. **[MUST]** Add derived Albums/Artists DB queries + pagination

  **What to do**:
  - Implement derived album/artist list queries from tracks using the identity rules above.
  - Implement cursor-based pagination for albums/artists.

  **Must NOT do**:
  - Do not query `albums`/`artists` tables.

  **References**:
  - `crates/library/src/db/mod.rs:52` - existing query module.

  **Acceptance Criteria**:
  - [ ] Rust tests confirm stable ordering and cursor behavior (no duplicates/gaps).

- [x] 3. **[MUST]** Implement paginated Tracks query (replace “load all tracks”)

  **What to do**:
  - Add a paginated `list_tracks_page` to replace unbounded `SELECT *` usage.
  - Expose via `cmd_library_list_tracks_page` per API contracts.

  **Must NOT do**:
  - Must not return full track list for large libraries.

  **References**:
  - Current unpaginated query: `crates/library/src/db/mod.rs:70`.
  - Current Tauri command: `src-tauri/src/commands/library.rs:68`.
  - Current UI call chain: `ui/src/lib/api/library.ts:12` → `ui/src/lib/state/library.ts:45`.

  **Acceptance Criteria**:
  - [ ] `cargo test -p library` → PASS.
  - [ ] Manual: Tracks view loads incrementally (Load more / infinite scroll).

- [x] 4. **[MUST]** Implement FTS-backed search queries (dropdown + full results)

  **What to do**:
  - Implement FTS5-backed search using the query normalization rules above.
  - Provide:
    - `cmd_library_search_suggest` (top 8–12)
    - paginated search endpoints for Tracks/Albums/Artists

  **Must NOT do**:
  - Do not silently fall back to slow LIKE scanning.

  **References**:
  - FTS5 docs: https://www.sqlite.org/fts5.html

  **Acceptance Criteria**:
  - [ ] Rust tests: FTS availability check + search smoke test + pagination determinism.

- [x] 5. **[MUST]** Add new Tauri commands and register them

  **What to do**:
  - Add all commands from the API contracts table to `src-tauri/src/commands/library.rs`.
  - Register in `src-tauri/src/lib.rs:120`.

  **References**:
  - Existing command patterns: `src-tauri/src/commands/library.rs:47`.
  - Handler registration: `src-tauri/src/lib.rs:120`.

  **Acceptance Criteria**:
  - [ ] `cargo test` → PASS.
  - [ ] `cargo tauri dev` starts and UI can call new commands.

- [x] 6. **[MUST]** Update UI routing to support detail + search routes

  **What to do**:
  - Implement the “Routing + Back Behavior” model above.
  - Update:
    - `ui/src/lib/state/route.ts:3`
    - `ui/src/App.svelte:44`
    - nav components (e.g., `ui/src/lib/components/LeftNav.svelte:4`).

  **Acceptance Criteria**:
  - [ ] Album detail and artist detail open and Back returns correctly.
  - [ ] Now Playing Back returns to previous route.

- [x] 7. **[MUST]** Implement global TopBar search input + dropdown + keyboard navigation

  **What to do**:
  - Replace placeholder in `ui/src/lib/components/TopBar.svelte:1`.
  - Add a `no-drag` region wrapper for input + dropdown.
  - Implement dropdown behavior (limit 8–12) and keyboard navigation.

  **Keyboard navigation spec (MUST, fully specified)**
  - State:
    - Dropdown opens when `query.length >= 1` and `results.length >= 1`.
    - Dropdown closes when query becomes empty.
  - `ArrowDown`:
    - If dropdown closed and results exist → open dropdown and set `activeIndex = 0`.
    - Else if `activeIndex == null` → set `activeIndex = 0`.
    - Else → `activeIndex = min(activeIndex + 1, lastIndex)` (no wrap).
  - `ArrowUp`:
    - If dropdown closed and results exist → open dropdown and set `activeIndex = lastIndex`.
    - Else if `activeIndex == null` → set `activeIndex = lastIndex`.
    - Else → `activeIndex = max(activeIndex - 1, 0)` (no wrap).
  - `Enter`:
    - If dropdown open and `activeIndex != null` → navigate to that result.
      - Track result: call Play Now on the selected track.
      - Album result: navigate to album detail route.
      - Artist result: navigate to artist detail route.
    - Else → navigate to Search Results route for current query.
  - `Esc`:
    - If dropdown open → close dropdown and clear `activeIndex`.
    - Else → clear query.
  - Mouse:
    - Hover sets `activeIndex`.
    - Click selects (same as Enter on that item).

  **References**:
  - Drag region: `ui/src/lib/components/TopBar.svelte:23`.

  **Acceptance Criteria**:
  - [ ] Typing does not freeze UI.
  - [ ] Keyboard spec above is implemented exactly (manual verification).

- [x] 8. **[MUST]** Implement Albums browse view (virtualized) + Album detail route

  **What to do**:
  - Replace placeholder albums view with real paginated data.
  - Implement virtualization with Virtua (`virtua/svelte`) or TanStack Virtual.
  - Optional alphabet jump is MAY.

  **References**:
  - Existing albums view: `ui/src/lib/views/AlbumsView.svelte:12`.

  **Acceptance Criteria**:
  - [ ] Smooth scrolling with Tier A dataset (>=1000).

- [x] 9. **[MUST]** Implement Artists browse view (virtualized) + Artist detail route

  **What to do**:
  - Replace placeholder artists view with real paginated data.
  - Implement virtualization.

  **Acceptance Criteria**:
  - [ ] Smooth scrolling with Tier A dataset (>=1000).

- [x] 10. **[MUST]** Implement Album Detail view with track list + Play Now / Add to Queue

  **What to do**:
  - Album detail track list is derived from tracks (no normalized album ID).
  - Implement Play Now/Add to Queue using existing playback calls.

  **References**:
  - Track actions pattern: `ui/src/lib/views/TracksView.svelte:72`.
  - Playback commands: `src-tauri/src/commands/playback.rs:204`.

  **Acceptance Criteria**:
  - [ ] In real (non-mock) mode with a small scanned library, album Play Now works.
  - [ ] Album Add to Queue works.

- [x] 11. **[MUST]** Implement Search Results view (full results) with infinite scroll

  **What to do**:
  - Create Search Results view that loads Artists/Albums/Tracks sections (paginated).

  **Acceptance Criteria**:
  - [ ] Enter navigates here; results load incrementally.

- [x] 12. **[MUST]** Add Library Stats card inside DiagnosticsView

  **What to do**:
  - Add a card in `ui/src/lib/views/DiagnosticsView.svelte:10` showing:
    - track count, derived album count, derived artist count
    - scan last run
    - DB size

  **Acceptance Criteria**:
  - [ ] Values render and update.

- [x] 13. **[MUST]** Update snapshot tooling for milestone 04 and produce snapshot pack

  **What to do**:
  - Update `scripts/snapshot.ps1:8` mapping and expected filenames to include milestone 04.
  - Update `package.json:11` so `pnpm ui:snapshots -- --milestone 04` forwards correctly.
  - Create `artifacts/ui/04-library-browse-search-polish/REVIEW.md`.
  - Capture:
    - `albums-grid.png`
    - `album-detail.png`
    - `artists.png`
    - `search-results.png`

  **Acceptance Criteria**:
  - [ ] Files exist with exact names.

- [x] 14. **[MUST]** Update `docs/perf-baseline.md` with milestone 04 perf notes

  **What to do**:
  - Cold start time: measure from app start to backend log `first_interactive` (`src-tauri/src/lib.rs:84`).
  - Typical search latency: measure from typing to dropdown visible.
  - Idle memory: measure from Windows Task Manager.

  **Acceptance Criteria**:
  - [ ] `docs/perf-baseline.md` contains all three metrics with at least one filled-in sample.

- [x] 15. **[MAY]** Apply SQLite pragma tuning (T8.2) - SKIPPED (not needed)

  **What to do**:
  - Only if perf baseline suggests it.

  **Acceptance Criteria**:
  - [ ] Any pragma changes are documented and justified.

- [x] 16. **[MUST]** Final verification pass

  **What to do**:
  - Run `cargo test`.
  - Run `cargo tauri dev` and verify Acceptance Criteria.
  - Run `pnpm ui:snapshots -- --milestone 04` and produce snapshot pack.

  **Acceptance Criteria**:
  - [ ] All milestone Acceptance Criteria satisfied.

### Source Additions

- Add a “Library Stats” debug panel (track count, album count, scan last run, DB size).
