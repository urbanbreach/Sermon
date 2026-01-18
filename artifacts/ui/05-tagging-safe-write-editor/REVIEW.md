# Milestone 05: Tagging & Safe Write Editor - Review Pack

## Overview

This milestone implements safe read/write tagging with a basic in-app tag editor, guaranteeing lossless behavior for unknown metadata fields and safe writes.

## Screenshots Required

| Screenshot | Description | How to Capture |
|------------|-------------|----------------|
| `tag-editor.png` | Tag editor modal in clean state | Open TracksView → Click ✎ on any track |
| `tag-editor-dirty.png` | Tag editor with modified fields | Edit Title/Artist fields, observe Apply button enabled |

## Features Implemented

### 1. Tag Write API (`crates/tags/src/writer.rs`)

- `TagPatch` enum: `Leave | Set(String) | Clear`
- `NumberPatch` enum: `Leave | Set(u32) | Clear`
- `TagPatches` struct for all 8 editable fields
- `write_tags()` using Lofty in-place tag mutation

### 2. Safe Write Helper (`crates/library/src/safe_write.rs`)

- Temp file → flush → atomic commit strategy
- Backup creation (ON by default): `file.ext.bak.YYYYMMDD-HHMMSSZ`
- Windows atomic commit: `ReplaceFileW` with `MoveFileExW` fallback
- Retry on `ERROR_SHARING_VIOLATION` (3 attempts, 75ms/200ms backoff)

### 3. Tag Editor UI (`ui/src/lib/components/TagEditor.svelte`)

- 8 editable fields: Title, Artist, Album, Album Artist, Genre, Track #, Disc #, Year
- Explicit Clear (×) button per field - no empty-string-means-clear ambiguity
- Validation: blocks Apply if field emptied without using Clear
- "Create backup" toggle (ON by default)
- Retry status display during transient lock retries

### 4. Modal Component (`ui/src/lib/components/Modal.svelte`)

- Glass-styled dialog matching design system
- Focus trap (Tab/Shift+Tab cycling within modal)
- ESC to close (unless `preventClose` set)
- Backdrop prevents background interaction
- `role="dialog"`, `aria-modal="true"` for accessibility

## Acceptance Criteria

- [x] Editing a tag updates the file on disk and updates UI/DB
- [x] Safe-write behavior: backup created when enabled
- [x] Safe-write behavior: no partial file writes (temp → atomic commit)
- [x] Unknown metadata not explicitly edited is preserved (per container policy)
- [x] Tests pass: `cargo test -p tags` (12 tests)
- [x] Tests pass: `cargo test -p library` (23 tests)

## Manual Verification Checklist

### Tag Editor Flow
- [ ] Open tag editor for a track (click ✎ in Actions column)
- [ ] Modify Title field → Apply button becomes enabled
- [ ] Click Apply → File is updated, UI refreshes
- [ ] Restart app → Changes persist

### Cancel Flow
- [ ] Open tag editor, modify fields
- [ ] Click Cancel → No changes saved
- [ ] Verify file and DB unchanged

### Clear Field Flow
- [ ] Open tag editor with existing tags
- [ ] Click × next to a field → Field shows as "cleared" (orange styling)
- [ ] Apply → Field is removed from file tags

### Backup Verification
- [ ] Edit a track with "Create backup" checked
- [ ] Verify backup file created: `original.ext.bak.YYYYMMDD-HHMMSSZ`

### Focus Trap (DEV mode)
- [ ] Run with `SERMON_DEBUG=1 cargo tauri dev`
- [ ] Open Diagnostics view → Focus Trap Harness section
- [ ] Open test modal → Verify Tab cycles within modal only
- [ ] ESC closes modal, focus returns to opening button

## Files Changed

### New Files
- `ui/src/lib/components/Modal.svelte`
- `ui/src/lib/components/TagEditor.svelte`
- `crates/tags/src/writer.rs`
- `crates/library/src/safe_write.rs`
- `crates/library/src/tag_edit.rs`
- `docs/tagging.md`
- `docs/adr/0007-tagging-lofty-safe-write.md`

### Modified Files
- `ui/src/lib/views/TracksView.svelte` - Added Edit Tags button
- `ui/src/lib/views/DiagnosticsView.svelte` - Added focus trap harness
- `ui/src/lib/types/library.ts` - Added tag patch types
- `ui/src/lib/api/library.ts` - Added updateTrackTags()
- `src-tauri/src/commands/library.rs` - Added cmd_library_update_track_tags
- `crates/library/src/scanner.rs` - Updated supported extensions
- `crates/tags/tests/metadata.rs` - Added tag write tests
- `docs/risk-register.md` - Added R014-R017

## Known Limitations

1. **WAV tagging**: RIFF INFO tags have poor interoperability
2. **Bulk editing**: Not supported (one track at a time)
3. **Advanced fields**: Composer, conductor not yet editable
4. **Artwork**: Not supported in this milestone (see M06)

## References

- [Tagging Documentation](/docs/tagging.md)
- [ADR 0007: Tagging with Lofty](/docs/adr/0007-tagging-lofty-safe-write.md)
- [Risk Register](/docs/risk-register.md)
