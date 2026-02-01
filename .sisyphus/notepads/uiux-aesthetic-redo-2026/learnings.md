
## 2026-02-01: Brutalist-Matte Core Principles
- The "Brutalist-Matte" aesthetic is defined by true black (#0a0a0a) bases, 1px white hairlines at 7% opacity, and matte surfaces (white 4% and 8%).
- Accent color usage is strictly limited to Primary Buttons and Focus Rings to prevent visual noise.
- Tabular numerals (`font-variant-numeric: tabular-nums`) are essential for maintaining alignment in audiophile-grade data tables and playback timers.
- Grayscale state language (hover at 10%, active at 6%) replaces colored highlights for a calmer experience.

## Bits UI Integration (2026-02-01)

### Successful Pattern: Compound Component Re-exports
For Bits UI, use module-level exports to create compound component pattern:
```svelte
<script lang="ts" module>
  import { DropdownMenu } from 'bits-ui';
  export const Root = DropdownMenu.Root;
  export const Trigger = DropdownMenu.Trigger;
  // ... etc
</script>
```

Import as: `import * as DropdownMenu from './DropdownMenu.svelte'`

### Styling Approach
- Use `:global(.class-name)` in component's `<style>` block
- Pass classes via `class="dropdown-content"` prop
- Bits UI uses `data-highlighted` and `data-disabled` attributes for states

### Version Compatibility
- Bits UI 2.15.5 works cleanly with Svelte 5.43.8
- No Svelte 5 warnings or deprecation notices
- Uses native runes pattern ($state, $derived)

## 2026-02-01: Baseline Screenshot Capture

### Tauri MCP Navigation Patterns
- Direct hash navigation (`window.location.hash = '#/route'`) may not work as expected in all cases
- Clicking navigation buttons directly using `tauri_webview_interact` is more reliable
- The settings button selector: `.system-controls button:nth-child(2)` works for Preferences
- Segmented control navigation: `.segmented-control button:nth-child(N)` for Albums(1), Artists(2), Tracks(3)

### View Selectors Discovered
- Albums/Artists/Tracks: `.view-container` with different child classes
- Preferences: `.preferences-view` with `.prefs-sidebar` and `.prefs-content`
- TopBar: `.top-bar`
- BottomBar: `.bottom-bar` or `.bottom-bar-wrapper`

### Window Sizing
- Resize to 1440×900 works reliably via `tauri_manage_window(action="resize")`
- App has a right rail panel that can be toggled (button `.rail-toggle`)
