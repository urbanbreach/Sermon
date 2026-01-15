# ADR 0005: Audio Decode with Symphonia

## Context
The application requires the ability to play back local audio files in various formats, primarily FLAC, WAV, and MP3. We need a decoding solution that is robust, performant, and easy to integrate into the Rust codebase. 

Options typically include:
1.  **FFmpeg bindings**: Comprehensive support but introduces complex build dependencies, requires external DLLs or static linking of large libraries, and complicates distribution.
2.  **Platform-native APIs**: Windows Media Foundation is an option but restricts cross-platform potential and API ergonomics can be challenging.
3.  **Pure Rust libraries**: `symphonia` is a modern, pure Rust audio decoding library.

## Decision
**ADOPT** `symphonia` for audio decoding.

We will use the `symphonia` crate to handle probing, container demuxing, and audio decoding.

## Consequences
-   **Broad Support**: Symphonia supports most common formats (FLAC, MP3, WAV, OGG/Vorbis, AAC, etc.) and containers.
-   **Zero External Dependencies**: Being pure Rust, it compiles directly into the binary without needing C libraries or external FFmpeg installations. This simplifies CI/CD and user installation.
-   **Safety**: Benefits from Rust's memory safety guarantees, unlike C-based decoders which can be prone to vulnerabilities.
-   **Performance**: Comparable to reference C implementations for most codecs.
-   **Integration**: Provides a consistent API for different codecs, simplifying the `AudioEngine` implementation.
