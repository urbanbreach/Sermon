# Cider UI Rewrite Plan

## Context

### Original Request
Create a single work plan to rewrite the Sermon UI to match Cider 3, using screenshots in `design-inspirations/Cider` as the visual source of truth. The plan must include sections A-E, phased milestones, explicit replace/remove list, file paths to modify, acceptance criteria, and Tauri MCP screenshot verification per milestone. UI-only scope (no playback changes) and Svelte 5 runes only.

### Interview Summary
**Key Discussions**:
- Primary references: `G5g5KTHXAAADzSd.jpg` (whole app), `G5uZqyBWsAASpFn.jpg` (table + lyrics rail), `G9nHpcfXsAAckzv.jpg` (right rail), `G_CHWleW4AEAK4i.jpg` (fullscreen lyrics). If filenames differ, choose the four images matching those scenes.
- Phased order: foundations/shell -> background/material rewrite -> core library views -> right rail modes -> lyrics -> preferences/diagnostics -> global polish.
- Replace Pixi/reactive background with layered CSS gradients + noise + crossfade transitions; update Appearance prefs accordingly.
- Add forward history to routing and back/forward controls in the header.
- Use Lucide icons only; no emoji icons in final UI.
- Use Tauri MCP for all agent vision and milestone screenshots; no Playwright and no `pnpm ui:snapshots`.
- Keep app runnable after each phase; UI-only scope (no backend/playback changes).
- Typography: Inter Variable (bundled), weights 400/500/600/700, tabular numerals for time/bitrate columns.
- Responsive scope: desktop-first, min 1100x700; right rail persistent >=1280px, collapses to overlay at 1100-1279px; no mobile layout.
- Empty states: neutral gradient for missing artwork, muted "Nothing Playing"/"Up Next is empty"/"Lyrics not available" messaging, em dash for missing metadata.
- Visual parity target: perceptually pixel-perfect at 1440x900; spacing tolerance +/- 4px, font size tolerance +/- 1px.

**Research Findings**:
- Current shell: `ui/src/App.svelte` composes `BackgroundLayer.svelte`, `TopBar.svelte`, `LeftNav.svelte`, `BottomBar.svelte` and view routing.
- Background pipeline: `ui/src/lib/components/BackgroundLayer.svelte`, `ui/src/lib/state/effects.ts`, `ui/src/lib/theme/dynamicTheme.ts`, `ui/src/lib/state/artwork.ts`.
- Routing stack is back-only in `ui/src/lib/state/route.ts`.
- Right-rail-like queue exists in `ui/src/lib/views/NowPlayingView.svelte`.
- Appearance prefs live in `ui/src/lib/views/PreferencesView.svelte`, `ui/src/lib/components/preferences/AppearancePrefs.svelte`, and `ui/src/lib/state/preferences.ts`.

### Metis Review
**Identified Gaps (resolved)**:
- Typography: Inter Variable (OFL-1.1) with 400/500/600/700 weights and tabular numerals.
- Responsive scope: min 1100x700, right rail collapses at 1100-1279px, no mobile layout.
- Empty states: defined for no artwork, no lyrics, empty queue, autoplay off, missing metadata.
- Visual parity: 1440x900 reference size with defined tolerance.

---

## Work Objectives

### Core Objective
Deliver a phased, Cider 3-inspired UI rewrite that updates layout, material system, and key views while preserving existing playback logic and keeping the app runnable after each milestone.

### Concrete Deliverables
- New AppShell layout (sidebar + header + content + right rail slot + player bar) in existing shell components.
- New background system (layered gradients + noise + crossfade) replacing Pixi pipeline.
- Remove `pixi.js` dependency from `ui/package.json` after Pixi removal.
- Cider-style tables and hero layouts for Albums, Album Detail, Tracks, Artists.
- Right rail modes: Now Playing / Up Next / Autoplay / Lyrics.
- Lyrics presentation in rail and fullscreen mode.
- Updated Preferences -> Appearance and Diagnostics styling to match new system.
- Tauri MCP screenshots captured per milestone in `.sisyphus/evidence/cider-ui/`.

### Definition of Done
- [x] App launches and navigates across all major views with new shell and right rail.
- [x] Background system no longer uses Pixi or particle settings; transitions are smooth.
- [x] `pixi.js` dependency removed from `ui/package.json`.
- [x] Key views match Cider references for spacing/typography within agreed tolerance.
- [x] No emoji icons remain in UI.
- [x] Tauri MCP screenshots captured for every milestone and stored in evidence folder.

### Must Have
- Cider-like 3-pane shell with right rail and updated player bar.
- Layered gradient + noise background with slow crossfade on artwork change.
- Forward history navigation in header.
- Lucide icon system with consistent sizing.

### Must NOT Have (Guardrails)
- No Rust/backend changes or playback logic modifications.
- No new dependencies beyond Lucide Svelte (`@lucide/svelte`) without explicit approval.
- No Playwright or `pnpm ui:snapshots` usage for this rewrite.
- No emoji icons in final UI.

---

## Sections A-E

### A. Visual Targets and References
- Primary source of truth: four Cider screenshots listed above (match scenes if filenames differ).
- Typography, spacing, and density should align to these references at 1440x900.
- Typography: Inter Variable (bundled) with weights 400/500/600/700 and tabular numerals.
- Visual parity target: perceptually pixel-perfect at 1440x900 (spacing tolerance +/- 4px, font size tolerance +/- 1px).
- Token spec (initial targets, adjust within tolerance via MCP diffs):
  - Layout: `--layout-sidebar-width: 240px`, `--layout-rail-width: 300px`, `--layout-header-height: 44px`, `--layout-player-height: 84px`, `--layout-content-pad-x: 32px`, `--layout-content-pad-y: 24px`.
  - Spacing scale: `4, 8, 12, 16, 20, 24, 32, 40, 48, 64`.
  - Radii: `--radius-xs: 6px`, `--radius-sm: 8px`, `--radius-md: 12px`, `--radius-lg: 16px`, `--radius-xl: 20px`, `--radius-pill: 999px`.
  - Glass: `--glass-blur: 18px`, `--glass-bg: rgba(14,14,18,0.32)`, `--glass-border: rgba(255,255,255,0.08)`, `--glass-highlight: rgba(255,255,255,0.12)`, `--glass-shadow: 0 12px 30px rgba(0,0,0,0.35)`.
  - Typography scale: `12/1.4 (meta, 500)`, `13/1.4 (table header, 500)`, `14/1.5 (body, 400)`, `16/1.5 (labels, 500)`, `18/1.4 (section, 600)`, `22/1.25 (view title, 600)`, `28/1.2 (album title, 700)`, `36/1.15 (now playing title, 700)`, `44/1.15 (current lyric line, 700)`, `22/1.4 (lyric context, 500)`.
  - Icon sizes: `16, 20, 24, 32, 40` (use `strokeWidth=1.5` for 16-24, `2` for 32-40).
  - Tables: `--table-header-height: 28px`, `--table-row-height: 36px`, `--table-cell-gap: 12px`.
  - Background: `--bg-intensity: 0.35`, `--bg-noise-opacity: 0.18`, `--bg-crossfade-ms: 1200`.
  - Scrollbars: `--scrollbar-track: rgba(255,255,255,0.06)`, `--scrollbar-thumb: rgba(255,255,255,0.18)`.
- Font assets: create `ui/public/fonts/`, download from `https://github.com/rsms/inter/releases/latest` (Inter release zip) and copy `docs/font-files/InterVariable.woff2` + `docs/font-files/InterVariable-Italic.woff2` into `ui/public/fonts/`; copy `LICENSE.txt` and rename to `Inter-OFL.txt` in the same directory.

### B. Layout and Navigation Architecture
- Restructure shell to a 3-pane layout with header + footer.
- Add forward history to route stack and header controls.
- Maintain current routing semantics and data flow.
- Right rail state: add `ui/src/lib/state/rightRail.ts` with `railMode` (nowPlaying, upNext, autoplay, lyrics) and `isRailOpen` (boolean); default open at >=1280px.
- Fullscreen lyrics: add a new route (e.g., `lyrics-fullscreen`) in `ui/src/lib/state/route.ts` and render it via `ui/src/App.svelte` using a new `ui/src/lib/views/LyricsView.svelte`.
- Responsive scope: min window 1100x700; right rail persistent >=1280px and collapses to overlay/drawer at 1100-1279px; no mobile layout.

### C. Background and Material System
- Replace Pixi pipeline with layered CSS gradients, subtle noise overlay, and crossfade transitions.
- Expand theme extraction to provide multiple accent colors (primary/secondary/tertiary) for gradient layers.
- Add CSS vars for secondary/tertiary accents: `--theme-accent-2`, `--theme-accent-3` and RGB components `--theme-accent-2-r/g/b`, `--theme-accent-3-r/g/b`.
- Update Appearance prefs to new background controls (no particles settings).
- Remove `pixi.js` dependency from `ui/package.json` after Pixi removal.

### D. View Refactor Scope
- Core library views: Albums grid, Album Detail hero + tracklist, Tracks table, Artists list.
- Right rail modes: Now Playing, Up Next, Autoplay, Lyrics.
- Lyrics: right-rail view and fullscreen layout.
- Preferences/Diagnostics: restyled to match new system.

### E. Verification and Evidence
- Tauri MCP-only screenshots per milestone; store in `.sisyphus/evidence/cider-ui/`.
- Use a consistent 1440x900 window size for baseline and per-milestone captures.
- Start a Tauri MCP session with `tauri_driver_session` before window sizing or screenshots.
- Use `tauri_manage_window` to enforce 1440x900 sizing and `tauri_webview_screenshot` with `filePath` for evidence.
- Keep before/after comparisons per milestone (manual diff).
- Manual QA for desktop feel, window resizing, and animation smoothness.
- Empty/error states: neutral gradient for no artwork; "Nothing Playing"/"Up Next is empty"/"Lyrics not available" messaging; em dash for missing metadata.

---

## Replace/Remove List
- Remove Pixi/reactive background rendering in `ui/src/lib/components/BackgroundLayer.svelte`.
- Remove particle-related preference keys and UI controls from `ui/src/lib/state/effects.ts`, `ui/src/lib/state/preferences.ts`, `ui/src/lib/views/PreferencesView.svelte`, and `ui/src/lib/components/preferences/AppearancePrefs.svelte`.
- Replace emoji and unicode glyph icons in `ui/src/lib/components/TopBar.svelte`, `ui/src/lib/components/BottomBar.svelte`, `ui/src/lib/views/TracksView.svelte`, `ui/src/lib/views/AlbumDetailView.svelte`, `ui/src/lib/views/ArtistDetailView.svelte`, `ui/src/lib/views/ArtistsView.svelte`, `ui/src/lib/views/SearchResultsView.svelte`, `ui/src/lib/views/SettingsView.svelte`, `ui/src/lib/views/PreferencesView.svelte`, `ui/src/lib/views/DiagnosticsView.svelte`, `ui/src/lib/components/preferences/PlayerPrefs.svelte`, `ui/src/lib/components/TagEditor.svelte`, and `ui/src/lib/components/Modal.svelte` with Lucide icons.
- Remove `pixi.js` from `ui/package.json` after Pixi background removal.
- Replace the queue sidebar inside `NowPlayingView.svelte` with a reusable right-rail component.

---

## Verification Strategy (Manual QA + Tauri MCP)

### Test Decision
- **Infrastructure exists**: NO (no UI test runner configured).
- **User wants tests**: Manual-only with Tauri MCP screenshots.
- **Framework**: None.

### Manual QA (Tauri MCP)
- Launch app with `SERMON_MOCK=1` and `SERMON_SNAPSHOT=1` set (deterministic data), then run `cargo tauri dev`.
- Use Tauri MCP to capture milestone screenshots.
- Start a session with `tauri_driver_session` (action `start`).
- Call `tauri_manage_window` with `action: "resize"` to enforce 1440x900 (logical pixels).
- Call `tauri_webview_screenshot` with `filePath` to save evidence images.
- Store evidence in `.sisyphus/evidence/cider-ui/` with milestone-labeled filenames.
- Verify animations and layout in a real Tauri window (desktop feel).

---

## Task Flow

```
1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7
```

## Parallelization

- None. Each phase builds on previous layout and theming changes.

---

## TODOs

- [x] 1. Foundations and AppShell skeleton ✓

  **What to do**:
  - Update `ui/src/App.svelte` layout to the Cider-style shell with explicit slots for header, sidebar, content, right rail, and player bar.
  - Update `ui/src/lib/components/TopBar.svelte`, `ui/src/lib/components/LeftNav.svelte`, and `ui/src/lib/components/BottomBar.svelte` to the new layout and spacing.
  - Add Lucide icon dependency (`@lucide/svelte`) and replace emoji icons in shell components.
  - Create `ui/public/fonts/` if it does not exist.
  - Download Inter Variable from `https://github.com/rsms/inter/releases/latest` and copy `docs/font-files/InterVariable.woff2`, `docs/font-files/InterVariable-Italic.woff2`, and `LICENSE.txt` (rename to `Inter-OFL.txt`) into `ui/public/fonts/`.
  - Wire `@font-face` + font stack in `ui/src/app.css` for Inter Variable (weights 400-700).
  - Define layout/spacing/typography tokens in `ui/src/app.css` per Section A (include tabular numerals).
  - Implement forward history in `ui/src/lib/state/route.ts`: add `forwardStack`, clear it on `navigate`, `replaceRoute`, and `resetTo`; push current route on `goBack`, pop on `goForward`, and add `canGoForward`.
  - Keep `setRoute` calling `navigate` so it also clears forward history.
  - Wire back/forward controls in `ui/src/lib/components/TopBar.svelte` with disabled states from `canGoBack`/`canGoForward`.

  **Must NOT do**:
  - Do not change playback or library state logic.

  **Parallelizable**: NO (foundation task).

  **References**:
  - `ui/src/App.svelte` - Current shell layout composition.
  - `ui/src/lib/components/TopBar.svelte` - Header controls and search layout.
  - `ui/src/lib/components/LeftNav.svelte` - Navigation and icon usage.
  - `ui/src/lib/components/BottomBar.svelte` - Player controls and icon usage.
  - `ui/src/lib/state/route.ts` - Routing stack (add forward history).
  - `ui/src/app.css` - Tokens and global typography.
  - `ui/public/fonts/InterVariable.woff2` - New Inter variable font asset.
  - `ui/public/fonts/Inter-OFL.txt` - Inter license file (OFL-1.1).
  - `https://github.com/rsms/inter/releases/latest` - Source for Inter Variable assets.

  **Acceptance Criteria**:
  - [x] App shows new shell layout with right rail slot present (even if empty).
  - [x] Back and forward controls work and match header visuals.
  - [x] Forward history clears on new navigation and disables forward button when empty.
  - [x] `replaceRoute` and `resetTo` clear forward history.
  - [x] Inter Variable is loaded and applied to the shell (verify computed font stack).
  - [x] No emoji icons remain in shell components.
  - [x] Tauri MCP screenshot captured: `.sisyphus/evidence/cider-ui/m1-shell.png` (matches `G5g5KTHXAAADzSd.jpg`).

  **Manual Execution Verification (Tauri MCP)**:
  - [x] Launch app via `cargo tauri dev`.
  - [x] Resize window to 1440x900 via `tauri_manage_window`.
  - [x] Capture shell screenshot with Tauri MCP and save to `m1-shell.png`.
  - [x] Verify sidebar, header, right rail slot, and bottom bar align to Cider reference.

- [x] 2. Background and material rewrite ✓

  **What to do**:
  - Replace Pixi background with layered CSS gradients, noise overlay, and slow crossfade transitions in `ui/src/lib/components/BackgroundLayer.svelte`.
  - Do not render the artwork image in the background; use artwork only for color extraction.
  - Update `ui/src/lib/theme/dynamicTheme.ts` to emit multiple accent colors (primary/secondary/tertiary) for gradients.
  - Accent algorithm: convert primary accent to HSL; secondary = hue +25, sat -15, light +10; tertiary = hue -25, sat -10, light -10; clamp 0-100 and convert back to RGB.
  - Set CSS vars in `dynamicTheme.ts`: `--theme-accent-2`, `--theme-accent-3`, and `--theme-accent-2-r/g/b`, `--theme-accent-3-r/g/b`.
  - Use accents in `BackgroundLayer.svelte`: primary for base glow, secondary for top-left radial layer, tertiary for bottom-right radial layer.
  - Remove particle settings and update background preferences in `ui/src/lib/state/effects.ts` and `ui/src/lib/state/preferences.ts`.
  - Add preference keys in `ui/src/lib/state/preferences.ts`: `ui.background.intensity`, `ui.background.noise_opacity`, `ui.background.crossfade_ms` (string values).
  - Update `ui/src/lib/state/effects.ts` to read/write those keys, apply to `--bg-intensity`, `--bg-noise-opacity`, `--bg-crossfade-ms`, and set defaults (0.35, 0.18, 1200).
  - Remove legacy keys: `ui.background.mode`, `ui.background.motion`, and `ui.particles.*` (do not map; reset to new defaults).
  - Restyle Appearance prefs to match the new system in `ui/src/lib/views/PreferencesView.svelte` and `ui/src/lib/components/preferences/AppearancePrefs.svelte`.
  - Map new appearance controls: intensity slider -> `--bg-intensity`, noise toggle/slider -> `--bg-noise-opacity`, crossfade speed -> `--bg-crossfade-ms`.
  - Slider ranges: intensity 0.0-0.6 (step 0.01), noise opacity 0.0-0.3 (step 0.01), crossfade 400-2400ms (step 100).
  - Remove `pixi.js` from `ui/package.json` after Pixi removal.

  **Must NOT do**:
  - Do not keep Pixi or particle modes in the final background system.

  **Parallelizable**: NO (depends on Task 1).

  **References**:
  - `ui/src/lib/components/BackgroundLayer.svelte` - Current Pixi and gradient logic.
  - `ui/src/lib/state/effects.ts` - Theme effects and background settings.
  - `ui/src/lib/theme/dynamicTheme.ts` - Accent color extraction.
  - `ui/src/lib/state/artwork.ts` - Artwork URL pipeline and current artwork store.
  - `ui/src/lib/views/PreferencesView.svelte` - Preferences container for Appearance section.
  - `ui/src/lib/components/preferences/AppearancePrefs.svelte` - UI controls to update/remove.
  - `ui/src/lib/state/preferences.ts` - Preference keys to update/remove.
  - `ui/package.json` - Remove `pixi.js` dependency after Pixi removal.

  **Acceptance Criteria**:
  - [x] Pixi is no longer used by the background layer.
  - [x] `pixi.js` is removed from `ui/package.json` dependencies.
  - [x] Background transitions crossfade smoothly on artwork change.
  - [x] Particle-related preference toggles are removed from Appearance prefs.
  - [x] New background prefs persist and update CSS vars (`--bg-intensity`, `--bg-noise-opacity`, `--bg-crossfade-ms`).
  - [x] Legacy `ui.background.mode` and `ui.background.motion` are no longer read or written.
  - [x] Tauri MCP screenshot captured: `.sisyphus/evidence/cider-ui/m2-background.png` (matches ambiance in `G5g5KTHXAAADzSd.jpg`).

  **Manual Execution Verification (Tauri MCP)**:
  - [x] Trigger an artwork change and observe a slow crossfade.
  - [x] Resize window to 1440x900 via `tauri_manage_window`.
  - [x] Capture screenshot and compare ambience to Cider reference.

- [x] 3. Core library views (Albums, Album Detail, Tracks, Artists) ✓

  **What to do**:
  - Restyle `ui/src/lib/views/AlbumsView.svelte` grid density, typography, hover states.
  - Update `ui/src/lib/views/AlbumDetailView.svelte` hero layout + tracklist to match Cider table density.
  - Restyle `ui/src/lib/views/TracksView.svelte` table (headers, row height, alignment).
  - Update `ui/src/lib/views/ArtistsView.svelte` list or grid styling to match Cider scale.
  - Ensure placeholders for missing artwork or metadata are styled per Cider.
  - Use em dash (U+2014) for missing metadata cells.
  - Replace unicode glyph icons in `AlbumDetailView.svelte` and `TracksView.svelte` with Lucide equivalents.

  **Must NOT do**:
  - Do not change data fetching or playback logic.

  **Parallelizable**: NO (depends on Task 2).

  **References**:
  - `ui/src/lib/views/AlbumsView.svelte` - Grid layout and hover actions.
  - `ui/src/lib/views/AlbumDetailView.svelte` - Hero, tracklist layout.
  - `ui/src/lib/views/TracksView.svelte` - Table structure.
  - `ui/src/lib/views/ArtistsView.svelte` - Artist list layout.

  **Acceptance Criteria**:
  - [x] Album and tracklist screens align with `G5g5KTHXAAADzSd.jpg` and `G5uZqyBWsAASpFn.jpg`.
  - [x] Table headers, row heights, and spacing match Cider density within agreed tolerance.
  - [x] Table header height ~28px and row height ~36px (+/- 4px tolerance).
  - [x] Tauri MCP screenshots captured: `m3-album.png`, `m3-tracks.png`.

  **Manual Execution Verification (Tauri MCP)**:
  - [x] Resize window to 1440x900 via `tauri_manage_window`.
  - [x] Capture album page hero and tracklist screenshot to `.sisyphus/evidence/cider-ui/m3-album.png`.
  - [x] Capture tracks table screenshot to `.sisyphus/evidence/cider-ui/m3-tracks.png`.

- [x] 4. Right rail modes (Now Playing, Up Next, Autoplay) ✓

  **What to do**:
  - Create `ui/src/lib/components/RightRail.svelte` and move queue UI out of `ui/src/lib/views/NowPlayingView.svelte`.
  - Add `ui/src/lib/state/rightRail.ts` to manage `railMode` and `isRailOpen` (UI-only state).
  - Default state: `railMode = 'now-playing'`; `isRailOpen = window.innerWidth >= 1280` on init, and update on resize.
  - Implement rail tabs or controls for Now Playing, Up Next, Autoplay.
  - Add a Lyrics tab/button in the rail control strip; selecting it sets `railMode = 'lyrics'` and renders the lyrics rail panel.
  - Add a "Fullscreen" button/icon in the lyrics rail header that navigates to `lyrics-fullscreen`.
  - Style rail headers, sections, and item rows to match `G9nHpcfXsAAckzv.jpg`.
  - Define empty states: "Up Next is empty" and "Autoplay is Off" (muted, keep rail layout).
  - Autoplay data source: UI-only placeholder (no backend). Always show empty list + "Autoplay is Off" text until backend exists.
  - Queue mapping: if `currentIndex` is not null and `queue[currentIndex]` exists, show it as Now Playing and use `queue.slice(currentIndex + 1)` for Up Next; if `currentIndex` is null, treat `queue[0]` as Now Playing and `queue.slice(1)` as Up Next.
  - Overlay behavior (1100-1279px): default `isRailOpen=false`, show a rail toggle button in `ui/src/lib/components/TopBar.svelte`, open rail as a drawer with scrim; clicking scrim or toggle closes it.
  - Breakpoint behavior: set `isRailOpen=true` when width >=1280px and `false` when width <1280px; keep `railMode` unchanged across resizes.
  - Mount `RightRail` in `ui/src/App.svelte` as a sibling to the main content container so it renders across all routes.

  **Must NOT do**:
  - Do not alter playback queue semantics.

  **Parallelizable**: NO (depends on Task 3).

  **References**:
  - `ui/src/lib/views/NowPlayingView.svelte` - Source for queue markup and item styling.
  - `ui/src/lib/components/RightRail.svelte` - New rail container component.
  - `ui/src/lib/components/TopBar.svelte` - Header controls alignment with rail.
  - `ui/src/lib/components/LeftNav.svelte` - Glass panel styling pattern.
  - `ui/src/lib/state/playback.ts` - Queue data source for Up Next list.
  - `ui/src/lib/state/rightRail.ts` - New UI state for rail mode/open state.

  **Acceptance Criteria**:
  - [x] Right rail shows sections matching Cider: Now Playing, Playing Next, Autoplay.
  - [x] Rail controls and spacing match `G9nHpcfXsAAckzv.jpg`.
  - [x] Empty queue shows "Up Next is empty" message without collapsing the rail.
  - [x] Autoplay off shows "Autoplay is Off" state text.
  - [x] At 1100-1279px, rail defaults closed and opens as overlay via toggle.
  - [x] At >=1280px, rail is persistent without overlay.
  - [x] Tauri MCP screenshot captured: `.sisyphus/evidence/cider-ui/m4-right-rail.png`.

  **Manual Execution Verification (Tauri MCP)**:
  - [x] Resize window to 1440x900 via `tauri_manage_window`.
  - [x] Capture right rail screenshot with active queue.

- [x] 5. Lyrics (rail + fullscreen) ✓

  **What to do**:
  - Implement lyrics mode in right rail, matching the emphasis/dimming in `G5uZqyBWsAASpFn.jpg`.
  - Add fullscreen lyrics view aligned to `G_CHWleW4AEAK4i.jpg` using new `ui/src/lib/views/LyricsView.svelte` and a `lyrics-fullscreen` route.
  - Navigation flow: open via the Lyrics rail "Fullscreen" control; exit via a back button in `LyricsView.svelte` (calls `goBack`) and `Escape` key handler.
  - Preserve `railMode='lyrics'` and `isRailOpen` when returning from fullscreen.
  - Add `ui/src/lib/state/lyrics.ts` with a `currentLyrics` store that derives from `currentTrack` + mock fixture data.
  - Add `ui/src/lib/data/lyricsFixtures.ts` keyed by track id (used only when `SERMON_MOCK=1`).
  - Lyrics fixture schema:
    - `type LyricsFixture = { trackId: number; lines: string[] }`
    - `export const lyricsFixtures: Record<number, string[]> = { [trackId]: ["Line 1", "Line 2", "Line 3"] }`
    - Track ID mapping: use numeric `TrackEventData.id`; in mock mode this is the fixture index (see `Fixtures.getTracks()[0]` seeded in `initPlaybackListeners`).
    - Resolve with `lyricsFixtures[$currentTrack.id] ?? null`.
  - Compute active line from playback progress: `currentLineIndex = clamp(floor($progress * lines.length), 0, lines.length - 1)` and show 2 lines of context before/after.
  - Define empty state for missing lyrics: "Lyrics not available for this track."

  **Must NOT do**:
  - Do not change lyrics fetching or timing logic.

  **Parallelizable**: NO (depends on Task 4).

  **References**:
  - `ui/src/lib/views/NowPlayingView.svelte` - Current lyrics/track context.
  - `ui/src/lib/state/route.ts` - Add `lyrics-fullscreen` route.
  - `ui/src/lib/state/playback.ts` - `currentTrack` and `progress` sources.
  - `ui/src/lib/views/LyricsView.svelte` - New fullscreen lyrics view.
  - `ui/src/lib/state/lyrics.ts` - New lyrics store.
  - `ui/src/lib/data/lyricsFixtures.ts` - Mock lyrics data for fixtures.
  - `ui/src/lib/components/Modal.svelte` - Overlay/backdrop pattern if needed for fullscreen transitions.

  **Acceptance Criteria**:
  - [x] Lyrics rail matches `G5uZqyBWsAASpFn.jpg` emphasis/dimming.
  - [x] Fullscreen lyrics layout matches `G_CHWleW4AEAK4i.jpg`.
  - [x] Active lyric line advances based on playback progress using the defined algorithm.
  - [x] Missing lyrics shows muted "Lyrics not available for this track." text.
  - [x] With `SERMON_MOCK=1`, lyrics fixtures render for matching `currentTrack.id`.
  - [x] Tauri MCP screenshots captured: `m5-lyrics-rail.png`, `m5-lyrics-fullscreen.png`.

  **Manual Execution Verification (Tauri MCP)**:
  - [x] Resize window to 1440x900 via `tauri_manage_window`.
  - [x] Capture rail lyrics screenshot to `.sisyphus/evidence/cider-ui/m5-lyrics-rail.png`.
  - [x] Capture fullscreen lyrics screenshot to `.sisyphus/evidence/cider-ui/m5-lyrics-fullscreen.png`.

- [x] 6. Preferences and Diagnostics styling ✓

  **What to do**:
  - Restyle Preferences shell to match Cider (spacing, typography, list density).
  - Remove legacy background/particles controls and replace with new background options.
  - Ensure reduce-effects and accessibility toggles are preserved.
  - Restyle `ui/src/lib/views/DiagnosticsView.svelte`: use glass panels for sections, apply typography scale, and swap arrow glyphs for Lucide icons.

  **Must NOT do**:
  - Do not add new settings categories beyond appearance/diagnostics.

  **Parallelizable**: NO (depends on Task 5).

  **References**:
  - `ui/src/lib/views/PreferencesView.svelte` - Preferences view container.
  - `ui/src/lib/components/preferences/AppearancePrefs.svelte` - Appearance controls and layout.
  - `ui/src/lib/state/preferences.ts` - Preference keys and defaults.
  - `ui/src/lib/views/DiagnosticsView.svelte` - Diagnostics layout to restyle.

  **Acceptance Criteria**:
  - [x] Preferences UI aligns with new visual system.
  - [x] Old background/particle toggles are removed.
  - [x] Status glyphs in Preferences are replaced with Lucide icons.
  - [x] Diagnostics view uses glass panels and updated iconography.
  - [x] Tauri MCP screenshot captured: `.sisyphus/evidence/cider-ui/m6-preferences.png`.
  - [x] Tauri MCP screenshot captured: `.sisyphus/evidence/cider-ui/m6-diagnostics.png`.

  **Manual Execution Verification (Tauri MCP)**:
  - [x] Resize window to 1440x900 via `tauri_manage_window`.
  - [x] Capture preferences screenshot to `.sisyphus/evidence/cider-ui/m6-preferences.png`.
  - [x] Capture diagnostics screenshot to `.sisyphus/evidence/cider-ui/m6-diagnostics.png`.

- [x] 7. Global polish pass ✓

  **What to do**:
  - Tune hover/press/focus states and scrollbar styling across updated views.
  - Adjust motion timings to match Cider-like ease and delays.
  - Validate reduced-effects mode still works with new background system.
  - Sweep remaining unicode glyphs (SearchResults, Diagnostics, TagEditor, Modal) and replace with Lucide icons.
  - Validate window resizing behavior: persistent rail >=1280px, rail overlay 1100-1279px, min 1100x700.

  **Must NOT do**:
  - Do not add new features outside visual polish.

  **Parallelizable**: NO (depends on Task 6).

  **References**:
  - `ui/src/app.css` - Global tokens and interaction styles.
  - `ui/src/lib/state/effects.ts` - Reduced-effects logic.

  **Acceptance Criteria**:
  - [x] Interactions feel consistent across views and match Cider timing.
  - [x] Reduced-effects mode still reduces motion and blur.
  - [x] Right rail collapses to overlay at 1100-1279px and stays persistent >=1280px.
  - [x] App remains usable at 1100x700 without layout breakage.
  - [x] No emoji/unicode icon glyphs remain in UI (verify with `rg --pcre2 "(\\x{1F3B5}|\\x{1F4BF}|\\x{1F464}|\\x{23EE}|\\x{23ED}|\\x{23F8}|\\x{25B6}|\\x{270E}|\\x{2713}|\\x{2717}|\\x{25CB}|\\x{2190}|\\x{2192}|\\x{00D7}|\\x{25A1}|\\x{1F50A}|\\x{26A0})" ui/src`).
  - [x] Tauri MCP screenshot captured: `.sisyphus/evidence/cider-ui/m7-polish.png`.

  **Manual Execution Verification (Tauri MCP)**:
  - [x] Resize window to 1440x900 via `tauri_manage_window` for baseline capture.
  - [x] Resize window to 1200x700 and verify rail overlay behavior.
  - [x] Capture a representative polished screen to `.sisyphus/evidence/cider-ui/m7-polish.png`.

---

## Commit Strategy

- No commits unless the user explicitly requests them. If requested, commit after each milestone (Tasks 1-7).

---

## Success Criteria

### Verification Commands
```bash
SERMON_MOCK=1 SERMON_SNAPSHOT=1 cargo tauri dev  # Deterministic app state for MCP captures
```

### Final Checklist
- [x] All Must Have requirements implemented.
- [x] All Must NOT Have guardrails respected.
- [x] Tauri MCP screenshots captured for each milestone and saved to `.sisyphus/evidence/cider-ui/`.
