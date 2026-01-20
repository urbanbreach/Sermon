# Learnings - Milestone 04: Library Browse & Search Polish

## Svelte 5 Patterns

### State Management with Runes
- Use `$state()` for reactive variables: `let items = $state<Item[]>([])`
- Use `$derived()` for computed values: `let count = $derived(items.length)`
- Use `$effect()` for side effects that react to state changes

### Virtualization with Virtua
- Import from `virtua/svelte`: `import { VList } from 'virtua/svelte'`
- VGrid is not available for Svelte - use VList with row chunking for grids
- Pattern for responsive grid with VList:
  ```typescript
  let columns = $derived(Math.max(1, Math.floor(containerWidth / minItemWidth)));
  let rows = $derived.by(() => {
    const res: Item[][] = [];
    for (let i = 0; i < items.length; i += columns) {
      res.push(items.slice(i, i + columns));
    }
    return res;
  });
  ```

### Infinite Scroll Pattern
```typescript
function handleScroll(e: Event) {
  const target = e.target as HTMLElement;
  const remaining = target.scrollHeight - target.scrollTop - target.clientHeight;
  if (remaining < 500) {
    loadMore();
  }
}
```

## API Patterns

### Tauri Command Invocation
- Always wrap request in `{ request: { ... } }` for complex parameters
- Example: `invoke('cmd_library_list_albums_page', { request: { limit, cursor } })`

### Pagination Cursors
- Cursor-based pagination for albums/artists (stable ordering)
- Offset-based pagination for tracks/search (simpler for sorted lists)
- Always check `nextCursor` existence to determine if more data available

## Type Safety

### Discriminated Unions for Search Results
```typescript
export type SearchHit = 
  | { type: 'track'; trackId: number; title?: string; ... }
  | { type: 'album'; albumArtistSort: string; ... }
  | { type: 'artist'; artistSort: string; ... };
```

### Generic Page Type
```typescript
export interface Page<T, C> {
  items: T[];
  nextCursor?: C;
}
```

## Routing

### Route Stack for Back Navigation
- Maintain array of routes for history
- `navigate()` pushes to stack
- `goBack()` pops from stack
- Limit stack size to prevent memory issues (20 max)

### Route Parameters via Store
```typescript
import { currentRoute } from '../state/route';

let albumArtistSort = $derived(
  $currentRoute.name === 'album-detail' ? $currentRoute.albumArtistSort : ''
);
```

## Styling

### Glass-morphism Variables
- `var(--glass-bg)` - transparent background
- `var(--glass-blur)` - backdrop blur amount
- `var(--glass-border)` - subtle border color
- `var(--glass-highlight)` - hover/active state
- `var(--glass-radius)` - border radius

### Window Drag Regions
- Parent: `-webkit-app-region: drag`
- Interactive children: `-webkit-app-region: no-drag`

## Testing

### Svelte Check vs LSP
- `pnpm run check` (svelte-check) is authoritative for type errors
- LSP diagnostics may show stale cache errors - trust svelte-check
- Always run svelte-check before considering a task complete

## Common Issues

### Subagent Failures
- Subagents sometimes claim completion without making changes
- Always verify with `glob` to check file existence
- Use `resume` parameter to retry failed delegations

### Import Errors
- Ensure all types are exported from types/library.ts
- Import order matters in some cases
- Check that API functions import correct types
