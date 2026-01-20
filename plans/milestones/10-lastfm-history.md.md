## Goal

- Add **Last.fm** integration (only) and a local play history view—no other accounts or streaming.
    

## Scope (in)

- Last.fm:
    
    - Auth flow (token/session key)
        
    - “Now Playing” updates
        
    - Scrobbling with correct timing thresholds
        
    - Retry queue when offline
        
- DB:
    
    - `play_history` table (track_id, started_at, played_ms, scrobbled_at, lastfm_status)
        
- UI:
    
    - Preferences → Internet → Last.fm connect/disconnect
        
    - History view: Recently played
        
    - Optional “Scrobble status” in Now Playing
        

## Non-scope (out)

- Any other integrations (Spotify, Qobuz, etc.).
    
- User accounts beyond Last.fm.
    

## Key decisions (with alternatives + why)

- **Store play history locally in SQLite**
    
    - Why: already have DB; enables UX without remote dependence.
        

## Dependencies (minimal; justify heavy deps)

- Reuse existing HTTP dependency; avoid adding a second client.
    

## Deliverables (concrete artifacts/files/features)

- Last.fm integration service (backend) + settings storage.
    
- Play history DB table + queries.
    
- UI:
    
    - Last.fm connect UI
        
    - History page
        
- `/docs/lastfm.md` (setup, privacy notes, failure modes)
    
- UI vision:
    
    - `/artifacts/ui/10-lastfm-history/prefs-lastfm.png`
        
    - `/artifacts/ui/10-lastfm-history/history.png`
        
    - `/artifacts/ui/10-lastfm-history/REVIEW.md`
        
- Risk register update.
    

## Acceptance criteria (pass/fail)

- User can connect Last.fm and see connection state persist after restart.
    
- Tracks scrobble after threshold; failures retry.
    
- History view populates with played tracks.
    
- Snapshot pack produced.
    

## Commands to run (dev/test/build)

- `cargo tauri dev`
    
- `pnpm ui:snapshots -- --milestone 10`
    
- `cargo test`
    

## Risks & mitigations

- **Rate limiting / API errors** → backoff + retry queue.
    
- **Privacy** → clear setting + ability to disable scrobbling entirely.
    

## Additions

- Add a small “Scrobble Diagnostics” section in logs for supportability.