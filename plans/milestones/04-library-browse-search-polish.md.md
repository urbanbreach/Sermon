## Goal

- Upgrade library UX to a polished audiophile-grade browser:
    
    - Albums grid (Roon/Audirvana-inspired)
        
    - Artist view
        
    - Album detail view
        
    - Global fast search
        

## Scope (in)

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
        

## Non-scope (out)

- Smart playlists (explicitly out of scope).
    
- Streaming services (out of scope).
    
- Tag editing (milestone 05).
    
- Artwork fetching (milestone 06; until then use placeholders/embedded art only).
    

## Key decisions (with alternatives + why)

- **Search: SQLite FTS5**
    
    - Why: high performance, embedded, no server; supports MATCH queries and ranking.
        
    - Alternative: external search engine → too heavy for “lightweight desktop app.”
        
- **UI virtualization**
    
    - Prefer a lightweight virtualization approach to keep UI smooth for 10k+ albums.
        

## Dependencies (minimal; justify heavy deps)

- No new heavy deps required; optional small virtualization helper if needed.
    

## Deliverables (concrete artifacts/files/features)

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
            

## Acceptance criteria (pass/fail)

- Albums grid loads and scrolls smoothly on a library with at least several thousand tracks.
    
- Search returns results quickly and does not freeze UI.
    
- Album detail page can “Play Now” and “Add to Queue” reliably.
    
- Snapshot pack produced.
    

## Commands to run (dev/test/build)

- `cargo tauri dev`
    
- `pnpm ui:snapshots -- --milestone 04`
    
- `cargo test`
    

## Risks & mitigations

- **FTS index bloat** → index only needed columns; keep normalization sane.
    
- **UI stutter from huge DOM** → virtualization + pagination.
    

## Additions

- Add a “Library Stats” debug panel (track count, album count, scan last run, DB size).