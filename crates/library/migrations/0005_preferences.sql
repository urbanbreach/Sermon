-- M07 Preferences Settings Migration
-- All booleans use 'on'/'off' format to match existing effects.ts pattern

-- General category (PLACEHOLDER - disabled in UI)
INSERT OR IGNORE INTO settings (key, value) VALUES ('general.startup.with_windows', 'off');
INSERT OR IGNORE INTO settings (key, value) VALUES ('general.startup.minimized', 'off');

-- Player category
INSERT OR IGNORE INTO settings (key, value) VALUES ('player.buffer_size_ms', '500');
INSERT OR IGNORE INTO settings (key, value) VALUES ('player.preload_next', 'on');

-- Now Playing category (PLACEHOLDER - values persisted but not wired)
INSERT OR IGNORE INTO settings (key, value) VALUES ('nowplaying.double_click', 'play_now');
INSERT OR IGNORE INTO settings (key, value) VALUES ('nowplaying.queue_add_position', 'end');
INSERT OR IGNORE INTO settings (key, value) VALUES ('nowplaying.shuffle_mode', 'off');

-- Library category
INSERT OR IGNORE INTO settings (key, value) VALUES ('library.scan_on_startup', 'on');
INSERT OR IGNORE INTO settings (key, value) VALUES ('library.continuous_monitoring', 'off');

-- Tags category (PLACEHOLDER - values persisted but not wired)
INSERT OR IGNORE INTO settings (key, value) VALUES ('tags.backup_before_write', 'on');
INSERT OR IGNORE INTO settings (key, value) VALUES ('tags.write_behavior', 'prompt');

-- Internet category (PLACEHOLDER - disabled in UI)
INSERT OR IGNORE INTO settings (key, value) VALUES ('internet.lastfm_enabled', 'off');

-- Devices category (PLACEHOLDER - disabled in UI)
INSERT OR IGNORE INTO settings (key, value) VALUES ('devices.dsd_dop_enabled', 'off');

PRAGMA user_version = 5;
