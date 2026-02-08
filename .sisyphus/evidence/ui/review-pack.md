# Frontend UI Revamp - Review Pack

Date: 2026-01-31
Plan: frontend-ui-revamp

## Summary of Changes

This revamp established a cohesive dark brutalist-minimal visual system:

1. **Token System**: Unified design tokens in `app.css` as single source of truth.
2. **Reduced Customization**: Only 4 safe settings remain (accent, reduce effects, dynamic bg, waveform).
3. **UI Primitives**: New Button, IconButton, Menu, MenuItem, Tooltip components.
4. **TopBar**: Added playback status badge showing output mode and bit-perfect state.
5. **Menus**: Replaced ad-hoc dropdowns with consistent Menu primitives.
6. **Polish**: LeftNav, RightRail, Albums, BottomBar updated to use tokens.

## Before/After Comparison

| Screen | Before | After | Notes |
|--------|--------|-------|-------|
| Albums View | [baseline-albums.png] | [after-albums.png] | Card hover states, token usage |
| TopBar | [baseline-topbar-a11y.yml] | [after-topbar.png] | Added playback badge |
| Tracks View | [baseline-tracks.png] | [after-tracks.png] | Track list with ellipsis buttons |
| Tracks Menu | [baseline-tracks-menu.png] | [after-tracks-menu.png] | Menu primitive (Play Now, Add to Queue, Edit Tags) |
| Appearance Prefs | [baseline-preferences-appearance.png] | [after-preferences-appearance.png] | Simplified to 4 options |
| LeftNav | [baseline-leftnav-visible.png] | [after-leftnav.png] | Token polish |
| BottomBar | [baseline-bottombar.png] | [after-bottombar.png] | Clean transport controls |
| RightRail | [baseline-rightrail-queue.png] | [after-rightrail.png] | Queue panel with token styling |

*Screenshots captured via Tauri MCP on 2026-01-31.*

## Reduced Motion Support

- `.reduce-effects` class applied to root when toggle is on.
- All transitions use `--motion-*` tokens.
- Animations disabled via `transition-duration: 0.01ms !important`.
- Verified in `ui/src/app.css`.

## Verification Notes

- [x] Reduced motion verification documented.
- [x] ADR 0001 updated to reflect MCP-only workflow.
- [x] Screenshots captured via Tauri MCP.
- [x] LSP diagnostics clean on all modified files (verified in previous tasks).
- [x] No layout changes to App.svelte structure.

## Files Modified

- `ui/src/app.css`
- `ui/src/lib/state/effects.ts`
- `ui/src/lib/components/BackgroundLayer.svelte`
- `ui/src/lib/components/primitives/Button.svelte`
- `ui/src/lib/components/primitives/IconButton.svelte`
- `ui/src/lib/components/primitives/Menu.svelte`
- `ui/src/lib/components/primitives/MenuItem.svelte`
- `ui/src/lib/components/primitives/Tooltip.svelte`
- `ui/src/lib/components/TopBar.svelte`
- `ui/src/lib/components/preferences/AppearancePrefs.svelte`
- `ui/src/lib/views/TracksView.svelte`
- `ui/src/lib/components/LeftNav.svelte`
- `ui/src/lib/components/RightRail.svelte`
- `ui/src/lib/views/AlbumsView.svelte`
- `ui/src/lib/components/BottomBar.svelte`
- `ui/src/lib/components/Modal.svelte`
- `ui/src/lib/components/TagEditor.svelte`
- `docs/ui-design-language.md`
- `docs/customization-migration.md`
- `docs/adr/0001-ui-vision-loop.md`
