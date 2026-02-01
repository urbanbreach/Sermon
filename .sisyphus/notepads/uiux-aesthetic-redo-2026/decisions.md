# UI/UX Aesthetic Redo 2026 - Decisions

## 2026-02-01 Decisions Captured

### D1: Headless UI Library
**Decision**: Use Bits UI 2.15.5
**Rationale**: Best Svelte 5 compatibility, good component coverage (Menu, Dialog, Tooltip), styling freedom
**Alternatives Considered**: Melt UI (less mature), extending existing primitives (more work)

### D2: Design Language
**Decision**: Brutalist-Matte (not Liquid Glass)
**Rationale**: User preference for crisp, high-legibility audiophile aesthetic
**Key Properties**:
- True black base (#0a0a0a)
- Minimal blur/gradients
- Sharp 1px hairlines
- Grayscale for non-interactive states

### D3: Accent Color Scope
**Decision**: Accent ONLY on buttons + focus ring
**Rationale**: Audiophile restraint, reduces visual noise
**Implementation**: Selection states use grayscale, progress bar grayscale

### D4: Frameless Window
**Decision**: Custom titlebar with window controls
**Rationale**: Full control over desktop presence
**Implementation**: 
- `decorations: false` in Tauri config
- WindowControls.svelte component
- TopBar acts as titlebar with drag region

### D5: Artwork Wash Location
**Decision**: Now Playing view ONLY
**Rationale**: Library views should be neutral for focus
**Implementation**: BackgroundLayer conditionally shows wash based on route

### D6: Reduced Motion
**Decision**: OS-level only (no in-app toggle)
**Rationale**: Simplicity, follows platform conventions
**Implementation**: `@media (prefers-reduced-motion: reduce)` in app.css
