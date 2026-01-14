ultrawork
Generate the plan.

# Milestone 00 - foundation

## Goal
- Create a **Windows-first** Tauri (Rust) + Svelte (Vite) desktop app skeleton that builds and runs.
    
- Establish the deterministic **UI vision loop** (screenshots + review pack) that will be used every milestone.
    
- Ship a first-pass “liquid glass” design system and base layout inspired by Apple Music + Roon/Audirvana/MusicBee.

## Scope (In)
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

## Non-scope (Out)
- Audio playback.
    
- Database, scanning, search.
    
- Tag read/write.
    
- Album art fetching and caching.
    
- Last.fm.

## Prerequisites/Dependencies
- 00: N/A

## Key Decisions
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

## Deliverables
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

## Acceptance Criteria
- App launches on Windows 10/11 via dev command.
    
- UI shell renders without backend.
    
- Running the snapshot command produces:
    
    - `/artifacts/ui/00-foundation/shell-library.png`
        
    - `/artifacts/ui/00-foundation/shell-now-playing.png`
        
    - `/artifacts/ui/00-foundation/shell-settings.png`
        
    - `/artifacts/ui/00-foundation/REVIEW.md`
        
- ADRs exist and risk register has 5 risks.

## Commands (Labeled)
- `pnpm install` (Pre-existing)
    
- `pnpm dev` (Pre-existing)
    
- `pnpm ui:snapshots -- --milestone 00` (Introduced)
    
- `cargo tauri dev` (Pre-existing)

## Verification (Tiered)
- **Tier A — Hardware-agnostic**:
  - `pnpm ui:snapshots` produces expected images.
  - `cargo tauri dev` launches successfully.
  - `lsp_diagnostics` is clean.
- **Tier B — Hardware-dependent**: N/A — not hardware dependent

## Risks & Mitigations
- **WebView2 blur/perf issues on Windows** → Provide a “reduced effects” toggle + keep blur radii capped; use snapshot mode with effects pinned.
    
- **Non-deterministic screenshots (timing/animations)** → Add a snapshot mode that disables animations and uses fixed data.
    
- **Cold start budget risk** → Measure from day 1 (log timestamps + UI “first interactive” marker).

## Tweaks (MUST/SHOULD/MAY)
- [SHOULD] T7 — Docs tweaks (Architecture map, Event contract)
  - Source: T7
  - Key points:
    - Include architecture map.
    - Define event contract.
  - Rationale: Architecture clarity for initial setup.

## Suggestions
- Consider launching explore agents for codebase pattern discovery
- Verify all acceptance criteria with lsp_diagnostics before completion

## Deferred/Backlog
[List MAY items with recommended defaults]

## Failure Recovery / Resume
- Checkpointing: note last completed deliverable after each major task group.
- Resume: if a session id is provided, use `sisyphus_task(resume="<session_id>", prompt="fix: <specific failure>")`.
- If resume fails: rerun the original task without category wrapper.

## Session Prompt Variables
- `milestone_id`: 00
- `milestone_title`: foundation
- `milestone_file`: C:\Obsidian Vaults\Sermon\prompts\Milestone 00 - foundation.md
- `special_emphasis`: none
- `dependencies`: N/A

## Milestone-specific Notes
### Source Additions
- **UI vision baseline views (00):**
    
    - `shell-library`, `shell-now-playing`, `shell-settings`
        
- **Review pack format requirement:**
    
    - `REVIEW.md` includes: list of screenshots, what changed, known UI issues, next UI focus.
