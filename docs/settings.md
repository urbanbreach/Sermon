# Settings Documentation

This document describes the settings system in Sermon, including all preference keys, their types, defaults, and behavior.

## Overview

Sermon uses SQLite for settings persistence via a key-value `settings` table. Settings are organized into 7 categories matching the Preferences UI.

### Storage Details

- **Location**: `settings` table in `library.db`
- **Schema**: `(key TEXT PRIMARY KEY, value TEXT, updated_at INTEGER)`
- **Versioning**: Uses `PRAGMA user_version` for migrations (current: v5)
- **Serialization**: Booleans use `'on'`/`'off'` strings (not `true`/`false`)

## Categories

### General

Startup behavior settings. **All placeholders - not yet wired.**

| Key | Type | Default | Description | Status |
|-----|------|---------|-------------|--------|
| `general.startup.with_windows` | bool | `'off'` | Start Sermon when Windows starts | Placeholder (disabled) |
| `general.startup.minimized` | bool | `'off'` | Start minimized to system tray | Placeholder (disabled) |

### Player

Audio output and buffer settings.

| Key | Type | Default | Description | Status |
|-----|------|---------|-------------|--------|
| `player.buffer_size_ms` | int | `'500'` | Ring buffer size in milliseconds (100-2000) | **Functional** - Restart required |
| `player.preload_next` | bool | `'on'` | Preload next track for gapless playback | Placeholder (deferred) |

**Audio Output Settings** (stored separately, managed by `playback.ts`):

| Key | Type | Default | Description | Status |
|-----|------|---------|-------------|--------|
| `audio.output.mode` | enum | `'exclusive'` | `'exclusive'` or `'shared'` | **Functional** - Hot-apply |
| `audio.output.policy` | enum | `'strict'` | `'strict'` or `'compatibility'` | **Functional** - Hot-apply |
| `audio.output.timing` | enum | `'polling'` | `'event'` or `'polling'` | **Functional** - Hot-apply |
| `audio.output.fade` | bool | `'off'` | Enable fade on format switch | **Functional** - Hot-apply |
| `audio.device.preference` | string | `'default'` | Preferred output device ID | **Functional** - Hot-apply |

### Now Playing

Queue and playback behavior settings. **All placeholders - not yet wired.**

| Key | Type | Default | Description | Status |
|-----|------|---------|-------------|--------|
| `nowplaying.double_click` | enum | `'play_now'` | Action on track double-click: `'play_now'`, `'add_to_queue'`, `'play_next'` | Placeholder (deferred) |
| `nowplaying.queue_add_position` | enum | `'end'` | Where to add tracks: `'end'`, `'next'` | Placeholder (deferred) |
| `nowplaying.shuffle_mode` | enum | `'off'` | Shuffle mode: `'off'`, `'tracks'`, `'albums'` | Placeholder (deferred) |

### Library

Library scanning and monitoring settings.

| Key | Type | Default | Description | Status |
|-----|------|---------|-------------|--------|
| `library.scan_on_startup` | bool | `'on'` | Run quick scan when app starts | **Functional** - Next startup |
| `library.continuous_monitoring` | bool | `'off'` | Watch folders for changes | Placeholder (disabled) |

### Tags

Tag writing behavior settings. **All placeholders - not yet wired.**

| Key | Type | Default | Description | Status |
|-----|------|---------|-------------|--------|
| `tags.backup_before_write` | bool | `'on'` | Create backup before modifying tags | Placeholder (deferred) |
| `tags.write_behavior` | enum | `'prompt'` | Save behavior: `'prompt'`, `'auto'`, `'never'` | Placeholder (deferred) |

### Internet

Network and online service settings.

| Key | Type | Default | Description | Status |
|-----|------|---------|-------------|--------|
| `internet.lastfm_enabled` | bool | `'off'` | Enable Last.fm scrobbling | Placeholder (disabled) |

**Artwork Provider Settings** (stored separately, managed by `effects.ts`):

| Key | Type | Default | Description | Status |
|-----|------|---------|-------------|--------|
| `artwork.provider.itunes` | bool | `'on'` | Enable iTunes artwork provider | **Functional** - Hot-apply |
| `artwork.provider.deezer` | bool | `'on'` | Enable Deezer artwork provider | **Functional** - Hot-apply |

### Devices

Hardware-specific audio settings. **All placeholders - not yet wired.**

| Key | Type | Default | Description | Status |
|-----|------|---------|-------------|--------|
| `devices.dsd_dop_enabled` | bool | `'off'` | Enable DSD over PCM (DoP) | Placeholder (disabled) |

### Appearance & Layout

Visual settings managed by `effects.ts`.

| Key | Type | Default | Description | Status |
|-----|------|---------|-------------|--------|
| `ui.bottombar.waveform_seekbar` | bool | `'off'` | Use waveform visualization in bottom bar | **Functional** - Hot-apply |
| `ui.bottombar.waveform_color` | hex | `'#4aafff'` | Color of the played portion in waveform seekbar | **Functional** - Hot-apply |

## Setting Status Legend

| Status | Meaning |
|--------|---------|
| **Functional** | Setting is wired to actual behavior |
| **Placeholder (disabled)** | UI control is disabled with "[Coming Soon]" label |
| **Placeholder (deferred)** | UI control is enabled, value persists, but behavior not yet implemented |

## Bit-Perfect Defaults

Sermon defaults to bit-perfect audio settings:

| Setting | Default | Rationale |
|---------|---------|-----------|
| `audio.output.mode` | `'exclusive'` | WASAPI Exclusive bypasses Windows mixer |
| `audio.output.policy` | `'strict'` | Requires exact format match with DAC |
| `player.buffer_size_ms` | `'500'` | Balance between latency and stability |

**Note**: In Exclusive + Strict mode, software volume control is disabled (unity gain) to preserve bit-perfect signal path.

## Restart-Required Settings

Some settings require application restart to take effect:

| Setting | Reason |
|---------|--------|
| `player.buffer_size_ms` | Ring buffer is created at audio thread initialization |

The UI displays a warning banner when these settings are changed.

## Migration Policy

1. **Forward-only migrations**: Settings schema only moves forward, never backward
2. **Additive changes**: New settings are added with sensible defaults
3. **No silent resets**: User preferences are never silently reset
4. **Version tracking**: `PRAGMA user_version` tracks schema version

### Migration History

| Version | Changes |
|---------|---------|
| 1 | Initial schema (tracks, folders) |
| 2 | Settings table |
| 3 | Full-text search |
| 4 | Artwork cache |
| 5 | M07 preferences (13 new keys) |

## API Reference

### Backend Commands

```typescript
// Get all settings for a category
invoke('cmd_settings_get_category', { category: 'player' })
// Returns: { "player.buffer_size_ms": "500", "player.preload_next": "on" }

// Set settings for a category
invoke('cmd_settings_set_category', {
  category: 'player',
  settings: { "player.buffer_size_ms": "1000" }
})

// Reset category to defaults
invoke('cmd_settings_reset_category', { category: 'player' })

// Export diagnostics JSON
invoke('cmd_settings_export_diagnostics')
// Returns: JSON string with system info, audio state, and all settings
```

### Frontend State

```typescript
// Load category settings
import { loadCategorySettings, saveCategorySetting } from '$lib/state/preferences';

await loadCategorySettings('player');
await saveCategorySetting('player', 'player.buffer_size_ms', '1000');

// Existing stores for audio settings
import { outputSettings, saveOutputSettings } from '$lib/state/playback';
import { providerItunes, setProviderItunes } from '$lib/state/effects';
```

## Export Diagnostics

The "Export Diagnostics" feature in Preferences generates a JSON file for debugging:

```json
{
  "version": "0.1.0",
  "timestamp": "2024-01-15T10:30:00Z",
  "system": {
    "os": "windows",
    "tauri_version": "2.x.x",
    "app_version": "0.1.0"
  },
  "audio": {
    "current_device": "Device Name",
    "output_mode": "exclusive",
    "wasapi_available": true
  },
  "settings": {
    "player.buffer_size_ms": "500",
    ...
  }
}
```
