# Learnings

## Windows API Dependencies
- When using `CreateFileW` from `windows` crate, you need `Win32_Security` feature in addition to `Win32_Storage_FileSystem` and `Win32_Foundation`. The error message from rustc is very helpful in identifying missing features for `windows` crate types.

## File Identity
- NTFS File ID is a robust way to track files on Windows.
- `GetFileInformationByHandle` provides both Volume Serial Number and File Index (High/Low).
- `FILE_FLAG_BACKUP_SEMANTICS` is needed to open directories if we ever need to identity directories, but here we are focusing on files. (Though our implementation uses it, which is fine for files too).

## Metadata Extraction
- Used `lofty` crate for audio metadata extraction.
- `lofty`'s `Probe` interface is convenient for detecting file types and reading tags.
- `AudioFile` trait provides access to properties like duration, sample rate, etc.
- `TaggedFileExt` provides access to tags.
- We map `lofty`'s generic tag interface to our `AudioMetadata` struct.
- Handling of missing tags is graceful (returning `None`).
- Used on-the-fly WAV generation for testing to avoid checking in binary files.

## UI/State Management
- Adopted a clean separation of concerns:
  - `types/*.ts`: TypeScript interfaces mirroring Rust structs
  - `api/*.ts`: Thin wrappers around `invoke()` calls
  - `state/*.ts`: Svelte stores + actions (business logic)
  - `views/*.svelte`: UI components consuming stores
- This pattern makes it easy to mock data by intercepting calls in the `state` layer (checking `SERMON_MOCK`).

## Snapshot Workflow
- Standardized snapshot workflow for milestone 01 using `pnpm ui:snapshots -- --milestone 01`.
- Expanded `scripts/snapshot.ps1` to handle milestone-specific mappings (directories and filenames).
- This ensures visual consistency checks are scalable across milestones.
