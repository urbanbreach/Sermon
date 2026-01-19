# ADR 0009: Settings Storage Strategy

## Context
Sermon requires persistent storage for user preferences including audiophile settings (WASAPI mode, sample rate handling, buffer sizes), library behavior (scan options, file handling), and UI options (theme, layout preferences). These settings must survive application restarts and be reliably persisted without data loss.

The application already uses SQLite for library management (ADR 0003), and a `settings` table exists in the schema (`crates/library/migrations/0002_settings.sql`).

## Decision
**ADOPT** the existing SQLite `settings` table with a key-value schema for all application preferences.

We will use `PRAGMA user_version` for schema versioning and forward-only migrations, following the pattern already established in `crates/library/src/migrations.rs`.

### Existing Schema
```sql
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
```

### Migration Pattern
Schema migrations are tracked via `PRAGMA user_version`:

```rust
let version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

if version < 2 {
    conn.execute_batch(include_str!("../../migrations/0002_settings.sql"))?;
}
// Future migrations: if version < 3 { ... }
```

Migrations are:
- **Forward-only**: No rollback support; each migration increments `user_version`
- **Sequential**: Applied in order on application startup
- **Idempotent**: Use `CREATE TABLE IF NOT EXISTS` and similar guards

## Alternatives Considered

### JSON Configuration File
- **Pros**: Human-readable, easy to edit manually, familiar pattern.
- **Cons**: No atomic writes without careful implementation (write-rename pattern). Harder to migrate schema changes. Separate backup from library database.

### Separate Settings Database
- **Pros**: Isolation from library data, independent versioning.
- **Cons**: Overkill for a single-user desktop application. Two files to manage and backup. Additional connection overhead.

### In-Memory with Periodic Flush
- **Pros**: Fast reads, simple API.
- **Cons**: Risk of data loss on crash or unexpected termination. Complex flush timing logic.

## Consequences
- **Positive**:
  - **Atomic Writes**: SQLite transactions ensure settings are never partially written.
  - **Unified Backup**: Settings are backed up alongside the library by copying a single `.sqlite` file.
  - **Transactional**: Can update multiple settings atomically within a transaction.
  - **Mature Tooling**: Can inspect/modify settings via DB Browser for SQLite during development.
  - **Existing Infrastructure**: Reuses the database connection and migration system already in place.
- **Negative**:
  - **Manual Migration Management**: Must implement migration logic ourselves using `PRAGMA user_version` rather than relying on a framework's migration tool.
  - **Serialization Overhead**: Complex settings (e.g., nested objects) must be serialized to JSON strings in the `value` column.
