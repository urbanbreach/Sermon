# Sermon

A high-fidelity music player for Windows, built with Tauri (Rust + Svelte).

## Features

### Completed (Milestone 03)
- **Library Management**: Scan folders for audio files, track metadata extraction
- **Audio Playback**: WASAPI Shared and Exclusive modes
- **Bit-Perfect Playback**: WASAPI Exclusive mode with format negotiation (matching source bit-depth/sample-rate)
- **Supported Formats**: FLAC, WAV, MP3 (via Symphonia decoder)
- **Playback Controls**: Play/Pause, Stop, Seek, Next/Previous, Volume
- **Queue Management**: Play Now, Add to Queue
- **Device Selection**: Choose audio output device, persisted preference
- **Audio Settings**: Toggle between Shared/Exclusive mode with live diagnostics

### Planned
- Album artwork and dynamic themes
- Last.fm scrobbling
- ASIO support
- DSD playback (DoP)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Tauri App                            │
├─────────────────────────────────────────────────────────────┤
│  Frontend (Svelte)           │  Backend (Rust)              │
│  ├── Views                   │  ├── src-tauri/              │
│  │   ├── TracksView          │  │   ├── commands/           │
│  │   ├── NowPlayingView      │  │   ├── state.rs            │
│  │   └── SettingsView        │  │   └── lib.rs (audio loop) │
│  ├── State (Svelte stores)   │  │                           │
│  └── API (Tauri invoke)      │  └── crates/                 │
│                              │      ├── audio-engine/       │
│                              │      ├── library/            │
│                              │      └── tags/               │
└─────────────────────────────────────────────────────────────┘
```

### Crates

| Crate | Purpose |
|-------|---------|
| `audio-engine` | Audio decode (Symphonia), WASAPI output, playback state machine, queue |
| `library` | SQLite database, folder/track management, settings persistence |
| `tags` | Audio file metadata extraction |

### Audio Pipeline

```
┌──────────┐    ┌─────────────┐    ┌─────────────┐    ┌──────────┐
│ Decoder  │───>│ Ring Buffer │───>│ WASAPI Out  │───>│ Speakers │
│(Symphonia)│    │  (500ms)    │    │ (Shared)    │    │          │
└──────────┘    └─────────────┘    └─────────────┘    └──────────┘
     │                                    │
     │         Decode Thread              │
     └────────────────────────────────────┘
```

- **Decoder**: Symphonia decodes FLAC/WAV/MP3 to f32 samples
- **Ring Buffer**: 500ms buffer decouples decode from output timing
- **WASAPI Output**: Shared mode with AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM

## Development

### Prerequisites
- Rust (stable)
- Node.js 18+
- pnpm

### Setup
```bash
# Install frontend dependencies
pnpm install

# Run development server
cargo tauri dev
```

### Testing
```bash
# Run all Rust tests
cargo test

# Run specific crate tests
cargo test -p audio-engine
cargo test -p library
```

### Project Structure
```
Sermon/
├── src-tauri/           # Tauri backend
│   ├── src/
│   │   ├── commands/    # IPC command handlers
│   │   ├── state.rs     # Application state
│   │   └── lib.rs       # Audio thread, initialization
│   └── Cargo.toml
├── ui/                  # Svelte frontend
│   └── src/
│       ├── lib/
│       │   ├── views/   # Page components
│       │   ├── state/   # Svelte stores
│       │   └── api/     # Tauri invoke wrappers
│       └── App.svelte
├── crates/              # Rust workspace crates
│   ├── audio-engine/    # Audio playback
│   ├── library/         # Database & scanning
│   └── tags/            # Metadata extraction
├── docs/                # ADRs, risk register
│   └── adr/             # Architecture Decision Records
├── prompts/             # Milestone specifications
└── artifacts/           # UI screenshots, review packs
```

## Documentation

- [ADR 0001: UI Vision Loop](docs/adr/0001-ui-vision-loop.md)
- [ADR 0002: Repo Architecture](docs/adr/0002-repo-architecture.md)
- [ADR 0003: SQLite Database](docs/adr/0003-db-sqlite.md)
- [ADR 0004: File Identity (Windows FileID)](docs/adr/0004-file-identity-windows-fileid.md)
- [ADR 0005: Audio Decode (Symphonia)](docs/adr/0005-audio-decode-symphonia.md)
- [ADR 0006: Audio Output (WASAPI)](docs/adr/0006-audio-output-wasapi.md)
- [Risk Register](docs/risk-register.md)
- [Database Schema](docs/db-schema-v1.md)

## Milestones

| # | Milestone | Status |
|---|-----------|--------|
| 00 | Foundation | ✅ Complete |
| 01 | Library DB Scan | ✅ Complete |
| 02 | Playback (WASAPI Shared) | ✅ Complete |
| 03 | WASAPI Exclusive (Bit-Perfect) | ✅ Complete |
| 04 | Gapless Playback | 🔜 Next |
| 05 | Library Browse & Search | Planned |
| 05 | Tagging & Safe Write Editor | Planned |
| 06 | Artwork Cache & Dynamic Theme | Planned |
| 07 | Preferences & Audiophile Settings | Planned |
| 08 | DSD (DoP) | Planned |
| 09 | ASIO Backend | Planned |
| 10 | Last.fm & History | Planned |
| 11 | Windows Productization | Planned |

## License

TBD
