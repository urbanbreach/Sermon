# ADR 0007: Tagging with Lofty and Safe Write

## Context

Sermon needs to support editing audio file metadata (tags) while guaranteeing:
1. No data loss during write operations
2. Preservation of unknown/unedited tag fields
3. Safe handling of file locks and system interruptions
4. Compatibility with the existing file identity system (ADR 0004)

## Decision

### Tag Library: Lofty

**ADOPT** the [Lofty](https://github.com/Serial-ATA/lofty-rs) crate for tag read/write operations.

**Rationale**:
- Rust-native, no FFI complexity
- Supports all required formats: FLAC, MP3, MP4/M4A, OGG/Vorbis/Opus, WAV, AIFF
- Provides in-place tag modification to maximize field preservation
- Active maintenance and good documentation
- `WriteOptions::remove_others(false)` preserves non-primary tag types

**Alternatives Considered**:
- **Format-specific libraries** (id3, metaflac, mp4ameta): Higher maintenance burden, inconsistent APIs
- **TagLib via FFI**: C++ dependency, build complexity on Windows
- **Symphonia**: Read-only, no tag writing support

### Safe Write Strategy

**ADOPT** a temp-file + atomic-commit strategy with optional backup.

**Algorithm**:
1. Copy original → temp file (`.sermon-tmp` suffix, same directory)
2. Apply tag patches to temp file via Lofty
3. Flush temp file to disk (`sync_all`)
4. If backup enabled: copy original → timestamped backup
5. Atomic commit: temp → original
   - Primary: `ReplaceFileW` (Windows API)
   - Fallback: `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH`
6. Post-commit verification: ensure destination exists

**Rationale**:
- Original file is never modified in-place until commit succeeds
- Backup provides recovery path if issues arise
- Atomic commit minimizes window for partial writes
- Same-directory temp file ensures same-volume operation (required for atomic move)

### Backup Policy

- **Default**: ON (backups created for every edit)
- **Naming**: `original.ext.bak.YYYYMMDD-HHMMSSZ`
- **Collision**: Append `.1`, `.2`, etc. if name exists
- **Policy**: NEVER overwrite existing backups

**Rationale**: For a user's music library, data safety is paramount. Default-on backups protect against bugs, user error, and system issues until the feature is proven stable.

### Transient Lock Handling

**ADOPT** bounded retry with backoff for commit-step lock errors.

- **Retry on**: `ERROR_SHARING_VIOLATION`, `ERROR_ACCESS_DENIED`
- **Attempts**: 3 total (initial + 2 retries)
- **Backoff**: 75ms, 200ms
- **Total wait**: < 300ms

**Rationale**: Music files are commonly locked by media players, indexers (Windows Search), or antivirus. Brief retry allows for transient locks to clear without blocking the UI excessively.

### Identity Implications

Atomic commit via `ReplaceFileW` may change the NTFS File ID. This is handled by:

1. Keeping `tracks.id` (database primary key) stable
2. Re-reading file identity after successful commit
3. Updating identity fields (`volume_serial`, `file_id`, `mtime_ms`, `size_bytes`) in place
4. Preflight duplication hazard check before commit

**Reference**: See [ADR 0004: File Identity](0004-file-identity-windows-fileid.md) for identity acquisition details.

## Consequences

### Positive

- **Data safety**: Original file never modified until atomic commit; backup provides recovery
- **Field preservation**: In-place tag modification preserves unknown fields per container capabilities
- **Lock resilience**: Brief retry handles common transient lock scenarios
- **Identity stability**: Database track IDs remain stable despite file replacement

### Negative

- **Performance overhead**: Copy-modify-replace is slower than in-place modification
- **Disk space**: Temp files and backups consume additional space temporarily
- **Windows-specific**: Atomic commit uses Windows APIs; would need platform abstraction for cross-platform

### Risks Mitigated

| Risk | Mitigation |
|------|------------|
| R004: Data loss during write | Temp file + atomic commit; backup before replace |
| R005: Unknown field loss | Lofty's `remove_others(false)`; in-place tag modification |
| R006: File lock errors | Bounded retry with user-friendly guidance |
| T3: Identity churn | Re-read identity post-commit; update DB in place |

## References

- [/docs/tagging.md](/docs/tagging.md) - User-facing tagging documentation
- [ADR 0004: File Identity](0004-file-identity-windows-fileid.md) - File identity strategy
- [Lofty documentation](https://docs.rs/lofty/latest/lofty/)
- [ReplaceFileW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew) - Windows API reference
