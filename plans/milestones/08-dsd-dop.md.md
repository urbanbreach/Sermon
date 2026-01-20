## Goal

- Support DSD playback on Windows via **DoP over WASAPI Exclusive** (and clearly document limitations).
    

## Scope (in)

- Library:
    
    - Identify DSF/DFF files and store DSD technical metadata (DSD rate, channels).
        
- Playback:
    
    - DoP packing:
        
        - Implement DoP framing with marker bytes 0x05 / 0xFA (per DoP framing spec).
            
    - Output as PCM stream in Exclusive mode at the appropriate rate (e.g., DSD64 → 176.4k DoP).
        
    - Device capability detection:
        
        - If endpoint doesn’t support needed DoP PCM format, follow policy:
            
            - default: refuse with message (strict)
                
            - optional: “convert DSD to PCM” (explicit opt-in, not default; may be deferred if too big)
                
- UI:
    
    - Show DSD badges in track details and Now Playing.
        
    - Preferences: DoP enable/disable; strict behavior.
        

## Non-scope (out)

- Native DSD via ASIO (milestone 09).
    
- DSP and resampling (must remain explicit opt-in only).
    

## Key decisions (with alternatives + why)

- **DSD strategy (Windows-first): DoP via WASAPI Exclusive**
    
    - Why: avoids reliance on ASIO; many DACs support DoP in exclusive mode.
        
- **No automatic DSD→PCM conversion by default**
    
    - Why: violates “no processing unless explicit.”
        

## Dependencies (minimal; justify heavy deps)

- Prefer minimal DSD parsing code or a small DSF/DFF reader crate; avoid pulling in ffmpeg.
    

## Deliverables (concrete artifacts/files/features)

- DSD metadata in DB.
    
- DoP playback path integrated into audio engine.
    
- `/docs/dsd.md`:
    
    - DoP explanation
        
    - device requirements
        
    - limitations and expected indicators
        
- Tests:
    
    - DoP packer correctness (markers, framing, byte ordering)
        
- UI vision:
    
    - `/artifacts/ui/08-dsd-dop/now-playing-dsd.png`
        
    - `/artifacts/ui/08-dsd-dop/prefs-devices-dsd.png`
        
    - `/artifacts/ui/08-dsd-dop/REVIEW.md`
        
- ADR:
    
    - `/docs/adr/0010-dsd-strategy.md`
        
- Risk register update.
    

## Acceptance criteria (pass/fail)

- On a DoP-capable DAC:
    
    - DSD file plays
        
    - Diagnostics indicates DoP output format
        
    - DAC indicates DSD (where supported)
        
- On non-capable device, app refuses playback in strict mode with clear guidance.
    
- Snapshot pack produced.
    

## Commands to run (dev/test/build)

- `cargo tauri dev`
    
- `pnpm ui:snapshots -- --milestone 08`
    
- `cargo test -p audio-engine`
    

## Risks & mitigations

- **Device compatibility fragmentation** → capability probing + good UX; recommend Shared fallback is not appropriate for DoP.
    
- **DoP framing bugs** → robust unit tests + known-good reference vectors.
    

## Additions

- Add “DSD test track” checklist section to `/docs/dsd.md` (user-provided files; no redistribution).