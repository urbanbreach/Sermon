# ADR 0003: SQLite as Embedded Database

## Context
The Sermon application requires a persistent storage solution for the local music library. The library may contain anywhere from 10,000 to over 100,000 tracks. We need to support efficient querying by metadata (artist, album, genre, year), and the solution must operate locally without requiring a separate server process. The architecture needs to be self-contained and easy to distribute.

## Decision
**ADOPT** SQLite via the `rusqlite` crate as the embedded database engine.

We will use a single `.sqlite` file stored in the user's application data directory to manage the library state.

## Alternatives Considered

### `sled`
- **Pros**: Native Rust key-value store, embedded, simple API.
- **Cons**: Weaker ergonomics for complex queries (filtering, sorting, aggregations) compared to SQL. No built-in Full Text Search (FTS). Requires more custom implementation for relational data.

### `sqlx`
- **Pros**: Strong type safety, compile-time query verification, async support.
- **Cons**: Significant compile-time overhead (macros). The async overhead is not strictly necessary for our local embedded use case where simple synchronous operations (or spawned threads) via `rusqlite` are sufficient and often simpler to reason about for a desktop app.

### `diesel`
- **Pros**: Powerful ORM, type safety.
- **Cons**: High complexity and boilerplate. For our relatively simple schema, an ORM adds unnecessary abstraction layers compared to raw SQL or a lightweight wrapper.

## Consequences
- **Positive**:
  - **Simplicity**: Zero-conf, serverless, single-file deployment.
  - **Ecosystem**: Robust tooling (DB Browser for SQLite), mature libraries, and extensive documentation.
  - **Backup**: Easy to backup/restore by simply copying the file.
  - **Performance**: Highly optimized for local read-heavy workloads.
- **Negative**:
  - **Migrations**: We must handle schema migrations ourselves (e.g., using `user_version` PRAGMA and directory-backed SQL files) rather than relying on a framework's built-in tool.
  - **Concurrency**: Write concurrency is limited (database lock), though WAL mode helps. Given the single-user nature of a desktop music player, this is acceptable.

## Evidence
- **Audirvana**: A high-end audiophile player uses a `.sqlite` database file (e.g., `AudirvanaDatabase.sqlite`) to manage large libraries effectively.
