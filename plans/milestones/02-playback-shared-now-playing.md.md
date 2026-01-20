## Goal

- Deliver the first end-to-end experience: **scan → click track → hear audio**, using **WASAPI Shared** as the initial output backend (non-bit-perfect fallback path).
    
- Implement a minimal audio engine architecture that can later swap in WASAPI Exclusive and ASIO.
    

## Scope (in)

- Audio decode:
    
    - Use Symphonia for demux/decode of common formats (lossless + lossy). Symphonia supports multiple codecs/containers including FLAC/WAV/MP3/AAC/ALAC/Vorbis and common containers.
        
- Audio output:
    
    - WASAPI Shared mode output via `wasapi` crate (simple path first).
        
    - Device selection: default output device + ability to choose another.
        
- Playback controls:
    
    - Play/pause/stop
        
    - Seek (basic)
        
    - Next/previous (queue-based)
        
    - Queue: “Play Now” and “Add to Queue”
        
- UI:
    
    - Now Playing bar becomes functional
        
    - Minimal Now Playing screen:
        
        - album art placeholder (real art later)
            
        - track details
            
        - timeline scrubber
            
        - queue panel (simple)
            

## Non-scope (out)

- WASAPI Exclusive (milestone 03).
    
- Bit-perfect validation (milestone 03).
    
- DSD (milestone 08).
    
- DSP features (off-scope unless explicitly opt-in later).
    

## Key decisions (with alternatives + why)

- **Decode: Symphonia (pure Rust)**
    
    - Why: broad common codec/container support, avoids bundling ffmpeg/libav; keeps runtime dependencies minimal.
        
    - Alternative: ffmpeg → heavy binary distribution, larger surface area.
        
- **Output (for this milestone): WASAPI Shared**
    
    - Why: easiest functional baseline and broad compatibility; explicitly labeled as non-bit-perfect fallback.
        
- **Output API wrapper: wasapi crate**
    
    - Why: provides safe Rust wrappers and supports shared/exclusive, event-driven/polling, etc.
        

## Dependencies (minimal; justify heavy deps)

- `symphonia` (decode)
    
- `wasapi` + `windows` (audio output; Windows-only)
    

## Deliverables (concrete artifacts/files/features)

- `crates/audio-engine/` (core playback state machine + decode + output abstraction).
    
- Tauri commands to:
    
    - start playback (track id)
        
    - pause/resume/seek
        
    - queue management
        
    - list/select output device
        
- UI:
    
    - Now Playing view + queue panel (functional)
        
- UI vision:
    
    - `/artifacts/ui/02-playback-shared-now-playing/now-playing-idle.png`
        
    - `/artifacts/ui/02-playback-shared-now-playing/now-playing-playing.png`
        
    - `/artifacts/ui/02-playback-shared-now-playing/queue.png`
        
    - `/artifacts/ui/02-playback-shared-now-playing/REVIEW.md`
        
- ADR:
    
    - `/docs/adr/0005-audio-decode-symphonia.md`
        
    - `/docs/adr/0006-audio-output-wasapi.md`
        
- Risk register update.
    

## Acceptance criteria (pass/fail)

- From the Tracks list, user can start playback and hear audio.
    
- Pause/resume and seek work.
    
- Output device can be switched (requires restart of stream).
    
- App remains responsive during playback (no UI freezes).
    
- UI snapshots generated.
    

## Commands to run (dev/test/build)

- `cargo tauri dev`
    
- `pnpm ui:snapshots -- --milestone 02`
    
- `cargo test -p audio-engine`
    

## Risks & mitigations

- **Shared mode resampling by Windows audio engine** → UI must clearly label Shared as fallback; Exclusive will be default later.
    
- **Glitches/dropouts** → basic buffering controls introduced in milestone 07.
    

## Additions

- Add a minimal “Audio Debug” overlay (UI) that shows:
    
    - output mode: Shared
        
    - device name
        
    - negotiated format (sample rate/bit depth/channels)