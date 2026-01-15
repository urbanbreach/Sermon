# Milestone 01 - library-db-scan

## Goal

- Implement the **local library database + incremental/resumable scanning** of user-selected folders.
- Show a minimal, fast “Tracks” view in the UI driven by real backend data.

## Scope (In)

- Database decision + schema v1:
  - Embedded DB with migrations.
  - Tables (outline):
    - `library_folders` (path, enabled, options)
    - `tracks` (path, stable file identity, basic tags, technical audio info)
    - `albums`, `artists` (minimal denormalization acceptable early)
    - `track_album`, `track_artist` mapping tables (if needed)
    - `scan_state` (last scan timestamps, resumable hints)
- Scanner:
  - Folder selection (UI → backend).
  - Incremental scan: only re-parse files whose identity+mtime/size changed.
  - Resumable: safe to stop/restart; re-walk is OK if it skips unchanged files quickly.
  - Rename/move tolerance on Windows using stable file identity:
    - Use Volume Serial + File ID when available (NTFS) to map the same file even if the path changes.
    - Fallback strategy for volumes/filesystems that don’t provide stable IDs (e.g., some network shares): path+mtime/size and optional lightweight hash.
- Metadata extraction during scan (read-only in this milestone):
  - Title, artist, album, album artist, track/disc numbers, year, genre (best-effort).
  - Technical info: codec/container, sample rate, bit depth, channels, duration.
- UI:
  - Settings: “Add Library Folder” + “Rescan”.
  - Track list view (simple table/list) with basic sort (title/artist/album).
  - Scan progress indicator.

## Non-scope (Out)

- Full “Albums grid” / “Artists view” polish (that’s milestone 04).
- Full-text search (milestone 04).
- Playback (milestone 02).
- Tag writing (milestone 05).

## Prerequisites/Dependencies

- 01: Milestone 00 — foundation

## Key Decisions

- **DB: SQLite (embedded) + migrations**
  - Why: ubiquitous embedded DB; strong indexing; great UX for local libraries; no server.
  - Evidence: Audirvana uses a `.sqlite` database file (e.g., `AudirvanaDatabase.sqlite`).
  - Alternative: `sled` (Rust-native) → weaker query ergonomics/FTS; more custom work.
- **Rust SQLite binding: rusqlite**
  - Why: minimal runtime overhead; straightforward; avoids async DB complexity early.
  - Alternative: sqlx → heavier compile-time/macro + async overhead.
- **Windows rename tolerance:** Volume Serial + File ID identity where available.
  - Alternative: full file hashing → expensive for large libraries.
- **SQLite file location:** `src-tauri` resolves app data dir and passes an explicit DB path to `crates/library` (keep library crate Tauri-free).
- **Migrations approach:** directory-backed raw SQL with schema version recorded in DB.
- **Metadata reader:** `lofty` (read-only, best-effort).
- **Testing strategy:** tests-after for Rust core; manual snapshots for UI (ADR 0001).
- **SERMON_MOCK behavior:** when `SERMON_MOCK=1`, UI uses fixtures only; otherwise use real DB.
- **Missing/deleted files:** mark missing, do not prune in M01.
- **Rescan concurrency:** single-flight; reject or queue one rescan request.
- **Windows long paths:** support `\\?\` prefix where required; store display path separately if needed for UI.

## Deliverables

- DB migrations directory (versioned).
- Schema doc: `/docs/db-schema-v1.md` (outline, indexes, migration strategy).
- Scanner service:
  - progress events → UI
  - incremental behavior verified
- UI updates:
  - “Library Folders” settings section
  - “Tracks” list view
- UI vision:
  - `/artifacts/ui/01-library-db-scan/tracks-empty.png`
  - `/artifacts/ui/01-library-db-scan/scanning.png`
  - `/artifacts/ui/01-library-db-scan/tracks-populated.png`
  - `/artifacts/ui/01-library-db-scan/REVIEW.md`
- ADRs:
  - `/docs/adr/0003-db-sqlite.md`
  - `/docs/adr/0004-file-identity-windows-fileid.md`
- Update `/docs/risk-register.md` (top 5 risks + burn-down note)

### Work Plan (Tasks)

- [x] 1. **[MUST]** Define schema v1 and migrations directory

  **What to do**:
  - Create `crates/library/migrations/0001_init.sql` with required tables and indexes.
  - Define `library_folders` columns: `id` (INTEGER PK), `path` (TEXT UNIQUE), `enabled` (INTEGER), `status` (TEXT), `last_error` (TEXT), `options_json` (TEXT).
  - Define `tracks` columns: `id` (INTEGER PK), `library_folder_id` (INTEGER FK), `path` (TEXT), `path_display` (TEXT), `path_lossy` (INTEGER), `identity_source` (TEXT), `volume_serial` (INTEGER), `file_id` (INTEGER), `mtime_ms` (INTEGER), `size_bytes` (INTEGER), `hash` (TEXT).
  - Define `tracks` metadata columns: `title`, `artist`, `album`, `album_artist`, `track_no`, `disc_no`, `year`, `genre`.
  - Define `tracks` technical columns: `codec`, `container`, `sample_rate`, `bit_depth`, `channels`, `duration_ms`.
  - Define `tracks` missing columns: `is_missing` (INTEGER), `missing_since_ms` (INTEGER).
  - Define `albums` columns: `id` (INTEGER PK), `title` (TEXT), `album_artist` (TEXT), `year` (INTEGER).
  - Define `artists` columns: `id` (INTEGER PK), `name` (TEXT).
  - Define `track_album` mapping table: `track_id` (INTEGER FK), `album_id` (INTEGER FK).
  - Define `track_artist` mapping table: `track_id` (INTEGER FK), `artist_id` (INTEGER FK).
  - Define `scan_state` columns: `folder_id` (FK to `library_folders.id`, UNIQUE), `last_scan_started_ms`, `last_scan_completed_ms`.
  - Default `options_json`: `{ "recursive": true, "include_extensions": [".flac", ".mp3", ".m4a", ".wav", ".ogg"], "exclude_patterns": [], "follow_symlinks": false }`.
  - Required indexes:
    - Unique `tracks` index on `(volume_serial, file_id)` WHERE `identity_source = "ntfs"` and both values are non-null.
    - Unique `tracks` index on `(path, mtime_ms, size_bytes)` WHERE `identity_source = "fallback"`.
    - Index on `tracks.library_folder_id`.
    - Unique composite on `track_album(track_id, album_id)` and `track_artist(track_id, artist_id)`.
  - Record schema version using `PRAGMA user_version` (no schema_migrations table).

  **Must NOT do**:
  - Do not add unrelated columns (playlists, artwork, lyrics, playback state).

  **References**:
  - `prompts/Milestone 01 - library-db-scan.md` (schema requirements).
  - https://www.sqlite.org/pragma.html#pragma_user_version (versioning option).

  **Acceptance Criteria**:
  - [ ] Migration file creates `tracks.path`, `albums`, `artists`, `track_album`, and `track_artist` tables.
  - [ ] Indexes match the specified identity + mapping requirements.

- [x] 2. **[MUST]** Write schema documentation + ADR 0003

  **What to do**:
  - Create `/docs/db-schema-v1.md` with sections: Overview, Tables (per-table columns), Indexes, Migration/versioning, Options JSON semantics.
  - Document `options_json` semantics: `recursive` boolean; `include_extensions` case-insensitive list (empty means use defaults); `exclude_patterns` uses `globset` with `/`-normalized paths (exclude wins over include); invalid globs are logged and ignored; `follow_symlinks` boolean.
  - Write `/docs/adr/0003-db-sqlite.md` in ADR format (context, decision, alternatives).
  - Document DB path ownership: `src-tauri` resolves app data dir, `crates/library` consumes a path.

  **Must NOT do**:
  - Do not document future features (playlists, search, artwork caching).

  **References**:
  - `docs/adr/0001-ui-vision-loop.md` (ADR format).
  - `docs/adr/0002-repo-architecture.md` (architecture context).
  - https://v2.tauri.app/reference/javascript/api/namespacepath/ (app data paths).

  **Acceptance Criteria**:
  - [ ] Schema doc lists all tables and required fields.
  - [ ] ADR includes decision and alternatives.

- [x] 3. **[MUST]** Implement DB layer + migration runner

  **What to do**:
  - Confirm workspace includes `crates/library` (root `Cargo.toml` uses `crates/*`).
  - Add `rusqlite`, `walkdir`, `blake3`, `tracing`, and `globset` to `crates/library/Cargo.toml`.
  - Add `tempfile` as a dev-dependency for `crates/library` tests (temp DB + fixture dirs).
  - Add `library` dependency to `src-tauri/Cargo.toml`.
  - Add `tags` dependency to `crates/library/Cargo.toml` (scanner calls `tags::read_metadata`).
  - Create modules:
    - `crates/library/src/db/mod.rs` (connection + queries)
    - `crates/library/src/db/migrations.rs` (apply SQL files)
    - `crates/library/src/models.rs` (TrackRow, LibraryFolder, ScanState)
  - Embed migrations with `include_str!` in `db/migrations.rs` and call `apply_migrations` during `LibraryState::new` at Tauri startup; `apply_migrations` reads/writes `PRAGMA user_version`.
  - Expose a minimal API from `crates/library/src/lib.rs` (open_db, apply_migrations, list_tracks, upsert_track, set_missing).
  - In `src-tauri`, resolve app data dir via `AppHandle.path().app_data_dir()` and pass `db_path` into library API.
  - Ensure DB directory exists before open.

  **Must NOT do**:
  - Do not introduce async DB layers or additional ORM frameworks.

  **References**:
  - `Cargo.toml` (workspace members already include `crates/*`).
  - `crates/library/Cargo.toml` (add `rusqlite` + `tags`).
  - `crates/tags/Cargo.toml` (tag reader crate).
  - `src-tauri/Cargo.toml` (add `library` dependency).
  - `src-tauri/src/lib.rs` (Tauri app setup entry point).
  - https://docs.rs/tauri/latest/tauri/struct.AppHandle.html#method.path (path resolver).

  **Acceptance Criteria**:
  - [ ] `crates/library/tests/migrations.rs` uses `Connection::open_in_memory()` or `tempfile` DB to cover migration idempotency.
  - [ ] `cargo test` passes for library crate tests.
  - [ ] DB file is created in app data dir and accessible on startup.

- [x] 4. **[MUST]** Implement file identity + ADR 0004

  **What to do**:
  - Create `crates/library/src/identity.rs` with identity extraction.
  - Add `windows` crate to `crates/library/Cargo.toml` with `Win32_Storage_FileSystem` + `Win32_Foundation` features.
  - Implement NTFS identity (Volume Serial + File ID) on Windows using `CreateFileW` (FILE_READ_ATTRIBUTES, share READ|WRITE|DELETE, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS) + `GetFileInformationByHandle`.
  - Implement fallback identity (path + mtime + size + optional lightweight hash).
  - Hash policy: compute blake3 over first 256KB when `identity_source = "fallback"` and `(mtime,size)` changed; store hash; treat hash mismatch as new file; no rename detection beyond path+mtime+size in M01.
  - Add long-path handling (`\\?\` prefix); store lossy display path in `tracks.path_display` with `path_lossy=1` when UTF-8 conversion fails.
  - Write `/docs/adr/0004-file-identity-windows-fileid.md` with fallback strategy.

  **Must NOT do**:
  - Do not hash full file contents for every scan.

  **References**:
  - `docs/risk-register.md` (R004 safe-write vs identity risk).
  - https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew (file handle).
  - https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfileinformationbyhandle (identity).
  - https://docs.rs/windows (Windows API bindings).
  - https://docs.rs/blake3 (optional lightweight hash).

  **Acceptance Criteria**:
  - [ ] `crates/library/tests/identity_windows.rs` (cfg windows) covers NTFS identity; `crates/library/tests/identity_fallback.rs` covers fallback identity.
  - [ ] Rename on NTFS updates path without duplicate row.

- [x] 5. **[MUST]** Implement metadata extraction with `lofty`

  **What to do**:
  - Add `lofty` to `crates/tags`.
  - Create `crates/tags/src/reader.rs` and expose `read_metadata(path)` from `crates/tags/src/lib.rs`.
  - Add fixture strategy: include a tiny CC0 test file (e.g., 1s sine WAV + tags in FLAC) in `crates/tags/tests/fixtures/` and document provenance in `crates/tags/tests/fixtures/README.md`.
  - Ensure parse failures are non-fatal; return partial/unknown values.

  **Must NOT do**:
  - Do not write tags or modify files.

  **References**:
  - https://docs.rs/lofty (metadata extraction).
  - `prompts/Milestone 01 - library-db-scan.md` (metadata scope).

  **Acceptance Criteria**:
  - [ ] Metadata parse errors do not abort a scan.
  - [ ] `crates/tags/tests/fixtures/README.md` documents fixture source/license.
  - [ ] `crates/tags/tests/metadata.rs` validates fixture file yields required fields.

- [x] 6. **[MUST]** Build incremental/resumable scanner service

  **What to do**:
  - Create `crates/library/src/scanner.rs` and wire into `crates/library/src/lib.rs`.
  - Walk enabled folders and apply filters from `options_json` (include_extensions case-insensitive; empty list uses defaults; exclude_patterns uses `globset` over `/`-normalized full paths; exclude overrides include; invalid globs logged and ignored).
  - Define `ScanProgress { scanned, total, errors }` and `ScanSummary { scan_id, scanned, total, skipped, errors, elapsed_ms }` in `crates/library/src/models.rs`.
  - Scanner accepts a progress callback (`scan_folder(path, on_progress)`) invoked with `ScanProgress`.
  - For each eligible file, call `tags::read_metadata` and map fields into `tracks` columns.
  - For scanner tests, copy the metadata fixture into a temp folder and scan that folder.
  - Upsert `albums`/`artists` by exact tag strings; insert mapping rows; keep `tracks.album`/`tracks.artist` for UI (no joins in M01).
  - Skip unchanged files based on identity + mtime + size.
  - Resumable scan: set `scan_state.last_scan_started_ms` on start; if previous scan did not complete, re-walk and rely on skip-unchanged; set `last_scan_completed_ms` on success.
  - Mark missing files (`is_missing=1`, `missing_since_ms`) rather than deleting.
  - When an identity match reappears, clear `is_missing` and `missing_since_ms`.
  - On root access failure, set `library_folders.status = "unavailable"` and `last_error` message; clear to `available` on successful scan; avoid mass-missing.
  - Batch inserts in transactions; avoid decoding audio.
  - Return scan summary (scanned, total, skipped, errors, elapsed_ms) for `evt_scan_complete` and logging.
  - Emit `tracing::info!` logs for `scan_progress` and `scan_complete` (so they appear in `artifacts/logs/sermon.log`).

  **Must NOT do**:
  - Do not run parallel scans or introduce background scheduling.

  **References**:
  - `prompts/Milestone 01 - library-db-scan.md` (scanner requirements).
  - `src-tauri/src/lib.rs` (log file location).
  - https://docs.rs/walkdir/latest/walkdir/ (directory walk).
  - https://docs.rs/globset (exclude pattern matching).

  **Acceptance Criteria**:
  - [ ] `crates/library/tests/scanner.rs` covers skip-unchanged, resume-after-interrupt, rescan without duplicates, missing-file marking, unavailable folder status, and album/artist mapping.
  - [ ] Scan progress logs include `scan_progress scanned=<n> total=<n>` and `scan_complete scanned=<n> total=<n> skipped=<n> errors=<n>` in `artifacts/logs/sermon.log`.

- [x] 7. **[MUST]** Define and implement IPC contract for library

  **What to do**:
  - Define naming rules: commands use `cmd_*`, events use `evt_*` (e.g., `cmd_library_*`, `evt_scan_progress`).
  - Create `src-tauri/src/state.rs` with `LibraryState` (db path + scan mutex) and manage via `app.manage(...)`.
  - LibraryState stores only `db_path`; commands open a new `rusqlite::Connection` per call; scan task opens its own connection (no shared Connection across threads).
  - Create `src-tauri/src/commands/mod.rs` and `src-tauri/src/commands/library.rs`.
  - Update `src-tauri/src/lib.rs` with `.setup(|app| { let db_path = app.path().app_data_dir()?; app.manage(LibraryState::new(db_path)); Ok(()) })` and register commands via `invoke_handler`.
  - Commands accept `State<'_, LibraryState>` and `AppHandle` and return `Result<T, String>`.
  - Example wiring:
    ```rust
    tauri::Builder::default()
        .manage(LibraryState::new(db_path))
        .invoke_handler(tauri::generate_handler![
            cmd_library_add_folder,
            cmd_library_list_folders,
            cmd_scan_start,
            cmd_library_list_tracks,
        ])
        .run(tauri::generate_context!())?;
    ```
  - Run scans on a background task (`tauri::async_runtime::spawn`) and emit progress via `AppHandle.emit` from the progress callback.
  - IPC contract (document in code and update `docs/adr/0002-repo-architecture.md`):
    - `cmd_library_add_folder { path: String } -> { id, path, enabled }`
    - `cmd_library_list_folders -> [{ id, path, enabled }]`
    - `cmd_scan_start { path: String } -> { scan_id }`
      - `cmd_scan_start` resolves `path` to `library_folders`: insert if missing; enable if disabled; return `scan_id` as epoch-ms `u64`.
    - `cmd_library_list_tracks { sort_by: "title"|"artist"|"album", direction: "asc"|"desc" } -> [TrackRow]` (TrackRow sourced from `tracks` table only).
    - `TrackRow` fields: `id`, `title`, `artist`, `album`, `duration_ms`, `sample_rate`, `bit_depth`, `path` (plus optional `album_artist`, `track_no`, `disc_no`, `year`, `genre`, `codec`, `channels`).
    - `evt_scan_progress { scanned: u32, total: u32 }`
      - `total` is the number of eligible files discovered so far during the current walk (dynamic). At scan end, `total = scanned + skipped + errors`.
      - `scanned` is processed count; emit at most 10/sec.
    - `evt_scan_complete { scan_id: u64, scanned: u32, total: u32, skipped: u32, errors: u32, elapsed_ms: u64 }`
      - Emitted once after scan completion to reset UI state.
  - Enforce single-flight scan in command layer: if scan is running, return `Err("scan already running")` and do not queue.
  - If `cmd_scan_start` receives an unknown path, return `Err("folder not found")` and do not emit events. If the path exists but is inaccessible, set status `unavailable` + `last_error` and return `Err("folder unavailable")`.

  **Must NOT do**:
  - Do not expose scanner internals beyond required IPC surface.

  **References**:
  - `docs/adr/0002-repo-architecture.md` (update with naming rules + payloads in this task).
  - `src-tauri/src/lib.rs` (builder location to add invoke_handler).

  **Acceptance Criteria**:
  - [ ] `docs/adr/0002-repo-architecture.md` updated with naming rules + payloads.
  - [ ] UI can invoke `cmd_scan_start` and receive `evt_scan_progress` + `evt_scan_complete` events.
  - [ ] Second scan request while scanning is rejected or queued.

- [x] 8. **[MUST]** Update UI data flow (folders, tracks, progress)

  **What to do**:
  - Add `@tauri-apps/plugin-dialog` to `ui/package.json`.
  - Add `tauri-plugin-dialog` to `src-tauri/Cargo.toml` and call `tauri_plugin_dialog::init()` in `src-tauri/src/lib.rs`.
  - Update `src-tauri/capabilities/default.json` to include `dialog:default` permission.
  - Create `ui/src/lib/types/library.ts` for `TrackRow`, `LibraryFolder`, `ScanProgress` types.
  - Create `ui/src/lib/api/library.ts` wrapping `invoke` calls.
  - Create `ui/src/lib/state/library.ts` store for folders, tracks, scan status with async helpers (`loadFolders`, `loadTracks`, `startScan`).
  - Store shape: `{ folders: LibraryFolder[], selectedFolderId: string | null, tracks: TrackRow[], scan: { status: "idle"|"scanning", scanned: number, total: number } }`.
  - Subscribe to `evt_scan_progress` and `evt_scan_complete` via `@tauri-apps/api/event.listen` inside the store.
  - When `SERMON_MOCK=1`, load fixtures from `ui/src/lib/data/fixtures.ts` / `ui/fixtures/library.json` and bypass IPC.
  - Add Folder flow: `cmd_library_add_folder` → `cmd_scan_start` with returned path; set `selectedFolderId` to the new folder.
  - On load, if `selectedFolderId` is null, default to the first enabled folder; Rescan uses `selectedFolderId` path; if none, show an error.
  - Surface IPC errors in store (e.g., `scan.error`) and display a simple error message.
  - Use store in `TracksView.svelte` and `SettingsView.svelte`.
  - Preserve `SERMON_MOCK=1` fixture-only behavior.

  **Must NOT do**:
  - Do not add search, playback, or complex sorting UI.

  **References**:
  - `ui/src/lib/state/route.ts` (store pattern).
  - `ui/src/lib/views/SettingsView.svelte` (settings layout).
  - `ui/src/lib/views/TracksView.svelte` (tracks list view; uses SERMON_MOCK).
  - `ui/src/lib/data/fixtures.ts` and `ui/fixtures/library.json` (fixture source).
  - `ui/src/App.svelte` (SERMON_SNAPSHOT handling).
  - `src-tauri/capabilities/default.json` (add `dialog:default`).
  - https://v2.tauri.app/plugin/dialog/ (directory picker).

  **Acceptance Criteria**:
  - [ ] Tracks view shows Title/Artist/Album/Duration/Sample rate/Bit depth.
  - [ ] Sorting by Title/Artist/Album produces deterministic ascending order; toggle reverses order.
  - [ ] Rescan with no selected folder shows a visible error message.
  - [ ] Scan errors (folder not found/unavailable) surface in UI error message.
  - [ ] Scan progress indicator updates during scan.

- [x] 9. **[MUST]** Align snapshot workflow with milestone 01

  **What to do**:
  - Update `scripts/snapshot.ps1` with an explicit mapping table:
    - `00` → `artifacts/ui/00-foundation/`
    - `01` → `artifacts/ui/01-library-db-scan/`
  - Update root `package.json` (repo root) `ui:snapshots` script so `pnpm ui:snapshots -- --milestone 01` passes through to the script correctly.
  - Update `scripts/snapshot.ps1` to print milestone 01 filenames (`tracks-empty.png`, `scanning.png`, `tracks-populated.png`) instead of `shell-*.png`.
  - Update `artifacts/README.md` and `docs/adr/0001-ui-vision-loop.md` to list milestone 01 filenames (`tracks-empty.png`, `scanning.png`, `tracks-populated.png`) and replace any `test/snapshots` or `npm run dev` references with the `pnpm ui:snapshots` flow.
  - Add `artifacts/ui/01-library-db-scan/REVIEW.md` template (mirror `artifacts/ui/00-foundation/REVIEW.md` structure: Screenshots checklist, What Changed, Known Issues, Next UI Focus).

  **Must NOT do**:
  - Do not add Playwright automation in M01.

  **References**:
  - `scripts/snapshot.ps1` (current snapshot script).
  - `artifacts/README.md` (snapshot instructions).
  - `docs/adr/0001-ui-vision-loop.md` (manual snapshot workflow).

  **Acceptance Criteria**:
  - [ ] `pnpm ui:snapshots -- --milestone 01` prepares output dir and prints required filenames.
  - [ ] ADR 0001 and `artifacts/README.md` reference the artifacts path + pnpm flow.
  - [ ] Captured images exist in `artifacts/ui/01-library-db-scan/`.

- [x] 10. **[MUST]** Update risk register + perf baseline

  **What to do**:
  - Update `/docs/risk-register.md` top 5 risks + burn-down note.
  - Add `/docs/perf-baseline.md` with measured `tracks/minute`:
    - Record `track_count`, `scan_start_ts`, `scan_end_ts`.
    - Compute `tracks/minute = track_count / minutes_elapsed`.
    - Note dataset size, storage type (SSD), and machine specs.

  **Must NOT do**:
  - Do not optimize performance beyond measurement.

  **References**:
  - `docs/risk-register.md` (existing risks).
  - `prompts/Milestone 01 - library-db-scan.md` (baseline requirement).

  **Acceptance Criteria**:
  - [ ] Risk register updated with current mitigation status.
  - [ ] Perf baseline recorded with dataset size and timing method.

- [x] 11. **[MUST]** Execute verification pass (tests + manual)

  **What to do**:
  - Run `cargo test`.
  - Gate NTFS-specific tests with `#[cfg(target_os = "windows")]`; skip on other platforms.
  - Run `cargo tauri dev` and manually verify folder add + scan + tracks list.
  - Rename a file in NTFS library folder; rescan; verify no duplicate.
  - Run `pnpm ui:snapshots -- --milestone 01` and capture required images.

  **Acceptance Criteria**:
  - [ ] All milestone Acceptance Criteria pass.
  - [ ] Snapshot images exist with required names.

- [x] 12. **[MAY]** Apply SQLite WAL/synchronous pragmas (T8.2)

  **What to do**:
  - If baseline shows need, enable WAL and tune synchronous settings.
  - Document the chosen pragmas and rationale.

  **Must NOT do**:
  - Do not enable pragmas without measuring baseline first.

  **References**:
  - https://www.sqlite.org/pragma.html#pragma_journal_mode (WAL).
  - https://www.sqlite.org/pragma.html#pragma_synchronous (synchronous).

  **Acceptance Criteria**:
  - [ ] Pragmas documented and gated by perf measurement.

## Acceptance Criteria

- User can pick a folder and start a scan.
- Scan populates DB and UI shows at least:
  - Title / Artist / Album / Duration / Sample rate / Bit depth (when available)
- Re-running scan does **not** duplicate tracks.
- Rename test:
  - Rename a file inside the library folder, rescan → DB updates path without duplicating (on NTFS/local).
- Snapshot command outputs required images.

## Commands (Labeled)

- `cargo tauri dev` (Pre-existing)
- `pnpm ui:snapshots -- --milestone 01` (Introduced)
- `cargo test` (scanner + DB unit tests) (Pre-existing)

## Verification (Tiered)

- **Tier A — Hardware-agnostic**:
  - `cargo test` passes (scanner logic).
  - `pnpm ui:snapshots` generates tracks/scanning images.
  - Manual verification of scan behavior (add folder -> see tracks).
- **Tier B — Hardware-dependent**: N/A — not hardware dependent

### Verification Strategy
- Use tests-after: implement features, then add targeted unit tests for migrations, identity, incremental scanning, and missing-file handling.
- UI verification is manual via snapshot workflow (ADR 0001).
- Task-level acceptance criteria are defined in the Work Plan (Tasks) section.

## Risks & Mitigations

- **Network share identity instability** → fallback identity strategy and “library health” UI warnings.
- **Scan performance on huge libraries** → cap concurrency; measure tracks/min; avoid decoding audio during scan.
- **DB locking** → single-writer model; batch inserts in transactions.

## Tweaks (MUST/SHOULD/MAY)

- [MUST] T3 — Safe-write tagging vs stable file identity
  - Source: T3
  - Key points:
    - Ensure scanning logic accounts for file identity stability.
    - Prepare for future safe-write strategies (Milestone 05).
  - Rationale: Fundamental for reliable library management.
  - Plan mapping: Tasks 4 and 6 enforce stable identity + non-dup behavior.

- [MAY] T8.2 — DB pragmas (WAL, synchronous)
  - Source: T8.2
  - Key points:
    - Enable WAL mode.
    - Tune synchronous settings.
  - Rationale: Performance optimization for SQLite.
  - Plan mapping: Task 12 (MAY).

## Suggestions

- Consider launching explore agents for codebase pattern discovery
- Verify all acceptance criteria with lsp_diagnostics before completion

## Deferred/Backlog

- [MAY] T8.2 — DB pragmas (WAL/synchronous). Recommended default: defer until perf baseline is recorded; keep SQLite defaults in M01.

## Failure Recovery / Resume

- Checkpointing: note last completed deliverable after each major task group.
- Resume: if a session id is provided, use `sisyphus_task(resume="<session_id>", prompt="fix: <specific failure>")`.
- If resume fails: rerun the original task without category wrapper.

## Session Prompt Variables

- `milestone_id`: 01
- `milestone_title`: library-db-scan
- `milestone_file`: C:\Obsidian Vaults\Sermon\prompts\Milestone 01 - library-db-scan.md
- `special_emphasis`: none
- `dependencies`: Milestone 00 — foundation

## Milestone-specific Notes

### Source Additions

- **Performance target (initial):** measure and record `tracks/minute` on SSD; set baseline in `/docs/perf-baseline.md`.
