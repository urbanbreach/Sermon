
- **Bits UI Integration**:
  - Use `import { Component } from 'bits-ui'` and export compound parts (Root, Trigger, Content, etc.) from a `module` script for cleaner consumption.
  - Use `:global(.class-name)` for styling Bits UI components since they are often rendered outside the component's scope or as portals.
  - `DropdownMenu` requires `Portal` for correct z-indexing and positioning, especially within `overflow: hidden` containers like virtual lists.
  - When replacing ad-hoc menus with Bits UI, ensure event propagation is handled (e.g., `onclick={(e) => e.stopPropagation()}` on Triggers) to prevent row selection conflicts.
  - Svelte 5 `onclick` handlers on components work if the component forwards them or if they are native elements. For Bits UI `Trigger`, it seems to work fine.
