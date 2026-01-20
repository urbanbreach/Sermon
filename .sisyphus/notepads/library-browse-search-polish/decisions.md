# Decisions - Milestone 04: Library Browse & Search Polish

## Architecture Decisions

### 1. Derived Albums/Artists (No Normalized Tables)
**Decision**: Derive album/artist lists from tracks table using GROUP BY queries instead of maintaining separate normalized tables.

**Rationale**:
- Scanner currently only populates tracks table
- Avoids schema migration complexity
- Keeps data model simple and consistent
- Allows future migration to normalized tables without UI changes

**Trade-off**: Slightly slower queries for large libraries, but acceptable with proper indexes.

### 2. Virtua for Virtualization
**Decision**: Use `virtua/svelte` for virtualized scrolling.

**Rationale**:
- Lightweight (~5KB)
- Native Svelte 5 support
- Simple API
- Good performance

**Alternative considered**: TanStack Virtual (heavier, more features than needed)

### 3. FTS5 for Search
**Decision**: Use SQLite FTS5 with external-content table for full-text search.

**Rationale**:
- Built into SQLite (no external dependencies)
- Fast prefix search with `prefix='2 3'`
- Proper Unicode handling with `unicode61 remove_diacritics 1`
- Ranking with `bm25()`

### 4. Route Stack for Navigation
**Decision**: Convert Route from string union to discriminated union with route stack.

**Rationale**:
- Type-safe route parameters
- Natural back navigation behavior
- Consistent with modern SPA patterns

### 5. Cursor-based vs Offset Pagination
**Decision**: 
- Cursor-based for albums/artists (stable ordering by sort keys)
- Offset-based for tracks/search (simpler, sorting changes often)

**Rationale**: Cursor-based prevents duplicates/gaps during concurrent modifications, but offset is simpler when ordering is stable.

### 6. Snapshot Tooling Node Wrapper
**Decision**: Create Node.js wrapper script to translate `--milestone` to PowerShell `-Milestone` parameter.

**Rationale**:
- Keeps CLI interface consistent (`pnpm ui:snapshots -- --milestone 04`)
- PowerShell script unchanged (backward compatible)
- Cross-platform potential (Node handles argument parsing)
