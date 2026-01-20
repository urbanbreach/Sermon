# Milestone 00 - foundation

## Context

### Original Request
- Build a Windows-first Tauri (Rust) + Svelte (Vite) desktop app skeleton that runs.
- Establish a deterministic UI vision loop (screenshots + review pack) used every milestone.
- Ship a first-pass “liquid glass” design system and base layout inspired by Apple Music + Roon/Audirvana/MusicBee.
- Provide repo scaffolding, mock UI shell, mock data fixtures, and ADRs/risk register.

### Interview Summary
**Key Decisions**:
- Toolchain: Tauri v2; Svelte + Vite with TypeScript (minimal types); Node 20 LTS + pnpm 9.x; `package.json` engines for node/pnpm; Rust 1.85.0 pinned in `rust-toolchain.toml` with edition 2024.
- UI: left nav routes between Library sections (Albums/Artists/Tracks) and Settings; “Now Playing” is a separate view reachable from the bottom bar, not a left-nav item; dark-only theme for milestone 00.
- Snapshots: fixed 1440×900 viewport at device scale 1.0; artifacts gitignored except `/artifacts/README.md`.
- Snapshot runner: `pnpm ui:snapshots -- --milestone 00` calls `scripts/snapshot.ps1 -Milestone 00`, sets `SERMON_MOCK=1` + `SERMON_SNAPSHOT=1`, launches app, ensures `/artifacts/ui/00-foundation/`, prints expected filenames; manual capture via Snipping Tool is acceptable.
- Fixtures: 6 albums, ~60 tracks; artwork 600×600 WebP preferred (JPG acceptable), consistent sizing.
- Logging: plain-text file + console logs; debug/diagnostics toggle via `SERMON_DEBUG=1` only.
- QA: manual-only; Playwright deferred for Milestone 00 (explicit override of prompt’s Playwright choice, documented in ADR 0001). `pnpm lint`, `cargo build`, and `cargo clippy` are nice-to-have, non-blocking checks.
- Docs tweaks: add architecture map, event contract, glossary, and diagnostics redaction policy for agent reliability; include deterministic snapshot discipline guidance. Defer other “Possible tweaks” items to backlog/risk register with one-line rationale.
- Windows minimum target: Windows 10 1909+ (64-bit).

**First Interactive Marker**:
- Define as `ui/src/App.svelte` `onMount` emitting `sermon://first-interactive` and logging `first_interactive` to console; backend logs the same message to file/console when event received.

### Research Findings
- Repo currently only contains markdown prompts and plans; no code scaffolding or test infrastructure exists yet.
- A milestone plan template exists at `plans/milestones/00-foundation.md.md`.

### Metis Review
**Identified Gaps (addressed)**:
- Toolchain pinning (Node/pnpm, Rust 1.85.0), snapshot viewport, routing vs static UI, fixture specs, artifact tracking, and manual QA scope were locked down.

---

## Work Objectives

### Core Objective
Deliver a runnable Windows-first Tauri v2 + Svelte TS skeleton with a liquid-glass UI shell, deterministic snapshot loop (manual capture), mock data fixtures, and baseline documentation for future milestones.

### Concrete Deliverables
- Root scaffolding: `ui/`, `src-tauri/`, `crates/` placeholders, root `package.json` + `pnpm-workspace.yaml`, `rust-toolchain.toml`.
- UI shell: Library / Now Playing / Settings views with routing, top bar, left nav, main area, bottom bar, optional right panel placeholder.
- Design system: liquid-glass tokens, global styles, snapshot-safe transitions, dark-only theme.
- Mock data: `ui/fixtures/` with 6 albums, ~60 tracks, artwork images.
- Snapshot loop: `scripts/snapshot.ps1`, `pnpm ui:snapshots -- --milestone 00`, `ui/tests/ui-snapshots.spec.ts` stub, `/artifacts/ui/00-foundation/REVIEW.md`, `/artifacts/README.md`.
- Logging: file + console logs with `SERMON_DEBUG=1` toggle; log file at `artifacts/logs/sermon.log`.
- Docs: `/docs/adr/0001-ui-vision-loop.md`, `/docs/adr/0002-repo-architecture.md`, `/docs/risk-register.md` (5 risks + owners + mitigations), plus architecture map, event contract, glossary, and diagnostics redaction policy.
- Backlog/risk register entries for deferred “Possible tweaks” items.

### Definition of Done
- [x] `pnpm install` completes at repo root.
- [x] `pnpm dev` renders the UI shell with working navigation.
- [x] `cargo tauri dev` launches the app on Windows 10/11 (1909+).
- [x] `pnpm ui:snapshots -- --milestone 00` runs the snapshot script, produces `/artifacts/ui/00-foundation/` and `REVIEW.md` template, and expected screenshot filenames are present.
- [x] ADRs and risk register exist with required sections, 5 risks, and deferred items documented.

### Must Have
- Repo scaffolding with Tauri v2 + Svelte TS and Rust 1.85.0 toolchain pin.
- Deterministic snapshot mode with `SERMON_MOCK=1` + `SERMON_SNAPSHOT=1`.
- Liquid-glass baseline styling and dark-only theme.
- Mock fixtures with consistent data and artwork sizes.
- Snapshot artifacts and review pack format as specified.
- First-interactive marker logged to console + `artifacts/logs/sermon.log`.

### Must NOT Have (Guardrails)
- No SvelteKit, no heavy UI frameworks (Tailwind, Bootstrap, etc.).
- No Playwright setup or automated visual diffing in milestone 00 (override of prompt decision; document deferral in ADR 0001).
- No backend logic beyond scaffolding and logging; no real data calls.
- No audio playback, DB, scanning, search, tagging, album art fetching, or Last.fm.
- No animation libraries; keep transitions simple and disable in snapshot mode.
- No additional Tauri plugins beyond core v2 setup.

---

## Verification Strategy (MANDATORY)

### Test Decision
- **Infrastructure exists**: NO
- **User wants tests**: NO
- **Framework**: none (Playwright deferred)
- **QA approach**: manual verification + snapshot runner stub

### Manual QA Only
- **Frontend/UI**: Use `pnpm dev` and verify nav routing and layout.
- **Tauri app**: Use `cargo tauri dev` to launch on Windows 10/11.
- **Snapshot loop**: Use `pnpm ui:snapshots -- --milestone 00` to set snapshot flags, launch the app, and manually capture screenshots into `/artifacts/ui/00-foundation/`.
- **Docs**: Open ADRs/risk register and confirm required sections and content.

---

## Task Flow

```
Task 1 → Task 2 → Task 3 → Task 4
Task 1 → Task 5
Task 1 → Task 6
```

## Parallelization

| Group | Tasks | Reason |
|------|-------|--------|
| A | 2, 5, 6 | Can proceed after scaffolding from Task 1 |

| Task | Depends On | Reason |
|------|------------|--------|
| 3 | 2 | Needs UI shell structure |
| 4 | 3 | Needs snapshot mode + fixtures |

---

## TODOs

- [x] 1. Scaffold repo + toolchain baseline

  **What to do**:
  - Create root `package.json` with `engines` (`node>=20`, `pnpm>=9`) and scripts:
    - `dev`: `pnpm -C ui dev`
    - `ui:snapshots`: `pwsh ./scripts/snapshot.ps1 -Milestone 00`
  - Add `pnpm-workspace.yaml` referencing `ui`.
  - Scaffold the frontend with `pnpm create vite ui --template svelte-ts` (or equivalent) and keep it in `/ui`.
  - Add `rust-toolchain.toml` pinned to 1.85.0; set Rust edition 2024 in backend crates.
  - Create `Cargo.toml` workspace with placeholder crates: `crates/audio`, `crates/library`, `crates/tags` (stubs only).
  - Initialize Tauri v2 in `src-tauri/` using `cargo tauri init`, configuring `devPath` to `http://localhost:5173`, `distDir` to `../ui/dist`, `beforeDevCommand` to `pnpm -C ui dev`, and `beforeBuildCommand` to `pnpm -C ui build`.
  - Set default window size to 1440×900 in `src-tauri/tauri.conf.json` for snapshot determinism.

  **Must NOT do**:
  - Do not add SvelteKit, Playwright, or additional Tauri plugins.
  - Do not add real backend logic beyond scaffolding.

  **Parallelizable**: NO (foundation for all tasks)

  **References**:
  - `prompts/Milestone 00 - foundation.md:14` - Repo scaffolding required.
  - `prompts/Milestone 00 - foundation.md:16` - Tauri + Svelte/Vite frontend.
  - `prompts/Milestone 00 - foundation.md:18` - Rust workspace layout for future crates.
  - `prompts/Milestone 00 - foundation.md:92` - `ui/` deliverable.
  - `prompts/Milestone 00 - foundation.md:94` - `src-tauri/` deliverable.
  - `prompts/Milestone 00 - foundation.md:96` - `crates/` placeholders.
  - Official Rust 1.85.0 (2024 edition): https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html
  - Vite scaffold docs: https://vitejs.dev/guide/#scaffolding-your-first-vite-project
  - Tauri v2 init docs: https://v2.tauri.app/start/

  **Acceptance Criteria**:
  - [ ] `rust-toolchain.toml` pins `channel = "1.85.0"`.
  - [ ] `Cargo.toml` workspace lists `crates/audio`, `crates/library`, `crates/tags` with stub `lib.rs`.
  - [ ] Root `package.json` includes `engines` for Node 20 and pnpm 9, and `dev`/`ui:snapshots` scripts point to `pnpm -C ui dev` and `pwsh ./scripts/snapshot.ps1 -Milestone 00`.
  - [ ] `src-tauri/tauri.conf.json` sets `devPath` to `http://localhost:5173`, `distDir` to `../ui/dist`, `beforeDevCommand` to `pnpm -C ui dev`, `beforeBuildCommand` to `pnpm -C ui build`, and window size to 1440×900.

  **Manual Execution Verification**:
  - [ ] Run `pnpm install` → completes without errors.
  - [ ] Run `cargo metadata` → returns workspace metadata.

  **Commit**: NO (unless requested)

- [x] 2. Build the Svelte UI shell with routing

  **What to do**:
  - Scaffold a Svelte + Vite + TypeScript app inside `ui/`.
  - Implement lightweight routing using a local Svelte store (e.g., `ui/src/lib/state/route.ts`) and `App.svelte` conditional rendering—no external router.
  - Left nav switches between Library sections (Albums/Artists/Tracks) and Settings; bottom bar switches to the “Now Playing” view.
  - Build the shell layout: left nav, top bar (search placeholder + window controls region), main content area, bottom “Now Playing” bar, and optional right panel placeholder (hidden by default).
  - Keep theme dark-only for milestone 00.

  **Must NOT do**:
  - Do not add a UI framework or animation library.
  - Do not connect to backend APIs or IPC beyond placeholders.

  **Parallelizable**: YES (with Task 5/6 after Task 1)

  **References**:
  - `prompts/Milestone 00 - foundation.md:22` - UI shell requirements.
  - `prompts/Milestone 00 - foundation.md:24` - Left nav items.
  - `prompts/Milestone 00 - foundation.md:26` - Top bar requirements.
  - `prompts/Milestone 00 - foundation.md:30` - Bottom bar requirement.
  - `prompts/Milestone 00 - foundation.md:98` - Placeholder views.

  **Acceptance Criteria**:
  - [ ] `pnpm dev` renders the shell with nav (Albums/Artists/Tracks + Settings), top bar, main area, and bottom bar.
  - [ ] Clicking Albums/Artists/Tracks/Settings updates the main placeholder view.
  - [ ] Clicking the bottom “Now Playing” bar opens the Now Playing placeholder view.

  **Manual Execution Verification**:
  - [ ] Run `pnpm dev`.
  - [ ] Navigate between Library/Now Playing/Settings and verify placeholder content updates.

  **Commit**: NO (unless requested)

- [x] 3. Implement liquid-glass design system + fixtures + snapshot mode

  **What to do**:
  - Create a CSS token system for the liquid-glass look with explicit values:
    - `--glass-bg: rgba(18, 20, 24, 0.60)`
    - `--glass-border: rgba(255, 255, 255, 0.12)`
    - `--glass-highlight: rgba(255, 255, 255, 0.08)`
    - `--glass-shadow: 0 12px 28px rgba(0, 0, 0, 0.35)`
    - `--glass-blur: 16px` (cap at 20px)
    - `--glass-radius: 12px`
  - Apply translucent panels and subtle depth to shell components using the tokens above.
  - Add mock data loader controlled by `SERMON_MOCK=1` and `SERMON_SNAPSHOT=1`.
  - Create `ui/fixtures/library.json` with a stable schema:
    - `albums[]`: `{ id, title, artistId, year, trackIds[], artworkFile }`
    - `artists[]`: `{ id, name }`
    - `tracks[]`: `{ id, title, albumId, artistId, durationMs, trackNumber, discNumber }`
  - Add `ui/fixtures/artwork/` with 6 album images (600×600 WebP preferred, JPG acceptable).
  - Configure `ui/vite.config.ts` with `envPrefix: ["VITE_", "SERMON_"]` so `SERMON_MOCK`/`SERMON_SNAPSHOT` are available via `import.meta.env`.
  - Create a fixture loader module (e.g., `ui/src/lib/data/fixtures.ts`) that imports `ui/fixtures/library.json` and exposes typed accessors used by the shell views when `import.meta.env.SERMON_MOCK === "1"`.
  - Disable transitions/animations when `import.meta.env.SERMON_SNAPSHOT === "1"` to stabilize screenshots.
  - Define the “first interactive” marker as `onMount` in `ui/src/App.svelte` emitting `sermon://first-interactive` (via `@tauri-apps/api/event`) and logging `first_interactive` to the browser console.

  **Must NOT do**:
  - Do not add complex animation or blur effects beyond CSS backdrop-filter/opacity.
  - Do not pull real data or network resources.

  **Parallelizable**: NO (depends on Task 2)

  **References**:
  - `prompts/Milestone 00 - foundation.md:34` - Liquid glass baseline.
  - `prompts/Milestone 00 - foundation.md:38` - Snapshot-safe transitions.
  - `prompts/Milestone 00 - foundation.md:50` - Mock data mode requirement.
  - `prompts/Milestone 00 - foundation.md:100` - Fixture deliverables.

  **Acceptance Criteria**:
  - [ ] `ui/vite.config.ts` includes `envPrefix: ["VITE_", "SERMON_"]` and UI reads `import.meta.env.SERMON_*`.
  - [ ] CSS tokens (`--glass-*`) are defined with the specified values and applied to nav/top/main/bottom panels.
  - [ ] With `SERMON_MOCK=1`, the UI uses fixture data (albums/tracks appear).
  - [ ] `ui/fixtures/library.json` contains `albums`, `artists`, and `tracks` arrays with the defined schema.
  - [ ] With `SERMON_SNAPSHOT=1`, transitions are disabled and blur radius stays ≤ 20px.
  - [ ] Artwork files exist at `ui/fixtures/artwork/` and are consistently sized at 600×600.

  **Manual Execution Verification**:
  - [ ] Run `SERMON_MOCK=1 pnpm dev` and verify fixture content.
  - [ ] Run `SERMON_SNAPSHOT=1 pnpm dev` and confirm transitions are disabled.
  - [ ] Inspect the CSS token file and confirm the `--glass-*` values match the plan.

  **Commit**: NO (unless requested)

- [x] 4. Add snapshot runner + artifacts + review pack template

  **What to do**:
  - Create `scripts/snapshot.ps1` to set `SERMON_MOCK=1` and `SERMON_SNAPSHOT=1`, run `cargo tauri dev` (which starts `pnpm -C ui dev` via `beforeDevCommand`), ensure `/artifacts/ui/00-foundation/`, and print the expected screenshot filenames plus the required 1440×900 viewport.
  - Wire `pnpm ui:snapshots -- --milestone 00` to call the PowerShell script.
  - Add `ui/tests/ui-snapshots.spec.ts` as a test-plan stub describing routes/steps (no Playwright implementation yet).
  - Create `/artifacts/README.md` documenting snapshot capture steps, naming conventions, and Windows display scaling set to 100% (device scale factor 1.0).
  - Create `/artifacts/ui/00-foundation/REVIEW.md` template with required sections (screenshots list, what changed, known issues, next UI focus).
  - Gitignore `/artifacts/ui/00-foundation/*` but keep `/artifacts/README.md` tracked.

  **Must NOT do**:
  - Do not add Playwright or automated visual diffing in M00.
  - Do not require CI to run snapshots.

  **Parallelizable**: NO (depends on Task 3)

  **References**:
  - `prompts/Milestone 00 - foundation.md:40` - Deterministic UI vision loop steps.
  - `prompts/Milestone 00 - foundation.md:44` - Screenshot output path.
  - `prompts/Milestone 00 - foundation.md:106` - Snapshot spec file deliverable.
  - `prompts/Milestone 00 - foundation.md:108` - Artifacts output directory.
  - `prompts/Milestone 00 - foundation.md:110` - REVIEW.md deliverable.
  - `prompts/Milestone 00 - foundation.md:125` - Expected screenshot filenames.
  - `prompts/Milestone 00 - foundation.md:193` - REVIEW.md format requirements.

  **Acceptance Criteria**:
  - [ ] `pnpm ui:snapshots -- --milestone 00` runs `scripts/snapshot.ps1 -Milestone 00`.
  - [ ] `/artifacts/ui/00-foundation/` exists with template `REVIEW.md`.
  - [ ] Script prints expected filenames: `shell-library.png`, `shell-now-playing.png`, `shell-settings.png`.
  - [ ] Script output includes the required 1440×900 viewport reminder.
  - [ ] Captured screenshots are 1440×900 pixels (verify via file properties).

  **Manual Execution Verification**:
  - [ ] Run `pnpm ui:snapshots -- --milestone 00` and confirm output folder and printed filename list.
  - [ ] Capture screenshots manually and save to `/artifacts/ui/00-foundation/` with exact filenames.
  - [ ] Verify each screenshot file is 1440×900 pixels.

  **Commit**: NO (unless requested)

- [x] 5. Implement backend logging + debug toggle

  **What to do**:
  - Add plain-text logging in `src-tauri` using `tracing` + `tracing_subscriber` with a file appender.
  - Write logs to `artifacts/logs/sermon.log` resolved from the repo root (`std::env::current_dir`) and mirror to console.
  - When `SERMON_DEBUG=1`, set log level to `debug` and include extra diagnostics lines.
  - In `src-tauri/src/main.rs`, register an event listener in `setup` using `tauri::Manager::listen` for `sermon://first-interactive` and log `first_interactive` with timestamp.

  **Must NOT do**:
  - Do not add log rotation or remote logging.
  - Do not introduce extra plugins or telemetry.

  **Parallelizable**: YES (after Task 1)

  **References**:
  - `prompts/Milestone 00 - foundation.md:20` - Logging + debug toggle requirement.
  - `.sisyphus/plans/foundation.md:23` - First-interactive marker definition for implementation.
  - Tauri event API: https://v2.tauri.app/reference/javascript/api/namespaceevent/

  **Acceptance Criteria**:
  - [ ] Running `SERMON_DEBUG=1 cargo tauri dev` prints debug logs to console.
  - [ ] `artifacts/logs/sermon.log` is created and contains a `first_interactive` log line.

  **Manual Execution Verification**:
  - [ ] Run `SERMON_DEBUG=1 cargo tauri dev` and confirm `artifacts/logs/sermon.log` exists with `first_interactive`.

  **Commit**: NO (unless requested)

- [x] 6. Write ADRs, risk register, and backlog deferrals

  **What to do**:
  - Create `/docs/adr/0001-ui-vision-loop.md` with sections: Context, Decision, Snapshot Flags (`SERMON_MOCK`, `SERMON_SNAPSHOT`), Manual Capture Steps, Review Pack Format.
  - Create `/docs/adr/0002-repo-architecture.md` with sections: Overview, Architecture Map (table of module → responsibility), Event Contract (table of command/event → payload), Glossary (term → definition), Diagnostics Redaction Policy (allowed vs redacted fields).
  - Create `/docs/risk-register.md` with top 5 risks, owners, mitigations (include WebView2 blur, snapshot determinism, cold start budget).
  - Append the following deferred items as new rows in the `prompts/BACKLOG.md` table (Source Milestone = `00`). Use the table columns as follows:
    - **Feature / Item**: item name
    - **Category**: pick one of `Architecture`, `Productization`, `UX`, `Docs/Legal`, `Perf`
    - **Recommended Default**: `Defer`
    - **Pull-in Trigger**: one-line rationale for when to pull in
    - Items to add:
      - Play session/time accounting spec (Category: Architecture)
      - Gapless playback decision (Category: Architecture)
      - Tagging safe-write vs file identity handling (Category: Architecture)
      - Last.fm scrobble rules + secret/session storage (Category: Docs/Legal)
      - Windows productization (installer choice, updater keys, WebView2 strategy, SMTC, single-instance) (Category: Productization)
      - Minor adds: history retention (UX), DB pragmas (Perf), bit-perfect indicator (UX), artwork attribution (Docs), ASIO licensing checklist (Docs/Legal)
  - If any deferred item is risk-relevant (e.g., tagging/file identity), note it in the risk register with owner/mitigation.

  **Must NOT do**:
  - Do not expand scope into full product design docs.
  - Do not include Playwright automation in ADR 0001 (document as deferred).

  **Parallelizable**: YES (after Task 1)

  **References**:
  - `prompts/Milestone 00 - foundation.md:114` - ADR 0001 deliverable.
  - `prompts/Milestone 00 - foundation.md:116` - ADR 0002 deliverable.
  - `prompts/Milestone 00 - foundation.md:118` - Risk register deliverable.
  - `prompts/Milestone 00 - foundation.md:160` - Docs tweaks (architecture map + event contract).
  - `prompts/BACKLOG.md:5` - Backlog table structure for new rows.
  - `plans/Possible tweaks to plan by GPT 5.2 Pro.md:227` - Docs tweaks for agent reliability.
  - `plans/Possible tweaks to plan by GPT 5.2 Pro.md:248` - Snapshot discipline emphasis.
  - `plans/Possible tweaks to plan by GPT 5.2 Pro.md:252` - Minor deferred adds list.
  - `plans/Possible tweaks to plan by GPT 5.2 Pro.md:267` - Top deferred items (play session, tagging identity, Last.fm storage).

  **Acceptance Criteria**:
  - [ ] ADR 0001 includes Context, Decision, Snapshot Flags, Manual Capture Steps, Review Pack Format.
  - [ ] ADR 0002 includes Overview, Architecture Map table, Event Contract table, Glossary, Diagnostics Redaction Policy.
  - [ ] Risk register lists 5 risks, each with owner + mitigation.
  - [ ] Deferred items are added as rows in `prompts/BACKLOG.md` with Category set, Recommended Default = `Defer`, and Pull-in Trigger containing the one-line rationale.

  **Manual Execution Verification**:
  - [ ] Open each doc and verify required sections and entries are present.

  **Commit**: NO (unless requested)

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 1 | `chore(scaffold): add toolchain and workspace` | workspace files | `pnpm install`, `cargo metadata` |
| 2-3 | `feat(ui): add shell and glass styles` | `ui/` | `pnpm dev` |
| 4 | `chore(snapshots): add snapshot runner and review pack` | `scripts/`, `artifacts/` | `pnpm ui:snapshots -- --milestone 00` |
| 5 | `chore(logging): add debug logging` | `src-tauri/` | `cargo tauri dev` |
| 6 | `docs: add ADRs and risk register` | `docs/`, `prompts/BACKLOG.md` | open docs |

---

## Success Criteria

### Verification Commands
```
pnpm install
pnpm dev
pnpm ui:snapshots -- --milestone 00
cargo tauri dev
```

### Final Checklist
- [x] UI shell renders without backend and routes are clickable.
- [x] Snapshot script generates deterministic UI state and required artifacts path.
- [x] Manual screenshots saved with exact filenames.
- [x] ADRs + risk register present with required content.
- [x] Logging works with `SERMON_DEBUG=1`.
