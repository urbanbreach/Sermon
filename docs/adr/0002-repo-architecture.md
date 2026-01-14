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

## Event Contract

| Command / Event | Payload | Description |
|-----------------|---------|-------------|
| `cmd_scan_start` | `{ path: String }` | Initiates a library scan on the given directory. |
| `evt_scan_progress` | `{ scanned: u32, total: u32 }` | Emits progress updates during scanning. |
| `cmd_audio_play` | `{ uri: String }` | Requests the audio engine to play a track. |
| `cmd_audio_pause` | `{}` | Requests the audio engine to pause playback. |
| `evt_player_state` | `{ state: "playing" \| "paused" \| "stopped" }` | Emits changes in player state. |

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
