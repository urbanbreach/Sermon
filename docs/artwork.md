# Artwork System

This document describes Sermon's album artwork pipeline: sources, caching, embedding, and the dynamic theme engine.

## Overview

Sermon's artwork system provides:
- **Multi-source artwork**: Embedded tags, iTunes Search API, Deezer API
- **Disk-based caching**: Persistent cache with eviction policy
- **Dynamic theming**: Album art-driven color palette for UI accents
- **Deterministic snapshots**: Fixed behavior for reproducible UI testing

## Artwork Sources

### 1. Embedded Artwork (Primary)

Artwork embedded in audio file tags is the preferred source.

**Selection algorithm** (when multiple pictures exist):
1. Prefer picture type "Cover Front" if present
2. Otherwise, choose the first picture
3. Tie-break by largest byte length, then lowest index

**Supported via Lofty**:
- FLAC: Vorbis Comments + PICTURE blocks
- MP3: ID3v2 APIC frames
- MP4/M4A: covr atoms
- OGG/Opus: METADATA_BLOCK_PICTURE

### 2. iTunes Search API

Query: `https://itunes.apple.com/search?term={artist}+{album}&entity=album&limit=5`

**Response fields used**:
- `collectionId` → `provider_item_id`
- `artworkUrl100` → `image_url` (used as-is, no size rewriting)

**No API key required.**

### 3. Deezer API

Query: `https://api.deezer.com/search/album?q={artist}%20{album}&limit=5`

**Response fields used**:
- `id` → `provider_item_id`
- `cover_big` or `cover_xl` → `image_url`

**No API key required.**

### 4. fanart.tv (Deferred)

fanart.tv requires an API key for all endpoints. The adapter is implemented but immediately returns "no results" and logs:
```
artwork_provider_skipped fanart reason=api_key_required
```

This satisfies the architecture without requiring user accounts.

## Caching

### Cache Location

```
{app_data_dir}/artwork-cache/
```

On Windows: `%APPDATA%/com.sermon.app/artwork-cache/`

### Cache Key Algorithm

```
cache_key = blake3("{album_artist_sort}||{album_title_sort}::{provider}:{provider_item_id}")
```

Cache files are stored without extensions. MIME type is stored in the database.

### Database Schema

```sql
-- Album-level artwork mapping
CREATE TABLE artwork_cache_map_album (
    album_artist_sort TEXT NOT NULL,
    album_title_sort TEXT NOT NULL,
    cache_key TEXT NOT NULL,
    mime TEXT NOT NULL,
    provider TEXT NOT NULL,
    provider_item_id TEXT NOT NULL,
    selected_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    PRIMARY KEY (album_artist_sort, album_title_sort)
);

-- Track-level artwork override
CREATE TABLE artwork_cache_map_track (
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    cache_key TEXT NOT NULL,
    mime TEXT NOT NULL,
    provider TEXT NOT NULL,
    provider_item_id TEXT NOT NULL,
    selected_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    PRIMARY KEY (track_id)
);
```

### Eviction Policy

- **Maximum size**: 256 MB
- **Enforcement**: After every cache write
- **Eviction order**: Oldest `selected_at` first across all mappings
- **Orphan cleanup**: Unreferenced cache files deleted by filesystem mtime

### Best Artwork Resolution

**For tracks** (`cmd_artwork_get_best_for_track`):
1. Track mapping (if exists and file present)
2. Embedded artwork from track file
3. Album mapping for track's album key
4. Return `source: 'none'`

**For albums** (`cmd_artwork_get_best_for_album`):
1. Album mapping (if exists and file present)
2. Embedded artwork from representative track (lowest disc/track/title/id)
3. Return `source: 'none'`

## UI Transport

Artwork is transported to the frontend via IPC-bytes strategy:

```typescript
// 1. Get best artwork info
const best = await getArtworkBestForAlbum(artistSort, titleSort);

// 2. If found, fetch bytes
if (best.source !== 'none' && best.cacheKey) {
  const bytes = await getArtworkBytes(best.cacheKey, best.mime);
  const dataUrl = `data:${bytes.mime};base64,${bytes.bytesBase64}`;
  // Use dataUrl as img src
}
```

## Dynamic Theme Engine

The theme engine extracts colors from album artwork to create a cohesive visual experience.

### Algorithm (Deterministic)

1. Load artwork into 48×48 canvas
2. Sample pixels on 3px grid (16×16 samples)
3. Filter: discard alpha < 0.8, luminance < 0.05 or > 0.95
4. Build 36-bucket hue histogram (10° each)
5. Select dominant bucket (highest count, tie-break by mean saturation)
6. Compute accent RGB as mean of pixels in dominant bucket
7. Clamp saturation to ≤ 0.85
8. Compute gradient stops: bg0 = accent × 0.65, bg1 = accent × 0.30

### CSS Variables

```css
--theme-accent: rgb(R, G, B);
--theme-bg-0: rgb(R, G, B);  /* Lighter gradient stop */
--theme-bg-1: rgb(R, G, B);  /* Darker gradient stop */
```

### Fallback Theme

When no valid pixels remain after filtering:
- Accent: `rgb(74, 175, 255)` (blue)
- bg0: `rgb(24, 32, 44)`
- bg1: `rgb(10, 10, 10)`

## Reduce Effects Mode

For accessibility and performance, users can enable "Reduce Motion / Reduce Transparency":

| Setting | Key | Default | Effect |
|---------|-----|---------|--------|
| Reduce Effects | `ui.reduce_effects` | off | Master toggle, disables all effects |
| Glass Blur | `ui.theme.blur` | on | Backdrop blur on glass panels |
| Theme Glow | `ui.theme.glow` | on | Accent-colored glow effects |
| Border Highlight | `ui.theme.border_highlight` | on | Accent-colored border highlights |

When `reduce_effects=on`, all individual toggles are ignored and `--glass-blur` is set to `0px`.

## Snapshot Mode

For deterministic UI testing, snapshot mode (`SERMON_SNAPSHOT=1`) provides:

1. **Fixed fixture artwork**: `solid-red.png` and `solid-blue.png`
2. **No network requests**: Provider fetchers return fixture candidates
3. **Deterministic seeding**: Now Playing shows first fixture track
4. **Theme consistency**: Same artwork → same theme colors

### Fixture Artwork

| File | Color | Expected Accent |
|------|-------|-----------------|
| `solid-red.png` | #ff0000 | R ≥ 200, G ≤ 60, B ≤ 60 |
| `solid-blue.png` | #0000ff | B ≥ 200, R ≤ 60, G ≤ 60 |

## Attribution Notes

Artwork fetched from external providers:
- **iTunes Search API**: Apple's terms of service apply
- **Deezer API**: Deezer's terms of service apply

Users should be aware that artwork is sourced from third-party services and may be subject to copyright. Sermon caches artwork locally for personal use only.

## Commands Reference

| Command | Input | Output |
|---------|-------|--------|
| `cmd_artwork_get_best_for_album` | `{ albumArtistSort, albumTitleSort }` | `{ source, cacheKey?, mime? }` |
| `cmd_artwork_get_best_for_track` | `{ trackId }` | `{ source, cacheKey?, mime? }` |
| `cmd_artwork_get_bytes` | `{ cacheKey, mime }` | `{ mime, bytesBase64 }` |
| `cmd_artwork_search_candidates` | `{ albumArtist?, albumTitle?, trackTitle? }` | `{ candidates: [...] }` |
| `cmd_artwork_select_candidate_for_album` | `{ albumArtistSort, albumTitleSort, provider, providerItemId, imageUrl }` | `{ cacheKey, cacheHit }` |
