# Decisions

## Audio Stack
- **Decoding**: Selected `symphonia` for pure Rust dependency-free decoding of FLAC/MP3/WAV.
- **Output**: Selected `wasapi` crate (Shared Mode) for Windows audio output. Exclusive mode deferred to M03.
