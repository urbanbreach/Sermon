# UI/UX Rework Plan — Apple-like Liquid Glass

## Context

### Original Request
Full UI/UX rework to a polished Apple-like “Liquid Glass” design with adjustable effects, dynamic reactive backgrounds (static frosted + music-reactive shader/particles), and consolidated Preferences. Remove Settings tab. Add a lightweight backend audio analysis pipeline to drive visuals. Use Tauri MCP for automated QA. Make a baseline git commit before refactor.

### Interview Summary
**Key Discussions**:
- Entire UI/UX is in scope (all views + shell).
- Effects must be visible but non-intrusive and adjustable by users via sliders/toggles.
- Backgrounds: two modes (static frosted album-art glass + music-reactive shader/particles). Reactive mode is optional and off by default.
- No performance tiers; granular user controls instead.
- Settings tab removed; all settings consolidated into Preferences with sectioned categories.
- Audio analysis pipeline in Rust backend (peak/RMS + FFT). No WebAudio fallback.
- WebGPU preferred with WebGL fallback.
- Automated verification via Tauri MCP only; no snapshot scripts.
- Baseline git commit required before refactor.

**Research Findings**:
- Entry points: `ui/src/main.ts`, `ui/src/App.svelte`.
- Shell layout: `ui/src/lib/components/TopBar.svelte`, `ui/src/lib/components/LeftNav.svelte`, `ui/src/lib/components/BottomBar.svelte`.
- Routing: `ui/src/lib/state/route.ts` and `currentRouteName` switching in `ui/src/App.svelte`.
- Settings/Preferences: `ui/src/lib/views/SettingsView.svelte`, `ui/src/lib/views/PreferencesView.svelte`, `ui/src/lib/components/preferences/*`.
- Theme engine: `ui/src/lib/theme/dynamicTheme.ts` (deterministic 48x48 canvas sampling), applied in `ui/src/lib/views/NowPlayingView.svelte`.
- Glass tokens: `ui/src/app.css` (`--glass-bg`, `--glass-border`, `--glass-shadow`, `--glass-blur`, `--glass-radius`).
- Effects toggles: `ui/src/lib/state/effects.ts` with UI controls in `ui/src/lib/views/SettingsView.svelte`.
- No existing WebGL/particle system or audio analysis pipeline.

### Metis Review (Applied)
**Guardrails**:
- No new navigation structure beyond removing Settings and consolidating Preferences.
- No new playback behavior or audio pipeline changes beyond analysis taps.
- No extra settings categories unless required for visuals/effects.
- Follow existing theme token patterns in `ui/src/app.css`.
- Explicit performance targets and fallback behavior defined.

---

## Work Objectives

### Core Objective
Deliver a cohesive Apple-like Liquid Glass UI/UX across the entire app, with controlled, adjustable effects and a responsive background system driven by album art and audio analysis.

### Concrete Deliverables
- Consolidated Preferences (Settings removed) with all effect controls.
- Updated liquid-glass design tokens and consistent application across all views.
- Background rendering system with two modes (static frosted + music-reactive shader/particles).
- Backend audio analysis pipeline (peak/RMS + FFT) exposed to UI.
- Automated UI verification via Tauri MCP after `cargo tauri dev`.

### Definition of Done
- All views in `ui/src/lib/views/` visually updated to the new style.
- Settings route removed; Preferences contains all settings including effects controls.
- Background modes toggle correctly with minimal default motion.
- Audio-reactive shader and particles respond smoothly to music and pause gracefully.
- Performance targets met (see Acceptance Criteria).

### Must Have
- Adjustable effects (sliders/toggles) with safe defaults.
- Two background modes with explicit user selection.
- Music-reactive visuals fed by Rust analysis pipeline.

### Must NOT Have (Guardrails)
- No new navigation architecture beyond removing Settings and updating Preferences.
- No WebAudio fallback pipeline.
- No new snapshot scripts or extra QA frameworks.
- No changes to playback behavior or audio output logic.

---

## Verification Strategy (Tauri MCP Only)

### Test Decision
- **Infrastructure exists**: Yes (Tauri + MCP).
- **User wants tests**: Automated QA via Tauri MCP; no snapshot scripts.
- **Framework**: Tauri MCP (UI automation), automated checks and screenshots.

### Automated QA (Tauri MCP)
All verification tasks must:
- Start the app with `cargo tauri dev`.
- Connect to Tauri MCP (`tauri_driver_session start`).
- Use `tauri_webview_*` actions to navigate, toggle settings, and verify UI state.
- Capture screenshots for each major view and mode.

---

## Performance Targets (Defaults Applied)
- **UI render budget**: background renderer < 2.5ms/frame on mid-tier hardware.
- **Frame rate target**: sustained 60 FPS on mid-tier hardware; 30+ FPS on low-end.
- **Audio analysis update rate**: 30 Hz default.

**Measurement Method**:
- Log `app.ticker.FPS` (Pixi) and average frame time via `performance.now()` deltas over 30s; optionally expose a dev-only on-screen stat overlay.
- Use Chromium performance overlay in Tauri devtools to confirm GPU frame time.
- For WebGPU fallback testing, temporarily force Pixi `preference: 'webgl'` and confirm renderer type.

---

## Task Flow

```
Foundations → Preferences Consolidation → Background Renderer → Audio Analysis → View-by-View Polish → QA/Verification
```

### Parallelization
- UI shell restyling and Preferences consolidation can proceed in parallel.
- Background renderer and audio analysis pipeline can proceed in parallel after design tokens are finalized.

---

## TODOs

> Implementation + verification are coupled. Each task includes Tauri MCP verification steps.

### 1) Baseline Commit + Branch Prep
**What to do**:
- Ensure working tree clean (or stash unrelated changes).
- Commit current state before refactor.

**References**:
- `ui/src/` (entire UI scope baseline).

**Acceptance Criteria**:
- Baseline commit created before any refactor changes.
- Commit message example: `chore(ui): baseline before liquid-glass rework`.

---

### 2) Liquid Glass Design Tokens Refresh
**What to do**:
- Refine `ui/src/app.css` glass tokens to match Apple-like depth, sheen, and saturation:
  - Target ranges: `--glass-blur` 16–28px, `--glass-bg` alpha 0.55–0.7, `--glass-border` alpha 0.08–0.18.
- Add optional variables for glow intensity, blur radius, and highlight strength (user-adjustable).
**References**:
- `ui/src/app.css` (existing tokens).
- `ui/src/lib/components/TopBar.svelte`
- `ui/src/lib/components/LeftNav.svelte`
- `ui/src/lib/components/BottomBar.svelte`

**Acceptance Criteria**:
- `--glass-bg`, `--glass-border`, `--glass-shadow` values updated in `ui/src/app.css`.
- TopBar/LeftNav/BottomBar backgrounds reference those tokens.
- Tauri MCP: open `Albums`, `Artists`, `Now Playing` and verify panels use glass classes (check computed styles for `backdrop-filter`).

---

### 3) Preferences Consolidation (Remove Settings)
**What to do**:
- Remove Settings route from `ui/src/lib/state/route.ts` and `ui/src/App.svelte`.
- Update `ui/src/lib/components/LeftNav.svelte` to remove Settings navigation entry, and remove `settings` from the `SimpleRouteName` union.
- Ensure all SettingsView functionality is reachable via Preferences categories (no loss of feature parity).
- Add Appearance category:
  - Create `ui/src/lib/components/preferences/AppearancePrefs.svelte`.
  - Add category to `ui/src/lib/views/PreferencesView.svelte` list and switch block, and update the local `PreferenceCategory` union type there.
- Initialize library listeners by calling `initLibrary()` in `ui/src/App.svelte` (scan progress events required for LibraryPrefs).
- Remove/guard `initLibrary()` call from `ui/src/lib/views/TracksView.svelte` to avoid duplicate listeners.
- Add `data-testid` attributes to Preferences categories and buttons for MCP automation.
  - Extend `PreferenceCategory` in `ui/src/lib/state/preferences.ts` and handle `resetCategoryToDefaults('appearance')`.
  - Add `AppearanceSettings` interface + `appearanceSettings` store in `ui/src/lib/state/preferences.ts`.
  - Update `loadCategorySettings()` and `loadAllPreferences()` to support `appearance`.
  - Update backend settings categories in `src-tauri/src/commands/settings.rs`: add `appearance` to `VALID_CATEGORIES`, add keys in `get_category_keys`, and defaults in `get_category_defaults`.
- Preserve all SettingsView features by mapping to existing Preferences tabs:
  - Library Folders → `ui/src/lib/components/preferences/LibraryPrefs.svelte` (reuse `open` dialog from `ui/src/lib/views/SettingsView.svelte`; include rescan + scan progress UI using `ui/src/lib/state/library`).
  - Audio → `ui/src/lib/components/preferences/PlayerPrefs.svelte` (output device/mode/policy/timing/fade + buffer size).
  - Theme → new `AppearancePrefs.svelte` (reduce effects, blur/glow/border + sliders).
  - Artwork Providers → `ui/src/lib/components/preferences/InternetPrefs.svelte` (provider toggles already wired to `effects.ts`).
  - General → `ui/src/lib/components/preferences/GeneralPrefs.svelte` (existing placeholders).
- Persistence strategy (explicit):
  - **Appearance category includes BOTH toggle keys and numeric keys** so Reset-to-Defaults works consistently.
  - Add toggle keys to `appearance` category in `src-tauri/src/commands/settings.rs`:
    - `ui.reduce_effects`, `ui.theme.blur`, `ui.theme.glow`, `ui.theme.border_highlight`.
  - Numeric keys live in `appearance` category as listed below.
  - Source of truth: `appearance` category values; `effects.ts` mirrors toggles + numeric values into stores and applies CSS.
  - `resetCategoryToDefaults('appearance')` must call `resetEffectsDefaults()` so `effects.ts` rehydrates toggles and CSS variables.
  - `loadEffectsSettings()` must call `loadCategorySettings('appearance')` to hydrate numeric stores.
  - All Appearance writes go through `saveCategorySetting` (no direct `cmd_settings_set` for appearance keys).

- Ensure settings persist via `ui/src/lib/state/effects.ts` (keep existing keys):
  - `ui.reduce_effects`
  - `ui.theme.blur`
  - `ui.theme.glow`
  - `ui.theme.border_highlight`
- Add numeric slider keys (string-encoded numbers) with explicit ranges/defaults:
  - `ui.theme.blur_px` (0–32, default 16)
  - `ui.theme.glow_strength` (0–1, default 0.35)
  - `ui.theme.border_strength` (0–1, default 0.2)
  - `ui.background.mode` (enum string: `static` | `reactive`, default `static`) **NOT a slider**
  - `ui.background.intensity` (0–1, default 0.3)
  - `ui.background.motion` (0–1, default 0.2)
  - `ui.particles.density` (0–1, default 0.2)
  - `ui.particles.speed` (0–1, default 0.3)
  - `ui.particles.intensity` (0–1, default 0.3)
  - Store values as strings (e.g., "0.3") consistent with existing settings API.
  - Add defaults for Appearance category in `src-tauri/src/commands/settings.rs` and include all keys in `get_category_keys("appearance")`.
- Background mode selector UI:
  - Add segmented control (Static | Reactive) in `AppearancePrefs.svelte` bound to a `backgroundMode` store in `effects.ts`.
  - Bind sliders to numeric stores from `effects.ts` (`bgIntensity`, `bgMotion`, `particlesDensity`, etc.).
  - Use `saveCategorySetting('appearance', key, value)` on `change` events (not `input`) to avoid IPC flooding.
  - `BackgroundLayer.svelte` subscribes to the `backgroundMode` store and toggles Pixi canvas on/off.
  - Call `resetCategoryToDefaults('appearance')` from Preferences and then `resetEffectsDefaults()` to reapply toggles and numeric defaults.
- Add `data-testid` hooks for MCP automation:
  - `data-testid="appearance-toggle-reduce"`, `appearance-slider-bg-intensity`, `appearance-mode-static`, `appearance-mode-reactive` in `AppearancePrefs.svelte`.
  - `data-testid="prefs-reset-defaults"` on the reset button in `PreferencesView.svelte`.
  - `data-testid="prefs-nav-{category}"` on each category button in `PreferencesView.svelte`.
  - `data-testid="glass-panel"` on shell panels (TopBar/LeftNav/BottomBar).
  - `data-testid="background-layer"` on BackgroundLayer root.


- Sync rules (explicit):
  - Appearance slider changes call `saveCategorySetting('appearance', key, value)` and then update effects.ts stores so CSS variables update immediately.
  - `resetCategoryToDefaults('appearance')` triggers `resetEffectsDefaults()` to reapply toggles and CSS.
- Extend `ui/src/lib/state/effects.ts`:
  - Add numeric stores and parsers (string → number with defaults).
  - `applyEffects()` must respect `reduce_effects` and set CSS variables even when toggles are on.
  - Call `loadEffectsSettings()` from `ui/src/App.svelte` on app start (Settings view removed), then `loadCategorySettings('appearance')` and `syncAppearanceToEffects()` in that order.
  - Add `resetEffectsDefaults()` to restore all Appearance defaults.
  - Provide helper `applyAppearanceToCSS()` called after any slider change to set `--glass-blur`, `--glass-glow-strength`, `--glass-border-strength`, `--bg-blur`, `--bg-intensity`.
- Map `appearanceSettings` → effects stores in a single function `syncAppearanceToEffects()` that runs after `loadCategorySettings('appearance')` and after save.
- Deprecate direct `cmd_settings_get/set` usage for appearance keys; only use category API for numeric sliders/mode.
- Sync sequence (explicit):
  1) `loadEffectsSettings()` loads boolean toggles.
  2) `loadCategorySettings('appearance')` loads numeric + mode.
  3) `syncAppearanceToEffects()` applies numeric + mode to stores and CSS.
  4) `saveCategorySetting()` on change triggers reload + `syncAppearanceToEffects()`.


**References**:
- `ui/src/lib/views/SettingsView.svelte` (source for effect controls).
- `ui/src/lib/views/PreferencesView.svelte` (tabs).
- `ui/src/lib/components/preferences/*` (pattern for tabs).
- `ui/src/lib/state/effects.ts` (persistence keys + new slider keys).
- `ui/src/lib/state/effects.ts` (background mode store ownership and defaults).

**Acceptance Criteria**:
- Settings tab removed from LeftNav and App route switch.
- Preferences includes “Appearance” tab with toggles and sliders.
- Reset to Defaults works for Appearance category (backend category defaults apply).
- Tauri MCP: use `data-testid="appearance-mode-static"` and `appearance-mode-reactive` to toggle mode; verify persistence after reload.
- Tauri MCP: adjust `data-testid="appearance-slider-bg-intensity"` and verify values persist after reload.

---

### 4) Background Rendering System (Static Frosted Mode)
**What to do**:
- Insert background layer in `ui/src/App.svelte` as the first child in `.app-shell`, before `TopBar`.
- Set stacking order:
  - Background layer: z-index 0.
  - Shell panels (TopBar/LeftNav/BottomBar/content): z-index 10.
  - NowPlaying overlay: z-index 100 (existing).
- Update `.app-shell` and `.content-area` background colors to `transparent` so the background layer is visible; keep other panels using glass tokens.
- Add `ui/src/lib/state/artwork.ts` (new) to centralize current artwork URL resolution and theme application, following the pattern of `ui/src/lib/state/effects.ts` (store + load/apply functions):
  - Exports: `currentArtworkUrl` store, `initArtworkStore()` initializer, `applyArtworkTheme()` helper.
  - Move artwork loading logic from `ui/src/lib/views/NowPlayingView.svelte` and `ui/src/lib/components/BottomBar.svelte` into this shared store.
  - Call `computeThemeFromImageSrc()` and `applyThemeToDocument()` inside `applyArtworkTheme()` so the theme updates even when Now Playing is not open.
  - Handle mock/snapshot fixtures using existing logic from `NowPlayingView.svelte` and `Fixtures`.
- Theme update trigger (single choice):
  - Implement in `ui/src/lib/state/artwork.ts` as a derived store on `currentTrack` that calls `applyArtworkTheme()` (single source of truth).
  - Initialize the artwork store in `ui/src/App.svelte` by importing and calling `initArtworkStore()` to ensure subscriptions are active on app start.
  - Do NOT duplicate theme application in `NowPlayingView.svelte` after this change.
- Update `NowPlayingView.svelte` and `BottomBar.svelte` to consume `currentArtworkUrl` from the new store and remove duplicate artwork loaders.
- BackgroundLayer static implementation (Task 4):
  - Ownership: component subscribes to stores internally (no external props).
  - Exports: none (uses internal stores only).
  - Render a full-bleed `div` with `background-image: url(currentArtworkUrl)`.
  - Apply CSS: `filter: blur(var(--bg-blur)) saturate(120%)` and overlay gradient from `--theme-bg-0/1`.
  - Map sliders to CSS/behavior:
    - `ui.background.intensity` → overlay opacity (0–1) for gradient layer.
    - `ui.background.motion` → CSS animation duration (higher value = faster drift).
    - `ui.theme.blur_px` → `--bg-blur` (0–32px).
  - Minimal motion using CSS keyframes (speed mapped from `ui.background.motion`).
  - Task 5 extends the same component with Pixi canvas for reactive mode.
  - Update `ui/src/app.css` to define `--glass-glow-strength`, `--glass-border-strength`, `--bg-blur`, `--bg-intensity` and use them in component styles.
  - Set `NowPlayingView.svelte` background to `transparent` so the global background layer remains visible.
  - Fallback behavior (implemented in `ui/src/lib/state/artwork.ts`):
    - If `currentArtworkUrl` becomes null, keep last artwork for 10s using a timer stored in the artwork store.
    - After 10s, clear artwork, fade to gradient-only background, and call `resetTheme()`.



**References**:
- `ui/src/App.svelte` (shell layout insertion point).
- `ui/src/lib/views/NowPlayingView.svelte` (artwork source to extract).
- `ui/src/lib/state/playback.ts` (currentTrack store).
- `ui/src/lib/api/artwork.ts` (artwork IPC helpers).
- `ui/src/lib/theme/dynamicTheme.ts` (theme colors).

**Acceptance Criteria**:
- Background is visible on all views and updates on track change.
- Default motion uses `ui.background.motion=0.2` (verify animation duration reflects this).
- Tauri MCP: change tracks, confirm background updates and no abrupt motion.

---

### 5) Background Rendering System (Music-Reactive Shader + Particles)
**What to do**:
- Add dependency in `ui/package.json`: `pixi.js` (PixiJS v8).
- Implement Pixi setup in `ui/src/lib/components/BackgroundLayer.svelte`:
  - Create a `div` container for Pixi canvas; initialize `new Application()` with `{ preference: 'webgpu', resizeTo: container }` and allow WebGL fallback.
  - Use `app.ticker` for render loop; clean up via `app.destroy(true)` on component teardown.
  - Map audio analysis values to shader uniforms and particle container properties.
  - Use `PIXI.Filter` with fragment shader for WebGL path; force WebGL for the filter if WebGPU cannot run GLSL filters.
  - Shader stub (fragment):
    ```glsl
    uniform float uTime, uBass, uMid, uHigh, uIntensity, uMotion;
    uniform sampler2D uNoise;
    void main(){
      vec2 uv = vTextureCoord;
      float n = texture2D(uNoise, uv + uTime * 0.02).r;
      float glow = (uBass*0.6 + uMid*0.3 + uHigh*0.1) * uIntensity;
      gl_FragColor = vec4(vec3(n) * glow, 1.0);
    }
    ```
  - Generate `uNoise` as a 64x64 random noise texture in code.
  - If WebGPU is active, run Pixi in WebGL mode for the background filter until WGSL filter support is confirmed.
  - Log renderer type: `app.renderer.type` or `app.renderer.name` for verification.
- Shader/particle spec (minimal):
  - FFT band mapping (from 512 bins):
    - `uBass`: average bins 0–8 (approx 0–172Hz)
    - `uMid`: average bins 9–64 (approx 172–1375Hz)
    - `uHigh`: average bins 65–128 (approx 1375–2750Hz)
  - Shader uniforms: `uTime`, `uBass`, `uMid`, `uHigh`, `uIntensity`, `uMotion`.
  - Particle count derived from `ui.particles.density` (0–1 mapped to 200–1200 sprites).
  - Motion speed from `ui.particles.speed`; opacity from `ui.particles.intensity`.
  - Particle texture: 8x8 soft circle generated in code (no external assets).
- Switch modes:
  - Static: shader uniforms fixed, particles disabled.
  - Reactive: shader uniforms updated from `audioLevels` store and particles enabled.

**References**:
- `ui/src/lib/components/BackgroundLayer.svelte` (new).
- `ui/src/lib/state/playback.ts` (new audio analysis store).
- PixiJS docs: https://pixijs.com/ (Application + WebGPU preference).
- Pixi Filter example (inline in plan): use the shader stub below with `new PIXI.Filter(vertex, fragment, uniforms)` applied to a full-screen sprite. (No external dependency required.)

**Acceptance Criteria**:
- Reactive mode toggles on/off with no stutter.
- Particles respond to audio energy and stop gracefully on pause.
- WebGPU/WebGL renderer type logged at startup and recorded in console.
- Tauri MCP: toggle reactive mode, verify shader and particles change with playback.
- Force fallback test: set Pixi preference to `webgl` and confirm renderer type changes.
- Shader uniforms update live when `audioLevels` values change.

---

### 6) Backend Audio Analysis Pipeline (Rust)
**What to do**:
- Modify `AudioPlayback::fill_ring_buffer()` in `src-tauri/src/lib.rs` to store the last decoded `converted` buffer on `AudioPlayback` (e.g., `last_samples: Vec<f32>`).
- In the audio thread loop (after `fill_ring_buffer()`), feed `playback.last_samples` into an analyzer owned by the audio thread.
- Analyzer buffering strategy:
  - Maintain a rolling `VecDeque<f32>` (mono samples) inside the analyzer.
  - On each tick, append samples from `last_samples`, drop oldest to keep 1024 window.
  - Run FFT every 512 samples (hop size) to match window + overlap.
  - Reuse preallocated FFT buffers to avoid allocations.
- Analyzer details:
  - Window size: 1024 samples, hop size: 512.
  - Mixdown to mono by averaging channels before FFT.
  - FFT: 1024-point FFT using `rustfft`, take bins 0–511 for the payload.
  - `peak` = max(abs(sample)) over the 1024-sample window.
  - `rms` = sqrt(mean(sample^2)) over the 1024-sample window.
  - Normalize bands to 0–1 using `magnitude / maxMagnitude` per window; apply EMA smoothing (alpha 0.2).
- Add FFT dependency to `src-tauri/Cargo.toml` (since analyzer lives in `src-tauri/src/lib.rs`): `rustfft = "6"`.
- Emit event from `src-tauri/src/lib.rs` (e.g., `evt_audio_levels`) at 30 Hz using a new tick alongside `position_tick` and `audio_tick`.
- Ensure analyzer runs only when playback state is Playing; send zeros when Paused/Stopped.

**Event Payload**:
```
{
  "peak": f32,
  "rms": f32,
  "bands": [f32; 512],
  "timestamp_ms": u64
}
```

**UI Wiring**:
- Define `AudioLevelsEvent` struct in `src-tauri/src/commands/playback.rs` (alongside other event payload structs) and export it for use in `src-tauri/src/lib.rs` emission.
- Add `AudioLevelsEvent` type in `ui/src/lib/types/playback.ts` with fixed `bands: number[]` length 512.
- Add `audioLevels` store in `ui/src/lib/state/playback.ts` and a listener for `evt_audio_levels` within `initPlaybackListeners()`.
- Initialize `audioLevels` with zeros on startup and when playback is stopped.
- Pass `audioLevels` into `BackgroundLayer.svelte` for shader/particle modulation.

**References**:
- `src-tauri/src/lib.rs` (audio thread loop, `fill_ring_buffer`, event emission).
- `src-tauri/Cargo.toml` (add FFT dependency).
- `ui/src/lib/types/playback.ts` (define new event payload type).
- `ui/src/lib/state/playback.ts` (listen to `evt_audio_levels` and store values).
- `ui/src/lib/components/BackgroundLayer.svelte` (consume `audioLevels`).

**Acceptance Criteria**:
- Audio analysis events emitted at ~30 Hz (configurable).
- UI receives values and uses them for shader/particles.
- Tauri MCP: verify values change during playback, stable at pause.

---

### 7) View-by-View UI/UX Rework
**What to do**:
- Update all views in `ui/src/lib/views/` to match new Liquid Glass style.
- Apply consistent typography, spacing, and glass depth across cards and panels.
- Ensure Now Playing and Bottom Bar feel premium and Apple-like.

**References**:
- `ui/src/lib/views/AlbumsView.svelte`
- `ui/src/lib/views/ArtistsView.svelte`
- `ui/src/lib/views/TracksView.svelte`
- `ui/src/lib/views/NowPlayingView.svelte`
- `ui/src/lib/views/AlbumDetailView.svelte`
- `ui/src/lib/views/ArtistDetailView.svelte`
- `ui/src/lib/views/SearchResultsView.svelte`
- `ui/src/lib/views/DiagnosticsView.svelte`

**Acceptance Criteria (Per-View Checklist)**:
- Glass panels use `--glass-bg`, `--glass-border`, `--glass-shadow` tokens.
- Typography scale matches shell (title 20–24px, subtitle 14–16px).
- Hover/active states have subtle motion (scale <= 1.02) and no flashing.
- Content remains readable over light artwork (text shadow or contrast overlay).
- At least one action control per view has visible focus/active state.
- Tauri MCP: visit each view and confirm checklist visually.
- Capture screenshots per view to confirm layout/spacing consistency.

---

### 8) Preferences UX Redesign + Sliders
**What to do**:
- Add sliders/toggles for effect intensity (blur, glow, border strength, particle density, motion speed, background intensity).
- Slider ranges + defaults (string-encoded numbers):
  - `ui.theme.blur_px` 0–32 (default 16)
  - `ui.theme.glow_strength` 0–1 (default 0.35)
  - `ui.theme.border_strength` 0–1 (default 0.2)
  - `ui.background.intensity` 0–1 (default 0.3)
  - `ui.background.motion` 0–1 (default 0.2)
  - `ui.particles.density` 0–1 (default 0.2)
  - `ui.particles.speed` 0–1 (default 0.3)
  - `ui.particles.intensity` 0–1 (default 0.3)
- Ensure live preview and immediate feedback.
- Group effects clearly: Background, Glass, Motion, Particles.
- Apply slider updates to CSS variables (blur/glow/border) and to `BackgroundLayer.svelte` uniforms.

**References**:
- `ui/src/lib/components/preferences/AppearancePrefs.svelte`
- `ui/src/lib/state/effects.ts`
- `ui/src/app.css` (CSS variables to drive)
- `ui/src/lib/components/BackgroundLayer.svelte` (apply uniforms)

**Acceptance Criteria**:
- All effect controls respond live and persist in settings.
- Default values reflect “calm” look.
- Tauri MCP: adjust sliders, observe real-time visual changes.

---

### 9) UI/UX Design Review Checkpoints (Frontend UI/UX Subagent)
**What to do**:
- Schedule 3 checkpoints: after tokens, after background modes, after full view refresh.
- Use frontend UI/UX subagent to review each checkpoint and list improvements.
- Record each checkpoint report in `artifacts/ui-ux-checkpoints/` as a dated `.md` note.
- Mark each report item as addressed or deferred in the next checkpoint note.

**Acceptance Criteria**:
- Each checkpoint yields a short change list and adjustments are incorporated.
- Each checkpoint note includes screenshots captured via Tauri MCP.

---

### 10) Final Tauri MCP Verification Sweep
**What to do**:
- Launch `cargo tauri dev`.
- Use Tauri MCP to verify each view, settings, and background mode.
- Capture screenshots for key screens and modes.

**Acceptance Criteria**:
- All screens verified and stable.
- Automated navigation shows no broken routes or missing sections.

---

## Commit Strategy
- Baseline commit before refactor.
- Additional commits after each major phase:
  - `feat(ui): refresh liquid-glass tokens`
  - `feat(ui): consolidate preferences and remove settings`
  - `feat(ui): add background renderer modes`
  - `feat(audio): add analysis pipeline for visuals`
  - `feat(ui): rework views to apple-like styling`

---

## Success Criteria
- Entire UI/UX updated to Apple-like liquid glass aesthetic.
- Settings removed; Preferences fully consolidated with effects controls.
- Backgrounds include static frosted mode + optional music-reactive shader/particles.
- Audio analysis pipeline operational with smooth reactivity.
- Automated QA via Tauri MCP completed with screenshots.

---

## Notes on Tool Usage
- Use **Svelte MCP** to validate Svelte 5 runes and transitions when adding motion or effects.
- Use **Tauri MCP** for all verification (no snapshot scripts).
- Always run `cargo tauri dev` before opening Tauri MCP sessions.
- Prefer adding analyzer logic in `src-tauri/src/lib.rs` audio thread to avoid changes to audio-engine APIs.
