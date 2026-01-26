# ADR 0011: ASIO Strategy

## Status

Accepted

## Context

Sermon currently supports audio output via WASAPI (Shared and Exclusive modes). To fulfill the requirements for professional audio interface support and advanced audiophile features, we need an ASIO (Audio Stream Input/Output) backend. 

ASIO is the industry standard for low-latency audio on Windows and is preferred by many high-end DAC users. Furthermore, native DSD (Direct Stream Digital) playback—beyond the DoP strategy implemented in [ADR 0010](0010-dsd-strategy.md)—is often only possible or more stable via ASIO.

### Options Considered

1. **CPAL (Cross-Platform Audio Library) ASIO feature**: Use the existing high-level abstraction.
2. **Direct `asio-sys` implementation**: Build a custom backend using the low-level FFI bindings to the Steinberg ASIO SDK.

## Decision

**ADOPT** a direct integration using the `asio-sys` crate rather than relying on the CPAL high-level wrapper.

## Rationale

While CPAL provides a convenient cross-platform abstraction, it has several limitations that conflict with Sermon's goals:

1. **Control Panel Access**: CPAL does not expose the `ASIOControlPanel()` function. Professional ASIO drivers rely on their own native control panels for buffer size adjustments, clock synchronization, and hardware-specific settings. Sermon must allow users to trigger this panel directly.
2. **Native DSD Support**: CPAL's ASIO backend currently does not support native DSD sample types (e.g., `ASIOSTDSDInt8LSB1`). While Sermon uses DoP as a primary DSD strategy, providing a path for native DSD bitstreaming is a long-term goal for the `audio-engine`.
3. **Transparency**: Direct use of `asio-sys` ensures we have full visibility into the driver's state and capabilities, which is critical for the "Audiophile Diagnostics" features planned for Sermon.

### Steinberg Licensing Update (Oct 2025)

As of October 2025, Steinberg has updated the ASIO SDK licensing to include a **GPLv3** option alongside the traditional proprietary agreement. This change significantly reduces the legal friction for open-source projects to redistribute headers or automate SDK downloads during the build process.

## Implementation

- **Build System**: ASIO support will be always compiled in on Windows (`cfg(target_os = "windows")`). It will not be gated behind a feature flag to ensure a consistent binary experience for Windows users.
- **Dependencies**: The `asio-sys` crate handles the ASIO SDK download and `bindgen` process. This requires **LLVM/Clang** to be present on the build machine.
- **Integration**: A new `AsioOutput` struct will be added to `crates/audio-engine/src/output.rs`, implementing a similar interface to the existing `WasapiOutput`.
- **DSD Support**: Initial ASIO implementation will reuse the DoP encoder from ADR 0010. Native DSD bitstreaming remains in the future scope but is facilitated by this decision.

## Consequences

### Positive
- **Professional Support**: Full compatibility with professional audio interfaces and drivers.
- **Native Experience**: Users can access driver-specific settings via the "Open Control Panel" button.
- **Future-Proof**: Clear architectural path to native DSD playback.
- **Redistribution**: GPLv3 compatibility simplifies project distribution.

### Negative
- **Build Complexity**: Developers and CI runners must have LLVM/Clang installed to compile the `asio-sys` bindings.
- **Maintenance**: We must maintain a custom ASIO backend instead of relying on CPAL's community-maintained abstraction.

## References

- [ADR 0006: Audio Output via WASAPI](0006-audio-output-wasapi.md)
- [ADR 0010: DSD Playback Strategy](0010-dsd-strategy.md)
- [Steinberg ASIO SDK](https://www.steinberg.net/developers/)
- [asio-sys crate](https://crates.io/crates/asio-sys)
