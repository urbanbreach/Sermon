# Layout Tokens Evidence

## Final Token Values
These values have been applied to `ui/src/app.css` to match Cider's design proportions.

| Token | Value | Description |
|-------|-------|-------------|
| `--layout-sidebar-width` | **220px** | Sleek, compact sidebar (reduced from 240px) |
| `--layout-rail-width` | **320px** | Spacious right rail for lyrics and queue (increased from 300px) |
| `--layout-header-height` | **48px** | Modern, slightly taller header (increased from 44px) |
| `--layout-player-height` | **90px** | Substantial player bar for artwork visibility (increased from 84px) |
| `--layout-content-pad-x` | **32px** | Standard horizontal padding |
| `--layout-content-pad-y` | **24px** | Standard vertical padding |

## Component Adjustments
- `LeftNav.svelte`: Automatically consumes `--layout-sidebar-width`.
- `RightRail.svelte`: Updated fallback width to match `320px` token.
- `TopBar.svelte`: Automatically consumes `--layout-header-height`.
- `BottomBar.svelte`: Automatically consumes `--layout-player-height`.

## Design Reasoning
- **Sidebar**: 220px provides enough space for navigation items without dominating the screen, matching modern music player aesthetics.
- **Right Rail**: 320px is wide enough to display lyrics legibly and show track metadata in the "Up Next" list without truncation.
- **Header**: 48px aligns with standard macOS title bar + toolbar heights, feeling native but custom.
- **Player**: 90px allows for a decent sized artwork thumbnail (e.g. 64x64 or 56x56) while leaving room for controls and volume.
