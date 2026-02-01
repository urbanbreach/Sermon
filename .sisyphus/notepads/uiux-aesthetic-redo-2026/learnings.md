# UI/UX Aesthetic Redo 2026 - Learnings

## 2026-02-01 Session Completion

### Design System Implementation

1. **Matte Brutalism Tokens**
   - True black base: `#0a0a0a`
   - Surfaces use white opacity layers: 4%, 8%, 12%
   - Text hierarchy: 92%, 64%, 48%, 32% white opacity
   - Borders: 8% and 12% white opacity

2. **Accent Color Contract**
   - Accent ONLY on buttons + focus ring
   - Selection states use grayscale (NOT accent)
   - Progress bar and waveform are grayscale
   - Artwork wash only in Now Playing view

### Bits UI Integration

- Use `import { Component } from 'bits-ui'` and export compound parts (Root, Trigger, Content, etc.) from a `module` script for cleaner consumption.
- Use `:global(.class-name)` for styling Bits UI components since they are often rendered outside the component's scope or as portals.
- `DropdownMenu` requires `Portal` for correct z-indexing and positioning, especially within `overflow: hidden` containers like virtual lists.
- When replacing ad-hoc menus with Bits UI, ensure event propagation is handled (e.g., `onclick={(e) => e.stopPropagation()}` on Triggers) to prevent row selection conflicts.
- Svelte 5 `onclick` handlers on components work if the component forwards them or if they are native elements. For Bits UI `Trigger`, it seems to work fine.

### Technical Gotchas

1. **node_modules Corruption**
   - Running app locks node_modules files
   - Must close app before `pnpm install`
   - Use `$env:CI='true'` for non-TTY pnpm runs

2. **Frameless Window**
   - Set `decorations: false` in tauri.conf.json
   - Custom WindowControls component for min/max/close
   - Double-click on TopBar to maximize
   - `-webkit-app-region: drag` for titlebar

3. **Svelte 5 A11y Warnings**
   - Div with click handlers need ARIA roles
   - Separator elements (resize handles) trigger warnings
   - These are warnings, not errors - acceptable for now

### MCP Evidence Workflow

1. **Baseline Screenshots**
   - Capture before any changes
   - Use consistent window size (1440x900)
   - Include accessibility snapshots

2. **After Screenshots**
   - Match baseline views
   - Store in same directory with `after-` prefix
   - Create review-pack.md with comparison tables

### Verification Commands

```bash
# Windows PowerShell from WSL
powershell.exe -NoProfile -Command "cd E:\\code\\sermon; pnpm -C ui check"
powershell.exe -NoProfile -Command "cd E:\\code\\sermon; pnpm -C ui build"
```
