# ADR 0004: File Identity Using Windows File ID

## Context
When scanning a music library, we need to track files even if they're renamed or moved within the library folder. On Windows NTFS, files have a stable File ID that persists across renames.

## Decision
**ADOPT** a two-tier identity strategy:
1. **Primary (NTFS)**: Use Volume Serial Number + File ID from `GetFileInformationByHandle`
2. **Fallback**: Use path + mtime + size + optional blake3 hash of first 256KB

## Implementation
- Use `windows` crate with `Win32_Storage_FileSystem` feature
- Call `CreateFileW` with `FILE_FLAG_BACKUP_SEMANTICS` to get handle
- Call `GetFileInformationByHandle` to retrieve volume serial and file index
- Support long paths with `\\?\` prefix

## Alternatives Considered
- **Full file hashing**: Too expensive for large libraries (10K+ files)
- **Path only**: Breaks on file renames
- **inode (Unix)**: Not available on Windows

## Consequences
- **Positive**: Rename tolerance on NTFS; fast identity lookup
- **Negative**: Requires Windows-specific code; fallback needed for network shares

## Fallback Strategy
For filesystems that don't provide stable IDs (some network shares, FAT32):
1. Use path + mtime + size as identity
2. Compute blake3 hash of first 256KB when mtime/size changes
3. Hash mismatch = treat as new file (no rename detection in M01)
