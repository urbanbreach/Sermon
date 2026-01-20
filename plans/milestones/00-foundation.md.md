## Goal

- Create a **Windows-first** Tauri (Rust) + Svelte (Vite) desktop app skeleton that builds and runs.
    
- Establish the deterministic **UI vision loop** (screenshots + review pack) that will be used every milestone.
    
- Ship a first-pass “liquid glass” design system and base layout inspired by Apple Music + Roon/Audirvana/MusicBee.
    

## Scope (in)

- Repo scaffolding:
    
    - Tauri app + Svelte+Vite frontend.
        
    - Rust workspace layout (prepare for `audio`, `library`, `tags` crates later).
        
    - Basic logging plumbing (file + console) and a debug/diagnostics toggle.
        
- UI shell (static/mock):
    
    - Left nav (Library: Albums/Artists/Tracks; Settings).
        
    - Top bar (search input placeholder, window controls region).
        
    - Main content area (placeholder “Library” view).
        
    - Bottom “Now Playing” bar (static).
        
    - Optional right-side panel placeholder (queue/track info), off by default.
        
- **Liquid glass baseline**:
    
    - Translucent panels, blur, subtle border highlights, depth/shadows.
        
    - Smooth route transitions (but controllable/disable-able for snapshot determinism).
        
- **Deterministic UI vision loop**:
    
    1. Render key UI routes/states automatically
        
    2. Capture screenshots to `/artifacts/ui/00-foundation/{view}.png`
        
    3. Create `/artifacts/ui/00-foundation/REVIEW.md` (screenshots + short notes)
        
    4. Optional visual diff vs previous baseline
        
- “Mock data mode” for UI:
    
    - Frontend can run with fixtures (no backend needed) to ensure deterministic screenshots.
        

## Non-scope (out)

- Audio playback.
    
- Database, scanning, search.
    
- Tag read/write.
    
- Album art fetching and caching.
    
- Last.fm.
    

## Key decisions (with alternatives + why)

- **UI vision tooling: Playwright snapshot runner (dev-only)**
    
    - Why: Playwright can deterministically render routes at fixed viewport sizes and capture screenshots; we can also use its built-in screenshot diffing.
        
    - Alternative: Cypress (heavier for component tests), custom Puppeteer scripts (more DIY).
        
- **Optional MCP complement: Playwright MCP server**
    
    - Why: In agent workflows, an MCP server can provide deterministic interaction via accessibility tree and deterministic tool application.
        
    - Note: This is _optional_; the required deterministic loop is satisfied by Playwright screenshot runs.
        
- **Frontend framework: Svelte + Vite (no SvelteKit)**
    
    - Why: minimal runtime, fast boot, low overhead; good for lightweight desktop.
        
- **Design system approach: CSS variables + a small component layer (no heavy UI framework)**
    
    - Why: better control for “glass” effects; avoid shipping large CSS frameworks.
        

## Dependencies (minimal; justify heavy deps)

- Required:
    
    - Tauri + Rust toolchain (core runtime).
        
    - Svelte + Vite.
        
- Dev-only (heavy but justified):
    
    - **Playwright** for deterministic UI rendering + screenshots + optional diffs (dev dependency only).
        

## Deliverables (concrete artifacts/files/features)

- Repo structure + scripts:
    
    - `ui/` (Svelte app)
        
    - `src-tauri/` (Tauri backend)
        
    - `crates/` (empty placeholders for upcoming `audio/`, `library/`, `tags/`)
        
- UI routes (mock-only): `Library`, `Now Playing`, `Settings` placeholders.
    
- UI fixtures:
    
    - `ui/fixtures/` with deterministic mock JSON and a small set of album art images.
        
- UI vision loop:
    
    - `ui/tests/ui-snapshots.spec.*` (test plan; no implementation code required here)
        
    - Output directory: `/artifacts/ui/00-foundation/`
        
    - `/artifacts/ui/00-foundation/REVIEW.md`
        
- Engineering hygiene:
    
    - `/docs/adr/0001-ui-vision-loop.md`
        
    - `/docs/adr/0002-repo-architecture.md`
        
    - `/docs/risk-register.md` (top 5 risks, with owners + mitigations)
        

## Acceptance criteria (pass/fail)

- App launches on Windows 10/11 via dev command.
    
- UI shell renders without backend.
    
- Running the snapshot command produces:
    
    - `/artifacts/ui/00-foundation/shell-library.png`
        
    - `/artifacts/ui/00-foundation/shell-now-playing.png`
        
    - `/artifacts/ui/00-foundation/shell-settings.png`
        
    - `/artifacts/ui/00-foundation/REVIEW.md`
        
- ADRs exist and risk register has 5 risks.
    

## Commands to run (dev/test/build)

- `pnpm install`
    
- `pnpm dev`
    
- `pnpm ui:snapshots -- --milestone 00`
    
- `cargo tauri dev`
    

## Risks & mitigations

- **WebView2 blur/perf issues on Windows** → Provide a “reduced effects” toggle + keep blur radii capped; use snapshot mode with effects pinned.
    
- **Non-deterministic screenshots (timing/animations)** → Add a snapshot mode that disables animations and uses fixed data.
    
- **Cold start budget risk** → Measure from day 1 (log timestamps + UI “first interactive” marker).
    

## Additions

- **UI vision baseline views (00):**
    
    - `shell-library`, `shell-now-playing`, `shell-settings`
        
- **Review pack format requirement:**
    
    - `REVIEW.md` includes: list of screenshots, what changed, known UI issues, next UI focus.