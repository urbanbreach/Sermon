-- Artwork cache mapping tables
-- Version: 4

-- Album-level artwork cache mapping
CREATE TABLE artwork_cache_map_album (
    album_artist_sort TEXT NOT NULL,
    album_title_sort TEXT NOT NULL,
    cache_key TEXT NOT NULL,
    mime TEXT NOT NULL,
    provider TEXT NOT NULL,
    provider_item_id TEXT NOT NULL,
    selected_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    PRIMARY KEY (album_artist_sort, album_title_sort)
);

-- Track-level artwork cache mapping (for individual track overrides)
CREATE TABLE artwork_cache_map_track (
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    cache_key TEXT NOT NULL,
    mime TEXT NOT NULL,
    provider TEXT NOT NULL,
    provider_item_id TEXT NOT NULL,
    selected_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    PRIMARY KEY (track_id)
);

-- Indexes for cache key lookups (used during eviction)
CREATE INDEX idx_artwork_album_cache_key ON artwork_cache_map_album(cache_key);
CREATE INDEX idx_artwork_track_cache_key ON artwork_cache_map_track(cache_key);

PRAGMA user_version = 4;
