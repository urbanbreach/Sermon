# ADR 0006: Audio Output via WASAPI

## Context
The application runs on Windows and needs a low-latency, reliable method to output audio to the user's speakers or headphones.

Options include:
1.  **CPAL (Cross-Platform Audio Library)**: A high-level Rust wrapper. While good for cross-platform, it can sometimes hide platform-specific details we might need to control (like specific WASAPI modes or event loop timing).
2.  **WASAPI (Windows Audio Session API)**: The native low-level API for audio on Windows. It offers two modes:
    -   **Shared Mode**: Audio is mixed by the Windows Audio Engine.
    -   **Exclusive Mode**: Application takes direct control of the audio device (bit-perfect).

## Decision
**ADOPT** WASAPI directly via the `wasapi` crate.
**USE** Shared Mode for Milestone 02.

We will implement the audio backend using the `wasapi` crate. For the current milestone (M02), we will exclusively use **Shared Mode**.

## Consequences
-   **System Integration**: Shared Mode allows the application to play audio simultaneously with other applications (browser, Spotify, system notifications). This is the expected behavior for a standard music player in most contexts.
-   **Audio Quality**: Shared Mode involves the Windows Audio Engine, which may resample audio to the shared format (e.g., 48kHz float). It is *not* guaranteed to be bit-perfect relative to the source file.
-   **Simplicity**: Shared Mode handles sample rate conversion and mixing automatically, reducing the complexity of the initial `AudioEngine` implementation.
-   **Future Proofing**: Using the `wasapi` crate directly exposes the necessary primitives to implement Exclusive Mode later (planned for Milestone 03) for audiophile/bit-perfect features.
-   **Device Handling**: We must handle device enumeration and initialization explicitly, selecting the system default or a user-specified endpoint.
