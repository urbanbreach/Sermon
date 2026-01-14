## Goal

- Build the **Preferences** experience (MusicBee-inspired categories) and wire up “audiophile-grade” settings—while keeping all DSP off by default.
    

## Scope (in)

- Settings persistence:
    
    - Decide storage: SQLite `settings` table vs simple JSON store.
        
    - Must support schema evolution/versioning.
        
- Preferences UI categories (minimum):
    
    - General (startup behavior, language later)
        
    - Player (output mode, device, buffer size, preload)
        
    - Now Playing (double-click behavior, queue rules, shuffle modes)
        
    - Library (folders, scan on startup, continuous monitoring toggle)
        
    - Tags (backup policy, write behavior)
        
    - Internet (artwork providers, Last.fm placeholder)
        
    - Devices (DSD/DoP toggles placeholder if not yet)
        
- Wire settings to actual behavior:
    
    - Output mode selection (Shared/Exclusive)
        
    - Buffer size (where applicable)
        
    - Library scan-on-startup
        
- UX:
    
    - Defaults must satisfy **bit-perfect requirements**: Exclusive on, DSP off.
        

## Non-scope (out)

- Full hotkey customization UI (optional later).
    
- Smart playlists.
    

## Key decisions (with alternatives + why)

- **Settings storage: SQLite** (preferred)
    
    - Why: already present; transactional; easy migration with schema versioning.
        
    - Alternative: JSON store → simpler, but needs careful migration strategy.
        
- **Settings model versioning**
    
    - Must have forward migrations; no silent resets.
        

## Dependencies (minimal; justify heavy deps)

- No new heavy deps expected.
    

## Deliverables (concrete artifacts/files/features)

- Preferences UI window with left-nav categories (like MusicBee screenshots).
    
- Settings persistence + migrations.
    
- `/docs/settings.md` (schema, defaults, migration policy)
    
- UI vision:
    
    - `/artifacts/ui/07-preferences-audiophile-settings/prefs-general.png`
        
    - `/artifacts/ui/07-preferences-audiophile-settings/prefs-player.png`
        
    - `/artifacts/ui/07-preferences-audiophile-settings/prefs-library.png`
        
    - `/artifacts/ui/07-preferences-audiophile-settings/prefs-tags.png`
        
    - `/artifacts/ui/07-preferences-audiophile-settings/REVIEW.md`
        
- ADR:
    
    - `/docs/adr/0009-settings-storage.md`
        
- Risk register update.
    

## Acceptance criteria (pass/fail)

- Settings persist across app restart.
    
- Changing output mode/device affects playback (with clear UX if restart required).
    
- Changing library scan settings affects scanning behavior.
    
- Snapshot pack produced.
    

## Commands to run (dev/test/build)

- `cargo tauri dev`
    
- `pnpm ui:snapshots -- --milestone 07`
    
- `cargo test`
    

## Risks & mitigations

- **Settings sprawl** → keep non-functional placeholders clearly labeled and gated.
    
- **Users accidentally disabling bit-perfect** → explain tradeoffs inline; defaults remain strict.
    

## Additions

- Add “Reset to Defaults” (per category) and “Export Diagnostics” placeholder.