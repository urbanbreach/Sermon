# Decisions

## Database Technology
- **Decision**: Use SQLite via `rusqlite`.
- **Rationale**: Zero-config, single file, sufficient performance for <100k tracks.
- **Reference**: See `docs/adr/0003-db-sqlite.md`.

## Database Schema
- **Decision**: Use a normalized schema for core entities (Tracks, Albums, Artists) but keep it simple.
- **Reference**: See `docs/db-schema-v1.md`.
