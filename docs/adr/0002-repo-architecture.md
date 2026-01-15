# ADR 0002: Repo Architecture

## Overview
Sermon follows a modular architecture using Tauri. The codebase is split into a Rust backend (Core) and a web frontend (UI), communicating via the Tauri Inter-Process Communication (IPC) bridge.

## Architecture Map

| Module | Responsibility |
|--------|----------------|
| `src-tauri` (Backend) | Core application logic, database access, file system operations, audio engine integration, and window management. |
| `src` (Frontend) | User interface, view logic, state management, and rendering. |
| `shared` | Shared types, constants, and IPC contract definitions (if extracted). |
| `prompts` | LLM prompts, backlog, and project documentation for AI assistants. |
| `docs` | Architecture Decision Records (ADRs), risk register, and developer guides. |

## IPC Contract

### Naming Convention
- **Commands**: `cmd_<domain>_<action>` (e.g., `cmd_library_add_folder`)
- **Events**: `evt_<event_name>` (e.g., `evt_scan_progress`)

### Library Commands

#### `cmd_library_add_folder`
- **Input**: `{ path: string }`
- **Output**: `{ id: number, path: string, enabled: boolean }`
- **Errors**: Database errors

#### `cmd_library_list_folders`
- **Input**: None
- **Output**: `[{ id: number, path: string, enabled: boolean }]`

#### `cmd_library_list_tracks`
- **Input**: `{ sort_by: "title"|"artist"|"album", direction: "asc"|"desc" }`
- **Output**: `[TrackRow]` with fields: id, title, artist, album, duration_ms, sample_rate, bit_depth, path, etc.

#### `cmd_scan_start`
- **Input**: `{ path: string }`
- **Output**: `{ scan_id: number }`
- **Errors**: "scan already running", "folder not found", "folder unavailable"
- **Behavior**: Adds folder if not exists, starts background scan

### Library Events

#### `evt_scan_progress`
- **Payload**: `{ scanned: number, total: number }`
- **Rate**: Max 10/second
- **Notes**: `total` is dynamic during scan (files discovered so far)

#### `evt_scan_complete`
- **Payload**: `{ scan_id: number, scanned: number, total: number, skipped: number, errors: number, elapsed_ms: number }`
- **Notes**: Emitted once when scan finishes (success or failure)

## Glossary

| Term | Definition |
|------|------------|
| **Sermon** | The name of this music player application. |
| **Crate** | A Rust package/compilation unit. |
| **IPC** | Inter-Process Communication; the bridge between the Rust backend and the WebView frontend. |
| **Vision Loop** | The process of verifying UI changes (see ADR 0001). |

## Diagnostics Redaction Policy
To protect user privacy in logs and diagnostics:
1. **File Paths**: All absolute file paths must be redacted or generalized (e.g., `/Users/alice/Music` -> `$MUSIC_DIR`).
2. **Usernames**: OS usernames must be replaced with generic identifiers (e.g., `[USER]`).
3. **Metadata**: Specific song titles/artists may be logged if necessary for debugging, but should be minimized in release builds.
