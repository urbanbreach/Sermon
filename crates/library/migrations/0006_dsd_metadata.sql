-- Add DSD technical metadata to tracks
ALTER TABLE tracks ADD COLUMN dsd_rate_hz INTEGER;
ALTER TABLE tracks ADD COLUMN dsd_channels INTEGER;

PRAGMA user_version = 6;
