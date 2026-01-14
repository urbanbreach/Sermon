## Goal

- Implement the **local library database + incremental/resumable scanning** of user-selected folders.
    
- Show a minimal, fast “Tracks” view in the UI driven by real backend data.
    

## Scope (in)

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
        

## Non-scope (out)

- Full “Albums grid” / “Artists view” polish (that’s milestone 04).
    
- Full-text search (milestone 04).
    
- Playback (milestone 02).
    
- Tag writing (milestone 05).
    

## Key decisions (with alternatives + why)

- **DB: SQLite (embedded) + migrations**
    
    - Why: ubiquitous embedded DB; strong indexing; great UX for local libraries; no server.
        
    - Evidence: Audirvana uses a `.sqlite` database file (e.g., `AudirvanaDatabase.sqlite`).
        
    - Alternative: `sled` (Rust-native) → weaker query ergonomics/FTS; more custom work.
        
- **Rust SQLite binding: rusqlite**
    
    - Why: minimal runtime overhead; straightforward; avoids async DB complexity early.
        
    - Alternative: sqlx → heavier compile-time/macro + async overhead.
        
- **Windows rename tolerance:** Volume Serial + File ID identity where available.
    
    - Alternative: full file hashing → expensive for large libraries.
        

## Dependencies (minimal; justify heavy deps)

- `rusqlite` (SQLite access)
    
- A directory walker (small, e.g., `walkdir`)
    
- Tag reading (temporary minimal reader acceptable now; full tagging library chosen in milestone 05)
    

## Deliverables (concrete artifacts/files/features)

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
    

## Acceptance criteria (pass/fail)

- User can pick a folder and start a scan.
    
- Scan populates DB and UI shows at least:
    
    - Title / Artist / Album / Duration / Sample rate / Bit depth (when available)
        
- Re-running scan does **not** duplicate tracks.
    
- Rename test:
    
    - Rename a file inside the library folder, rescan → DB updates path without duplicating (on NTFS/local).
        
- Snapshot command outputs required images.
    

## Commands to run (dev/test/build)

- `cargo tauri dev`
    
- `pnpm ui:snapshots -- --milestone 01`
    
- `cargo test` (scanner + DB unit tests)
    

## Risks & mitigations

- **Network share identity instability** → fallback identity strategy and “library health” UI warnings.
    
- **Scan performance on huge libraries** → cap concurrency; measure tracks/min; avoid decoding audio during scan.
    
- **DB locking** → single-writer model; batch inserts in transactions.
    

## Additions

- **Performance target (initial):** measure and record `tracks/minute` on SSD; set baseline in `/docs/perf-baseline.md`.