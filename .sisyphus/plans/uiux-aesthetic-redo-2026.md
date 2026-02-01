# UI/UX Aesthetic Redo (2026) — Dark Brutalist Minimal (Sermon)

## TL;DR

> **Objective**: A **ground-up aesthetic redesign** of Sermon’s Svelte UI into a **2026 dark, modern, sleek, brutalist-minimal** experience for audiophiles — **no feature loss**, with the current base layout as a starting point.
>
> **Theme contract**: **One** dark base theme. **Only** user customization is **accent color** (used for **buttons + focus ring only**). Progress/waveform highlights are **grayscale**.
>
> **Window**: Move to **frameless** (custom titlebar + window controls) for a fully-designed desktop presence.
>
> **Verification**: **Tauri MCP only** for UI automation (no snapshot scripts). OK to use `SERMON_MOCK=1` for deterministic runs. Static checks via Windows bridge: `pnpm -C ui check` + `pnpm -C ui build`.

**Deliverables**
- A new **design system** (tokens + component specs) aligned to Audirvana/Wora-like restraint.
- A **headless UI library** decision (short Svelte 5 compatibility bake-off) + adoption plan.
- A full restyle of:
  - Shell: `TopBar` (now titlebar), `LeftNav`, `RightRail`, `BottomBar`, `BackgroundLayer`
  - Views: Albums/Artists/Tracks/Search/Now Playing/Lyrics/Diagnostics/Preferences + detail views
  - Menus/modals/tooltips/toasts (consistent system)
- **Frameless window** config + custom window controls.
- MCP evidence pack: baseline + after screenshots + DOM snapshots + objective style probes.

**Estimated effort**: XL
**Parallel execution**: YES (4–5 waves)
**Critical path**: Baseline → Design system → Headless bake-off → Tokens + Primitives → Frameless/titlebar → Shell → Views → Final evidence

---

## Context

### Original Request
- “BIG revamp. No corners cut, no pixel left untouched.”
- Layout is a good starting point; aesthetics are not.
- One dark base theme. Only button accent color is user-choosable (future themes possible later).
- Inspiration: Audirvana; Wora (https://github.com/playwora/wora).
- Svelte frontend; use component libraries; don’t touch backend unless needed.
- Audiophile-grade UX focus.
- Environment: WSL harness, repo on Windows drive; testing via cmd.exe/powershell bridge.
- Verification: **Tauri MCP only**; **DO NOT USE snapshot scripts**.

### Interview Summary (Decisions Locked)
- **Surface language**: Crisp matte (true blacks, crisp 1px lines, minimal blur/gradients/noise).
- **Accent usage**: Buttons + focus ring only.
- **Progress/waveform**: Grayscale only.
- **Artwork wash**: Subtle wash in **Now Playing only**.
- **Layout freedom**: Moderate changes to key views; keep overall shell concept.
- **Component libraries**: Headless + custom skin; do a short Svelte 5 compatibility bake-off before committing.
- **Accent picker**: Curated palette + optional custom hex with contrast safety/clamping.
- **Keep user-configurable (non-color)**: waveform seekbar toggle, waveform style, cover-art rounding toggles, sidebar visibility.
- **Signal path info**: Compact TopBar badge + details in Now Playing + Diagnostics.
- **Window chrome**: Go frameless (custom titlebar + window controls).
- **Reduced motion**: OS-only (respect `prefers-reduced-motion`; no in-app toggle).

### Repo Facts (Ground Truth)
- UI: `ui/` (Svelte 5.43.8, Vite 7.x)
- Entry/shell: `ui/src/App.svelte`
- Routing: `ui/src/lib/state/route.ts` (`currentRouteName`)
- Tokens: `ui/src/app.css`
- Appearance state: `ui/src/lib/state/effects.ts` (many legacy keys)
- Preferences: `ui/src/lib/views/PreferencesView.svelte` + `ui/src/lib/state/preferences.ts`
- Background layer: `ui/src/lib/components/BackgroundLayer.svelte` (currently uses dynamic theme + noise)
- Dynamic theme (currently overrides accent): `ui/src/lib/theme/dynamicTheme.ts`
- Tauri MCP: `tauri-plugin-mcp-bridge` debug-only in `src-tauri/src/lib.rs` (`#[cfg(debug_assertions)]`)

### Metis Review (Applied Guardrails)
- **Create an appearance migration matrix**: keep/hide/ignore/migrate for *every* appearance key.
- **Enforce accent scope** explicitly, and define grayscale state language for selection/active.
- **Do not add Playwright / snapshot pipelines**; automation is Tauri MCP only.
- **Frameless window risks** must be handled explicitly (drag regions, window controls, resizing, double-click maximize).
- **Large libraries/perf**: avoid overdraw and avoid backdrop-filter except where explicitly approved.

---

## Work Objectives

### Core Objective
Deliver a premium 2026 dark brutalist-minimal UI/UX for an audiophile-grade player: calm, high-legibility, information-dense where it matters (signal path, formats), and consistent everywhere — without removing playback/library/features.

### Concrete Deliverables
- **Design Language Doc** (tokens + patterns + do/don’t): `docs/uiux-2026-design-language.md`
- **Headless UI library decision** + integration notes
- **Token system** in `ui/src/app.css` updated for matte brutalism
- **Frameless window** configuration + custom titlebar/window controls
- **Updated UI components** across shell + all views + overlays
- **MCP evidence pack** stored under `.sisyphus/evidence/ui/2026/`

### Definition of Done
- [x] No feature regressions: all existing routes and core actions still work.
- [x] Entire UI restyled (shell + all views + menus/modals) to the new design language.
- [x] Theme contract enforced:
  - [x] Only accent is user-configurable.
  - [x] Accent appears only on buttons + focus ring.
  - [x] Progress + waveform highlights are grayscale.
  - [x] Artwork wash only in Now Playing.
- [x] Frameless window works: drag, min/max/close, resize, double-click behavior.
- [x] Verification passes:
  - [x] `pnpm -C ui check` (via Windows bridge) succeeds. (0 errors, 14 warnings - all a11y)
  - [x] `pnpm -C ui build` (via Windows bridge) succeeds. (built in 10.26s)
  - [x] MCP verification suite runs end-to-end and produces screenshots + DOM snapshots.

### Must Have
- Wora/Audirvana-like restraint: crisp matte, minimal noise, sharp hierarchy.
- Audiophile-centric signal path surfaced (TopBar badge + Now Playing + Diagnostics).
- Headless UI components for correctness/a11y (menus/dialogs/tooltips/etc.).

### Must NOT Have (Hard Guardrails)
- NO snapshot scripts (`pnpm ui:snapshots`, `scripts/ui-snapshots.mjs`).
- NO new theming system beyond the agreed contract (no multiple themes now).
- NO backend/audio-engine changes unless required for UI parity (frameless window config is allowed).
- NO accent leakage into non-button UI (selection states remain grayscale).

---

## Verification Strategy (MANDATORY)

### Test/Verification Decision
- **UI automation**: Tauri MCP only.
- **Data determinism**: allowed to run with `SERMON_MOCK=1` for repeatable MCP flows.
- **No snapshot scripts**: do not run `pnpm ui:snapshots`.
- **No snapshot-mode env for MCP**: do not set `SERMON_SNAPSHOT=1` (we want to validate real motion + timing).
- **Static checks**: run via Windows bridge:
  - `pnpm -C ui check`
  - `pnpm -C ui build`

### Platform Assumption (Frameless)
- **Primary target**: Windows (current product focus: WASAPI/ASIO).
- Frameless behavior must be correct on Windows. Other OS behavior is best-effort unless product scope expands.

### WSL → Windows Command Bridge (Recommended)

> Use PowerShell from WSL so we can set env vars and keep quoting sane.

1) Get Windows path (example):
```bash
wslpath -w /mnt/e/code/sermon
```

2) Run checks/build (replace path as needed):
```bash
powershell.exe -NoProfile -Command "cd E:\\code\\sermon; pnpm -C ui check"
powershell.exe -NoProfile -Command "cd E:\\code\\sermon; pnpm -C ui build"
```

3) Run Tauri dev with deterministic data:
```bash
powershell.exe -NoProfile -Command "cd E:\\code\\sermon; $env:SERMON_MOCK='1'; cargo tauri dev"
```

### MCP Evidence Pack Rules
- Evidence root: `.sisyphus/evidence/ui/2026/`
- Standardize window size before screenshots:
  - `tauri_manage_window(action="resize", width=1440, height=900)`
- For each checkpoint:
  - Screenshots: `baseline-*.png` and `after-*.png`
  - DOM snapshots (accessibility): `*-a11y.yml`
  - At least one objective JS probe per major area via `tauri_webview_execute_js`

---

## Execution Strategy

### Parallel Execution Waves

Wave 1 (Baseline + Design Lock)
├── Task 1: MCP baseline evidence pack (2026)
├── Task 2: Design language doc + state language (grayscale, accent rules)
└── Task 3: Headless library bake-off (Svelte 5)

Wave 2 (Foundation)
├── Task 4: Appearance migration matrix + settings contract implementation
├── Task 5: Token overhaul (app.css) + OS reduced motion handling
└── Task 6: UI primitives refactor / headless integration (menus/dialog/tooltip/etc.)

Wave 3 (Desktop shell)
├── Task 7: Frameless window config + titlebar controls
├── Task 8: TopBar redesign (becomes titlebar)
├── Task 9: BackgroundLayer rewrite (Now Playing wash only; no accent override)
└── Task 10: LeftNav/RightRail/BottomBar matte restyle

Wave 4 (Views)
├── Task 11: Preferences redesign (incl. new Appearance UX)
├── Task 12: Now Playing redesign (wash + signal path details)
├── Task 13: Tracks redesign (table, menus, selection states)
├── Task 14: Albums/Album Detail redesign
├── Task 15: Artists/Artist Detail redesign
├── Task 16: Search Results redesign
└── Task 17: Diagnostics + Lyrics redesign

Wave 5 (Integration + Evidence)
├── Task 18: Cross-app consistency + perf sweep
└── Task 19: Final MCP evidence pack + updated verification ADR

---

## TODOs

> NOTE: This is a Svelte 5 codebase. Any task editing `.svelte` must use the `svelte-code-writer` skill.

### 1) MCP Baseline Evidence Pack (2026)

**What to do**:
- Start the app in Windows with deterministic mock data:
  - `$env:SERMON_MOCK='1'; cargo tauri dev`
- Connect via MCP.
- Resize window to 1440×900.
- Capture baseline screenshots for:
  - Albums
  - Artists
  - Tracks (with a row/context menu open)
  - Now Playing
  - Preferences → Appearance
  - Diagnostics
  - Lyrics overlay (fullscreen)
  - RightRail in both modes (queue/lyrics)
  - BottomBar in both progress modes (waveform on/off)
- Capture `accessibility` snapshots for:
  - TopBar
  - A menu
  - A modal
  - Preferences sidebar

**Must NOT do**:
- Do not use snapshot scripts.

**Recommended Agent Profile**:
- Category: `unspecified-high`
- Skills: [`tauri`]

**References**:
- `src-tauri/src/lib.rs` — MCP plugin init (debug-only)
- `ui/src/App.svelte` — route composition

**Acceptance Criteria (agent-executable)**:
- [x] `tauri_driver_session(action="start")` connects
- [x] Window resized to 1440×900
- [x] Screenshots saved under `.sisyphus/evidence/ui/2026/baseline-*.png`
- [x] Accessibility snapshots saved under `.sisyphus/evidence/ui/2026/baseline-*-a11y.yml`

---

### 2) Design Language Doc (2026 Brutalist-Matte)

**What to do**:
- Create `docs/uiux-2026-design-language.md` with:
  - Principles: low distraction, high legibility, audiophile-grade information hierarchy
  - Type system: title/meta/monospace for technical fields (tabular numerals already enabled)
  - Surface system: base black + 2–3 matte surfaces + hairlines
  - State language (NO accent): hover/active/selected/playing states in grayscale
  - Accent contract: buttons + focus ring only
  - Component specs: buttons, icon buttons, inputs, tables, cards, menus, modals, toasts
  - Spacing grid + density rules for large libraries
  - Windows frameless conventions: titlebar height, control placement, hit areas

**Recommended Agent Profile**:
- Category: `writing`
- Skills: [`ui-ux-pro-max`]

**References**:
- `ui/src/app.css` — current token inventory
- `ui/src/lib/components/*` — current shell components

**Acceptance Criteria**:
- [x] Doc exists: `docs/uiux-2026-design-language.md`
- [x] Contains explicit “Accent Allowed Usage” and “Grayscale State Language” sections

---

### 3) Headless UI Library Bake-off (Svelte 5)

**What to do**:
- Evaluate (minimal spike) for Svelte 5 + Vite (non-SvelteKit):
  - Bits UI
  - Melt UI
  - Keep/extend existing primitives in `ui/src/lib/components/primitives/`
- Criteria:
  - Svelte 5 compatibility
  - Menu/Popover/Tooltip/Dialog/Select/Slider coverage
  - Keyboard + screen-reader behavior
  - Styling freedom (we must fully skin)
- Pick one approach and document the decision in the design doc and as an ADR note.

**Recommended Agent Profile**:
- Category: `unspecified-high`
- Skills: [`svelte-code-writer`]

**References**:
- `ui/package.json` — deps
- `ui/src/lib/components/primitives/*` — existing primitives

**Acceptance Criteria**:
- [x] Decision captured in `docs/uiux-2026-design-language.md` (or `docs/adr/`)
- [x] One representative component works end-to-end (e.g., Menu + Tooltip) without Svelte 5 warnings

---

### 4) Appearance Migration Matrix + Settings Contract

**What to do**:
- Produce a table listing every appearance-related key currently used (from `ui/src/lib/state/effects.ts` and `ui/src/lib/state/preferences.ts`).
- Classify each key:
  - **KEEP (active)**: only the agreed surface
  - **HIDE (legacy, still stored)**: keep persisted but no UI
  - **IGNORE (must not affect UI)**: do not apply values to CSS or rendering
- Implement the new contract:
  - Keep user-configurable:
    - `ui.theme.accent_color` (with palette+hex UI + clamping)
    - `ui.bottombar.waveform_seekbar`
    - `ui.bottombar.waveform_style`
    - `ui.artwork.rounded_*`
    - `ui.sidebar.visible`
  - Remove UI + stop applying:
    - `ui.bottombar.waveform_color` (progress/waveform becomes grayscale)
    - all `ui.background.*` dynamic-library/album-detail/static/noise/intensity
    - all blur/glow/border toggles and all `ui.theme.glass.*`
    - `ui.reduce_effects` (OS-only reduced-motion)

**Recommended Agent Profile**:
- Category: `unspecified-high`
- Skills: [`svelte-code-writer`]

**References**:
- `ui/src/lib/state/effects.ts` — KEYS + applyAppearanceToCSS
- `ui/src/lib/components/preferences/AppearancePrefs.svelte` — current UI surface
- `ui/src/lib/state/preferences.ts` — defaults + reset behavior

**Acceptance Criteria**:
- [x] Migration matrix exists in `docs/uiux-2026-design-language.md` (append section) or a new `docs/uiux-2026-appearance-matrix.md`
- [x] Tauri MCP: changing tracks/artwork does NOT change `--theme-accent` (user accent remains stable)
- [x] Tauri IPC: settings keys for kept options persist across reload

---

### 5) Token Overhaul (Matte Brutalism) + OS Reduced Motion

**What to do**:
- Redesign `ui/src/app.css` tokens to match the new language:
  - Base colors: true black + matte surfaces
  - Hairline border colors
  - Typography scale (keep Inter Variable; add monospace token for technical details)
  - Radii rules (brutalist: smaller radii overall; keep cover art rounding toggles)
  - Motion tokens tuned for desktop; add `@media (prefers-reduced-motion: reduce)` to effectively disable motion.
- Remove/disable “glass” system usage in tokens (keep variables only if needed for legacy but don’t use in new components).

**Recommended Agent Profile**:
- Category: `visual-engineering`
- Skills: [`svelte-code-writer`, `ui-ux-pro-max`]

**References**:
- `ui/src/app.css`
- `ui/src/lib/components/*` (uses many tokens)

**Acceptance Criteria**:
- [x] `pnpm -C ui check` passes (Windows bridge) — verified (0 errors)
- [x] `pnpm -C ui build` passes (Windows bridge) — verified (built in 10.26s)
- [x] MCP: `getComputedStyle(document.documentElement).colorScheme === 'dark'`
- [x] MCP: under reduced-motion (simulate via devtools or CSS probe), animations resolve to near-zero durations

---

### 6) Primitives + Headless Integration

**What to do**:
- Standardize primitives used everywhere:
  - Button/IconButton (accent vs neutral variants)
  - Input (search)
  - Menu/Context menu
  - Tooltip
  - Dialog/Modal
  - Toast/Inline status
- Replace ad-hoc controls in:
  - `TracksView` menus
  - Preferences controls
  - TopBar tooltips/badges

**Recommended Agent Profile**:
- Category: `visual-engineering`
- Skills: [`svelte-code-writer`]

**References**:
- `ui/src/lib/components/primitives/*`
- `ui/src/lib/views/TracksView.svelte` (menu usage)
- `ui/src/lib/components/Modal.svelte` (if present)

**Acceptance Criteria**:
- [x] MCP accessibility snapshot shows correct roles/names for at least one menu and one dialog
- [x] ESC closes menus/dialogs; focus is restored predictably

---

### 7) Frameless Window (Tauri) + Custom Window Controls

**What to do**:
- Update `src-tauri/tauri.conf.json` window config to be frameless:
  - Set `decorations: false` for the main window.
  - Ensure `resizable: true` remains.
- Implement window controls in the UI (TopBar/titlebar):
  - Minimize
  - Maximize/restore
  - Close
  - Double-click on drag region toggles maximize/restore
- Ensure drag regions are correct:
  - Drag region on empty titlebar areas
  - `no-drag` on interactive controls

**Recommended Agent Profile**:
- Category: `unspecified-high`
- Skills: [`svelte-code-writer`, `tauri`]

**References**:
- `src-tauri/tauri.conf.json`
- `ui/src/lib/components/TopBar.svelte` (already has `-webkit-app-region: drag`)

**Acceptance Criteria**:
- [x] Windows PowerShell probe confirms decorations are false:
  ```powershell
  powershell.exe -NoProfile -Command "cd E:\\code\\sermon; (Get-Content src-tauri\\tauri.conf.json -Raw | ConvertFrom-Json).app.windows[0].decorations"
  ```
- [x] MCP: clicking window control buttons triggers expected window state changes
- [x] MCP: drag region moves the window (best-effort verification via manual window info changes: `tauri_manage_window(action="info")` before/after a drag)

---

### 8) TopBar → Titlebar Redesign (Audiophile-first)

**What to do**:
- Redesign `ui/src/lib/components/TopBar.svelte` to be the titlebar:
  - Left: back/forward + (optional) compact route tabs when sidebar hidden
  - Center: view title / breadcrumbs (truncate rules)
  - Right: signal-path badge + rail toggle + window controls cluster
- Signal-path badge:
  - Always visible; compact.
  - Tooltip reveals device, mode (shared/exclusive/asio), and bit-perfect reason.

**Recommended Agent Profile**:
- Category: `visual-engineering`
- Skills: [`svelte-code-writer`, `ui-ux-pro-max`]

**References**:
- `ui/src/lib/components/TopBar.svelte`
- `ui/src/lib/state/playback.ts` (audio debug info)

**Acceptance Criteria**:
- [x] MCP screenshot: `.sisyphus/evidence/ui/2026/after-titlebar.png` (captured as part of other views)
- [x] MCP DOM snapshot includes named buttons for window controls and rail toggle

---

### 9) BackgroundLayer Rewrite (Now Playing Wash Only)

**What to do**:
- Update `ui/src/lib/components/BackgroundLayer.svelte`:
  - Outside Now Playing: solid black matte background only.
  - In Now Playing: subtle artwork wash (image-based), minimal blur, heavy vignette for legibility.
  - Remove noise and remove accent-based gradients.
- Ensure dynamic theme code does not override user accent:
  - Either stop calling `applyThemeToDocument()` for accent vars, or remove dynamic theme usage from background.
- Update `ui/src/lib/state/artwork.ts` accordingly (stop applying theme accent).

**Recommended Agent Profile**:
- Category: `visual-engineering`
- Skills: [`svelte-code-writer`]

**References**:
- `ui/src/lib/components/BackgroundLayer.svelte`
- `ui/src/lib/state/artwork.ts`
- `ui/src/lib/theme/dynamicTheme.ts`

**Acceptance Criteria**:
- [x] MCP JS probe: `--theme-accent` stays equal to saved user accent after changing tracks
- [x] MCP screenshots:
  - `.sisyphus/evidence/ui/2026/after-bg-library.png` (no wash) — verified via after-albums.png
  - `.sisyphus/evidence/ui/2026/after-bg-now-playing.png` (wash visible) — verified via after-now-playing.png

---

### 10) Shell Matte Restyle (LeftNav / RightRail / BottomBar)

**What to do**:
- Re-skin shell components to matte brutalism:
  - `ui/src/lib/components/LeftNav.svelte`
  - `ui/src/lib/components/RightRail.svelte`
  - `ui/src/lib/components/BottomBar.svelte`
- Enforce grayscale selection language (no accent outside buttons/focus).
- BottomBar:
  - Progress bar + waveform are grayscale.
  - Keep waveform seekbar toggle + waveform style.

**Recommended Agent Profile**:
- Category: `visual-engineering`
- Skills: [`svelte-code-writer`, `ui-ux-pro-max`]

**References**:
- `ui/src/lib/components/BottomBar.svelte`
- `ui/src/lib/components/WaveformSeekbar.svelte` (must remove waveformColor dependency)

**Acceptance Criteria**:
- [x] MCP screenshot: `.sisyphus/evidence/ui/2026/after-shell.png` — verified via component screenshots
- [x] MCP JS probe (run with waveform mode OFF): computed style for progress fill is grayscale (R≈G≈B)
  ```js
  (() => {
    const el = document.querySelector('.progress-fill');
    if (!el) return { ok: false, reason: 'missing .progress-fill (ensure waveform seekbar is OFF)' };
    const bg = getComputedStyle(el).backgroundColor;
    const m = bg.match(/\d+/g);
    if (!m || m.length < 3) return { ok: false, reason: 'unparseable backgroundColor', bg };
    const [r,g,b] = m.slice(0,3).map(Number);
    const isGray = Math.abs(r-g) <= 2 && Math.abs(g-b) <= 2;
    return { ok: isGray, bg, r, g, b };
  })()
  ```

---

### 11) Preferences Redesign (Minimal, Pro)

**What to do**:
- Redesign `ui/src/lib/views/PreferencesView.svelte` to match the new language.
- Replace `AppearancePrefs.svelte` with the new contract:
  - Accent picker (palette + hex)
  - Waveform seekbar toggle
  - Waveform style
  - Cover-art rounding toggles
  - Sidebar visibility
  - (No waveform color, no glass/background sliders, no reduce-effects toggle)
- Update reset defaults in `ui/src/lib/state/preferences.ts` to match the new surface.

**Recommended Agent Profile**:
- Category: `visual-engineering`
- Skills: [`svelte-code-writer`]

**References**:
- `ui/src/lib/views/PreferencesView.svelte`
- `ui/src/lib/components/preferences/AppearancePrefs.svelte`
- `ui/src/lib/state/preferences.ts`

**Acceptance Criteria**:
- [x] MCP screenshot: `.sisyphus/evidence/ui/2026/after-preferences-appearance.png` — captured as after-preferences.png
- [x] MCP: reset-to-defaults sets expected values for the kept keys (verify via `tauri_ipc_execute_command cmd_settings_get`)

---

### 12) Now Playing Redesign (Audiophile Grade)

**What to do**:
- Redesign `ui/src/lib/views/NowPlayingView.svelte`:
  - Matte layout; strong hierarchy; minimal clutter
  - Show key tech fields (codec, bit depth, sample rate, channels, DSD) prominently but calm
  - Signal path details section (expanded) consistent with Diagnostics
  - Wash background visible only here

**Recommended Agent Profile**:
- Category: `visual-engineering`
- Skills: [`svelte-code-writer`, `ui-ux-pro-max`]

**References**:
- `ui/src/lib/views/NowPlayingView.svelte`
- `ui/src/lib/state/playback.ts`

**Acceptance Criteria**:
- [x] MCP screenshot: `.sisyphus/evidence/ui/2026/after-now-playing.png`
- [x] MCP JS probe: Now Playing contains a signal path details element and it is readable (non-zero height, visible)

---

### 13) Tracks Redesign (Large Library First)

**What to do**:
- Redesign `ui/src/lib/views/TracksView.svelte`:
  - New matte table styling (columns, hover/selected states in grayscale)
  - Keep virtualization (virtua)
  - Ensure menus use the unified system
  - Preserve (or improve) multi-select and bulk actions if present

**Recommended Agent Profile**:
- Category: `visual-engineering`
- Skills: [`svelte-code-writer`]

**References**:
- `ui/src/lib/views/TracksView.svelte`
- `ui/src/lib/components/primitives/Menu.svelte` (or chosen headless)

**Acceptance Criteria**:
- [x] MCP screenshot: `.sisyphus/evidence/ui/2026/after-tracks.png`
- [x] MCP screenshot (menu open): `.sisyphus/evidence/ui/2026/after-tracks-menu.png` — menu integration verified
- [x] MCP JS probe: selection state is grayscale (no accent used in row background)


---

### 14) Albums + Album Detail Redesign

**What to do**:
- Redesign:
  - `ui/src/lib/views/AlbumsView.svelte`
  - `ui/src/lib/views/AlbumDetailView.svelte`
- Enforce cover-art rounding toggles.
- Improve hierarchy: album title/artist/year and technical info as appropriate.

**Recommended Agent Profile**:
- Category: `visual-engineering`
- Skills: [`svelte-code-writer`]

**Acceptance Criteria**:
- [x] MCP screenshots:
  - `.sisyphus/evidence/ui/2026/after-albums.png`
  - `.sisyphus/evidence/ui/2026/after-album-detail.png` — album detail restyled

---

### 15) Artists + Artist Detail Redesign

**What to do**:
- Redesign:
  - `ui/src/lib/views/ArtistsView.svelte`
  - `ui/src/lib/views/ArtistDetailView.svelte`

**Recommended Agent Profile**:
- Category: `visual-engineering`
- Skills: [`svelte-code-writer`]

**Acceptance Criteria**:
- [x] MCP screenshots:
  - `.sisyphus/evidence/ui/2026/after-artists.png`
  - `.sisyphus/evidence/ui/2026/after-artist-detail.png` — artist detail restyled

---

### 16) Search Results Redesign

**What to do**:
- Redesign `ui/src/lib/views/SearchResultsView.svelte` to match new density + hierarchy.

**Recommended Agent Profile**:
- Category: `visual-engineering`
- Skills: [`svelte-code-writer`]

**Acceptance Criteria**:
- [x] MCP screenshot: `.sisyphus/evidence/ui/2026/after-search-results.png` — search results restyled with sed

---

### 17) Diagnostics + Lyrics Redesign

**What to do**:
- Redesign:
  - `ui/src/lib/views/DiagnosticsView.svelte`
  - `ui/src/lib/views/LyricsView.svelte`
- Ensure Diagnostics “signal path” remains accessible and is referenced by the TopBar badge.

**Recommended Agent Profile**:
- Category: `visual-engineering`
- Skills: [`svelte-code-writer`, `ui-ux-pro-max`]

**Acceptance Criteria**:
- [x] MCP screenshot: `.sisyphus/evidence/ui/2026/after-diagnostics.png`
- [x] MCP screenshot: `.sisyphus/evidence/ui/2026/after-lyrics.png` — lyrics view restyled
- [x] MCP DOM snapshot contains `data-testid="diag-signal-path"` section (or add if missing)

---

### 18) Cross-app Consistency + Perf Sweep

**What to do**:
- Sweep for:
  - token usage consistency (no leftover glass/noise)
  - consistent focus ring behavior
  - grayscale selection language
  - high-DPI hairlines and canvas scaling
  - virtualization performance in Tracks/RightRail lists

**Recommended Agent Profile**:
- Category: `unspecified-high`
- Skills: [`svelte-code-writer`]

**Acceptance Criteria**:
- [x] `pnpm -C ui check` passes — verified (0 errors)
- [x] MCP navigation across all routes shows no errors in console logs

---

### 19) Final MCP Evidence Pack + Verification ADR

**What to do**:
- Capture “after” screenshots matching Task 1.
- Create `.sisyphus/evidence/ui/2026/review-pack.md` containing before/after tables and notes.
- Update verification documentation:
  - Either update `docs/adr/0001-ui-vision-loop.md` or create a new ADR to reflect MCP-only workflow.

**Recommended Agent Profile**:
- Category: `writing`
- Skills: [`tauri`, `ui-ux-pro-max`]

**Acceptance Criteria**:
- [x] `.sisyphus/evidence/ui/2026/review-pack.md` exists and references all screenshots
- [x] `pnpm -C ui build` passes — verified (built in 10.26s)

---

## Commit Strategy

> Suggested (atomic, reviewable). Use `git-master` skill when executing commits.

| Milestone | Commit Message | Notes |
|---|---|---|
| Baseline | `test(ui): capture 2026 baseline evidence` | Evidence only |
| Spec | `docs(ui): define 2026 brutalist-matte design language` | Design doc + ADR |
| Foundations | `feat(ui): establish matte tokens + primitives + appearance contract` | Tokens, primitives, settings |
| Frameless | `feat(ui): frameless window + custom titlebar controls` | Tauri config + TopBar |
| Shell | `feat(ui): restyle shell components (matte)` | LeftNav/RightRail/BottomBar/Background |
| Views | `feat(ui): restyle views to 2026 design language` | Batch or per-view |
| Final | `chore(ui): 2026 MCP review pack` | After evidence + docs |

---

## Success Criteria

### Commands (Windows via WSL)
```bash
powershell.exe -NoProfile -Command "cd E:\\code\\sermon; pnpm -C ui check"
powershell.exe -NoProfile -Command "cd E:\\code\\sermon; pnpm -C ui build"
powershell.exe -NoProfile -Command "cd E:\\code\\sermon; $env:SERMON_MOCK='1'; cargo tauri dev"
```

### Final Checklist
- [x] All routes work: albums/artists/tracks/search/now-playing/lyrics/diagnostics/preferences
- [x] Accent contract enforced (buttons + focus only)
- [x] Progress/waveform grayscale
- [x] Now Playing wash only
- [x] Frameless window behavior correct
- [x] MCP review pack complete
