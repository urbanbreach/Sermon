## Goal

- Implement album art pipeline (embedded + fetch + cache + optional embed) and the Apple-like **dynamic liquid-glass theming** that reacts to album art.
    

## Scope (in)

- Artwork sources:
    
    - Read embedded art from file tags (via tagging pipeline).
        
    - Fetch art from iTunes Search API and Deezer API (no accounts).
        
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
            

## Non-scope (out)

- Lyrics fetching (not requested; lyrics UI can remain placeholder).
    
- Streaming services.
    

## Key decisions (with alternatives + why)

- **Avoid heavy image processing in Rust initially**
    
    - Prefer doing palette extraction in frontend canvas (small, fast, avoids `image` crate weight).
        
- **Cache on disk, not in DB blobs**
    
    - Why: avoids DB bloat; easy eviction.
        

## Dependencies (minimal; justify heavy deps)

- Minimal HTTP client (Rust or frontend fetch); choose the lighter option that keeps Rust runtime slim.
    
- No heavyweight image processing crates unless absolutely required.
    

## Deliverables (concrete artifacts/files/features)

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
    

## Acceptance criteria (pass/fail)

- Embedded art displays when available.
    
- Fetching art works for a known album and is cached for subsequent loads.
    
- Dynamic theme changes when changing tracks/albums.
    
- Snapshot pack produced and deterministic (re-running produces identical images).
    

## Commands to run (dev/test/build)

- `cargo tauri dev`
    
- `pnpm ui:snapshots -- --milestone 06`
    
- `cargo test`
    

## Risks & mitigations

- **Provider rate limits / availability** → caching + graceful fallback; allow provider toggles.
    
- **Visual nondeterminism from palette extraction** → snapshot mode forces fixed sampling parameters.
    

## Additions

- Add a “Reduce Motion / Reduce Transparency” toggle (helps perf and accessibility).