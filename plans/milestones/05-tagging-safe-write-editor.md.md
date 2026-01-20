## Goal

- Implement safe **read/write tagging** with a basic in-app tag editor.
    
- Guarantee lossless behavior for unknown metadata fields (as feasible per container) and safe writes.
    

## Scope (in)

- Tag I/O library choice and integration:
    
    - Use **Lofty** for tag read/write across common audio formats.
        
- Safe write strategy (non-negotiable):
    
    - Write to temp file → fsync/flush → atomic replace
        
    - Optional “create backup before write”
        
- Supported container policy (must document):
    
    - FLAC, MP3, MP4/M4A, OGG/Vorbis/Opus, WAV, AIFF (best-effort)
        
    - Document known limitations (e.g., WAV tagging interoperability)
        
- UI:
    
    - Track Info panel: “Edit Tags”
        
    - Editable fields: Title, Artist, Album, Album Artist, Track #, Disc #, Year, Genre
        
    - Apply/Cancel, and “Write backup” toggle
        
- DB sync:
    
    - After tag write, update DB metadata.
        

## Non-scope (out)

- Bulk tag editing across thousands of tracks (can be later).
    
- Advanced tag fields (composer, conductor, classical, etc.) unless trivial.
    
- Artwork embedding (milestone 06).
    

## Key decisions (with alternatives + why)

- **Tags: Lofty**
    
    - Why: Rust-native tag read/write across multiple formats; avoids per-format bespoke writers.
        
    - Alternative: format-specific tag libs → higher maintenance, inconsistent behavior.
        
- **Atomic write helper**
    
    - Consider using a small “safe write” helper crate (or implement internally) that enforces temp + atomic replace. (Example: `safe-write` exists as a crate concept.)
        
    - Decision: prefer minimal internal implementation unless crate clearly reduces risk.
        

## Dependencies (minimal; justify heavy deps)

- `lofty` (tags)
    
- Optional: `safe-write`-style helper (small)
    

## Deliverables (concrete artifacts/files/features)

- Tag read/write pipeline.
    
- `/docs/tagging.md`:
    
    - supported formats
        
    - limitations
        
    - unknown-field preservation policy
        
    - backup + atomic replace behavior
        
- Test corpus + acceptance tests:
    
    - “read → write (change one field) → read” invariants
        
    - Unknown-field preservation tests (where feasible)
        
- UI vision:
    
    - `/artifacts/ui/05-tagging-safe-write-editor/tag-editor.png`
        
    - `/artifacts/ui/05-tagging-safe-write-editor/tag-editor-dirty.png`
        
    - `/artifacts/ui/05-tagging-safe-write-editor/REVIEW.md`
        
- ADR:
    
    - `/docs/adr/0007-tagging-lofty-safe-write.md`
        
- Risk register update.
    

## Acceptance criteria (pass/fail)

- Editing a tag updates the file on disk and updates UI/DB.
    
- Safe-write behavior confirmed:
    
    - backup created when enabled
        
    - no partial file writes (simulate interruption via test harness where possible)
        
- Unknown metadata not explicitly edited is preserved per documented policy.
    
- Snapshot pack produced.
    

## Commands to run (dev/test/build)

- `cargo test -p tags`
    
- `cargo tauri dev`
    
- `pnpm ui:snapshots -- --milestone 05`
    

## Risks & mitigations

- **WAV tagging interoperability** → clearly message constraints; default to conservative behavior.
    
- **User data loss risk** → backups ON by default until proven stable; strong tests.
    

## Additions

- Add “Show raw tags” (read-only) for debugging (optional but helpful for audiophiles).