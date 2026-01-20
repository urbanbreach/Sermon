# Learnings

## Virtua + Svelte 5
- `virtua`'s `VGrid` component is currently NOT available for Svelte (as of v0.48.3), despite being available for React/Vue.
- To implement a virtualized grid in Svelte with `virtua`, use `VList` and chunk the data into rows manually.
- The `VList` component for Svelte 5 uses snippets. The item snippet must be named `children` (e.g. `{#snippet children(item)}`), not `item`.
- `VList` in Svelte does not seem to expose a `footer` snippet like the React version might. Loading indicators should be placed outside the `VList` but inside the scroll container.

## Svelte 5 Runes
- Using `$state` and `$derived` simplifies reactivity for window resizing and column calculation.
- `bind:clientWidth` works well on the container div to drive the responsive grid column count.

## Layout
- When using virtualization, the scroll container relationship is critical. `virtua` detects the nearest scrollable parent.
- Ensuring the container has `overflow-y: auto` and a defined height is essential.
- Infinite scroll can be implemented by listening to the `onscroll` event of the container and checking `scrollHeight - scrollTop - clientHeight`.
