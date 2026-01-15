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

## Implementation Details

### Ring Buffer Architecture
A critical learning from implementation: WASAPI expects the application to fill the *entire* available buffer space when signaled. Writing partial buffers causes audio distortion and playback speed issues.

**Solution**: Use a ring buffer between the decoder and WASAPI output:

```
┌──────────┐    ┌─────────────┐    ┌─────────────┐
│ Decoder  │───>│ Ring Buffer │───>│ WASAPI Out  │
│(Symphonia)│    │  (500ms)    │    │ (Shared)    │
└──────────┘    └─────────────┘    └─────────────┘
```

- **Producer**: Decoder fills ring buffer with decoded f32 samples
- **Consumer**: WASAPI drains ring buffer, filling its entire available space
- **Buffer Size**: ~500ms capacity to handle decode timing variations

### WASAPI Shared Mode Configuration
```rust
let mode = StreamMode::EventsShared {
    autoconvert: true,  // AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM
    buffer_duration_hns: def_time,
};
```

- `autoconvert: true` enables automatic sample rate and format conversion
- We provide audio in the source file's native format (sample rate, channels)
- Windows Audio Engine handles conversion to the device's mix format

### Event-Driven Loop Pattern
Based on wasapi-rs examples and CamillaDSP reference implementation:

1. Fill ring buffer with decoded samples (keep ~50% full)
2. Query available space: `get_available_space_in_frames()`
3. Drain ring buffer to fill available space
4. Write to device: `write_to_device(frames, data, None)`
5. Repeat on timer tick (~10ms)

## Consequences
-   **System Integration**: Shared Mode allows the application to play audio simultaneously with other applications (browser, Spotify, system notifications). This is the expected behavior for a standard music player in most contexts.
-   **Audio Quality**: Shared Mode involves the Windows Audio Engine, which may resample audio to the shared format (e.g., 48kHz float). It is *not* guaranteed to be bit-perfect relative to the source file.
-   **Simplicity**: Shared Mode handles sample rate conversion and mixing automatically, reducing the complexity of the initial `AudioEngine` implementation.
-   **Future Proofing**: Using the `wasapi` crate directly exposes the necessary primitives to implement Exclusive Mode later (planned for Milestone 03) for audiophile/bit-perfect features.
-   **Device Handling**: We must handle device enumeration and initialization explicitly, selecting the system default or a user-specified endpoint.
-   **Ring Buffer Requirement**: Direct decode-to-output without buffering causes distortion. The ring buffer is essential for proper WASAPI operation.

## References
- [wasapi-rs examples](https://github.com/HEnquist/wasapi-rs/tree/master/examples)
- [CamillaDSP WASAPI implementation](https://github.com/HEnquist/camilladsp/blob/master/src/wasapidevice.rs)

