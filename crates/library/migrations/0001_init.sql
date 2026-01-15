-- Initial Schema for Library
-- Version: 1

-- Library folders (scan roots)
CREATE TABLE library_folders (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL DEFAULT 'available',
    last_error TEXT,
    options_json TEXT NOT NULL DEFAULT '{"recursive":true,"include_extensions":[".flac",".mp3",".m4a",".wav",".ogg"],"exclude_patterns":[],"follow_symlinks":false}'
);

-- Core tracks table
CREATE TABLE tracks (
    id INTEGER PRIMARY KEY,
    library_folder_id INTEGER NOT NULL REFERENCES library_folders(id),
    path TEXT NOT NULL,
    path_display TEXT,
    path_lossy INTEGER NOT NULL DEFAULT 0,
    identity_source TEXT NOT NULL, -- 'ntfs' or 'fallback'
    volume_serial INTEGER, -- nullable, for NTFS
    file_id INTEGER, -- nullable, for NTFS
    mtime_ms INTEGER NOT NULL,
    size_bytes INTEGER NOT NULL,
    hash TEXT, -- nullable, blake3 of first 256KB for fallback
    
    -- Metadata
    title TEXT,
    artist TEXT,
    album TEXT,
    album_artist TEXT,
    track_no INTEGER,
    disc_no INTEGER,
    year INTEGER,
    genre TEXT,
    
    -- Technical
    codec TEXT,
    container TEXT,
    sample_rate INTEGER,
    bit_depth INTEGER,
    channels INTEGER,
    duration_ms INTEGER,
    
    -- Missing
    is_missing INTEGER NOT NULL DEFAULT 0,
    missing_since_ms INTEGER
);

-- Albums table
CREATE TABLE albums (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    album_artist TEXT,
    year INTEGER
);

-- Artists table
CREATE TABLE artists (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

-- Track -> Album mapping
CREATE TABLE track_album (
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    album_id INTEGER NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    PRIMARY KEY (track_id, album_id)
);

-- Track -> Artist mapping
CREATE TABLE track_artist (
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    artist_id INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    PRIMARY KEY (track_id, artist_id)
);

-- Scan state table
CREATE TABLE scan_state (
    folder_id INTEGER PRIMARY KEY REFERENCES library_folders(id) ON DELETE CASCADE,
    last_scan_started_ms INTEGER,
    last_scan_completed_ms INTEGER
);

-- Indexes
CREATE UNIQUE INDEX idx_tracks_ntfs_identity ON tracks(volume_serial, file_id) 
    WHERE identity_source = 'ntfs' AND volume_serial IS NOT NULL AND file_id IS NOT NULL;

CREATE UNIQUE INDEX idx_tracks_fallback_identity ON tracks(path, mtime_ms, size_bytes) 
    WHERE identity_source = 'fallback';

CREATE INDEX idx_tracks_library_folder ON tracks(library_folder_id);

CREATE UNIQUE INDEX idx_albums_dedup ON albums(title, album_artist);

-- Set schema version
PRAGMA user_version = 1;
