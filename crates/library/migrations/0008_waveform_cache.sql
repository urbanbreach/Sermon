-- Waveform cache tracking table for LRU eviction (1GB cap)
CREATE TABLE waveform_cache_map (
    cache_key TEXT PRIMARY KEY NOT NULL,
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    size_bytes INTEGER NOT NULL,
    mtime_ms INTEGER NOT NULL,
    last_access_ms INTEGER NOT NULL DEFAULT (strftime('%s','now') * 1000)
);

-- Index for eviction queries (oldest last_access first)
CREATE INDEX idx_waveform_cache_last_access ON waveform_cache_map(last_access_ms);

-- Index for size aggregation
CREATE INDEX idx_waveform_cache_track ON waveform_cache_map(track_id);

PRAGMA user_version = 8;
