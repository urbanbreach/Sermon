# Milestone 06 Issues & Blockers

## 2026-01-18 Session

### MUST-14: UI Snapshot Pack - BLOCKED (FINAL REMAINING TASK)

**Blocker**: Requires manual execution - cannot be automated

The snapshot capture requires:
1. Running Tauri app with visible GUI window
2. `SERMON_MOCK=1` and `SERMON_SNAPSHOT=1` environment variables
3. Manual screenshot capture at 1440×900 resolution

**To complete manually**:
```powershell
cd E:\Code\Sermon
$env:SERMON_MOCK = "1"
$env:SERMON_SNAPSHOT = "1"
cargo tauri dev

# Capture screenshots:
# 1. Navigate to Albums → capture albums-themed.png
# 2. Navigate to Now Playing → capture now-playing-themed.png
# 3. Click "Choose Artwork..." on album card → capture artwork-picker.png

# Save to: artifacts/ui/06-artwork-cache-dynamic-theme/
# Create REVIEW.md following existing milestone format
```

**Determinism verification**:
```powershell
# Run twice and compare hashes
Get-FileHash artifacts/ui/06-artwork-cache-dynamic-theme/*.png
```

**Expected artifacts**:
- `artifacts/ui/06-artwork-cache-dynamic-theme/albums-themed.png`
- `artifacts/ui/06-artwork-cache-dynamic-theme/now-playing-themed.png`
- `artifacts/ui/06-artwork-cache-dynamic-theme/artwork-picker.png`
- `artifacts/ui/06-artwork-cache-dynamic-theme/REVIEW.md`

### Pre-existing Warnings (Not Blockers)

- TracksView.svelte: Old event handler syntax (on:click) - pre-existing, not from this milestone
- NowPlayingView.svelte: `isDebugging` not using $state() - pre-existing
- Various a11y warnings for labels - pre-existing

### MAY-13: Embed Art into File - COMPLETED

**Status**: ✅ Implemented

**Implementation completed**:
1. Extended `TagPatches` with `PicturePatch` enum (`Leave`, `SetCover`, `ClearAll`)
2. Modified `write_tags` in `crates/tags/src/writer.rs` to handle Lofty's `Picture` API
3. Added backend command `cmd_artwork_embed_to_file` in `src-tauri/src/commands/artwork.rs`
4. Added frontend API `embedArtworkToFile()` in `ui/src/lib/api/artwork.ts`

**Files modified**:
- `crates/tags/src/writer.rs` - Added PicturePatch enum and picture embedding logic
- `crates/tags/src/lib.rs` - Exported PicturePatch
- `crates/library/src/tag_edit.rs` - Updated to include picture field
- `src-tauri/src/commands/artwork.rs` - Added cmd_artwork_embed_to_file
- `src-tauri/src/commands/mod.rs` - Exported new command
- `src-tauri/src/lib.rs` - Registered command in invoke handler
- `ui/src/lib/api/artwork.ts` - Added embedArtworkToFile function
