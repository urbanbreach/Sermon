# Milestone 06 Learnings

## 2026-01-18 Session

### Conventions Discovered

- **Event handler syntax**: Svelte 5 requires consistent use of new syntax (`onclick`, `onchange`) - cannot mix with old `on:click` syntax in the same file
- **Reactive state in Svelte 5**: Use `$state()` for reactive variables, `$effect()` for reactive side effects
- **Artwork transport**: IPC-bytes strategy (base64 over Tauri invoke) works well for artwork - simple and reliable
- **Theme engine placement**: Theme computation should be driven by Now Playing only (single source of truth) to avoid nondeterminism from multiple album grid images

### Successful Approaches

- **Layered artwork resolution**: Track override → Embedded → Album selection → None provides good UX
- **Deterministic palette extraction**: Fixed 48×48 canvas, 3px grid, exact integer math ensures reproducible themes
- **Settings store pattern**: Separate effects.ts store with load/save/apply functions keeps concerns separated
- **Modal component reuse**: Existing Modal.svelte with focus trap works well for artwork picker

### Technical Gotchas

- **Canvas crossOrigin**: Must set `img.crossOrigin = 'anonymous'` before loading for canvas pixel access
- **Image decode timing**: Use `await img.decode()` before drawing to canvas for deterministic rendering
- **HSV math edge cases**: Handle `delta === 0` case in RGB→HSV conversion to avoid division by zero
- **Saturation clamping**: Recompute HSV from averaged RGB before clamping, not from original samples

### Commands Added

- `cmd_settings_get(key)` → `{ value?: string }`
- `cmd_settings_set(key, value)` → void
- `cmd_artwork_embed_to_file(trackId, cacheKey, mime)` → `{ success: boolean }`
- Uses existing `library::db::get_setting/set_setting` helpers

### Session 2 Additions (SHOULD-06 & MAY-13)

**Provider Toggles (SHOULD-06)**:
- Settings keys: `artwork.provider.itunes`, `artwork.provider.deezer`
- Stores in `ui/src/lib/state/effects.ts`: `providerItunes`, `providerDeezer`
- Backend checks settings before HTTP calls in `cmd_artwork_search_candidates`

**Embed Art to File (MAY-13)**:
- Added `PicturePatch` enum: `Leave`, `SetCover { bytes, mime }`, `ClearAll`
- Extended `TagPatches` struct with `picture: PicturePatch` field
- Uses Lofty's `Picture` API with `PictureType::CoverFront`
- Frontend API: `embedArtworkToFile(trackId, cacheKey, mime)`
