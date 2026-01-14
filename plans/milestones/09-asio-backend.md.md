## Goal

- Add **ASIO** as an advanced output option, including native DSD where drivers support it—while managing licensing/build/distribution risk.
    

## Scope (in)

- ASIO backend design:
    
    - Build-time feature flag: `asio`
        
    - When disabled, app ships without ASIO (default).
        
- ASIO integration strategy:
    
    - Prefer leveraging CPAL’s optional ASIO backend (only when `asio` feature enabled) to avoid implementing an ASIO host from scratch.
        
        - CPAL documents an optional `asio` feature on Windows and notes build requirements (drivers + LLVM/Clang for bindings generation).
            
- Licensing documentation and guardrails:
    
    - ASIO SDK is dual-licensed (GPLv3 or proprietary) according to publicly mirrored license text; proprietary terms historically restricted redistribution.
        
    - Clearly document what is required to legally build/distribute an ASIO-enabled build.
        
    - Provide CI/build defaults that do **not** enable ASIO.
        
- UI:
    
    - Preferences → Player → Output Mode includes ASIO only when feature enabled.
        
    - Device selection for ASIO drivers.
        
    - Buffer size control (ASIO panel).
        

## Non-scope (out)

- Making ASIO the default path (WASAPI Exclusive remains default for Windows-first).
    
- Shipping an ASIO-enabled binary without clear licensing compliance.
    

## Key decisions (with alternatives + why)

- **ASIO as optional feature, not baseline**
    
    - Why: reduces legal and build complexity risk; keeps core app lightweight.
        
- **Use CPAL only for ASIO**
    
    - Why: CPAL’s WASAPI backend is shared-mode oriented; we keep our WASAPI implementation for Exclusive. (CPAL still helps for ASIO only.)
        
    - Alternative: implement ASIO host directly → more code + higher risk.
        

## Dependencies (minimal; justify heavy deps)

- Heavy (optional, only under `asio` feature):
    
    - `cpal` with `asio` feature + build tooling requirements (LLVM/Clang).
        
- Justification: only enabled for advanced users; default build remains slim.
    

## Deliverables (concrete artifacts/files/features)

- ASIO backend behind feature flag.
    
- `/docs/asio.md`:
    
    - build prerequisites
        
    - licensing notes (GPLv3 vs proprietary)
        
    - distribution guidance
        
- UI updates:
    
    - ASIO settings panel when enabled
        
- UI vision:
    
    - `/artifacts/ui/09-asio-backend/prefs-player-asio.png`
        
    - `/artifacts/ui/09-asio-backend/REVIEW.md`
        
- ADR:
    
    - `/docs/adr/0011-asio-strategy.md`
        
- Risk register update.
    

## Acceptance criteria (pass/fail)

- Default build (no ASIO feature) compiles and runs unchanged.
    
- ASIO-enabled build:
    
    - lists ASIO drivers (when installed)
        
    - can play PCM through selected ASIO device
        
- Clear documentation exists for licensing/build requirements.
    
- Snapshot pack produced.
    

## Commands to run (dev/test/build)

- Default:
    
    - `cargo tauri dev`
        
- ASIO feature build:
    
    - `cargo tauri dev --features asio` (or equivalent build command)
        
- UI snapshots:
    
    - `pnpm ui:snapshots -- --milestone 09`
        

## Risks & mitigations

- **Licensing/distribution uncertainty** → keep ASIO off by default; document options; require explicit enable.
    
- **Toolchain friction (clang/bindgen)** → ASIO build path documented; CI not required to enable ASIO by default.
    

## Additions

- Add a “Legal/Compliance checklist” section to `/docs/asio.md` referencing the dual licensing and restrictions.