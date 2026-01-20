# Cider UI Rewrite - Decisions

## 2026-01-20 Session Start

### Task Order
- Sequential execution required (each phase builds on previous)
- No parallelization possible per plan analysis

### Font Strategy
- Download Inter Variable from GitHub releases
- Bundle woff2 files in ui/public/fonts/
- Include OFL license as Inter-OFL.txt

## Background System Rewrite (Task 2)
- **Decision:** Replace Pixi.js with CSS-only implementation.
- **Rationale:** 
  - Pixi.js was overkill for a background effect and caused initialization issues.
  - CSS gradients + noise overlay provide the desired "Cider-style" aesthetic with much lower resource usage.
  - Removes a large dependency (`pixi.js`).
- **Implementation:**
  - Added HSL color conversion to `dynamicTheme.ts` to generate complementary accent colors (Secondary: +25 hue, Tertiary: -25 hue).
  - Used base64 SVG for noise texture to avoid extra network requests or asset files.
  - Exposed customization for Intensity, Noise Opacity, and Crossfade Speed.
