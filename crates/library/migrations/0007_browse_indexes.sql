-- Migration 0007: Add indexes for browse/search performance
-- Optimizes list_albums_page, list_artists_page, list_tracks_page queries

-- Index for is_missing filtering (used in all browse queries)
CREATE INDEX IF NOT EXISTS idx_tracks_is_missing ON tracks(is_missing);

-- Covering index for album browsing (GROUP BY album_artist, album with is_missing filter)
CREATE INDEX IF NOT EXISTS idx_tracks_album_browse 
    ON tracks(is_missing, album, album_artist, artist, year);

-- Covering index for artist browsing (GROUP BY artist with is_missing filter)
CREATE INDEX IF NOT EXISTS idx_tracks_artist_browse 
    ON tracks(is_missing, artist);

-- Index for tracks page sorting (common sort columns with is_missing filter)
CREATE INDEX IF NOT EXISTS idx_tracks_title_sort 
    ON tracks(is_missing, title, id);

CREATE INDEX IF NOT EXISTS idx_tracks_artist_sort 
    ON tracks(is_missing, artist, id);

CREATE INDEX IF NOT EXISTS idx_tracks_album_sort 
    ON tracks(is_missing, album, id);

-- Update schema version
PRAGMA user_version = 7;
