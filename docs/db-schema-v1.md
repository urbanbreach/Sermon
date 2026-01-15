# Database Schema V1

## Overview
This document details the V1 database schema for the Sermon music library. The database is implemented using SQLite and is designed to store metadata for local music files, library configuration, and scanning state. It serves as the single source of truth for the user's music collection.

## Tables

### `library_folders`
Stores the file system locations that the user wants to include in their library.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | Unique identifier |
| `path` | TEXT | NOT NULL UNIQUE | Absolute path to the folder |
| `enabled` | BOOLEAN | NOT NULL DEFAULT 1 | Whether the folder is active |
| `status` | TEXT | NOT NULL DEFAULT 'IDLE' | Current scan status (e.g., IDLE, SCANNING, ERROR) |
| `last_error` | TEXT | | Message describing the last error encountered, if any |
| `options_json` | TEXT | NOT NULL DEFAULT '{}' | JSON configuration for scanning behavior (see Options JSON Semantics) |

### `tracks`
Represents individual audio files and their metadata.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | Unique identifier |
| `library_folder_id` | INTEGER | NOT NULL REFERENCES library_folders(id) ON DELETE CASCADE | The source folder |
| `path` | TEXT | NOT NULL | Absolute path to the file |
| `path_display` | TEXT | NOT NULL | Display-friendly path (useful for UI) |
| `path_lossy` | TEXT | NOT NULL | Lossy conversion of path for searching/matching |
| `identity_source` | TEXT | NOT NULL | Method used to identify track (e.g., 'inode', 'path_hash') |
| `volume_serial` | TEXT | | Volume serial number for drive identification |
| `file_id` | INTEGER | | File system inode or equivalent ID |
| `mtime_ms` | INTEGER | NOT NULL | Last modification time in milliseconds |
| `size_bytes` | INTEGER | NOT NULL | File size in bytes |
| `hash` | TEXT | | Content hash (optional/lazy) |
| `title` | TEXT | | Track title |
| `artist` | TEXT | | Track artist (denormalized) |
| `album` | TEXT | | Album name (denormalized) |
| `album_artist` | TEXT | | Album artist (denormalized) |
| `track_no` | INTEGER | | Track number |
| `disc_no` | INTEGER | | Disc number |
| `year` | INTEGER | | Release year |
| `genre` | TEXT | | Genre |
| `codec` | TEXT | | Audio codec (e.g., 'flac', 'mp3') |
| `container` | TEXT | | Container format |
| `sample_rate` | INTEGER | | Sample rate in Hz |
| `bit_depth` | INTEGER | | Bits per sample |
| `channels` | INTEGER | | Number of audio channels |
| `duration_ms` | INTEGER | | Duration in milliseconds |
| `is_missing` | BOOLEAN | NOT NULL DEFAULT 0 | Flag if file is not found during scan |
| `missing_since_ms` | INTEGER | | Timestamp when file was first noted missing |

### `albums`
Normalized album entities for grouping tracks.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | Unique identifier |
| `title` | TEXT | NOT NULL | Album title |
| `album_artist` | TEXT | | Primary artist for the album |
| `year` | INTEGER | | Release year |

### `artists`
Normalized artist entities.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | Unique identifier |
| `name` | TEXT | NOT NULL UNIQUE | Artist name |

### `track_album`
Many-to-many mapping between tracks and albums.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `track_id` | INTEGER | NOT NULL REFERENCES tracks(id) ON DELETE CASCADE | |
| `album_id` | INTEGER | NOT NULL REFERENCES albums(id) ON DELETE CASCADE | |
| | | PRIMARY KEY (track_id, album_id) | Composite PK |

### `track_artist`
Many-to-many mapping between tracks and artists.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `track_id` | INTEGER | NOT NULL REFERENCES tracks(id) ON DELETE CASCADE | |
| `artist_id` | INTEGER | NOT NULL REFERENCES artists(id) ON DELETE CASCADE | |
| | | PRIMARY KEY (track_id, artist_id) | Composite PK |

### `scan_state`
Tracks the progress of folder scans for resumability.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `folder_id` | INTEGER | PRIMARY KEY REFERENCES library_folders(id) ON DELETE CASCADE | The folder being scanned |
| `last_scan_started_ms` | INTEGER | NOT NULL | Timestamp when scan started |
| `last_scan_completed_ms` | INTEGER | | Timestamp when scan completed |

## Indexes

- `idx_library_folders_path`: UNIQUE on `library_folders(path)`
- `idx_tracks_path`: on `tracks(path)` for file lookup
- `idx_artists_name`: UNIQUE on `artists(name)`
- `idx_tracks_identity_inode`: UNIQUE on `tracks(volume_serial, file_id)` WHERE `identity_source = 'inode'`
- `idx_tracks_identity_path`: UNIQUE on `tracks(hash)` WHERE `identity_source = 'path_hash'` (Note: 'hash' column usage for path hash if strictly path identity)

## Migration/Versioning

We do not use a separate `schema_migrations` table. Instead, we utilize SQLite's built-in `PRAGMA user_version`.
- The application checks `user_version` on startup.
- If the version is lower than the current schema version, migration scripts are applied sequentially.
- The `user_version` is updated atomically with the schema changes.

## Options JSON Semantics

The `options_json` column in `library_folders` stores configuration as a JSON object:

- **`recursive`** (`boolean`): If `true`, subdirectories are scanned recursively. Default: `true`.
- **`include_extensions`** (`array<string>`): Case-insensitive list of file extensions to include. If empty or null, defaults are used: `['.flac', '.mp3', '.m4a', '.wav', '.ogg']`.
- **`exclude_patterns`** (`array<string>`): List of glob patterns to exclude. Paths are normalized to use `/` separators before matching. Exclusion takes precedence over inclusion. Invalid globs are logged and ignored.
- **`follow_symlinks`** (`boolean`): If `true`, symbolic links are followed. Default: `false` (to avoid loops/duplicates).

## DB Path Ownership

- The database file is stored in the application data directory.
- `src-tauri` resolves the app data directory using `AppHandle.path().app_data_dir()`.
- The resolved path is passed explicitly to the `crates/library` crate during initialization.
- This ensures the `library` crate remains decoupled from Tauri-specific APIs.
