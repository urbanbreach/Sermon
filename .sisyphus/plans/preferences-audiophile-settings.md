# Milestone 07 - Preferences & Audiophile Settings

## Context

### Original Request
Build the **Preferences** experience (MusicBee-inspired categories) and wire up "audiophile-grade" settings—while keeping all DSP off by default. Leverage frontend UI/UX engineer subagent and Tauri MCP server for design tasks and UI polish.

### Interview Summary
**Key Discussions**:
- User provided explicit milestone file with complete requirements
- No scope changes permitted; all details must be preserved
- MUST/SHOULD/MAY rubric to be applied from milestone

**Research Findings**:
- **Settings Storage**: SQLite `settings` table already exists (`crates/library/migrations/0002_settings.sql`) with key-value schema
- **Current Settings UI**: `ui/src/lib/views/SettingsView.svelte` has basic sections (Library, Audio, Theme, Artwork Providers)
- **UI Framework**: Svelte 5 with Runes, custom stack-based routing, Glassmorphism design system
- **Audio Settings**: WASAPI exclusive/shared mode already wired via `outputSettings` store
- **No test infrastructure**: No `*.test.*` files found; manual QA required
- **ADR format**: Established in `docs/adr/` with consistent structure

### Dependencies
- **Milestone 03 — wasapi-exclusive-bit-perfect**: ✅ Complete (WASAPI exclusive mode implemented)

---

## Work Objectives

### Core Objective
Create a MusicBee-inspired Preferences window with left-nav category navigation and wire all audiophile-grade settings to actual behavior, with bit-perfect defaults.

### Concrete Deliverables
1. **Preferences UI**: New window/view with left-nav categories (General, Player, Now Playing, Library, Tags, Internet, Devices)
   - **Note**: Theme settings remain in existing SettingsView.svelte (not migrated to Preferences)
   - This avoids breaking existing `effects.ts` integration and keeps Preferences focused on audiophile/playback settings
2. **Settings Persistence**: Schema versioning and migrations for new settings
3. **Settings Wiring**: Output mode, buffer size, library scan-on-startup affect actual behavior
4. **Documentation**: `/docs/settings.md` (schema, defaults, migration policy)
5. **ADR**: `/docs/adr/0009-settings-storage.md`
6. **UI Vision Artifacts**:
   - `/artifacts/ui/07-preferences-audiophile-settings/prefs-general.png`
   - `/artifacts/ui/07-preferences-audiophile-settings/prefs-player.png`
   - `/artifacts/ui/07-preferences-audiophile-settings/prefs-library.png`
   - `/artifacts/ui/07-preferences-audiophile-settings/prefs-tags.png`
   - `/artifacts/ui/07-preferences-audiophile-settings/REVIEW.md`
  7. **Snapshot Pack**: Manual capture via Tauri MCP to `/artifacts/ui/07-preferences-audiophile-settings/`

8. **Risk Register Update**: Add M07-specific risks

### Definition of Done
- [x] Settings persist across app restart (Tier A verification)
- [x] Changing output mode/device affects playback
- [x] Changing library scan settings affects scanning behavior
- [x] Snapshot pack produced successfully
- [x] All acceptance criteria verified via manual QA

### Must Have
- Preferences UI with left-nav categories
- Settings persistence with schema versioning
- Output mode wired to actual WASAPI behavior
- Library scan-on-startup wired to actual behavior
- Bit-perfect defaults (Exclusive ON, DSP OFF)
- Reset to Defaults (per category)
- Export Diagnostics implemented (Task 9)

### Reset to Defaults Behavior Per Category

| Category | Backed By | Reset Action | Default Values |
|----------|-----------|--------------|----------------|
| General | `preferences.ts` | `cmd_settings_reset_category('general')` | `startup.with_windows: 'off'`, `startup.minimized: 'off'` |
| Player (buffer/preload) | `preferences.ts` | `cmd_settings_reset_category('player')` | `buffer_size_ms: '500'`, `preload_next: 'on'` |
| Player (output) | `playback.ts` | `saveOutputSettings({ mode: 'exclusive', policy: 'strict', timing: 'polling', fade: true })` | Bit-perfect defaults |
| Now Playing | `preferences.ts` | `cmd_settings_reset_category('nowplaying')` | `double_click: 'play_now'`, `queue_add_position: 'end'`, `shuffle_mode: 'off'` |
| Library | `preferences.ts` | `cmd_settings_reset_category('library')` | `scan_on_startup: 'on'`, `continuous_monitoring: 'off'` |
| Tags | `preferences.ts` | `cmd_settings_reset_category('tags')` | `backup_before_write: 'on'`, `write_behavior: 'prompt'` |
| Internet (providers) | `effects.ts` | `setProviderItunes(true)`, `setProviderDeezer(true)` | Both enabled |
| Internet (lastfm) | `preferences.ts` | `cmd_settings_reset_category('internet')` | `lastfm_enabled: 'off'` |
| Devices | `preferences.ts` | `cmd_settings_reset_category('devices')` | `dsd_dop_enabled: 'off'` |

**Reset Button Implementation**:
- Each category header has "Reset to Defaults" button
- Button calls appropriate reset action(s) based on backing store
- For mixed categories (Player, Internet), call both backend reset AND store-specific reset
- Show inline status message below button: "✓ Reset complete" (fades after 3s)

### Export Diagnostics (Implemented in Task 9)

**UI**: Button in Preferences footer labeled "Export Diagnostics"

**Trigger**: Calls `cmd_settings_export_diagnostics()` → opens save dialog → downloads JSON file

**Schema** (placeholder, expandable later):

**Note**: Export Diagnostics is fully implemented in Task 9 (not a placeholder feature).
```json
{
  "version": "0.1.0",
  "timestamp": "2024-01-15T10:30:00Z",
  "system": {
    "os": "Windows 11",
    "tauri_version": "2.x.x",
    "app_version": "0.1.0"
  },
  "audio": {
    "current_device": "Device Name",
    "output_mode": "exclusive",
    "wasapi_available": true
  },
  "settings": {}
}
```

**Behavior**: Backend-driven export (simpler, no additional frontend deps needed):
1. Frontend calls `cmd_settings_export_diagnostics()` 
2. Backend generates JSON and returns it as string
3. Frontend uses existing `@tauri-apps/plugin-dialog` `save()` to get path
4. Frontend uses `@tauri-apps/plugin-fs` `writeTextFile()` to save

**Required Dependencies** (add if not present):
- `ui/package.json`: Add `"@tauri-apps/plugin-fs": "^2"` to dependencies
- `src-tauri/Cargo.toml`: Add `tauri-plugin-fs = "2"` to dependencies  
- `src-tauri/Cargo.toml`: Add `chrono = { version = "0.4", features = ["serde"] }` to dependencies
- `src-tauri/src/lib.rs`: Add `.plugin(tauri_plugin_fs::init())` to Builder
- `src-tauri/capabilities/default.json`: Add `"fs:default"` permission

**Diagnostics JSON Field Sources**:
| Field | Source | Notes |
|-------|--------|-------|
| `version` | Hardcoded `"0.1.0"` | Placeholder, update when versioning exists |
| `timestamp` | `chrono::Utc::now().to_rfc3339()` | Requires chrono crate (add to Cargo.toml) |
| `system.os` | `std::env::consts::OS` | Returns "windows" |
| `system.tauri_version` | `tauri::VERSION` | Tauri constant |
| `system.app_version` | Hardcoded `"0.1.0"` | Placeholder |
| `audio.current_device` | Read from settings: `get_setting(conn, "audio.device.preference")` | Already persisted |
| `audio.output_mode` | Read from settings: `get_setting(conn, "audio.output.mode")` | Already persisted |
| `audio.wasapi_available` | Hardcoded `true` | Windows-only app, always available |
| `settings` | Read all from SQLite | Via `get_setting()` for each key |

**Note**: Audio diagnostics use persisted settings, not live engine state (simpler, avoids thread sync).

### UI Feedback Pattern

**No toast system exists in this codebase.** Use inline status messages instead:

| Action | Feedback Pattern | Example Location |
|--------|-----------------|------------------|
| Reset to Defaults | Inline text below button: "✓ Reset complete" (fades after 3s) | New pattern |
| Export Diagnostics | Inline text: "✓ Saved to {path}" or "✗ Export failed" | New pattern |
| Setting saved | No feedback (silent save, matching existing SettingsView behavior) | `ui/src/lib/views/SettingsView.svelte` |

**Implementation**: Add `statusMessage` state variable to PreferencesView, display conditionally below action buttons, auto-clear with `setTimeout`.

### Preferences Loading Lifecycle

**Initial Load**:
- `PreferencesView.svelte` calls `loadCategorySettings()` for active category in `onMount`
- `loadEffectsSettings()` is called in `PreferencesView.svelte` onMount (ensures fresh provider values)
- Each category component loads its specific data on mount

**After Reset**:
- `resetCategoryToDefaults(category)` calls backend reset command
- Then immediately calls `loadCategorySettings(category)` to refresh UI
- For mixed categories (Player, Internet), also refresh relevant existing stores

**Load Sequence in PreferencesView.svelte**:
```svelte
<script>
  import { onMount } from 'svelte';
  import { loadEffectsSettings } from '../state/effects';
  import { loadOutputSettings, loadDevices } from '../state/playback';
  import { loadFolders } from '../state/library';
  import { loadCategorySettings } from '../state/preferences';
  
  let activeCategory = 'general';
  
  onMount(async () => {
    // Load all data needed by category components
    await Promise.all([
      loadEffectsSettings(),  // For Internet category (artwork providers)
      loadOutputSettings(),   // For Player category (output mode, policy, etc.)
      loadDevices(),          // For Player category (device dropdown)
      loadFolders(),          // For Library category (folder list)
      loadCategorySettings(activeCategory)
    ]);
  });
  
  async function handleCategoryChange(category) {
    activeCategory = category;
    await loadCategorySettings(category);
  }
</script>
```

**Data Loading Per Category**:
| Category | Data Loaded | Source Function |
|----------|-------------|-----------------|
| General | Category settings | `loadCategorySettings('general')` |
| Player | Devices, output settings, category settings | `loadDevices()`, `loadOutputSettings()`, `loadCategorySettings('player')` |
| Now Playing | Category settings | `loadCategorySettings('nowplaying')` |
| Library | Folders, category settings | `loadFolders()`, `loadCategorySettings('library')` |
| Tags | Category settings | `loadCategorySettings('tags')` |
| Internet | Effects settings, category settings | `loadEffectsSettings()`, `loadCategorySettings('internet')` |
| Devices | Category settings | `loadCategorySettings('devices')` |

**Note**: All data is loaded on PreferencesView mount (guarded by `SERMON_MOCK`) to ensure all categories are ready without requiring user to visit other views first.

### Restart-Required UX

**Owner Component**: `PlayerPrefs.svelte` (where buffer size lives)

**Settings Requiring Restart**:
- `player.buffer_size_ms` - Ring buffer size change

**State Management**:
```svelte
<script>
  let restartRequired = false;
  let originalBufferSize = null;
  
  function handleBufferSizeChange(newValue) {
    if (originalBufferSize === null) {
      originalBufferSize = currentBufferSize;
    }
    if (newValue !== originalBufferSize) {
      restartRequired = true;
    }
    // Save setting...
  }
</script>

{#if restartRequired}
  <div class="restart-banner">
    ⚠️ Restart required for buffer size change to take effect. Please close and reopen the application.
  </div>
{/if}
```

**Restart Implementation**:
- **Approach**: Manual restart instruction only (no plugin dependency)
- **Rationale**: No `tauri-plugin-process` exists in project; adding dependency for one feature is overkill
- **UX**: Show warning banner with text only - NO button (user manually restarts)
- Banner persists until app restart or value reverted to original

### Snapshot/Mock Mode Handling

**PreferencesView must include SERMON_MOCK guard** (matching SettingsView pattern):

```svelte
<script>
  const isMock = import.meta.env.SERMON_MOCK === '1';
  
  // Disable backend calls in mock mode
  async function handleSave() {
    if (isMock) return;
    // ... actual save logic
  }
</script>
```

**Why**: Ensures deterministic snapshot output by preventing real API calls that could vary between runs.

### Must NOT Have (Guardrails)
- Full hotkey customization UI (deferred)
- Smart playlists (out of scope)
- Silent settings resets (must have forward migrations)
- DSP enabled by default (bit-perfect requirements)
- Over-complicated placeholder settings (clearly label non-functional items)
- Moving theme settings out of SettingsView (stay in existing location)
- Adding preference categories beyond the 7 listed (General, Player, Now Playing, Library, Tags, Internet, Devices)

### Settings Validation Rules

| Setting | Type | Default | Validation | Hot-Apply |
|---------|------|---------|------------|-----------|
| `general.startup.with_windows` | bool | `'off'` | `'on'` or `'off'` only | N/A (placeholder) |
| `general.startup.minimized` | bool | `'off'` | `'on'` or `'off'` only | N/A (placeholder) |
| `player.buffer_size_ms` | int | `'500'` | 100-2000 range, integer only | NO (restart required) |
| `player.preload_next` | bool | `'on'` | `'on'` or `'off'` only | N/A (placeholder) |
| `nowplaying.double_click` | enum | `'play_now'` | `'play_now'`\|`'add_to_queue'`\|`'play_next'` | N/A (placeholder) |
| `nowplaying.queue_add_position` | enum | `'end'` | `'end'`\|`'next'` | N/A (placeholder) |
| `nowplaying.shuffle_mode` | enum | `'off'` | `'off'`\|`'tracks'`\|`'albums'` | N/A (placeholder) |
| `library.scan_on_startup` | bool | `'on'` | `'on'` or `'off'` only | NO (next startup) |
| `library.continuous_monitoring` | bool | `'off'` | `'on'` or `'off'` only | N/A (placeholder, disabled) |
| `tags.backup_before_write` | bool | `'on'` | `'on'` or `'off'` only | N/A (placeholder) |
| `tags.write_behavior` | enum | `'prompt'` | `'prompt'`\|`'auto'`\|`'never'` | N/A (placeholder) |
| `internet.lastfm_enabled` | bool | `'off'` | `'on'` or `'off'` only | N/A (placeholder) |
| `devices.dsd_dop_enabled` | bool | `'off'` | `'on'` or `'off'` only | N/A (placeholder) |

**Hot-Apply Summary**: Functional settings include:
- `audio.output.*` (already wired via `playback.ts` stores; not part of new preferences table)
- `player.buffer_size_ms` (new preference; requires restart)
- `library.scan_on_startup` (new preference; takes effect next startup)

All other settings are PLACEHOLDER (persist value only).

---

## Verification Strategy (MANDATORY)

### Test Decision
- **Infrastructure exists**: NO
- **User wants tests**: Manual-only (no test files found in project)
- **Framework**: N/A

### Manual QA Protocol

Each TODO includes detailed verification procedures using:
- **Tauri MCP Server**: For UI interaction and screenshots (primary tool)
- **Terminal Commands**: For build/compile verification

**Note**: Playwright is NOT used - the project uses Tauri MCP for all UI automation.

**Evidence Required:**
- Screenshots saved to `/artifacts/ui/07-preferences-audiophile-settings/` (deliverables)
- Command output captured in verification notes
- Settings persistence verified across app restarts

---

## Task Flow

```
Task 1 (ADR) ──────────────────────────────────────────────┐
Task 2 (Schema/Migrations) ────────────────────────────────┤
                                                           ├──> Task 7 (Wiring)
Task 3 (Backend Settings API) ────────────────────────────┤
Task 4 (Preferences Shell UI) ────────────────────────────┤
Task 5 (Category Components) ─────────────────────────────┤
Task 6 (Svelte State/Stores) ─────────────────────────────┘
                                                           │
                                                           v
Task 8 (UI Vision/Snapshots) ──> Task 9 (Export Diagnostics) ──> Task 10 (Docs) ──> Task 11 (Risk Register)
```

## Parallelization

| Group | Tasks | Reason |
|-------|-------|--------|
| A | 1, 2, 3 | Independent backend work |
| B | 4, 5, 6 | Can start after schema exists |
| C | 9, 10, 11 | Documentation after implementation |

| Task | Depends On | Reason |
|------|------------|--------|
| 3 | 2 | API needs schema |
| 5 | 4 | Categories need shell |
| 7 | 3, 5, 6 | Wiring needs all pieces |
| 8 | 7 | Vision needs working UI |
| 9 | 3 | Export needs settings API |
| 10 | 7 | Docs need implementation |
| 11 | 10 | Risk register is final |

---

## TODOs

### Phase 1: Foundation

- [x] 1. Create ADR 0009: Settings Storage Strategy

  **What to do**:
  - Document decision to use SQLite settings table (already exists)
  - Explain schema versioning strategy using `user_version` PRAGMA
  - Document migration approach (forward-only, no silent resets)
  - Compare with JSON alternative (simpler but harder migration)

  **Must NOT do**:
  - Change existing settings table structure fundamentally
  - Introduce breaking changes to existing settings

  **Parallelizable**: YES (with 2, 3)

  **References**:
  
  **Pattern References**:
  - `docs/adr/0003-db-sqlite.md` - ADR format and SQLite rationale
  - `docs/adr/0006-audio-output-wasapi.md` - ADR structure example
  
  **Schema References**:
  - `crates/library/migrations/0002_settings.sql` - Existing settings table schema
  - `crates/library/src/db/mod.rs:get_setting()` - Current settings access pattern
  
  **Documentation References**:
  - Milestone file lines 37-42 - Settings storage decision rationale

  **Acceptance Criteria**:
  
  **File Verification:**
  - [ ] File exists: `docs/adr/0009-settings-storage.md`
  - [ ] Contains sections: Context, Decision, Alternatives Considered, Consequences
  - [ ] Documents migration strategy with `user_version` PRAGMA
  - [ ] References existing `0002_settings.sql` schema

  **Commit**: YES
  - Message: `docs(adr): add ADR 0009 settings storage strategy`
  - Files: `docs/adr/0009-settings-storage.md`

---

- [x] 2. Add Settings Schema Migration for M07 Settings

  **What to do**:
  - Create migration `0005_preferences.sql` (or next number)
  - **CRITICAL**: Update `crates/library/src/db/migrations.rs` to include the new migration
    - Add `if version < 5 { ... }` block after line 29 to apply `0005_preferences.sql`
    - This file currently stops at version 4; without this change, migration won't run
  - Add default values for all new preference keys
  - Ensure bit-perfect defaults: `audio.output.mode = 'exclusive'`, no DSP
  - **Serialization format**: Use `'on'`/`'off'` for booleans (matching existing pattern in `effects.ts:49,79`)
  - Settings keys to add:
    - `general.startup.with_windows` (bool → `'off'`) **[PLACEHOLDER]**
    - `general.startup.minimized` (bool → `'off'`) **[PLACEHOLDER]**
    - `player.buffer_size_ms` (int → `'500'`)
    - `player.preload_next` (bool → `'on'`)
    - `nowplaying.double_click` (enum → `'play_now'`)
      - Values: `'play_now'` | `'add_to_queue'` | `'play_next'`
      - UI Labels: "Play Now" | "Add to Queue" | "Play Next"
    - `nowplaying.queue_add_position` (enum → `'end'`)
      - Values: `'end'` | `'next'`
      - UI Labels: "Add to End" | "Play Next"
    - `nowplaying.shuffle_mode` (enum → `'off'`)
      - Values: `'off'` | `'tracks'` | `'albums'`
      - UI Labels: "Off" | "Shuffle Tracks" | "Shuffle Albums"
    - `library.scan_on_startup` (bool → `'on'`)
    - `library.continuous_monitoring` (bool → `'off'`) **[PLACEHOLDER - DISABLED]**
    - `tags.backup_before_write` (bool → `'on'`)
    - `tags.write_behavior` (enum → `'prompt'`)
      - Values: `'prompt'` | `'auto'` | `'never'`
      - UI Labels: "Ask Before Saving" | "Save Automatically" | "Never Save"
    - `internet.lastfm_enabled` (bool → `'off'`) **[PLACEHOLDER]**
    - `devices.dsd_dop_enabled` (bool → `'off'`) **[PLACEHOLDER]**

  **Placeholder vs Functional Settings**:
  | Setting | Status | Behavior |
  |---------|--------|----------|
  | `general.startup.*` | PLACEHOLDER | UI disabled, value persisted but not wired |
  | `player.buffer_size_ms` | FUNCTIONAL | Wired to ring buffer (restart required) |
  | `player.preload_next` | PLACEHOLDER | UI enabled, value persisted, wiring deferred (no preload logic exists) |
  | `nowplaying.*` | PLACEHOLDER | UI enabled, values persisted, wiring deferred (requires TracksView/AlbumDetailView changes) |
  | `library.scan_on_startup` | FUNCTIONAL | Wired to startup scan |
  | `library.continuous_monitoring` | PLACEHOLDER | UI disabled (no persistence toggle) |  | `tags.*` | PLACEHOLDER | UI enabled, values persisted, wiring deferred (requires TagEditor integration) |
  | `internet.lastfm_enabled` | PLACEHOLDER | UI disabled, value persisted |
  | `devices.dsd_dop_enabled` | PLACEHOLDER | UI disabled, value persisted |
  
  **Placeholder UI Pattern**:
  - Disabled placeholders: `[Coming Soon]` label, control disabled (startup, continuous monitoring, Last.fm, DSD/DoP)
  - Deferred wiring placeholders: control enabled, persist value, hint "Takes effect in future update" (preload, nowplaying, tags)
  
  **Future Wiring Points (NOT in M07 scope, documented for reference)**:
  - `nowplaying.double_click`: `ui/src/lib/views/TracksView.svelte` row click handler
  - `nowplaying.queue_add_position`: `ui/src/lib/state/playback.ts:addToQueue()`
  - `tags.backup_before_write`: `ui/src/lib/components/TagEditor.svelte` save handler
  - `tags.write_behavior`: `ui/src/lib/components/TagEditor.svelte` save confirmation flow
  - `player.preload_next`: Would require new preload module in audio thread

  **Must NOT do**:
  - Modify existing settings keys
  - Break backward compatibility
  - Reset existing user preferences
  - Use `true`/`false` for booleans (must use `'on'`/`'off'` to match existing pattern)

  **Parallelizable**: YES (with 1, 3)

  **References**:
  
  **Pattern References**:
  - `crates/library/migrations/0001_init.sql` - Migration file format
  - `crates/library/migrations/0002_settings.sql` - Settings table creation
  - `crates/library/migrations/0003_fts.sql` - PRAGMA user_version pattern
  
  **Migration Wiring (CRITICAL)**:
  - `crates/library/src/db/migrations.rs:5-32` - **MUST UPDATE**: Add version 5 block after line 29
    ```rust
    if version < 5 {
        info!("Applying migration 0005_preferences");
        conn.execute_batch(include_str!("../../migrations/0005_preferences.sql"))?;
        version = 5;
    }
    ```
  
  **Serialization Pattern**:
  - `ui/src/lib/state/effects.ts:49-51` - `parseBool()` expects `'on'` for true, anything else for false
  - `ui/src/lib/state/effects.ts:79` - `value ? 'on' : 'off'` pattern for saving
  
  **Milestone References**:
  - Lines 12-19 - Categories and their settings

  **Acceptance Criteria**:
  
  **File Verification:**
  - [ ] Migration file exists: `crates/library/migrations/0005_preferences.sql`
  - [ ] Contains INSERT OR IGNORE for all new settings keys
  - [ ] Sets `PRAGMA user_version = 5`
  - [ ] Default values use `'on'`/`'off'` for booleans (not `true`/`false`)
  - [ ] `crates/library/src/db/migrations.rs` updated with version 5 block
  
  **Build Verification:**
  - [ ] `cargo build -p library` → Compiles successfully
  - [ ] `cargo test -p library` → All tests pass
  
  **Runtime Verification:**
  - [ ] Start app with fresh DB → Settings initialized with correct defaults
  - [ ] Start app with existing DB → No data loss, new settings added
  - [ ] Query setting via dev console → Returns `'on'` or `'off'` format

  **Commit**: YES
  - Message: `feat(library): add settings migration for M07 preferences`
  - Files: `crates/library/migrations/0005_preferences.sql`, `crates/library/src/db/migrations.rs`

---

- [x] 3. Extend Backend Settings API

  **What to do**:
  - Add typed settings structs in Rust for each category
  - Create Tauri commands for bulk get/set per category
  - Implement `cmd_settings_get_category(category: &str)` → JSON
  - Implement `cmd_settings_set_category(category: &str, settings: Value)`
  - Add `cmd_settings_reset_category(category: &str)` for per-category reset

  **API Contract Definition (CRITICAL)**:
  
  Each category returns a JSON object with keys matching Task 2 settings. All values are strings (matching SQLite TEXT storage):
  
  ```typescript
  // cmd_settings_get_category('general') returns:
  {
    "general.startup.with_windows": "off",
    "general.startup.minimized": "off"
  }
  
  // cmd_settings_get_category('player') returns:
  {
    "player.buffer_size_ms": "500",
    "player.preload_next": "on"
  }
  
  // cmd_settings_get_category('nowplaying') returns:
  {
    "nowplaying.double_click": "play_now",
    "nowplaying.queue_add_position": "end",
    "nowplaying.shuffle_mode": "off"
  }
  
  // cmd_settings_get_category('library') returns:
  {
    "library.scan_on_startup": "on",
    "library.continuous_monitoring": "off"
  }
  
  // cmd_settings_get_category('tags') returns:
  {
    "tags.backup_before_write": "on",
    "tags.write_behavior": "prompt"
  }
  
  // cmd_settings_get_category('internet') returns:
  {
    "internet.lastfm_enabled": "off"
  }
  
  // cmd_settings_get_category('devices') returns:
  {
    "devices.dsd_dop_enabled": "off"
  }
  ```
  
  **Serialization rules**:
  - Booleans: `"on"` or `"off"` (matching `effects.ts` pattern)
  - Integers: String representation (e.g., `"500"`)
  - Strings: Direct value (e.g., `"play_now"`)
  
  **Reset behavior**: 
  - `cmd_settings_reset_category` **reinserts defaults directly** (not just deletes)
  - The command maintains a hardcoded defaults map per category in Rust
  - After reset, calls `set_setting()` for each key with default value
  - This ensures `get_setting()` always returns a value after reset
  
  **Defaults Map (in Rust command handler)**:
  ```rust
  fn get_category_defaults(category: &str) -> HashMap<&'static str, &'static str> {
      match category {
          "general" => HashMap::from([
              ("general.startup.with_windows", "off"),
              ("general.startup.minimized", "off"),
          ]),
          "player" => HashMap::from([
              ("player.buffer_size_ms", "500"),
              ("player.preload_next", "on"),
          ]),
          // ... etc for each category
          _ => HashMap::new(),
      }
  }
  ```
  
  **Category Validation**:
  - Valid categories: `["general", "player", "nowplaying", "library", "tags", "internet", "devices"]`
  - Invalid category → Return error: `Err("Invalid category: {name}. Valid: general, player, nowplaying, library, tags, internet, devices")`
  - Empty settings object for set → No-op (not an error)

  **Must NOT do**:
  - Break existing `cmd_settings_get` / `cmd_settings_set` commands
  - Remove backward compatibility
  - Use `true`/`false` for booleans (must use `"on"`/`"off"`)

  **Parallelizable**: YES (with 1, 2 initially; then needs 2 complete)

  **References**:
  
  **Pattern References**:
  - `src-tauri/src/commands/library.rs:cmd_settings_get` - Existing settings command pattern
  - `src-tauri/src/commands/library.rs:cmd_settings_set` - Existing settings command pattern
  - `src-tauri/src/commands/playback.rs` - Command structure patterns
  
  **State References**:
  - `src-tauri/src/state.rs` - Application state management
  - `crates/library/src/db/mod.rs:229-246` - `get_setting()` and `set_setting()` DB access pattern
  
  **Type References**:
  - `ui/src/lib/api/playback.ts:AudioOutputSettings` - Frontend type example

  **Acceptance Criteria**:
  
  **Build Verification:**
  - [ ] `cargo build` → Compiles successfully
  - [ ] `cargo test` → All tests pass
  - [ ] No clippy warnings on new code
  
  **API Verification (via Tauri dev console):**
  - [ ] `invoke('cmd_settings_get_category', {category: 'general'})` → Returns `{"general.startup.with_windows": "off", ...}`
  - [ ] `invoke('cmd_settings_set_category', {category: 'general', settings: {"general.startup.with_windows": "on"}})` → Persists
  - [ ] `invoke('cmd_settings_get_category', {category: 'general'})` → Returns `{"general.startup.with_windows": "on", ...}`
  - [ ] `invoke('cmd_settings_reset_category', {category: 'general'})` → Resets to defaults
  - [ ] Export diagnostics command is NOT part of Task 3 (handled in Task 9)

  **Commit**: YES
  - Message: `feat(tauri): add category-based settings API`
  - Files: `src-tauri/src/commands/settings.rs` (new), `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`

  **Note**: Export diagnostics command is implemented and registered in Task 9 only.  
  **Command Registration (CRITICAL)**:
  - Create new file: `src-tauri/src/commands/settings.rs` with all new commands
  - Update `src-tauri/src/commands/mod.rs`:
    - Add `pub mod settings;`
    - Add `pub use settings::*;`
  - Update `src-tauri/src/lib.rs:172-215` invoke_handler list:
    - Add `cmd_settings_get_category,`
    - Add `cmd_settings_set_category,`
    - Add `cmd_settings_reset_category,`
  
---

### Phase 2: UI Foundation

- [x] 4. Create Preferences Shell Component with Left-Nav

  **What to do**:
  - Create `ui/src/lib/views/PreferencesView.svelte` as new view
  - Implement MusicBee-style left sidebar navigation
  - Categories: General, Player, Now Playing, Library, Tags, Internet, Devices
  - Use Glassmorphism design tokens (`--glass-bg`, `--glass-border`, etc.)
  - **CRITICAL**: Add `'preferences'` route to the Route union type in `ui/src/lib/state/route.ts:4-13`
    - Add `| { name: 'preferences' }` to the Route type
  - **CRITICAL**: Add preferences entry to LeftNav in `ui/src/lib/components/LeftNav.svelte:4,29-42`
    - Add `'preferences'` to `SimpleRouteName` type on line 4
    - Add button for Preferences in the settings nav-section (lines 29-42)
  - **CRITICAL**: Add view rendering in `ui/src/App.svelte`
    - Import PreferencesView
    - Add `{:else if $currentRouteName === 'preferences'}` block
  - Conditionally render category content area
  - Add "Reset to Defaults" button in each category header
  - Add "Export Diagnostics" button in footer (implemented in Task 9)

  **Must NOT do**:
  - Remove or break existing SettingsView.svelte (keep for backward compat)
  - Use external UI libraries (stick to native Svelte + CSS)

  **Parallelizable**: YES (after schema exists)

  **References**:
  
  **Pattern References**:
  - `ui/src/lib/views/SettingsView.svelte` - Current settings layout patterns
  - `ui/src/lib/components/LeftNav.svelte` - Left navigation patterns
  - `ui/src/App.svelte` - View routing pattern
  
  **Route Type Definition (MUST UPDATE)**:
  - `ui/src/lib/state/route.ts:4-13` - Add `| { name: 'preferences' }` to Route union type
  - `ui/src/lib/state/route.ts:92` - Add `'preferences'` to `setRoute` function signature
  
  **LeftNav Integration (MUST UPDATE)**:
  - `ui/src/lib/components/LeftNav.svelte:4` - Add `'preferences'` to `SimpleRouteName` type
  - `ui/src/lib/components/LeftNav.svelte:29-42` - Add Preferences button in settings section
  
  **App Shell Integration (MUST UPDATE)**:
  - `ui/src/App.svelte` - Import PreferencesView and add route conditional
  
  **Style References**:
  - `ui/src/lib/views/SettingsView.svelte:262-389` - CSS class patterns (`.section`, `.setting`, `.btn`)
  - `ui/src/app.css` - Global glass variables

  **Acceptance Criteria**:
  
  **Route Integration Verification:**
  - [ ] `ui/src/lib/state/route.ts` contains `| { name: 'preferences' }` in Route type
  - [ ] `ui/src/lib/components/LeftNav.svelte` has Preferences button in nav
  - [ ] `ui/src/App.svelte` imports and renders PreferencesView
  
  **Visual Verification (via Tauri MCP):**
  - [ ] Preferences view renders with left sidebar
  - [ ] All 7 categories visible in sidebar: General, Player, Now Playing, Library, Tags, Internet, Devices
  - [ ] Clicking category highlights it and shows content area
  - [ ] Glass styling applied consistently
  - [ ] "Reset to Defaults" button visible per category
  - [ ] "Export Diagnostics" button in footer
  
  **Navigation Verification:**
  - [ ] LeftNav shows "Preferences" button in settings section
  - [ ] Clicking "Preferences" in LeftNav navigates to preferences view
  - [ ] Route to preferences works: `navigate({ name: 'preferences' })`
  - [ ] Back navigation returns to previous view
  
  **Commit**: YES
  - Message: `feat(ui): add Preferences view shell with left-nav categories`
  - Files: `ui/src/lib/views/PreferencesView.svelte`, `ui/src/lib/state/route.ts`, `ui/src/lib/components/LeftNav.svelte`, `ui/src/App.svelte`

---

- [x] 5. Implement Category Content Components

  **What to do**:
  - Create component per category in `ui/src/lib/components/preferences/`:
    - `GeneralPrefs.svelte` - Startup behavior (all PLACEHOLDER, disabled)
    - `PlayerPrefs.svelte` - Output mode, device, buffer size, preload
    - `NowPlayingPrefs.svelte` - Double-click behavior, queue rules, shuffle modes
    - `LibraryPrefs.svelte` - Folders, scan-on-startup, continuous monitoring (PLACEHOLDER)
    - `TagsPrefs.svelte` - Backup policy, write behavior
    - `InternetPrefs.svelte` - Artwork providers (from effects.ts), Last.fm (PLACEHOLDER)
    - `DevicesPrefs.svelte` - DSD/DoP (PLACEHOLDER)
  - Migrate relevant sections from SettingsView.svelte
  - Mark placeholders clearly using two patterns:
  - Disabled placeholders: `[Coming Soon]` label, control disabled (startup, continuous monitoring, Last.fm, DSD/DoP)
  - Deferred wiring placeholders: control enabled, hint "Takes effect in future update" (preload, nowplaying, tags)
  - Use consistent `.setting` and `.setting-hint` class patterns

  **Player Category Data Sourcing**:
  | Field | Source Store | Read/Write Pattern |
  |-------|-------------|-------------------|
  | Output Device | `playback.ts:devices`, `currentDevice` | `loadDevices()`, `selectDevice()` |
  | Output Mode | `playback.ts:outputSettings.mode` | `loadOutputSettings()`, `saveOutputSettings()` |
  | Policy | `playback.ts:outputSettings.policy` | `loadOutputSettings()`, `saveOutputSettings()` |
  | Timing Mode | `playback.ts:outputSettings.timing` | `loadOutputSettings()`, `saveOutputSettings()` |
  | Fade | `playback.ts:outputSettings.fade` | `loadOutputSettings()`, `saveOutputSettings()` |
  | Buffer Size | `preferences.ts` (NEW) | `loadCategorySettings('player')`, `saveCategorySettings()` |
  | Preload Next | `preferences.ts` (NEW) | `loadCategorySettings('player')`, `saveCategorySettings()` |

  **Internet Category Data Sourcing**:
  | Field | Source Store | Read/Write Pattern |
  |-------|-------------|-------------------|
  | iTunes Provider | `effects.ts:providerItunes` | `loadEffectsSettings()`, `setProviderItunes()` |
  | Deezer Provider | `effects.ts:providerDeezer` | `loadEffectsSettings()`, `setProviderDeezer()` |
  | Last.fm | `preferences.ts` (NEW, PLACEHOLDER) | Disabled, value persisted |

  **Must NOT do**:
  - Implement full functionality for placeholder items (Last.fm, DSD/DoP, startup, monitoring)
  - Break existing audio settings that are already wired
  - Mix data sources incorrectly (use existing stores where they exist)

  **Parallelizable**: NO (depends on 4)

  **References**:
  
  **Pattern References**:
  - `ui/src/lib/views/SettingsView.svelte:92-169` - Audio settings section pattern
  - `ui/src/lib/views/SettingsView.svelte:171-230` - Theme settings pattern
  - `ui/src/lib/views/SettingsView.svelte:232-259` - Artwork providers pattern
  
  **Component References**:
  - `ui/src/lib/components/Modal.svelte` - Svelte 5 snippet patterns
  
  **State References**:
  - `ui/src/lib/state/playback.ts:outputSettings` - Output settings store
  - `ui/src/lib/state/effects.ts` - UI preferences store
  - `ui/src/lib/state/library.ts:folders` - Library folders store

  **Acceptance Criteria**:
  
  **Component Verification:**
  - [ ] All 7 category components created in `ui/src/lib/components/preferences/`
  - [ ] Each component follows `.setting` / `.setting-hint` patterns
  - [ ] Disabled placeholders show `[Coming Soon]` label and disabled controls
  - [ ] Deferred placeholders show hint "Takes effect in future update" and are enabled
  
  **Visual Verification (via Tauri MCP):**
  - [ ] GeneralPrefs shows: Start with Windows, Minimize to tray (placeholder)
  - [ ] PlayerPrefs shows: Output device, Output mode, Policy, Timing, Buffer size, Preload
  - [ ] NowPlayingPrefs shows: Double-click behavior, Queue position, Shuffle mode
  - [ ] LibraryPrefs shows: Folder list, Add folder, Scan on startup, Continuous monitoring
  - [ ] TagsPrefs shows: Backup before write, Write behavior options
  - [ ] InternetPrefs shows: iTunes/Deezer toggles, Last.fm placeholder
  - [ ] DevicesPrefs shows: DSD/DoP placeholder
  
  **Commit**: YES
  - Message: `feat(ui): implement preference category components`
  - Files: `ui/src/lib/components/preferences/*.svelte`

---

- [x] 6. Create Preferences State Management

  **What to do**:
  - Create `ui/src/lib/state/preferences.ts` store
  - Define typed interfaces for each category settings
  - Implement `loadCategorySettings(category)` async action
  - Implement `saveCategorySettings(category, settings)` async action
  - Implement `resetCategoryToDefaults(category)` async action
  - Wire to new backend commands from Task 3
  - Handle loading states and errors
  - **Serialization**: Use `parseBool()` pattern from effects.ts (expects `'on'`/`'off'`)

  **Existing Store Integration Strategy**:
  The existing `effects.ts` store manages UI effects and artwork providers using `cmd_settings_get`/`cmd_settings_set`.
  
  **Decision**: 
  - **Theme settings stay in SettingsView.svelte** (not moved to Preferences)
  - `effects.ts` continues to manage theme effects (blur, glow, border) and artwork providers
  - Preferences focuses on audiophile/playback categories only
  - `InternetPrefs.svelte` will import artwork provider toggles from `effects.ts` (iTunes/Deezer)
  - `PlayerPrefs.svelte` will use existing `outputSettings` from `playback.ts` for audio settings
  - New `preferences.ts` handles categories NOT covered by existing stores: General, Now Playing, Library, Tags, Devices
  - This avoids breaking existing functionality while adding new category support

  **Must NOT do**:
  - Break existing `outputSettings` store in playback.ts
  - Break existing `effects.ts` store (artwork providers, theme effects)
  - Duplicate stores - reuse `effects.ts` exports in InternetPrefs, `playback.ts` exports in PlayerPrefs

  **Parallelizable**: YES (after 3 is complete for API)

  **References**:
  
  **Pattern References**:
  - `ui/src/lib/state/playback.ts` - Store patterns, async actions
  - `ui/src/lib/state/effects.ts` - Settings persistence pattern with `'on'`/`'off'` serialization
  - `ui/src/lib/state/library.ts` - Event listeners pattern
  
  **Existing Store Integration**:
  - `ui/src/lib/state/effects.ts:10-18` - KEYS constant for existing settings (artwork providers, theme)
  - `ui/src/lib/state/effects.ts:49-51` - `parseBool()` function to reuse
  - `ui/src/lib/state/effects.ts:77-109` - Individual setters pattern
  - `ui/src/lib/state/playback.ts:28,115-131` - `outputSettings` store and load/save functions
  
  **API References**:
  - `ui/src/lib/api/playback.ts` - API wrapper patterns
  - `src-tauri/src/commands/library.rs` - Existing `cmd_settings_get`/`cmd_settings_set` commands

  **Acceptance Criteria**:
  
  **Code Verification:**
  - [ ] File exists: `ui/src/lib/state/preferences.ts`
  - [ ] Exports typed interfaces for General, NowPlaying, Library, Tags, Devices categories
  - [ ] Exports `loadCategorySettings`, `saveCategorySettings`, `resetCategoryToDefaults`
  - [ ] Uses `'on'`/`'off'` serialization matching `effects.ts` pattern
  
  **Integration Verification:**
  - [ ] `InternetPrefs.svelte` imports artwork providers from `effects.ts`
  - [ ] `PlayerPrefs.svelte` imports `outputSettings` from `playback.ts`
  - [ ] No duplication of existing store functionality
  
  **Build Verification:**
  - [ ] `pnpm check` → No TypeScript errors
  
  **Runtime Verification:**
  - [ ] Load preferences view → Settings loaded without errors
  - [ ] Change a setting → Persisted to backend
  - [ ] Reset category → Values return to defaults
  - [ ] Existing artwork provider toggles still work

  **Commit**: YES
  - Message: `feat(ui): add preferences state management with category support`
  - Files: `ui/src/lib/state/preferences.ts`

---

### Phase 3: Wiring & Integration

- [x] 7. Wire Settings to Actual Behavior

  **What to do**:
  - **Output Mode**: Ensure changing mode in PlayerPrefs triggers audio engine reconfiguration
  - **Buffer Size**: Wire `player.buffer_size_ms` to ring buffer initialization (may require restart)
  - **Library Scan-on-Startup**: Wire `library.scan_on_startup` to startup logic in `src-tauri/src/lib.rs`
  - **Continuous Monitoring**: 
    - This is a PLACEHOLDER feature (file watcher not implemented)
    - Control is **disabled** with `[Coming Soon]` label (no persistence toggle)
    - Do NOT persist changes in M07 (disabled control)
    - The actual `notify` crate integration is out of scope for M07
    - When implemented later, the wiring point is: after `quick_scan` in `src-tauri/src/lib.rs:133-158`, spawn a watcher thread if `library.continuous_monitoring` is `'on'`
  - Add UX for restart-required settings (text-only banner, no restart button)
  - Ensure defaults satisfy bit-perfect: Exclusive ON, DSP OFF

  **Must NOT do**:
  - Enable any DSP by default
  - Break existing audio output behavior
  - Make restart-required changes silently
  - Implement full file watcher (placeholder only for continuous monitoring)

  **Parallelizable**: NO (depends on 3, 5, 6)

  **References**:
  
  **Pattern References**:
  - `src-tauri/src/lib.rs:133-158` - Startup quick scan logic (future watcher integration point)
  - `src-tauri/src/lib.rs:651` - Bit-perfect gain mode setting
  - `crates/audio-engine/src/output.rs:157` - WASAPI exclusive mode
  
  **Settings References**:
  - `ui/src/lib/state/playback.ts:saveOutputSettings` - Current output settings flow
  - `src-tauri/src/commands/playback.rs` - Playback command handlers
  
  **Audio References**:
  - `src-tauri/src/lib.rs:1043` - Ring buffer creation (buffer size)
  
  **Continuous Monitoring (Future Integration Point)**:
  - `src-tauri/src/lib.rs:133-158` - After quick_scan, spawn watcher if enabled
  - Requires adding `notify` crate to `Cargo.toml` (NOT in M07 scope)
  - Setting persisted now, functionality implemented in future milestone

  **Scan-on-Startup Wiring (CRITICAL)**:
  - Location: `src-tauri/src/lib.rs:129-158` - Startup quick scan thread
  - **Current behavior**: Always runs `library::quick_scan()` after 500ms delay
  - **Required change**: Before calling `quick_scan`, read `library.scan_on_startup` setting:
    ```rust
    // In the spawned thread, before quick_scan:
    let conn = library::open_db(&db_path_clone).ok();
    let scan_enabled = conn.as_ref()
        .and_then(|c| library::db::get_setting(c, "library.scan_on_startup").ok())
        .flatten()
        .map(|v| v == "on")
        .unwrap_or(true); // Default: scan if setting missing
    
    if scan_enabled {
        match library::quick_scan(&db_path_clone) { ... }
    }
    ```
  - Uses `crates/library/src/db/mod.rs:229` `get_setting()` function
  - Default behavior when key missing: scan (matches migration default `'on'`)

  **Buffer Size Wiring (CRITICAL)**:
  - **TWO locations** must be updated to use `player.buffer_size_ms`:
    1. `src-tauri/src/lib.rs:469-473` - Ring buffer creation during format negotiation
    2. `src-tauri/src/lib.rs:1043-1047` - Ring buffer recreation after output reconfiguration
  - Current hardcoded value: `500` (ms)
  
  **Integration approach**: 
  - `AudioPlayback` struct needs a `buffer_size_ms: u32` field (cached value)
  - On `spawn_audio_thread()`, read setting from DB and pass to `AudioPlayback::new()`
  - `db_path` is already passed to `spawn_audio_thread()` at `src-tauri/src/lib.rs:168`
  - Read once at thread start: 
    ```rust
    let buffer_ms = {
        let conn = library::open_db(&db_path).ok();
        conn.and_then(|c| library::db::get_setting(&c, "player.buffer_size_ms").ok())
            .flatten()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(500)
    };
    ```
  - Store in `AudioPlayback` struct and use in both ring buffer creation sites
  - Since buffer size change requires recreating the ring buffer, show restart warning in UI

  **Restart UX**:
  - Show warning banner with text only: "Restart required for buffer size change to take effect. Please close and reopen the application."
  - NO "Restart Now" button (no plugin dependency)
  - Banner persists until app restart or value reverted

  **Acceptance Criteria**:
  
  **Behavior Verification (Tier A - Hardware Agnostic):**
  - [ ] Change output mode in preferences → Audio engine mode changes
  - [ ] Toggle scan-on-startup OFF → Restart app → No auto-scan occurs
  - [ ] Toggle scan-on-startup ON → Restart app → Auto-scan runs
  - [ ] Change buffer size → Warning shown about restart requirement
  
  **Bit-Perfect Defaults:**
  - [ ] Fresh install → Output mode defaults to "Exclusive"
  - [ ] Fresh install → Policy defaults to "Strict"
  - [ ] No DSP options enabled by default
  
  **Persistence (Tier A):**
  - [ ] Change output mode → Restart app → Setting persisted
  - [ ] Change library setting → Restart app → Setting persisted
  
  **Commit**: YES
  - Message: `feat: wire preferences to actual app behavior`
  - Files: `src-tauri/src/lib.rs`, `ui/src/lib/components/preferences/PlayerPrefs.svelte`, `ui/src/lib/components/preferences/LibraryPrefs.svelte`

---

### Phase 4: Vision & Documentation

- [x] 8. Generate UI Vision Artifacts & Snapshots (manual MCP capture)

  **What to do**:
  - Use Tauri MCP server to capture polished screenshots
  - Invoke frontend UI/UX engineer subagent for design review
  - Create artifacts directory: `/artifacts/ui/07-preferences-audiophile-settings/`
  - Capture screenshots:
    - `prefs-general.png` - General category selected
    - `prefs-player.png` - Player category with all audio settings visible
    - `prefs-library.png` - Library category with folder management
    - `prefs-tags.png` - Tags category with backup options
  - Create `REVIEW.md` with design observations and improvement suggestions
  - Add `data-category` attributes to PreferencesView sidebar buttons for reliable MCP selection

  **Snapshot Capture Process** (manual via MCP tools):
  1. Run `cargo tauri dev` with `SERMON_MOCK=1 SERMON_SNAPSHOT=1` environment
  2. Use Tauri MCP tools:
     ```
     tauri_driver_session({ action: 'start' })
     tauri_webview_interact({ action: 'click', selector: '[data-category="general"]' })
     tauri_webview_screenshot({ filePath: 'artifacts/ui/07-preferences-audiophile-settings/prefs-general.png' })
     // repeat for each category
     tauri_driver_session({ action: 'stop' })
     ```
  
  **Category Selectors**:
  - General: `[data-category="general"]`
  - Player: `[data-category="player"]`
  - Library: `[data-category="library"]`
  - Tags: `[data-category="tags"]`

  **Must NOT do**:
  - Ship low-quality or cropped screenshots
  - Skip the UI/UX review process
  - Create non-deterministic snapshots

  **Parallelizable**: NO (depends on 7)

  **References**:
  
  **Pattern References**:
  - `docs/adr/0001-ui-vision-loop.md` - Snapshot mode documentation, artifact path convention
  - `docs/risk-register.md:R002` - Non-deterministic screenshot mitigation
  - `ui/src/lib/state/playback.ts:136-155` - SERMON_SNAPSHOT mode handling
  - Milestone file lines 49-54 - Required artifact list
  
  **Tool References**:
  - Tauri MCP `tauri_webview_screenshot` - Screenshot capture
  - Tauri MCP `tauri_driver_session` - App connection
  - Frontend UI/UX engineer subagent - Design review

  **Acceptance Criteria**:
  
  **File Verification:**
  - [ ] Directory exists: `/artifacts/ui/07-preferences-audiophile-settings/`
  - [ ] `prefs-general.png` exists and shows General category
  - [ ] `prefs-player.png` exists and shows Player category
  - [ ] `prefs-library.png` exists and shows Library category
  - [ ] `prefs-tags.png` exists and shows Tags category
  - [ ] `REVIEW.md` exists with design observations
  
  **Quality Verification:**
  - [ ] Screenshots are full-resolution, not cropped
  - [ ] Glass styling visible in all screenshots
  - [ ] REVIEW.md contains actionable feedback
  - [ ] SERMON_SNAPSHOT=1 flag properly freezes animations/time

  **Commit**: YES
  - Message: `docs(ui): add M07 preferences UI vision artifacts`
  - Files: `artifacts/ui/07-preferences-audiophile-settings/*`, `ui/src/lib/views/PreferencesView.svelte` (add data-category attrs)

  **Evidence Note**:
  - No `.sisyphus/evidence` screenshots required for this milestone; artifacts are the source of truth.
---

- [x] 9. Implement Export Diagnostics Feature (fully wired)

  **What to do**:
  - Add "Export Diagnostics" button to PreferencesView footer
  - Implement `cmd_settings_export_diagnostics` command in backend
  - Wire button to call command, open save dialog, write JSON file

  **Implementation Flow**:
  1. User clicks "Export Diagnostics" button in Preferences footer
  2. Frontend calls `invoke('cmd_settings_export_diagnostics')`
  3. Backend generates JSON and returns it as string
  4. Frontend uses `@tauri-apps/plugin-dialog` `save()` to get file path
  5. Frontend uses `@tauri-apps/plugin-fs` `writeTextFile()` to save
  6. Show inline status: "✓ Saved to {filename}" or "✗ Export failed"

  **Required Dependencies** (add if not present):
  - `ui/package.json`: Add `"@tauri-apps/plugin-fs": "^2"` to dependencies
  - `src-tauri/Cargo.toml`: Add `tauri-plugin-fs = "2"` to dependencies  
  - `src-tauri/Cargo.toml`: Add `chrono = { version = "0.4", features = ["serde"] }` to dependencies
  - `src-tauri/src/lib.rs`: Add `.plugin(tauri_plugin_fs::init())` to Builder
  - `src-tauri/capabilities/default.json`: Add `"fs:default"` permission

  **Diagnostics JSON Schema**:
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
    "settings": {}
  }
  ```

  **Field Sources**:
  | Field | Source |
  |-------|--------|
  | `version` | Hardcoded `"0.1.0"` |
  | `timestamp` | `chrono::Utc::now().to_rfc3339()` |
  | `system.os` | `std::env::consts::OS` |
  | `system.tauri_version` | `tauri::VERSION` |
  | `audio.current_device` | `get_setting(conn, "audio.device.preference")` |
  | `audio.output_mode` | `get_setting(conn, "audio.output.mode")` |
  | `audio.wasapi_available` | Hardcoded `true` (Windows-only app) |
  | `settings` | All settings from SQLite |

  **Must NOT do**:
  - Access live engine state (use persisted settings only)
  - Block UI during export

  **Parallelizable**: NO (depends on 3)

  **References**:
  
  **Pattern References**:
  - `ui/src/lib/views/SettingsView.svelte:2` - Dialog plugin import pattern
  - `crates/library/src/db/mod.rs:229` - `get_setting()` function
  
  **Command Registration**:
  - Add to `src-tauri/src/commands/settings.rs`
  - Register in `src-tauri/src/lib.rs` invoke_handler

  **Acceptance Criteria**:
  
  **UI Verification:**
  - [ ] "Export Diagnostics" button visible in Preferences footer
  - [ ] Click opens save dialog
  - [ ] JSON file saved to selected location
  - [ ] Inline status message shown after export
  
  **Content Verification:**
  - [ ] Exported JSON contains all required fields
  - [ ] Timestamp is valid ISO 8601 format
  - [ ] Settings section contains current preference values

  **Commit**: YES
  - Message: `feat: add export diagnostics feature to preferences`
  - Files: `src-tauri/src/commands/settings.rs`, `ui/src/lib/views/PreferencesView.svelte`, `src-tauri/Cargo.toml`, `ui/package.json`

---

- [x] 10. Create Settings Documentation

  **What to do**:
  - Create `/docs/settings.md` with:
    - Complete settings schema (all keys, types, defaults)
    - Category organization
    - Migration policy documentation
    - Default values table with bit-perfect rationale
  - Document restart-required vs hot-reload settings
  - Document placeholder vs functional settings

  **Must NOT do**:
  - Document unimplemented features as complete
  - Miss any setting keys

  **Parallelizable**: NO (depends on 7)

  **References**:
  
  **Pattern References**:
  - `docs/db-schema-v1.md` - Schema documentation style (if exists)
  - `docs/tagging.md` - Feature documentation style (if exists)
  
  **Implementation References**:
  - `crates/library/migrations/0005_preferences.sql` - All settings keys
  - Task 2 - Settings key list

  **Acceptance Criteria**:
  
  **File Verification:**
  - [ ] File exists: `/docs/settings.md`
  - [ ] Contains complete settings schema table
  - [ ] Documents all 7 categories
  - [ ] Documents migration policy
  - [ ] Marks placeholders clearly
  
  **Accuracy Verification:**
  - [ ] All settings keys from migration match documentation
  - [ ] Default values match code

  **Commit**: YES
  - Message: `docs: add comprehensive settings documentation`
  - Files: `docs/settings.md`

---

- [x] 11. Update Risk Register

  **What to do**:
  - Add M07-specific risks:
    - **R021**: Settings sprawl - non-functional placeholders confusing users
    - **R022**: Users accidentally disabling bit-perfect mode
  - Add mitigations per milestone file
  - Add M07 burn-down notes section

  **Must NOT do**:
  - Remove or modify existing risks
  - Mark risks as mitigated without evidence

  **Parallelizable**: NO (final task)

  **References**:
  
  **Pattern References**:
  - `docs/risk-register.md` - Existing format and structure
  - Lines 25-32 - Milestone burn-down notes pattern
  
  **Milestone References**:
  - Lines 82-83 - M07 risks and mitigations

  **Acceptance Criteria**:
  
  **File Verification:**
  - [ ] R021 added with mitigation (placeholders clearly labeled)
  - [ ] R022 added with mitigation (explain tradeoffs inline, strict defaults)
  - [ ] M07 Burn-Down Notes section added
  
  **Format Verification:**
  - [ ] Follows existing table format
  - [ ] Status column properly set

  **Commit**: YES
  - Message: `docs(risk): add M07 risks and burn-down notes`
  - Files: `docs/risk-register.md`

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 1 | `docs(adr): add ADR 0009 settings storage strategy` | `docs/adr/0009-settings-storage.md` | File exists |
| 2 | `feat(library): add settings migration for M07 preferences` | `crates/library/migrations/0005_preferences.sql` | `cargo build` |
| 3 | `feat(tauri): add category-based settings API` | `src-tauri/src/commands/settings.rs` | `cargo build` |
| 4 | `feat(ui): add Preferences view shell with left-nav` | `ui/src/lib/views/PreferencesView.svelte` | `pnpm check` |
| 5 | `feat(ui): implement preference category components` | `ui/src/lib/components/preferences/*.svelte` | `pnpm check` |
| 6 | `feat(ui): add preferences state management` | `ui/src/lib/state/preferences.ts` | `pnpm check` |
| 7 | `feat: wire preferences to actual app behavior` | Multiple files | Manual QA |
| 8 | `docs(ui): add M07 preferences UI vision artifacts` | `artifacts/ui/07-*` | Files exist |
| 9 | `feat: add export diagnostics feature to preferences` | `src-tauri/src/commands/settings.rs`, `ui/src/lib/views/PreferencesView.svelte`, `src-tauri/Cargo.toml`, `ui/package.json` | Manual QA |
| 10 | `docs: add comprehensive settings documentation` | `docs/settings.md` | File complete |
| 11 | `docs(risk): add M07 risks and burn-down notes` | `docs/risk-register.md` | Format valid |

---

## Success Criteria

### Verification Commands
```bash
# Build verification
cargo build                     # Expected: Success
cargo test                      # Expected: All tests pass
pnpm check                      # Expected: No TypeScript errors

# Runtime verification
cargo tauri dev                 # Expected: App launches

# Snapshot verification
# Manual MCP capture to artifacts/ui/07-preferences-audiophile-settings/
```

### Tier A Verification (Hardware-Agnostic)
1. Open Preferences → Change Output Mode to "Shared"
2. Restart application
3. Open Preferences → Verify "Shared" is still selected
4. Change back to "Exclusive" → Verify persists after restart

### Final Checklist
- [x] All 11 tasks completed
- [x] Settings persist across app restart
- [x] Output mode affects playback behavior
- [x] Library scan settings affect scanning behavior
- [x] Snapshot pack produced
- [x] All documentation created
- [x] Risk register updated
- [x] All commits made with conventional format
