# Learnings - Foundation Milestone

## CSS Tokens (Liquid Glass)
- Implemented standard liquid-glass tokens using CSS variables.
- Using `backdrop-filter: blur()` works well with `rgba` backgrounds for the glass effect.
- Tokens centered in `:root` allow for easy theme switching later.
- Exact values: `--glass-bg: rgba(18, 20, 24, 0.60)`, `--glass-blur: 16px`, `--glass-radius: 12px`

## Fixtures
- Created a static JSON library with relational data (Albums -> Tracks, Artists).
- Using `import.meta.glob` with `{ eager: true, as: 'url' }` is a robust way to load assets in Vite without hardcoding paths or relying on `public` directory magic during dev.
- `resolveJsonModule` in `tsconfig` ensures type-safe JSON imports.
- **Note**: `as: 'url'` is deprecated - use `query: '?url', import: 'default'` in future

## Snapshot Mode
- `import.meta.env.SERMON_SNAPSHOT` allows conditional logic at build/runtime.
- CSS class `.snapshot-mode` with `!important` on `transition: none` effectively kills animations for snapshots.

## Routing
- Store-based routing in Svelte works well without external router
- Simple `writable<Route>` store with conditional rendering in App.svelte
- No need for svelte-routing or SvelteKit for this use case

## Tauri v2
- Config uses `build.devUrl` (not `devPath`) and `build.frontendDist` (not `distDir`)
- Event listener uses `app.listen()` not `app.on()`
- Rust edition 2024 requires Rust 1.85.0+

## Logging
- Tracing with `tracing_appender::rolling::never` for simple log files without rotation
- `tracing_subscriber` with both file and console layers works well
- SERMON_DEBUG=1 controls log level (debug vs info)

## Commands Reference
- `pnpm install` - Install all workspace dependencies
- `pnpm run build` (in ui/) - Build production UI
- `cargo build --package sermon` - Build Tauri backend
- `pnpm ui:snapshots -- --milestone 00` - Run snapshot mode

## Session Summary (2026-01-14)
- All 6 tasks completed
- UI builds successfully, Rust workspace compiles
- Remaining: Manual screenshot capture (requires human interaction)
