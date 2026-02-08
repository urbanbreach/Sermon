# Motion Tokens & Ambiance Tuning Evidence

## Overview
As part of the final UI/UX polish (Task 7), we have introduced a centralized motion system and tuned the background ambiance to match the "Cider" aesthetic (fluid, warm, Apple-like).

## Motion System
We introduced the following CSS variables in `ui/src/app.css` to standardize animations across the application.

### Tokens
```css
/* Motion tokens - Cider-like fluid animations */
--motion-fast: 140ms;    /* Micro-interactions (hover, press) */
--motion-medium: 200ms;  /* Standard UI transitions (opacity, color) */
--motion-slow: 320ms;    /* Larger movements (layout, lyrics) */

/* Easings */
--ease-standard: cubic-bezier(0.22, 0.61, 0.36, 1); /* Natural deceleration */
--ease-emphasis: cubic-bezier(0.2, 0.8, 0.2, 1);    /* Bouncy/expressive for transforms */
```

## Background Tuning
The `BackgroundLayer.svelte` component was tuned to be warmer and smoother, aligning with the reference material.

- **Glow Intensity**: Increased from `0.35` to `0.42` for more vibrancy.
- **Noise Opacity**: Reduced from `0.18` to `0.14` for a cleaner look.
- **Crossfade Duration**: Increased from `1200ms` to `1400ms` for smoother artwork transitions.

## Component Updates
The following components were refactored to use the new motion tokens instead of hardcoded magic numbers:

| Component | Usage | Previous | New Token |
|-----------|-------|----------|-----------|
| `TopBar.svelte` | Button hover | `0.2s` | `var(--motion-medium)` |
| `LeftNav.svelte` | Nav item hover | `0.2s` | `var(--motion-medium)` |
| `RightRail.svelte` | Tab switching | `0.2s` | `var(--motion-medium)` |
| `BottomBar.svelte` | Controls | `0.2s` | `var(--motion-medium)` |
| `AlbumsView.svelte` | Card hover | `0.2s cubic...` | `var(--motion-medium) var(--ease-emphasis)` |
| `AlbumDetailView.svelte` | Button/Row hover | `0.2s` / `0.15s` | `var(--motion-medium)` / `var(--motion-fast)` |
| `TracksView.svelte` | Row hover | `0.15s` | `var(--motion-fast)` |
| `NowPlayingView.svelte` | Controls | `0.2s` | `var(--motion-medium)` |
| `LyricsView.svelte` | Line highlight | `0.3s` | `var(--motion-slow)` |

## Accessibility
The `reduce-effects` mode (controlled via `ui/src/lib/state/effects.ts`) continues to work correctly. The global override in `app.css` ensures that when the `.reduce-effects` class is present, all transitions are forced to near-zero duration:

```css
.reduce-effects,
.reduce-effects * {
  transition-duration: 0.01ms !important;
  animation-duration: 0.01ms !important;
}
```

This overrides the CSS variables used in the components, ensuring accessibility requirements are met without complex per-component logic.
