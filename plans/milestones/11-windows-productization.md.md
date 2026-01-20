## Goal

- Ship a minimal Windows product: installer build, basic update strategy, and key Windows integrations (media keys + device UX polish), while hitting performance budgets.
    

## Scope (in)

- Packaging:
    
    - Choose Windows installer target(s): MSI (WiX) and/or NSIS via Tauri.
        
    - Tauri docs note Windows distribution is via MSI (WiX Toolset v3) or NSIS setup executables.
        
- Update strategy (explicit):
    
    - Decide: implement Tauri updater plumbing now (even if hosted update server is deferred).
        
    - Document signing requirements and a “local update feed” demo for acceptance.
        
- Windows integrations:
    
    - Media keys (System Media Transport Controls or equivalent)
        
    - Audio device selection UX polish (clear mode + format display)
        
    - Optional: tray/miniplayer (only if small; otherwise defer explicitly)
        
- Performance checkpoints:
    
    - Cold start → interactive: measure and record (target ≤ 2s)
        
    - Idle memory with library loaded: measure (target 250–400MB)
        
    - Scan throughput: record baseline tracks/min
        
- Release hygiene:
    
    - “Export diagnostics bundle” (logs + DB stats + config snapshot, no private data unless user opts in)
        

## Non-scope (out)

- MSIX Store distribution (optional later; if not done, document why).
    
- Crash reporting service (unless trivial/local).
    

## Key decisions (with alternatives + why)

- **Installer: MSI (WiX) first**
    
    - Why: standard enterprise-friendly Windows packaging; Tauri supports MSI via WiX on Windows.
        
    - Alternative: NSIS (also supported) for simpler setup-exe distribution.
        
- **Updater: implement config hooks now, hosting later**
    
    - Why: keeps milestone small while ensuring the architecture is ready.
        

## Dependencies (minimal; justify heavy deps)

- Tauri bundling tooling already in stack.
    
- Optional updater plugin if not already in use.
    

## Deliverables (concrete artifacts/files/features)

- Windows installer build configured (MSI and/or NSIS).
    
- `/docs/release.md`:
    
    - build steps
        
    - signing notes
        
    - update feed strategy (even if “defer hosting”)
        
- Media key support.
    
- Diagnostics export.
    
- UI vision:
    
    - `/artifacts/ui/11-windows-productization/about-diagnostics.png`
        
    - `/artifacts/ui/11-windows-productization/REVIEW.md`
        
- ADR:
    
    - `/docs/adr/0012-windows-packaging-updates.md`
        
- Risk register final update.
    

## Acceptance criteria (pass/fail)

- `tauri build` produces a Windows installer artifact (MSI and/or NSIS) on Windows.
    
- Installed app launches and plays audio in WASAPI Exclusive (on compatible device).
    
- Media keys control playback.
    
- Diagnostics export creates a bundle file with logs + metadata.
    
- Performance baseline documented and compared to targets.
    
- Snapshot pack produced.
    

## Commands to run (dev/test/build)

- `cargo tauri build`
    
- `cargo tauri dev`
    
- `pnpm ui:snapshots -- --milestone 11`
    

## Risks & mitigations

- **Signing / SmartScreen friction** → document required signing steps; allow unsigned dev builds.
    
- **Update hosting complexity** → defer hosting; provide local feed demo.
    
- **WebView2 deployment** → follow Tauri guidance on WebView2 installation options (bootstrapper vs fixed) (documented in release notes).
    

## Additions

- **Global Definition of Done checklist** (in `/docs/release.md`):
    
    - Build passes
        
    - WASAPI Exclusive validated
        
    - Library scan + search works
        
    - Tags safe write works
        
    - UI snapshot packs exist for all milestones
        
    - Minimal release artifact exists