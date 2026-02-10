-- Add parsed loudness metadata (ReplayGain track gain in dB)
ALTER TABLE tracks ADD COLUMN loudness_db REAL;

PRAGMA user_version = 10;
