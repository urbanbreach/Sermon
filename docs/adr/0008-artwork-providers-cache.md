# ADR 0008: Artwork Providers and Cache

## Context

Sermon needs to display album artwork in the UI. Artwork may come from:
1. Embedded tags in audio files
2. External providers (iTunes, Deezer, fanart.tv)

Key requirements:
- Minimize network requests (cache aggressively)
- Support offline operation after initial fetch
- Deterministic behavior for snapshot testing
- No user accounts or API keys required

## Decision

### Artwork Source Priority

**ADOPT** a layered resolution strategy:

1. **Track-level override** (user explicitly set artwork for this track)
2. **Embedded artwork** (extracted from file tags on-demand)
3. **Album-level selection** (user chose from provider candidates)
4. **Fallback**: No artwork (placeholder shown)

**Rationale**:
- Embedded artwork is authoritative and works offline
- User selections are persisted and respected
- Explicit overrides allow per-track customization

### Provider Selection

**ADOPT** iTunes Search API and Deezer API as primary providers.

| Provider | API Key | Quality | Coverage |
|----------|---------|---------|----------|
| iTunes | Not required | High | Excellent for mainstream |
| Deezer | Not required | High | Good European coverage |
| fanart.tv | Required | Very High | Limited to popular artists |

**DEFER** fanart.tv integration until API key management is implemented.

**Rationale**:
- iTunes and Deezer provide good coverage without authentication
- No user accounts simplifies onboarding
- fanart.tv can be added later when settings infrastructure supports API keys

### Cache Strategy

**ADOPT** disk-based cache with database mapping.

**Cache location**: `{app_data_dir}/artwork-cache/`

**Cache key algorithm**:
```
blake3("{album_artist_sort}||{album_title_sort}::{provider}:{provider_item_id}")
```

**Why blake3**:
- Fast, cryptographically secure hash
- Deterministic across platforms
- Already a dependency (used elsewhere in Rust ecosystem)

**Why not store in SQLite**:
- Image blobs would bloat the database
- Disk files are easier to evict individually
- Filesystem provides natural caching semantics

### Eviction Policy

**ADOPT** LRU-style eviction based on `selected_at` timestamp.

- **Limit**: 256 MB total cache size
- **Trigger**: After every successful cache write
- **Order**: Evict oldest `selected_at` first
- **Cleanup**: Remove orphaned files (not in any mapping)

**Rationale**:
- 256 MB is reasonable for ~500-1000 albums at ~300KB each
- LRU ensures frequently-used artwork stays cached
- Post-write enforcement prevents unbounded growth

### UI Transport

**ADOPT** IPC-bytes strategy (base64-encoded bytes over Tauri invoke).

**Alternatives considered**:
- **Custom protocol** (`sermon://artwork/{key}`): More elegant but adds complexity
- **File URLs**: Cross-origin issues in WebView2
- **Blob URLs**: Lifecycle management complexity

**Rationale**:
- IPC-bytes is simple and works reliably
- Base64 overhead (~33%) is acceptable for artwork sizes
- No custom protocol registration needed

### Dynamic Theme Engine

**ADOPT** frontend canvas-based palette extraction.

**Why frontend, not Rust**:
- Avoids `image` crate weight in Rust binary
- Canvas API is fast enough for 48×48 sampling
- Keeps theming logic close to CSS application

**Algorithm properties**:
- Deterministic (fixed sampling grid, exact math)
- Filters extreme luminance (avoids white/black accents)
- Clamps saturation (prevents oversaturated UI)

### Snapshot Mode

**ADOPT** environment-variable gating for deterministic snapshots.

When `SERMON_SNAPSHOT=1`:
- Provider fetchers return fixture candidates only
- No HTTP requests are made
- Theme computation uses fixture artwork

**Rationale**:
- Reproducible screenshots for visual regression testing
- Matches existing `SERMON_MOCK` pattern
- Backend and frontend both respect the flag

## Consequences

### Positive

- Works offline after initial artwork fetch
- No user accounts required
- Fast artwork display (local cache hit path)
- Deterministic snapshot testing
- Extensible to additional providers

### Negative

- 256 MB cache may be insufficient for very large libraries
- Base64 transport adds ~33% overhead per image
- fanart.tv high-quality artwork not available without API key

### Risks Mitigated

- **R018**: Provider rate limits → Aggressive caching, no repeated fetches
- **R019**: Provider unavailability → Graceful fallback, embedded art preferred
- **R020**: Non-deterministic theme → Fixed algorithm, snapshot mode

## References

- [iTunes Search API](https://developer.apple.com/library/archive/documentation/AudioVideo/Conceptual/iTuneSearchAPI/)
- [Deezer API](https://developers.deezer.com/api)
- [blake3](https://github.com/BLAKE3-team/BLAKE3)
- ADR 0001: UI Vision Loop (snapshot mode)
