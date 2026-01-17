-- FTS5 Full-Text Search for tracks
-- Version: 3

-- Create external-content FTS5 table
CREATE VIRTUAL TABLE tracks_fts USING fts5(
    title,
    artist,
    album,
    album_artist,
    genre,
    content='tracks',
    content_rowid='id',
    tokenize="unicode61 remove_diacritics 1",
    prefix='2 3'
);

-- Triggers to keep FTS in sync with tracks table

-- INSERT trigger
CREATE TRIGGER tracks_ai AFTER INSERT ON tracks BEGIN
    INSERT INTO tracks_fts(rowid, title, artist, album, album_artist, genre)
    VALUES (NEW.id, NEW.title, NEW.artist, NEW.album, NEW.album_artist, NEW.genre);
END;

-- UPDATE trigger
CREATE TRIGGER tracks_au AFTER UPDATE ON tracks BEGIN
    INSERT INTO tracks_fts(tracks_fts, rowid, title, artist, album, album_artist, genre)
    VALUES ('delete', OLD.id, OLD.title, OLD.artist, OLD.album, OLD.album_artist, OLD.genre);
    INSERT INTO tracks_fts(rowid, title, artist, album, album_artist, genre)
    VALUES (NEW.id, NEW.title, NEW.artist, NEW.album, NEW.album_artist, NEW.genre);
END;

-- DELETE trigger
CREATE TRIGGER tracks_ad AFTER DELETE ON tracks BEGIN
    INSERT INTO tracks_fts(tracks_fts, rowid, title, artist, album, album_artist, genre)
    VALUES ('delete', OLD.id, OLD.title, OLD.artist, OLD.album, OLD.album_artist, OLD.genre);
END;

-- Rebuild FTS index for existing data
INSERT INTO tracks_fts(tracks_fts) VALUES('rebuild');

-- Expression indexes for browse sorting (derived album/artist)

-- Artist sort index: LOWER(COALESCE(NULLIF(TRIM(artist), ''), 'Unknown Artist'))
CREATE INDEX idx_tracks_artist_sort ON tracks(
    (LOWER(COALESCE(NULLIF(TRIM(artist), ''), 'Unknown Artist')))
);

-- Album browse index: (album_artist_sort, album_title_sort)
-- album_artist_sort = LOWER(COALESCE(NULLIF(TRIM(album_artist), ''), NULLIF(TRIM(artist), ''), 'Unknown Artist'))
-- album_title_sort = LOWER(COALESCE(NULLIF(TRIM(album), ''), 'Unknown Album'))
CREATE INDEX idx_tracks_album_sort ON tracks(
    (LOWER(COALESCE(NULLIF(TRIM(album_artist), ''), NULLIF(TRIM(artist), ''), 'Unknown Artist'))),
    (LOWER(COALESCE(NULLIF(TRIM(album), ''), 'Unknown Album')))
);

PRAGMA user_version = 3;
