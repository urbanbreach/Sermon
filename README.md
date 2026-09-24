# Sermon

A desktop music player for local audio files on Windows, built with Rust, Tauri 2, Svelte 5, TypeScript, and SQLite.

**Archived:** Sermon is a personal project and is no longer actively developed. The source is public for reference. No installers, release binaries, or packages were published, and Windows productization was not completed.

## Features

- Local music library with folder scanning, metadata extraction, SQLite search, and album, artist, and track views.
- Audio playback through WASAPI shared and exclusive modes, plus an ASIO backend with driver selection and control-panel access.
- FLAC, WAV, and MP3 decoding through Symphonia, with playback controls, seeking, volume, and queue management.
- DSD playback from DSF and uncompressed DFF files using DSD over PCM (DoP) on compatible output hardware.
- Tag editing with temporary-file writes, optional backups, and atomic replacement of the original file.
- Album artwork discovery, caching, selection, and embedding, with artwork backgrounds and configurable appearance effects.
- Persistent preferences for library folders, playback, output devices, tagging, and appearance, plus live audio diagnostics.
- A waveform seek bar and lyrics lookup with local caching.

WASAPI exclusive mode supports matching the source format when the device supports it. Bit-perfect playback depends on the selected output settings and hardware. DoP requires a compatible DAC and driver; DSD-to-PCM fallback and DST-compressed DFF decoding are not implemented. Some preference controls, such as preloading the next track, remain placeholders.

## Milestones

This table records the implementation present when development stopped. "Implemented" means the core feature exists in the source; it does not imply a packaged release or complete validation across audio devices. These are historical milestones, not an active roadmap.

| # | Milestone | Status | What is present |
|---|-----------|--------|-----------------|
| 00 | Foundation | Implemented | Rust workspace, Tauri shell, and Svelte frontend. |
| 01 | Library database and scanning | Implemented | SQLite migrations, folder scanning, metadata extraction, and file identity tracking. |
| 02 | WASAPI shared playback | Implemented | Audio decoding, shared-mode output, playback controls, and queue management. |
| 03 | WASAPI exclusive playback | Implemented | Source-format negotiation, strict and compatibility policies, and diagnostics. |
| 04 | Library browsing and search | Implemented | Album, artist, and track views, search, and paginated library queries. |
| 05 | Tagging and safe-write editor | Implemented | Tag editor connected to metadata writes, temporary files, backups, and atomic replacement. |
| 06 | Artwork cache and dynamic theme | Implemented | Artwork lookup and caching, artwork backgrounds, and configurable visual effects. |
| 07 | Preferences and audio settings | Implemented | Persisted settings and output controls; some individual options remain placeholders. |
| 08 | DSD over PCM (DoP) | Implemented | DSF/DFF decoding and DoP output through compatible WASAPI exclusive or ASIO devices. |
| 09 | ASIO backend | Implemented | Driver enumeration, callback-based output, format handling, and control-panel access. |
| 10 | Last.fm and listening history | Not implemented | A Last.fm setting exists, but there is no authentication, scrobbling, or listening-history implementation. |
| 11 | Windows productization | Not completed | Tauri bundle configuration exists, but no installers, release binaries, or packages were published. |

## Architecture

The Svelte frontend calls Tauri commands for library access, playback, settings, artwork, lyrics, and waveform data. The Rust workspace separates the desktop integration from audio processing, library storage, and metadata handling.

| Component | Responsibility |
|-----------|----------------|
| [`ui/`](ui/) | Svelte and TypeScript interface, views, state, and Tauri API calls. |
| [`src-tauri/`](src-tauri/) | Desktop application, command handlers, playback coordination, and artwork, lyrics, and waveform services. |
| [`crates/audio-engine/`](crates/audio-engine/) | Symphonia decoding, DSD decoding and DoP packing, WASAPI and ASIO output, resampling, and playback queues. |
| [`crates/library/`](crates/library/) | SQLite database and migrations, folder scanning, settings, and safe tag-write coordination. |
| [`crates/tags/`](crates/tags/) | Audio metadata reading and writing, including DSD metadata support. |

PCM audio is decoded through Symphonia and sent to the selected WASAPI or ASIO output. DSD follows a separate decoding and DoP-packing path. Output format negotiation and conversion depend on the selected device and playback policy.

## Development

The full desktop player targets Windows. These instructions are for working with the archived source, not installing a released application.

### Prerequisites

- Windows with the native build prerequisites for Tauri 2 installed.
- Rust stable; the application declares Rust 1.85 as its minimum version.
- Node.js 20+ and pnpm 9+, as declared in `package.json`; the installed Node version must also satisfy the frontend dependencies.
- LLVM/Clang available for the ASIO bindings. The Windows CI workflow uses LLVM 18 and sets `LIBCLANG_PATH` to its `bin` directory.
- A compatible ASIO driver or DoP-capable device to use those output modes.

### Run from source

From the repository root:

```bash
pnpm install
pnpm exec tauri dev
```

The Tauri command starts the frontend development server before launching the desktop app. To run only the Vite frontend server:

```bash
pnpm dev
```

The frontend alone does not provide the native playback and library services.

### Checks

```bash
# Svelte and TypeScript checks
pnpm run check:ui

# Frontend tests
pnpm run test:ui

# Rust workspace tests, including the Windows audio backend
pnpm run test:rust

# All of the above
pnpm run ci:local
```

The [CI workflow](.github/workflows/ci.yml) runs frontend checks and tests on Ubuntu, then Rust workspace tests on Windows with LLVM. Run the full native checks on Windows.

## License

TBD.
