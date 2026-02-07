-- Lyrics cache table for LRCLIB fetched lyrics
-- Version: 9

CREATE TABLE lyrics_cache (
    track_id INTEGER PRIMARY KEY NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    synced_lyrics TEXT,
    plain_lyrics TEXT,
    source TEXT NOT NULL DEFAULT 'lrclib',
    fetched_at INTEGER NOT NULL DEFAULT (strftime('%s','now') * 1000)
);

CREATE INDEX idx_lyrics_cache_source ON lyrics_cache(source);

PRAGMA user_version = 9;
