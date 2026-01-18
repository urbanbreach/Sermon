# Tagging

Sermon supports reading and writing audio file metadata (tags) with a focus on safe, lossless operations.

## Supported Formats

| Format | Extensions | Tag Type | Notes |
|--------|------------|----------|-------|
| FLAC | `.flac` | Vorbis Comments | Full support |
| MP3 | `.mp3` | ID3v2 | Full support |
| MP4/M4A | `.mp4`, `.m4a` | MP4 ilst | Full support |
| OGG/Vorbis | `.ogg` | Vorbis Comments | Full support |
| Opus | `.opus` | Vorbis Comments | Full support |
| WAV | `.wav` | RIFF INFO / ID3 | Best-effort (see limitations) |
| AIFF | `.aiff`, `.aif` | ID3 chunk | Best-effort |

## Editable Fields

The tag editor supports the following fields:

| Field | Type | Notes |
|-------|------|-------|
| Title | String | Track title |
| Artist | String | Performing artist |
| Album | String | Album name |
| Album Artist | String | Album-level artist |
| Track # | Number | Track number within disc |
| Disc # | Number | Disc number |
| Year | Number | Release year |
| Genre | String | Musical genre |

## Tag Edit Semantics

Each field uses an explicit patch model:

- **Leave**: Do not modify this field
- **Set(value)**: Set the field to a new value (must be non-empty for strings)
- **Clear**: Remove the field entirely from the tag

**Important**: Empty input is NOT treated as "clear". To remove a field, you must explicitly use the Clear action.

## Unknown-Field Preservation Policy

When editing tags, Sermon preserves fields that are not being edited:

| Container | Primary Tag Type | Preserve Other Tag Types | Preserve Unknown Items | Notes |
|-----------|------------------|--------------------------|------------------------|-------|
| FLAC | Vorbis Comments | SHOULD | SHOULD | |
| OGG/Vorbis/Opus | Vorbis Comments | SHOULD | SHOULD | |
| MP3 | ID3v2 | SHOULD | SHOULD | |
| MP4/M4A | MP4 ilst | SHOULD | BEST-EFFORT | Some atoms may be reserialized |
| WAV | RIFF INFO / ID3 | BEST-EFFORT | BEST-EFFORT | Interoperability varies |
| AIFF | ID3 chunk | BEST-EFFORT | BEST-EFFORT | |

**Definitions**:
- **SHOULD**: Expected to preserve in normal operation
- **BEST-EFFORT**: May not preserve due to format/library limitations
- **Unknown**: Any tag items/frames/atoms not touched by the edit operation

## Safe Write Behavior

Sermon uses a safe write strategy to prevent data loss:

### Backup Creation

- **Default**: ON (backups are created before each edit)
- **Naming**: `original.ext.bak.YYYYMMDD-HHMMSSZ`
- **Collision handling**: Appends `.1`, `.2`, etc. if timestamp exists
- **Policy**: NEVER overwrites existing backups

### Atomic Commit

1. Original file is copied to a temporary file (`.sermon-tmp` suffix)
2. Tag modifications are applied to the temporary file
3. Temporary file is flushed to disk (`sync_all`)
4. If backup enabled, original is copied to backup location
5. Temporary file atomically replaces original:
   - Primary: `ReplaceFileW` (Windows)
   - Fallback: `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING`

### Transient Lock Handling

If the file is locked by another process (e.g., media player, indexer):

- **Retry attempts**: 3 total (initial + 2 retries)
- **Backoff**: 75ms, 200ms between attempts
- **Error handling**: After retries exhausted, displays guidance:
  > "Close any app using the file (player/editor/indexer) and try again."

### Post-Commit Verification

After atomic commit, the system verifies:
- Destination file exists
- If missing but backup exists: restores from backup
- If missing and no backup: reports "manual recovery required"

## Known Limitations

1. **WAV tagging**: RIFF INFO tags have poor interoperability across different players. Some players may not recognize tags written by Sermon.

2. **Bulk editing**: Not yet supported. Edit one track at a time.

3. **Advanced fields**: Composer, conductor, and other specialized fields are not yet editable (may be added in future milestones).

4. **Artwork embedding**: Not supported in this milestone. See Milestone 06 for artwork features.

5. **File identity**: Atomic commit may change the NTFS File ID. Sermon handles this by updating the database identity fields while keeping the track ID stable.

## References

- [ADR 0007: Tagging with Lofty and Safe Write](adr/0007-tagging-lofty-safe-write.md)
- [ADR 0004: File Identity (Windows FileID)](adr/0004-file-identity-windows-fileid.md)
