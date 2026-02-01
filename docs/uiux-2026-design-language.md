# 2026 Brutalist-Matte Design Language

## Overview
The 2026 Brutalist-Matte design language is a specialized UI system for **Sermon**, an audiophile-grade music player. It prioritizes information density, legibility, and a calm, professional aesthetic inspired by high-end audio equipment and modern minimal interfaces (Audirvana, Wora).

## A. Design Principles
- **Low Distraction:** Interfaces should disappear, leaving only the music and its metadata.
- **High Legibility:** Crisp typography and clear hierarchy for complex library navigation.
- **Audiophile Hierarchy:** Information relevant to playback quality (bit depth, sample rate, format) is treated with high priority.
- **Crisp Matte Surfaces:** Moving away from "glass" and "blur" effects in favor of true black bases, matte surface levels, and 1px hairlines.
- **Zero Visual Noise:** No gradients, no noise textures, and no unnecessary animations that could distract from the listening experience.

## B. Color System
The system is built on a "True Black" foundation to maximize contrast and reduce eye strain in focused listening sessions.

- **Base:** `#0a0a0a` (True Black)
- **Matte Surface Level 1:** `rgba(255, 255, 255, 0.04)`
- **Matte Surface Level 2:** `rgba(255, 255, 255, 0.08)`
- **Hairline Borders:** `rgba(255, 255, 255, 0.07)` (1px solid)
- **Text Hierarchy:**
    - **Primary:** `rgba(255, 255, 255, 0.92)`
    - **Secondary:** `rgba(255, 255, 255, 0.70)`
    - **Tertiary:** `rgba(255, 255, 255, 0.45)`
    - **Disabled:** `rgba(255, 255, 255, 0.25)`

## C. Accent Allowed Usage
The user-definable accent color is a high-contrast element that must be used with extreme discipline.

- **Accent color appears ONLY on:**
  1. **Primary Buttons:** Solid fill or high-contrast border.
  2. **Focus Rings:** 2px outline for keyboard navigation.
- **Accent color is FORBIDDEN on:**
  - Progress bars (must use grayscale).
  - Waveform highlights (must use grayscale).
  - Selection states (must use grayscale).
  - Active/playing indicators (must use grayscale).
  - Track titles, icons, or background washes.

## D. Grayscale State Language
States are communicated through variations in grayscale opacity and subtle layout shifts, preserving the calm aesthetic.

- **Hover:** `surface-hover` (`rgba(255, 255, 255, 0.10)`)
- **Active/Pressed:** `surface-active` (`rgba(255, 255, 255, 0.06)`)
- **Selected:** `surface-2` (`rgba(255, 255, 255, 0.08)`) + subtle 2px left border.
- **Playing/Current:** Increased text brightness (Primary 92%) + optional subtle grayscale pulse.
- **Focused:** 2px accent ring (this is the **ONLY** accent use outside buttons).

## E. Typography System
- **Primary Font:** Inter Variable (Variable axes: Weight 100-900).
- **Monospace:** Used for technical data (sample rates, bit depths, timecodes).
- **Tabular Numerals:** Enabled globally (`font-variant-numeric: tabular-nums`) to ensure alignment in tables and lists.
- **Scale:**
    - **Meta:** 12px (Small labels, technical data)
    - **Body:** 14px (Standard text, list items)
    - **Label:** 16px (Buttons, sidebar items)
    - **Section:** 18px (Sub-headings)
    - **Title:** 22px (View titles)

## F. Surface System
- **Level 0:** `#0a0a0a` - The base application background.
- **Level 1:** `white 4%` - Used for main content cards, sidebar, and panels.
- **Level 2:** `white 8%` - Used for elevated elements like menus, tooltips, and modals.
- **Hairlines:** `white 7%` - Used for 1px dividers and borders to define boundaries without visual weight.

## G. Component Specs
- **Buttons:**
    - **Primary:** Accent fill, high-contrast text.
    - **Secondary:** Surface 2 fill, Primary text.
    - **Ghost:** Transparent background, Surface hover on hover.
- **Icon Buttons:** 32x32px hit area, Secondary text color, Primary on hover.
- **Inputs:** Surface 1 fill, 1px Hairline border. Focus state uses the Accent ring.
- **Tables:** 28px header height, 12px cell gap, subtle horizontal hairlines only.
- **Cards:** Level 1 surface base, transitioning to Level 2 on hover.
- **Menus/Modals:** Level 2 surface with 1px Hairline border and `shadow-3`.

## H. Spacing Grid
- **Base Unit:** 4px.
- **Scale:** 4px, 8px, 12px, 16px, 24px, 32px.
- **Density:** High-density views (Library) use 12px spacing; relaxed views (Settings) use 24px+.

## I. Windows Frameless Conventions
As a Tauri application on Windows, the window shell must feel native yet custom.
- **Titlebar Height:** 44px.
- **Window Controls:** Standard cluster (Minimize, Maximize, Close) right-aligned.
- **Drag Regions:** The entire titlebar area is draggable except for interactive elements (buttons, search).
- **Hit Areas:** All window control buttons must have a minimum hit area of 32x32px.

## J. Motion Tokens
Animations are minimal and purposeful, following the "calm" principle.
- **Fast:** 0.15s (Micro-interactions, hover states).
- **Medium:** 0.25s (Panel slides, page transitions).
- **Slow:** 0.4s (Modal entries).
- **Easing:** 
  - `ease-out`: `cubic-bezier(0.16, 1, 0.3, 1)` (Standard).
  - `ease-in-out`: `cubic-bezier(0.4, 0, 0.2, 1)` (Modals).
- **Accessibility:** Respect `prefers-reduced-motion`. When enabled, all durations are set to 0s.

## K. Headless UI Library Decision

**Chosen:** Bits UI
**Version:** 2.15.5
**Rationale:** Bits UI v2 is fully rewritten for Svelte 5, using native runes ($state, $derived) and the snippet/render pattern. It's the community standard for headless Svelte components and powers shadcn-svelte. Excellent ARIA support out of the box.

### Components to Use
- **DropdownMenu** - For track context menus, settings dropdowns
- **ContextMenu** - For right-click menus on tracks, albums
- **Tooltip** - For icon buttons, truncated text
- **Dialog** - For modals (tag editor, confirmations)
- **Select** - For device selection, format pickers
- **Slider** - For volume, seek bar (future)
- **Popover** - For floating panels, color pickers

### Styling Approach
Bits UI components are unstyled by default. We apply brutalist-matte styles via:
1. **CSS classes** - Pass `class="dropdown-content"` etc. to components
2. **Global styles** - Define `.dropdown-content`, `.dropdown-item` in component's `<style>` block with `:global()` selector
3. **CSS custom properties** - Components use our existing design tokens (`--surface-floating`, `--divider`, `--text-primary`, etc.)

### Migration Notes
- Import compound components: `import * as DropdownMenu from '$lib/components/primitives/DropdownMenu.svelte'`
- Use Bits UI's `data-highlighted` attribute (not `:hover`) for keyboard-navigated highlight states
- Use `data-disabled` attribute for disabled item styling
- State is managed via `bind:open` for two-way binding
- Bits UI handles focus management, keyboard navigation, and ARIA automatically

---

## L. Appearance Settings Migration Matrix

This matrix documents every appearance-related setting key from `effects.ts` and its classification under the Brutalist-Matte design system.

### Classification Key
- **KEEP (active)**: Setting has UI controls and applies to CSS
- **IGNORE (legacy)**: Setting persists in DB but no longer affects UI or CSS. No UI controls exposed.

### Settings Matrix

| Key | Classification | New Default | Notes |
|-----|----------------|-------------|-------|
| `ui.theme.accent_color` | **KEEP** | `#4aafff` | Palette + hex picker with contrast clamping. Accent for primary buttons and focus rings ONLY. |
| `ui.bottombar.waveform_seekbar` | **KEEP** | `off` | Toggle waveform visualization on/off |
| `ui.bottombar.waveform_style` | **KEEP** | `pills` | `'pills'` or `'raw'` style selection |
| `ui.bottombar.waveform_color` | **IGNORE** | — | Waveform is now **grayscale only** (`rgba(255,255,255,0.6)` played, `0.15` unplayed) per design language Section C |
| `ui.artwork.rounded_sidebar` | **KEEP** | `on` | Toggle rounded corners on sidebar artwork |
| `ui.artwork.rounded_albums` | **KEEP** | `on` | Toggle rounded corners on album grid artwork |
| `ui.artwork.rounded_album_detail` | **KEEP** | `on` | Toggle rounded corners on album detail artwork |
| `ui.sidebar.visible` | **KEEP** | `on` | Toggle sidebar visibility |
| `ui.reduce_effects` | **IGNORE** | — | Now **OS-only** via `prefers-reduced-motion` media query. No JS toggle. |
| `ui.theme.blur` | **IGNORE** | — | No blur effects in Brutalist-Matte |
| `ui.theme.glow` | **IGNORE** | — | No glow effects in Brutalist-Matte |
| `ui.theme.border_highlight` | **IGNORE** | — | No glass border highlights in Brutalist-Matte |
| `ui.theme.blur_px` | **IGNORE** | — | Blur slider removed |
| `ui.theme.glow_strength` | **IGNORE** | — | Glow slider removed |
| `ui.theme.border_strength` | **IGNORE** | — | Border slider removed |
| `ui.background.intensity` | **IGNORE** | — | Background is now static true black `#0a0a0a` |
| `ui.background.noise_opacity` | **IGNORE** | — | No noise texture in Brutalist-Matte |
| `ui.background.crossfade_ms` | **IGNORE** | — | No dynamic background transitions |
| `ui.background.static_color` | **IGNORE** | — | Fixed to true black `#0a0a0a` |
| `ui.background.dynamic_library` | **IGNORE** | — | Dynamic backgrounds disabled |
| `ui.background.dynamic_now_playing` | **IGNORE** | — | Dynamic backgrounds disabled |
| `ui.background.dynamic_album_detail` | **IGNORE** | — | Dynamic backgrounds disabled |
| `ui.theme.glass.main_blur` | **IGNORE** | — | Liquid Glass system removed |
| `ui.theme.glass.edge_blur` | **IGNORE** | — | Liquid Glass system removed |
| `ui.theme.glass.edge_width` | **IGNORE** | — | Liquid Glass system removed |
| `ui.theme.glass.main_bg` | **IGNORE** | — | Liquid Glass system removed |
| `ui.theme.glass.edge_bg` | **IGNORE** | — | Liquid Glass system removed |
| `ui.theme.glass.sheen_blur` | **IGNORE** | — | Liquid Glass system removed |
| `ui.theme.glass.sheen_bg` | **IGNORE** | — | Liquid Glass system removed |
| `ui.theme.glass.sheen_width` | **IGNORE** | — | Liquid Glass system removed |
| `ui.theme.glass.edge_gradient_width` | **IGNORE** | — | Liquid Glass system removed |

### Non-Appearance Settings (unchanged)
| Key | Classification | Notes |
|-----|----------------|-------|
| `artwork.provider.itunes` | **KEEP** | Artwork provider toggle (not appearance) |
| `artwork.provider.deezer` | **KEEP** | Artwork provider toggle (not appearance) |

### Implementation Notes

1. **KEYS and stores remain in `effects.ts`** — we don't delete them to preserve user data and maintain backward compatibility.

2. **`applyAppearanceToCSS()` changes:**
   - REMOVE: All `--blur-px`, `--glow-strength`, `--border-strength` CSS var settings
   - REMOVE: All `--bg-intensity`, `--bg-noise-opacity`, `--bg-crossfade-ms`, `--bg-static-color` CSS var settings
   - REMOVE: All `--glass-*` CSS var settings
   - KEEP: `--theme-accent`, `--theme-accent-r/g/b` for accent color
   - KEEP: `--artwork-radius-*` for artwork rounding

3. **`applyEffects()` changes:**
   - REMOVE: Class toggling for `effects-blur`, `effects-glow`, `effects-border`
   - The `reduce-effects` class is kept but only responds to `prefers-reduced-motion` via CSS

4. **Waveform color:**
   - `WaveformSeekbar.svelte` no longer reads `$waveformColor` store
   - Uses hardcoded grayscale: `rgba(255,255,255,0.6)` for played, `rgba(255,255,255,0.15)` for unplayed

5. **CSS respects `prefers-reduced-motion`:**
   ```css
   @media (prefers-reduced-motion: reduce) {
     * { transition-duration: 0s !important; animation-duration: 0s !important; }
   }
   ```
