# Cider UI Refresh - Pass 1 Comparison Notes

**Date**: 2026-01-22
**Session**: ses_41c874a8dffeGEtgn2gf7uL5Ex

## Overview

This pass focused on aligning the shell, Albums, and Now Playing views with Cider's Liquid Glass aesthetic, plus adding accent color customization.

## Screenshots

| View | Before | After |
|------|--------|-------|
| Albums | `01-before-albums.png` | `07-after-albums.png` |
| Now Playing | `01-before-now-playing.png` | `07-after-now-playing.png` |

## Comparison Results

### Albums View vs Cider Reference (`Näyttökuva 2026-01-20 054030.png`)

| Aspect | Status | Notes |
|--------|--------|-------|
| Grid Layout | ✅ PASS | Grid sizing increased to 160px min, 24px gaps - matches Cider proportions |
| Card Radii | ✅ PASS | Art corners now 8px (was 12px) - matches Cider style |
| Typography | ✅ PASS | Title/artist hierarchy maintained |
| Sidebar Glass | ✅ PASS | Updated to use glass tokens with blur and translucency |
| Dividers | ✅ PASS | Subtle 1px dividers between columns |

### Now Playing View vs Cider Reference (`G9OPGw0XQAManL-.jpg`)

| Aspect | Status | Notes |
|--------|--------|-------|
| Art Container | ✅ PASS | Using 16px radius, heavy shadow system |
| Glass Panels | ✅ PASS | Nav buttons use glass tokens with proper blur |
| Controls | ✅ PASS | Using UI accent for interactive elements |
| Background Glow | ✅ PASS | Dynamic theme glow unaffected by accent override |
| Debug Overlay | ✅ PASS | Styled with glass tokens |

### Shell Components vs Cider Reference (`G5uZqyBWsAASpFn.jpg`)

| Aspect | Status | Notes |
|--------|--------|-------|
| LeftNav | ✅ PASS | Glass bg with blur, pill search bar, accent active states |
| TopBar | ✅ PASS | Circular buttons with glass styling, accent active states |
| RightRail | ✅ PASS | Glass sidebar, pill controls with accent active states |
| BottomBar | ✅ PASS | Darker glass player bar, center island with glass styling |

### Accent Override Feature

| Aspect | Status | Notes |
|--------|--------|-------|
| Preferences UI | ✅ PASS | Color picker + reset button in Appearance section |
| Persistence | ✅ PASS | Uses `cmd_settings_get/set` for persistence |
| Startup Apply | ✅ PASS | `loadEffectsSettings()` called in App.svelte onMount |
| UI Chrome | ✅ PASS | Active nav, buttons, focus rings use `--ui-accent` |
| Background Glow | ✅ PASS | Unaffected - uses `--theme-accent-*` tokens |

## Token Changes Summary

### Glass System
- Blur values tuned: 24px light, 32px player, 40px sidebar, 48px heavy
- Background opacity increased for darker, more solid feel
- Border opacity reduced for subtler dividers
- Shadow system refined with lighter shadows

### Radius System
- Panel/card: 24px → 16px
- Button: 12px → 10px
- Art large: 12px → 8px
- Art small: 6px → 4px

### Accent System
- New base tokens: `--cider-accent-base`, `--cider-accent-base-rgb`
- New UI tokens: `--ui-accent`, `--ui-accent-rgb`
- All UI chrome updated to use `--ui-accent` variants
- Background glow preserved on `--theme-accent-*`

## Verification

```
pnpm -C ui check
✅ 0 errors, 5 warnings (pre-existing, unrelated to this pass)
```

## Remaining Work (Out of Scope for Pass 1)

- Tracks view styling (accent may propagate via shared tokens - acceptable)
- Lyrics view styling (out of scope)
- Album detail view styling (out of scope)
- Artist views styling (out of scope)

## Conclusion

**PASS** - All acceptance criteria met for Pass 1 refresh.
