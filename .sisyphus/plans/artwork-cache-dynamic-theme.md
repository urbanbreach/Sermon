# Milestone 06 - artwork-cache-dynamic-theme

## Goal

- Implement album art pipeline (embedded + fetch + cache + optional embed) and the Apple-like **dynamic liquid-glass theming** that reacts to album art.

## Scope (In)

- Artwork sources:
  - Read embedded art from file tags (via tagging pipeline).
  - Fetch art from iTunes Search API, fanart.tv and Deezer API (no accounts).
- Caching:
  - Local cache directory with stable keys (album/artist/title match + provider id).
  - DB mapping from album/track → cached art key.
  - Cache size policy (simple; user-configurable later).
- Optional: “Embed art into file” (uses milestone 05 write-back).
- UI:
  - Album grid and Now Playing show artwork.
  - Artwork picker modal (choose among results).
  - **Dynamic theme engine**:
    - Derive accent color + gradient background from current album art.
    - Apply to glass panels (subtle glow/border highlight).
    - Ensure deterministic snapshot behavior (fixed fixture images → fixed theme outputs).

## Non-scope (Out)

- Lyrics fetching (not requested; lyrics UI can remain placeholder).
- Streaming services.

## Prerequisites/Dependencies

- Milestone 01 — library-db-scan
- Milestone 04 — library-browse-search-polish

## Key Decisions

- **Avoid heavy image processing in Rust initially**
  - Prefer doing palette extraction in frontend canvas (small, fast, avoids `image` crate weight).
- **Cache on disk, not in DB blobs**
  - Why: avoids DB bloat; easy eviction.

## Deliverables

- Artwork service (fetch + cache + map to albums).
- UI:
  - artwork picker modal
  - dynamic theming applied across shell
- `/docs/artwork.md` (providers, caching, embedding options, attribution notes)
- UI vision:
  - `/artifacts/ui/06-artwork-cache-dynamic-theme/albums-themed.png`
  - `/artifacts/ui/06-artwork-cache-dynamic-theme/now-playing-themed.png`
  - `/artifacts/ui/06-artwork-cache-dynamic-theme/artwork-picker.png`
  - `/artifacts/ui/06-artwork-cache-dynamic-theme/REVIEW.md`
- ADR:
  - `/docs/adr/0008-artwork-providers-cache.md`
- Risk register update.

### Work Plan (Single Plan)

> Each task includes task-level acceptance criteria.
> Labels:
> - **MUST**: required to satisfy the milestone scope/deliverables/acceptance criteria.
> - **SHOULD**: recommended (risk mitigation / quality), but not strictly required by acceptance criteria.
> - **MAY**: optional.

- [x] MUST-00: Add deterministic UI fixtures for snapshot mode

  **What to do**:
  - Ensure the UI fixture directories exist and are used:
    - `ui/fixtures/library.json` (already present)
    - `ui/fixtures/artwork/` (already present; contains `album*.jpg`)
    so the existing fixture loader resolves correctly:
    - `ui/src/lib/data/fixtures.ts:31` expects `../../../fixtures/library.json`
    - `ui/src/lib/data/fixtures.ts:36` expects `../../../fixtures/artwork/*`
  - Add deterministic theming fixtures and wire them into `ui/fixtures/library.json`:
    - Add `solid-red.png` and `solid-blue.png` under `ui/fixtures/artwork/`.
    - Update at least two fixture albums’ `artworkFile` to reference these (e.g. set album 1 → `solid-red.png`, album 2 → `solid-blue.png`).
  - Fixture file schema (must match `ui/src/lib/data/fixtures.ts:1` types):
    - `artists: Array<{ id: string, name: string }>`
    - `albums: Array<{ id: string, title: string, artistId: string, year: number, trackIds: string[], artworkFile: string }>`
      - Fixture year rule (explicit): use `year: 0` to represent “unknown year” (fixtures must still include the field).
    - `tracks: Array<{ id: string, title: string, albumId: string, artistId: string, durationMs: number, trackNumber: number, discNumber: number }>`
  - Include at least **two** deterministic fixture artwork images for theming:
    - `solid-red.png` (dominant #ff0000)
    - `solid-blue.png` (dominant #0000ff)
  - Deterministic ID mapping (required due to mixed fixture/string IDs vs UI numeric IDs):
    - Decision (this milestone): match existing mock behavior in `ui/src/lib/state/library.ts:49`.
    - Define `fixtureTrackNumericId(trackId: string) = fixture index order` (0-based index in `fixtures.library.json` `tracks[]`).
    - Use this same mapping everywhere mock tracks are exposed (Tracks list, Now Playing seed, queue).
    - Guardrail: do NOT derive numeric IDs by sorting string IDs in this milestone (to avoid noisy snapshot diffs).
  - Ensure snapshot capture can render Albums + Now Playing with artwork without a real DB scan and without any network:
    - **Albums (mock mode)**: implement mock gating at the lowest-friction callsite used by `ui/src/lib/views/AlbumsView.svelte:24`:
      - In `ui/src/lib/api/library.ts`, when `import.meta.env.SERMON_MOCK === '1'`, `listAlbumsPage()` MUST return a fixture-derived `Page<AlbumListItem, AlbumCursor>` instead of invoking `cmd_library_list_albums_page`.
      - Fixture → `AlbumListItem` mapping MUST match the TS type (no ambiguity):
        - `albumTitleDisplay = (album.title && album.title.trim()) || 'Unknown Album'`
        - `albumArtistDisplay = (artist.name && artist.name.trim()) || 'Unknown Artist'`
        - `albumTitleSort = lower(trim(album.title))` with fallback `'unknown album'`
        - `albumArtistSort = lower(trim(artist.name))` with fallback `'unknown artist'`
        - `year = album.year` unless `album.year === 0`, in which case omit `year` (leave it `undefined`)
        - `trackCount = album.trackIds.length`
      - Pagination MUST match backend cursor rules:
        - sort by `(albumArtistSort, albumTitleSort)`
        - if cursor provided, only return items `>(cursor.albumArtistSort, cursor.albumTitleSort)` lexicographically
        - compute `nextCursor` by taking `limit + 1` items and popping last (same as `crates/library/src/db/mod.rs:321`).
    - **Now Playing (mock + snapshot mode)**: seed playback state deterministically in the frontend (no Tauri events required):
      - Deterministic Now Playing seed choice (explicit): use the first fixture track (`tracks[0]` from `ui/fixtures/library.json`) as the seeded track in snapshot mode.
      - In `ui/src/lib/state/playback.ts`, when `import.meta.env.SERMON_MOCK === '1'` and `import.meta.env.SERMON_SNAPSHOT === '1'`, seed stores deterministically (use the real DTO field names from `ui/src/lib/types/playback.ts:9`):
        - `currentTrack` (type `TrackEventData`) must be set to:
          - `id = fixtureTrackNumericId(fixtureTrack.id)` (0-based fixture index)
          - `title = fixtureTrack.title`
          - `artist = Fixtures.getArtist(fixtureTrack.artistId)?.name`
          - `album = Fixtures.getAlbum(fixtureTrack.albumId)?.title`
          - `duration_ms = fixtureTrack.durationMs`
        - `playbackState` to `'playing'`.
        - `durationMs` store to `fixtureTrack.durationMs`; `positionMs` store to `0`.
        - Optional but recommended for stable UI: `queue` store set to empty `[]` and `currentIndex` set to `null`.
      - Deterministic Now Playing artwork lookup (explicit join key):
        - Compute `(albumArtistSort, albumTitleSort)` from seeded `currentTrack`:
          - `albumTitleSort = lower(trim(currentTrack.album || 'Unknown Album'))`
          - `albumArtistSort = lower(trim(currentTrack.artist || 'Unknown Artist'))`
        - Find the fixture album whose derived sorts match these values.
        - Render artwork using fixture `albums[].artworkFile` + `Fixtures.getArtworkPath(artworkFile)`.
      - Snapshot-mode guardrail (single entrypoint, no contradictions): implement snapshot behavior inside `initPlaybackListeners()` itself (because `ui/src/App.svelte:22` calls it unconditionally).
        - If `import.meta.env.SERMON_MOCK === '1'` AND `import.meta.env.SERMON_SNAPSHOT === '1'`, then `initPlaybackListeners()` MUST:
          1. Seed the playback stores exactly as specified above (set `currentTrack`, `playbackState`, `durationMs`, `positionMs`, optionally `queue` and `currentIndex`).
          2. Return immediately.
        - In this snapshot path, `initPlaybackListeners()` MUST NOT:
          - register any `listen(...)` handlers
          - call backend APIs (`api.getVolume()` or `loadOutputSettings()`)
    - **Artwork source for snapshots**:
      - In snapshot mode, the displayed album art MUST come from deterministic fixture artwork files.
      - Transport rule (explicit):
        - Snapshot-mode artwork rendering MUST use **asset URLs** from `Fixtures.getArtworkPath()`.
        - Runtime (non-snapshot) artwork rendering MUST use IPC-bytes (`MUST-03`).
      - Theme engine MUST support reading pixels from:
        - asset URL (snapshot/fixtures)
        - data URL (IPC-bytes) (runtime/cache)
            - Albums snapshot artwork mapping (no new DTO required):
        - In mock/snapshot mode, AlbumsView MUST resolve per-album artwork deterministically without changing `AlbumListItem`:
          - Join key:
            - Compute fixture `albumArtistSort` and `albumTitleSort` using the same rules as the mock `AlbumListItem` mapping.
            - Match `(albumArtistSort, albumTitleSort)` from the `AlbumListItem` to the fixture album.
          - Then use fixture `albums[].artworkFile` and `Fixtures.getArtworkPath(artworkFile)`.
      - Provider search/fetch UI MUST be disabled or no-op in snapshot mode to prevent nondeterminism.
      - In snapshot mode, the artwork picker MUST be populated offline (single source of truth):
        - `cmd_artwork_search_candidates` MUST return the deterministic fixture candidate list.
          - Backend contract (no ambiguity): in snapshot mode, the command returns candidates whose `image_url` values are simple filenames (e.g. `solid-red.png`, `solid-blue.png`).
          - Frontend contract: UI converts these to actual asset URLs by calling `Fixtures.getArtworkPath(image_url)`.
            - This avoids requiring the Rust backend to understand Vite asset URLs, and still keeps a single source of truth for the candidate list.
        - The UI MUST NOT independently create its own candidate list in snapshot mode (to avoid split-brain behavior).

  **Must NOT do**:
  - Do not require any network calls to produce milestone 06 screenshots.

  **References**:
  - `ui/src/lib/data/fixtures.ts:31` - fixture JSON import location.
  - `ui/src/lib/data/fixtures.ts:36` - fixture artwork glob.
  - `ui/src/lib/state/library.ts:7` - `SERMON_MOCK` gating pattern.
  - `ui/src/App.svelte:25` - `SERMON_SNAPSHOT` snapshot-mode class.
  - `scripts/snapshot.ps1:27` - snapshot runner sets `SERMON_MOCK` and `SERMON_SNAPSHOT`.

  **Acceptance Criteria**:
  - [ ] With `SERMON_MOCK=1`, Albums renders a non-empty grid using fixtures.
  - [ ] With `SERMON_MOCK=1` + `SERMON_SNAPSHOT=1`, Now Playing renders a deterministic fixture track with visible artwork.
  - [ ] Snapshot artifacts can be captured without network access.

- [x] MUST-01: Introduce snapshot runner support for Milestone 06

  **What to do**:
  - Update `scripts/snapshot.ps1` to recognize milestone `06`.
  - Add expected screenshot list for milestone 06:
    - `albums-themed.png`
    - `now-playing-themed.png`
    - `artwork-picker.png`
  - Ensure artifacts directory is created at `artifacts/ui/06-artwork-cache-dynamic-theme/`.

  **References**:
  - `scripts/snapshot.ps1:7` - milestone and screenshot mapping patterns.
  - `scripts/ui-snapshots.mjs:15` - CLI wrapper for `--milestone`.
  - `docs/adr/0001-ui-vision-loop.md:18` - manual snapshot loop and flags.
  - `artifacts/README.md:5` - artifacts directory structure.

  **Acceptance Criteria**:
  - [ ] `pnpm ui:snapshots -- --milestone 06` launches successfully.
  - [ ] `artifacts/ui/06-artwork-cache-dynamic-theme/` exists after running.

- [x] MUST-02: Add DB support for artwork cache mapping (and fix migrations version bug)

  **What to do**:
  - Add a new library migration (version 4) with **explicit schema**:
    - Table `artwork_cache_map_album`:
      - `album_artist_sort TEXT NOT NULL`
      - `album_title_sort TEXT NOT NULL`
      - `cache_key TEXT NOT NULL`
      - `mime TEXT NOT NULL`
      - `provider TEXT NOT NULL`
      - `provider_item_id TEXT NOT NULL`
      - `selected_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))`
      - `PRIMARY KEY (album_artist_sort, album_title_sort)`
    - Table `artwork_cache_map_track`:
      - `track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE`
      - `cache_key TEXT NOT NULL`
      - `mime TEXT NOT NULL`
      - `provider TEXT NOT NULL`
      - `provider_item_id TEXT NOT NULL`
      - `selected_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))`
      - `PRIMARY KEY (track_id)`
    - Indexes:
      - `idx_artwork_album_cache_key` on `artwork_cache_map_album(cache_key)`
      - `idx_artwork_track_cache_key` on `artwork_cache_map_track(cache_key)`
  - Add a new migration SQL file:
    - `crates/library/migrations/0004_artwork_cache.sql` with `PRAGMA user_version = 4;`
  - Fix migration bookkeeping in `crates/library/src/db/migrations.rs` so version is correctly advanced after applying `0003_fts.sql`:
    - restore `version = 3;` (currently commented at `crates/library/src/db/migrations.rs:23`).
  - Update migration runner to apply `0004_artwork_cache.sql` when `version < 4` and set `version = 4;`.
  - Add/extend migration tests to prove:
    - idempotency
    - `user_version` reaches 4
    - new tables exist

  **References**:
  - `crates/library/src/db/migrations.rs:20` - current bug: `version = 3` is commented out.
  - `crates/library/src/db/migrations.rs:23` - exact line where `version = 3` must be restored.
  - `crates/library/migrations/0003_fts.sql:57` - migration pattern (`PRAGMA user_version = 3`).
  - `crates/library/tests/migrations.rs:36` - migration idempotency and `user_version` checks.
  - `crates/library/src/db/mod.rs:319` - derived album identity in app (`album_artist_sort`, `album_title_sort`).

  **Acceptance Criteria**:
  - [ ] `cargo test -p library` passes.
  - [ ] Migration test asserts `user_version == 4`.

- [x] MUST-03: Implement on-disk artwork cache with stable keys (and define UI transport + eviction)

  **What to do**:
  - **Ownership decision (locked)**: implement artwork cache + provider fetchers in the **Tauri backend (`src-tauri`)**, because:
    - it already owns `app_data_dir` (`src-tauri/src/lib.rs:93`)
    - it avoids frontend CORS issues
    - it keeps “cache on disk, not DB blobs” centralized
  - Cache root:
    - `{app_data_dir}/artwork-cache/`
  - App data directory access (must be explicit):
    - Extend `LibraryState` in `src-tauri/src/state.rs:7` to also store `app_data_dir: PathBuf` (or store `artwork_cache_dir: PathBuf`).
    - Populate it during `.setup()` in `src-tauri/src/lib.rs:93`, alongside `db_path`.
  - Cache key algorithm (deterministic):
    - `album_key = `${album_artist_sort}||${album_title_sort}` (use derived values consistent with `crates/library/src/db/mod.rs:319`).
    - `source_key = `${provider}:${provider_item_id}`.
    - `cache_key = blake3(album_key + '::' + source_key)` (hex).
  - Cache file layout + lookup rule (no ambiguity):
    - Cache file path is exactly: `{cache_root}/${cache_key}` (NO extension).
  - Cache MIME is stored in DB mapping tables as `mime TEXT NOT NULL`.
  - MIME derivation rules (explicit):
    - For provider-downloaded images: prefer HTTP `Content-Type` header (strip parameters), else infer from `image_url` extension (`.jpg/.jpeg -> image/jpeg`, `.png -> image/png`, `.webp -> image/webp`), else `application/octet-stream`.
    - For embedded images: use Lofty picture mime/type if available, else `application/octet-stream`.
  - When reading, `cmd_artwork_get_*` loads bytes from `{cache_root}/${cache_key}` and uses the DB `mime` value.
    - On corruption/missing file: treat as cache miss and fall back per `MUST-07`.
  - UI transport method (deterministic and simple):
    - **IPC-bytes strategy**: UI calls a Tauri command that returns `{ mime, bytesBase64 }` for a given `cache_key`.
    - UI constructs a `data:` URL for `<img>`.
  - Cache size policy (simple, explicit):
    - Enforce **max total bytes**.
    - Default limit: **256 MB**.
    - Enforcement trigger (no ambiguity): run enforcement **after every successful cache write** (i.e., after downloading or after extracting embedded artwork into cache).
    - Size accounting algorithm:
      - Walk `{app_data_dir}/artwork-cache/` and sum file sizes.
      - If total <= limit: no-op.
      - Else: delete cached files in eviction order until total <= limit.
    - Eviction unit (explicit): eviction operates on **cache keys** (i.e. files), not individual mapping rows.
    - Eviction order (explicit):
      1. Build a list of unique `cache_key`s referenced by either mapping table.
      2. For each `cache_key`, compute `oldest_selected_at = MIN(selected_at)` across all referencing rows in:
         - `artwork_cache_map_track`
         - `artwork_cache_map_album`
      3. Sort by `oldest_selected_at ASC` (oldest first) and evict in that order until total bytes <= limit.
      4. Then delete any remaining unreferenced cache files (not present as `cache_key` in either mapping table) oldest-by-filesystem-mtime.
    - Eviction → DB semantics (explicit and non-contradictory):
      - When evicting a `cache_key`, delete **all** mapping rows (album + track) that reference that `cache_key`, then delete the cache file.
      - Never delete a cache file without first removing all DB references to it.
    - Concurrency/locking (explicit, to avoid eviction-read races):
    - Introduce a single `artwork_cache_lock: parking_lot::Mutex<()>` in Tauri state (match existing `src-tauri/src/state.rs:3` usage).
    - All cache reads (`cmd_artwork_get_bytes`) and writes/evictions MUST hold this lock.
    - This guarantees eviction cannot delete a file while it is being read.
  - Concurrency expectation:
      - If two writes race, the final post-write enforcement run MUST bring size back <= 256 MB.
    - If a DB mapping exists but the cache file is missing/corrupt, treat as cache miss and fall back per `MUST-07`.

  **Must NOT do**:
  - Do not store image blobs in SQLite.
  - Do not implement custom protocol URLs unless IPC-bytes becomes unworkable.

  **References**:
  - `src-tauri/src/lib.rs:93` - app data dir ownership.
  - `src-tauri/Cargo.toml:20` - backend dependency manifest where HTTP + hashing deps will be added.
  - `crates/library/src/db/mod.rs:319` - canonical derived album identity keys.

  **Acceptance Criteria**:
  - [ ] Cache directory is created on demand under app data dir.
  - [ ] Given the same `(album_artist_sort, album_title_sort, provider, provider_item_id)`, the computed `cache_key` is stable across runs.
  - [ ] UI can display a cached image via the chosen IPC-bytes transport.
  - [ ] Cache never grows beyond 256 MB after enforcement.

- [x] MUST-04: Read embedded artwork from file tags (and define embedded→cache flow)

  **What to do**:
  - Extend the tagging pipeline to extract embedded picture bytes (primary embedded artwork) via Lofty.
  - Define embedded artwork handling explicitly:
    - Embedded artwork is read **on-demand** (when UI requests best artwork) using the track’s `path`.
    - Embedded artwork selection rule (deterministic):
      1. Prefer picture type “Cover Front” if present.
      2. Else choose the first picture.
      3. If multiple candidates in step 1, pick the one with largest byte length; ties by lowest index.
    - If embedded artwork exists, write it into the same on-disk artwork cache (`{app_data_dir}/artwork-cache/`) using a deterministic key:
      - `provider = 'embedded'`
      - `provider_item_id = String(track_id)`
      - `album_artist_sort`/`album_title_sort` derived from DB track row
      - cache key algorithm is identical to `MUST-03`.
      - `mime` MUST be set based on the embedded picture type; if unknown, use `application/octet-stream`.
    - DB mapping behavior (explicit and compatible with MUST-07):
      - When embedded artwork is successfully extracted and cached for a track, upsert `artwork_cache_map_track` for that `track_id` with:
        - `provider = 'embedded'`
        - `provider_item_id = String(track_id)`
        - `cache_key` + `mime` set to the cached embedded image
      - `cmd_artwork_get_best_for_track` MUST treat a mapping row with `provider='embedded'` as `source: 'embedded'` (NOT `trackOverride`) and return it without hitting provider fetchers.
  - Expose embedded artwork to the UI using the IPC-bytes strategy (`{ mime, bytesBase64 }`).

  **Must NOT do**:
  - No heavy Rust image processing (no decoding/resizing pipeline in Rust).

  **References**:
  - `crates/tags/src/reader.rs:48` - current metadata reader; extend to include picture extraction API.
    - Required API surface (explicit): add a new struct + function in `crates/tags/src/reader.rs`:
      - `pub struct ArtworkPicture { pub mime: Option<String>, pub picture_type: String, pub bytes: Vec<u8> }`
      - `pub fn read_embedded_pictures(path: &Path) -> Result<Vec<ArtworkPicture>, lofty::error::LoftyError>`
  - `crates/tags/src/lib.rs:4` - re-export the new function from the crate root.
  - `docs/adr/0007-tagging-lofty-safe-write.md:13` - Lofty is the tagging library.
  - `crates/library/src/db/mod.rs:290` - `get_track_by_id()` can be used to obtain the track path.

  **Acceptance Criteria**:
  - [ ] Tier A.1: Embedded art displays when available.
  - [ ] When embedded art exists, no provider/network fetch is performed for that track.

- [x] MUST-05: Implement provider fetchers (iTunes Search API, fanart.tv, Deezer API) with explicit HTTP stack

  **What to do**:
  - Implement provider fetching in **Rust backend** (to avoid browser CORS issues and keep a single networking layer).
  - Execution model (explicit; avoids UI hangs): any command that performs HTTP MUST run inside `tauri::async_runtime::spawn_blocking`.
    - Repo precedent: `src-tauri/src/commands/library.rs:594` uses `spawn_blocking` for DB work.
    - Return model: the `#[tauri::command]` may be `async fn` and `await` the blocking task, or may return immediately and use events; for this milestone, prefer `async fn` that awaits `spawn_blocking` so UI can `await invoke(...)`.
  - HTTP stack (inside the blocking task):
    - Use `reqwest::blocking` with `rustls-tls` for HTTPS GETs.
    - Timeouts (explicit): connect timeout 3s, total request timeout 10s.
    - Retries: NONE (rate limits are mitigated via caching; failures surface to UI).
    - User-Agent: `Sermon/0.0.0`.
  - Provider query strategy:
    - Input: album + album artist (fallback to track artist) + optional year.
    - Return a list of candidates with `{ provider, provider_item_id, image_url, mime_hint?, width?, height? }`.
  - Provider specifics (provider_item_id rules are explicit):
    - **iTunes Search API**:
      - Query the search endpoint.
      - Candidate:
        - `provider_item_id = String(results[0].collectionId)`
        - `image_url = results[0].artworkUrl100` (use exactly as returned; do NOT rewrite sizes)
    - **Deezer API**:
      - Query the album search endpoint.
      - Candidate:
        - `provider_item_id = String(data[0].id)`
        - `image_url = data[0].cover_big` (or `cover_xl`)
    - **fanart.tv**: **NO-KEY ONLY**. If API key is required for all endpoints, skip fanart.tv gracefully and continue.

  **fanart.tv integration rule (resolves scope vs no-key conflict)**:
  - Implement a fanart.tv provider adapter module, but behavior is:
    - If fanart.tv endpoints require an API key, the adapter MUST immediately return “no results” (no error) and emit one log line: `artwork_provider_skipped fanart reason=api_key_required`.
    - This satisfies “provider exists” while respecting “no accounts/no key”.
  - Download selected artwork to disk cache using `MUST-03` cache keys.
  - Cache write MUST record into the appropriate DB mapping table row:
    - `cache_key`
    - `mime`
    - `selected_at` semantics (explicit):
      - On initial insert, `selected_at` defaults to now.
      - On re-selecting the same candidate or hitting cache, update `selected_at = now` so “recently used” items are less likely to be evicted.
  - Derived key algorithm for backend use (copy/paste, no ambiguity):
    - `album_title_sort = lower(trim(track.album))` with fallback `'unknown album'`
    - `album_artist_sort = lower(trim(coalesce(track.album_artist, track.artist)))` with fallback `'unknown artist'`
    - These must match the browse SQL semantics in `crates/library/src/db/mod.rs:321`.

  **Snapshot-mode override (determinism requirement)**:
  - In `SERMON_SNAPSHOT=1`, provider networking MUST be disabled at the backend enforcement point:
    - All commands that would perform HTTP (`cmd_artwork_search_candidates`, `cmd_artwork_select_candidate_for_*`) MUST check `std::env::var("SERMON_SNAPSHOT") == Ok("1".to_string())` (or equivalent) and MUST NOT perform any HTTP.
    - The artwork picker MUST be populated from deterministic fixture candidates (either UI-sourced or command-sourced), and MUST NOT perform HTTP requests.
    - In snapshot mode, selecting a candidate MAY be a no-op (no download) as long as screenshots remain deterministic.

  **Must NOT do**:
  - Do not implement fetching in frontend `fetch()`.
  - Do not add provider authentication.

  **References**:
  - `src-tauri/Cargo.toml:20` - current dependencies (no HTTP client yet; executor adds one).
  - iTunes sample endpoint: `https://itunes.apple.com/search?term=nirvana+nevermind&entity=album&limit=1`.
  - Deezer sample endpoint: `https://api.deezer.com/search/album?q=nirvana%20nevermind&limit=1`.

  **Acceptance Criteria**:
  - [ ] Tier A.2: For a known album without embedded art, Fetch returns at least 1 candidate from iTunes or Deezer.
  - [ ] If fanart.tv is unreachable or requires a key, Fetch still succeeds via other providers.
  - [ ] In snapshot mode, opening the artwork picker shows exactly 2 fixture candidates and performs no network calls.
  - [ ] Selecting a candidate downloads it once and caches it.
    - Verification method (objective): repeat the same selection twice and verify one of:
      - `cmd_artwork_select_candidate_for_album` response includes `cacheHit=true` on the second attempt, OR
      - logs include exactly `artwork_cache_hit` on the second attempt.
    - Logging requirement (explicit): backend MUST emit `artwork_cache_hit` and `artwork_cache_miss` log lines for each candidate selection attempt.

- [x] SHOULD-06: Provider toggles (risk mitigation for rate limits/availability)

  **What to do**:
  - Add a way to disable/enable providers at runtime (initially via settings UI or config flags).

  **References**:
  - `docs/risk-register.md:5` - R001 reduced effects toggle pattern.

  **Acceptance Criteria**:
  - [ ] Providers can be disabled to avoid flaky or rate-limited sources.

- [x] MUST-07: Add backend IPC + frontend API types for artwork (explicit contracts + precedence rules)

  **What to do**:
  - Add new Tauri commands (names are suggestions; executor may adjust to repo conventions):
    - `cmd_artwork_get_best_for_album`:
      - input: `{ albumArtistSort: string, albumTitleSort: string }`
      - output: `{ source: 'trackOverride'|'albumSelection'|'embedded'|'cache'|'none', cacheKey?: string }`
    - `cmd_artwork_get_best_for_track`:
      - input: `{ trackId: number }`
      - output: `{ source: 'trackOverride'|'albumSelection'|'embedded'|'cache'|'none', cacheKey?: string }`
    - `cmd_artwork_get_bytes`:
      - input: `{ cacheKey: string }`
      - output: `{ mime: string, bytesBase64: string }`
    - `cmd_artwork_search_candidates`:
      - input: `{ albumArtist?: string, albumTitle?: string, trackTitle?: string }`
      - output: `{ candidates: Array<{ provider: string, providerItemId: string, imageUrl: string }> }`
    - `cmd_artwork_select_candidate_for_album`:
      - input: `{ albumArtistSort: string, albumTitleSort: string, provider: string, providerItemId: string, imageUrl: string }`
      - output: `{ cacheKey: string, cacheHit: boolean }`
    - `cmd_artwork_select_candidate_for_track` (optional but recommended to match schema):
      - input: `{ trackId: number, provider: string, providerItemId: string, imageUrl: string }`
      - output: `{ cacheKey: string, cacheHit: boolean }`

  - Define “best artwork” precedence rules (no ambiguity):
    - **For track** (`cmd_artwork_get_best_for_track`):
      1. Track mapping (`artwork_cache_map_track.track_id`) if present and file exists
      2. Embedded artwork from that track (via `MUST-04` embedded→cache flow)
      3. Album mapping for that track’s derived album key (if present and file exists)
      4. If none: return `source: 'none'`
    - **For album** (`cmd_artwork_get_best_for_album`):
      1. Album mapping (`artwork_cache_map_album`) if present and file exists
      2. Embedded artwork from a deterministic representative track for that album:
         - choose the lowest `(disc_no_sort, track_no_sort, title_sort, id)` track in that album key using the same ordering semantics as `crates/library/src/db/mod.rs:442`.
      3. If none: return `source: 'none'`
    - If mapping exists but cache file missing: treat as cache miss and proceed to next fallback.

  - Add invoke wrappers + TS DTOs to match.
  - UI rendering rules (no ambiguity):
    - In runtime mode, AlbumsView MUST use `cmd_artwork_get_best_for_album` to obtain a `cacheKey`, then call `cmd_artwork_get_bytes(cacheKey)` to render.
    - Now Playing MUST use `cmd_artwork_get_best_for_track` similarly.
    - `cmd_artwork_get_best_*` MUST NOT return bytes; bytes only come from `cmd_artwork_get_bytes`.
  - Ensure all runtime (non-snapshot) UI image rendering uses IPC-bytes strategy from `MUST-03`.

  **References**:
  - `src-tauri/src/commands/library.rs:214` - serde request/response patterns.
  - `ui/src/lib/api/library.ts:1` - invoke wrapper patterns.
  - `ui/src/lib/types/library.ts:1` - DTO patterns.
  - `crates/library/src/db/mod.rs:442` - album tracks paging entrypoint (ordering includes `disc_no_sort`, `track_no_sort`, `title_sort`, `id`).

  **Acceptance Criteria**:
  - [ ] Albums grid can resolve and render album artwork via `cmd_artwork_get_best_for_album`.
  - [ ] Now Playing can resolve and render track artwork via `cmd_artwork_get_best_for_track`.
  - [ ] Artwork picker can list candidates and persist selection to DB mapping.

- [x] MUST-08: Albums grid shows artwork

  **What to do**:
  - Replace the placeholder art in the albums grid with the resolved artwork.
  - Keep a placeholder when missing.

  **References**:
  - `ui/src/lib/views/AlbumsView.svelte:74` - current `artwork-placeholder`.
  - `artifacts/ui/04-library-browse-search-polish/REVIEW.md:15` - baseline albums view.

  **Acceptance Criteria**:
  - [ ] Album grid displays embedded/cached art when present.

- [x] MUST-09: Now Playing shows artwork

  **What to do**:
  - Replace placeholder in Now Playing with resolved artwork.
  - Ensure changing tracks updates displayed artwork.

  **References**:
  - `ui/src/lib/views/NowPlayingView.svelte:44` - current `art-placeholder`.

  **Acceptance Criteria**:
  - [ ] Changing tracks changes artwork.

- [x] MUST-10: Artwork picker modal (choose among results)

  **What to do**:
  - Add a deterministic UI entry point for the picker (so `artwork-picker.png` is reproducible):
    - Add a "Choose Artwork…" button on the Albums grid card in `ui/src/lib/views/AlbumsView.svelte`.
      - The button must stop propagation so clicking it does not navigate into Album Detail.
      - This avoids needing Album Detail tracks to load in snapshot mode.
    - This button opens the artwork picker modal for that album key `(albumArtistSort, albumTitleSort)`.
  - Implement a modal that lists provider results and allows selection.
  - Picker scope (explicit): selection is **per-album** for this milestone (writes `artwork_cache_map_album`).
    - Track-level overrides (`artwork_cache_map_track`) are supported by the backend schema but the UI can defer exposing per-track overrides.
  - Snapshot-mode picker behavior (explicit):
    - When `SERMON_SNAPSHOT=1`, the picker must show exactly 2 candidates using fixture artwork URLs (`solid-red.png`, `solid-blue.png`) and must not perform HTTP.
    - Source-of-truth rule (no ambiguity; must match MUST-00):
      - In snapshot mode, the backend `cmd_artwork_search_candidates` MUST return exactly two fixture candidates whose `image_url` values are simple filenames: `solid-red.png` and `solid-blue.png`.
      - Frontend MUST map these filenames to asset URLs using `Fixtures.getArtworkPath(image_url)`.
      - Provider ids can still be synthetic like `fixture:red`, `fixture:blue`.
    - Selecting a candidate in snapshot mode may simply update local UI preview state (no download) as long as the screenshot is stable.
  - Persist selection to DB mapping so it becomes the new “best artwork” in runtime mode.

  **References**:
  - `ui/src/lib/components/Modal.svelte:102` - modal component patterns (focus trap, ESC close).
  - `artifacts/ui/05-tagging-safe-write-editor/REVIEW.md:38` - modal usage patterns.

  **Acceptance Criteria**:
  - [ ] Selecting a result updates album art in UI.
  - [ ] Selection persists across app restart.

- [x] MUST-11: Dynamic theme engine (deterministic palette extraction in frontend canvas)

  **What to do**:
  - Implement a theme engine that derives:
    - an **accent color** (single RGB)
    - a **background gradient** (two RGB stops)
    from the currently displayed album art.
  - Deterministic algorithm (no latitude):
    1. Load artwork image into an offscreen `<canvas>`.
    2. Resize/draw into a fixed working size: **48×48**.
    3. Sample pixels on a fixed grid: every 3px (16×16 samples).
    4. Alpha filter (exact): discard pixels where `alphaByte < 204` (i.e., alpha < 0.8).
    5. Relative luminance formula (exact):
       - `lum = 0.2126*R + 0.7152*G + 0.0722*B` where R/G/B are in `[0,1]` from sRGB bytes/255 (no linearization).
       - discard pixels with `lum < 0.05` or `lum > 0.95`.
    6. RGB→HSV conversion (exact, all inputs in bytes 0..255):
       - `r = R/255`, `g = G/255`, `b = B/255`
       - `max = max(r,g,b)`, `min = min(r,g,b)`, `delta = max-min`
       - `V = max`
       - `S = (max == 0) ? 0 : (delta / max)`
       - `H`:
         - if `delta == 0`: `H = 0`
         - else if `max == r`: `H = 60 * (((g-b)/delta) % 6)`
         - else if `max == g`: `H = 60 * (((b-r)/delta) + 2)`
         - else: `H = 60 * (((r-g)/delta) + 4)`
         - if `H < 0`: `H += 360`
    7. Hue histogram:
       - bucket count: 36
       - `bucket = floor(H / 10)`
    8. Pick dominant bucket by count; if tie, pick higher mean `S`; if still tied, pick lower bucket index.
    9. Accent RGB computation (exact):
       - Compute mean of original RGB bytes for pixels in dominant bucket:
         - `accentR = round(sumR / n)`, same for G/B.
    10. Saturation clamp (exact, no latitude):
       - After step 9 computes `accent` as averaged RGB bytes, recompute `H,S,V` from that **accent RGB** using the exact RGB→HSV conversion in step 6.
       - If `S > 0.85`, set `S = 0.85` (keep `H` and `V` from the recomputed accent HSV).
       - Recompute the accent RGB via the HSV→RGB conversion in step 11.
    11. HSV→RGB conversion (exact):
       - `C = V*S`
       - `X = C*(1 - abs(((H/60) % 2) - 1))`
       - `m = V - C`
       - Choose `(r1,g1,b1)` based on sector:
         - 0<=H<60: (C,X,0)
         - 60<=H<120: (X,C,0)
         - 120<=H<180: (0,C,X)
         - 180<=H<240: (0,X,C)
         - 240<=H<300: (X,0,C)
         - 300<=H<360: (C,0,X)
       - `R = round((r1+m)*255)` (same for G/B), clamp 0..255.
    12. Gradient stops (exact): multiply RGB bytes by factors and round:
       - stop A = `round(accent * 0.65)`
       - stop B = `round(accent * 0.30)`
    13. Empty-sample fallback (exact):
       - If no pixels remain after filtering, set:
         - `accent = (74, 175, 255)`
         - stop A = `(24, 32, 44)`
         - stop B = `(10, 10, 10)`
  - Apply theme via CSS variables at shell level (explicit integration point):
    - Implement a small frontend module (suggested path): `ui/src/lib/theme/dynamicTheme.ts` that exports:
      - `computeThemeFromImageSrc(src: string): Promise<{ accent: [number,number,number], bg0: [number,number,number], bg1: [number,number,number] }>`
      - `applyThemeToDocument(theme): void` (writes CSS variables)
    - Wire this module from a **single source of truth** to avoid nondeterminism:
      - Theme MUST be driven only by Now Playing’s current track artwork (i.e. `currentTrack` changes in `ui/src/lib/state/playback.ts`).
      - Albums grid MUST NOT drive global theme (because many images render and decode in nondeterministic order).
      - For the `albums-themed.png` snapshot: ensure snapshot mode seeds Now Playing with a deterministic fixture track (MUST-00) and ensure the app is navigated such that Now Playing is visible or otherwise the theme is applied before capturing the Albums screenshot.
    - CSS variable names (locked for this milestone; avoids guessing):
      - `--theme-accent`
      - `--theme-bg-0`
      - `--theme-bg-1`
  - Apply to glass panels:
    - subtle border highlight using `--theme-accent` with low alpha
    - optional glow/blur effects controlled by toggles in `MUST-12`
  - Snapshot determinism requirements:
    - In `SERMON_SNAPSHOT=1`, theme computation MUST run only after the artwork image is fully decoded.
      - Required mechanism: call `await img.decode()` (or equivalent) before drawing to canvas.
    - Theme recomputation trigger (explicit): recompute whenever the artwork `src` changes (asset URL in snapshot mode, data URL in runtime mode).
    - Theme engine output MUST be written to CSS variables on `document.documentElement`.

  **References**:
  - `ui/src/App.svelte:25` - existing snapshot-mode body class.
  - `ui/src/app.css:16` - glass CSS tokens to extend via theme variables.
  - `docs/risk-register.md:6` - R002 nondeterminism mitigation.

  **Acceptance Criteria**:
  - [ ] With fixture art `solid-red.png`, computed `--theme-accent` satisfies:
    - `R >= 200`, `G <= 60`, `B <= 60`.
  - [ ] With fixture art `solid-blue.png`, computed `--theme-accent` satisfies:
    - `B >= 200`, `R <= 60`, `G <= 60`.
  - [ ] Tier A.3 passes: theme colors update to match displayed artwork.
  - [ ] Re-running milestone 06 snapshots yields identical images.

- [x] MUST-12: Add Reduce Motion / Reduce Transparency + toggleable theme effects

  **What to do**:
  - Add a “Reduce Motion / Reduce Transparency” toggle (helps perf and accessibility).
  - Add toggleable effects for dynamic theme (blur, glow, border highlight).
  - Persist settings using the existing `settings` table pattern.

  **Settings keys (exact)**:
  - `ui.reduce_effects` = `on|off`
  - `ui.theme.blur` = `on|off`
  - `ui.theme.glow` = `on|off`
  - `ui.theme.border_highlight` = `on|off`

  **Backend contract (must exist)**:
  - Add Tauri commands (names are suggestions; executor may align with existing `cmd_output_*` patterns):
    - `cmd_settings_get(key: string) -> { value?: string }`
    - `cmd_settings_set(key: string, value: string) -> void`
  - These commands MUST use `library::get_setting` / `library::set_setting` on the existing DB.

  **Frontend contract (must exist)**:
  - Add a small settings store that:
    - loads the four keys on app start
    - updates DB on toggle changes
    - applies effects by toggling a body class and/or CSS variables

  **Objective effect rules (verifiable)**:
  - When `ui.reduce_effects=on`:
    - `--glass-blur` MUST be set to `0px` (or nearest equivalent) for all panels.
    - Glow/border highlight MUST be disabled regardless of individual toggles.
  - When `ui.reduce_effects=off`:
    - individual toggles control blur/glow/border highlight independently.

  **References**:
  - `docs/risk-register.md:5` - R001 mitigation suggests reduced effects toggle.
  - `crates/library/src/db/mod.rs:229` - `get_setting()` helper.
  - `crates/library/src/db/mod.rs:239` - `set_setting()` helper.
  - `crates/library/migrations/0002_settings.sql:1` - settings table schema.
  - `ui/src/lib/views/SettingsView.svelte:160` - existing Theme section placeholder to extend.

  **Acceptance Criteria**:
  - [ ] When `ui.reduce_effects=on`, computed `--glass-blur` is `0px` and no glow is visible in screenshots.
  - [ ] When `ui.reduce_effects=off`, blur/glow/border-highlight toggles each change the UI.
  - [ ] Settings persist across restart (read from DB and applied in UI).

- [x] MAY-13: “Embed art into file” (Milestone 05 write-back)

  **What to do**:
  - Add explicit UI action to embed selected artwork into the file.
  - Reuse safe write strategy from milestone 05.

  **References**:
  - `docs/adr/0007-tagging-lofty-safe-write.md:31` - safe write algorithm.

  **Acceptance Criteria**:
  - [ ] Embedded artwork persists after restart (file-level, not just cached).

- [x] MUST-14: Produce UI snapshot pack + review pack for Milestone 06

  **What to do**:
  - Run `pnpm ui:snapshots -- --milestone 06`.
  - Capture screenshots:
    - `albums-themed.png`
    - `now-playing-themed.png`
    - `artwork-picker.png`
  - Confirm determinism by running twice and comparing hashes.
  - Write `artifacts/ui/06-artwork-cache-dynamic-theme/REVIEW.md` following existing format.

  **References**:
  - `artifacts/README.md:5` - capture requirements (1440x900, 100% scale).
  - `docs/adr/0001-ui-vision-loop.md:18` - snapshot loop.

  **Acceptance Criteria**:
  - [ ] All required artifacts exist under `artifacts/ui/06-artwork-cache-dynamic-theme/`.
  - [ ] PowerShell `Get-FileHash` results match between run 1 and run 2 for each PNG.

- [x] MUST-15: Docs + ADR + risk register update

  **What to do**:
  - Create `docs/artwork.md` covering:
    - providers, caching, embedding option
    - attribution notes
    - deterministic theme snapshot notes
  - Create `docs/adr/0008-artwork-providers-cache.md` documenting the key decisions.
  - Update `docs/risk-register.md` with milestone 06 risks.

  **References**:
  - `docs/risk-register.md:3` - risk register table format.
  - `docs/adr/0007-tagging-lofty-safe-write.md:1` - ADR writing style.

  **Acceptance Criteria**:
  - [ ] `docs/artwork.md` exists.
  - [ ] `docs/adr/0008-artwork-providers-cache.md` exists.
  - [ ] `docs/risk-register.md` includes updated provider + determinism risks.

## Acceptance Criteria

- Embedded art displays when available.
- Fetching art works for a known album and is cached for subsequent loads.
- Dynamic theme changes when changing tracks/albums.
- Snapshot pack produced and deterministic (re-running produces identical images).

## Commands (Labeled)

- `cargo tauri dev` (Pre-existing)
- `pnpm ui:snapshots -- --milestone 06` (Introduced)
- `cargo test` (Pre-existing)

## Verification (Tiered)

- **Tier A — Hardware-agnostic**:
  1. Load track with known embedded art, verify display.
     - This is satisfied when `cmd_artwork_get_best_for_track` returns `source: 'embedded'` or `source: 'cache'` with bytes that render.
  2. Load track without art, use "Fetch" to find artwork from provider.
     - This is satisfied when `cmd_artwork_search_candidates` returns at least 1 candidate from iTunes or Deezer.
  3. Verify application theme colors update to match displayed artwork.
     - This is satisfied when CSS variables (`--theme-accent`, `--theme-bg-0`, `--theme-bg-1`) change when switching between two different artworks.
- **Tier B — Hardware-dependent**: N/A — not hardware dependent

### Verification Strategy

  - **Automated tests (Rust)**:
  - Run `cargo test`.
  - Add/extend tests in `crates/library/tests/*` for DB migrations only (migrations live in `crates/library`).
  - Add/extend tests in `crates/tags/tests/*` for embedded picture extraction (tags logic lives in `crates/tags`).
    - Test fixture strategy (must match repo policy in `crates/tags/tests/fixtures/README.md:8`): do NOT commit binary audio fixtures.
      - Generate an on-disk test file inside a temp dir during the test.
      - If embedding artwork into a WAV is not feasible via Lofty, then:
        - Add the embedded-art extraction test as a minimal integration test that only asserts “no panic + returns no pictures” for WAV, OR
        - Switch the generated container to one Lofty can write embedded pictures for (executor chooses, but must be deterministic and generated on-the-fly).
    - Test assertions (explicit):
      - Extraction returns either 0 pictures (for formats without embedded art) or >=1 picture with non-empty bytes.
      - If >=1 picture returned, MIME must be a concrete `image/*` or `application/octet-stream` per MIME rules.
  - Cache key logic lives in `src-tauri`; verify determinism via command-level tests (if present) or via manual QA:
    - run the same `cmd_artwork_select_candidate_for_album` twice and confirm `cacheHit=true` on the second run (or `artwork_cache_hit` log line).

- **Manual QA (UI + determinism)**:
  - Run `pnpm ui:snapshots -- --milestone 06`.
  - Ensure **no network is required** to produce screenshots:
    - run once with the machine offline (or with providers disabled) and verify the app still renders fixture artwork + theme.
  - Capture all required screenshots into `artifacts/ui/06-artwork-cache-dynamic-theme/`.
  - Determinism check:
    - Run the snapshot command twice.
    - Compute hashes for each screenshot with PowerShell `Get-FileHash`.
    - Confirm hashes match between runs.

## Risks & Mitigations

- **Provider rate limits / availability** → caching + graceful fallback; allow provider toggles.
- **Visual nondeterminism from palette extraction** → snapshot mode forces fixed sampling parameters.

## Tweaks (MUST/SHOULD/MAY)

- **T8.4** (MAY): Artwork attribution note. Ensure UI or docs credit the artwork providers (iTunes/Deezer) if their terms require it.

## Suggestions

- Consider launching explore agents for codebase pattern discovery
- Verify all acceptance criteria with lsp_diagnostics before completion
- Use frontend UI/UX engineer subagent to design and polish the UI/UX.
- Use the installed Tauri MCP server to view and edit the UI/UX.

## Deferred/Backlog

- Lyrics fetching (MAY).

## Failure Recovery / Resume

- Checkpointing: note last completed deliverable after each major task group.
- Resume: if a session id is provided, use `sisyphus_task(resume="<session_id>", prompt="fix: <specific failure>")`.
- If resume fails: rerun the original task without category wrapper.

## Session Prompt Variables

- `milestone_id`: 06
- `milestone_title`: artwork-cache-dynamic-theme
- `milestone_file`: C:\Obsidian Vaults\Sermon\prompts\Milestone 06 - artwork-cache-dynamic-theme.md
- `special_emphasis`: none
- `dependencies`: Milestone 01 — library-db-scan; Milestone 04 — library-browse-search-polish

## Milestone-specific Notes

### Source Additions

- Add a “Reduce Motion / Reduce Transparency” toggle (helps perf and accessibility).
- Toggleable effects for the dynamic theme (e.g. blur, glow, border highlight).
