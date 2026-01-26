# Learnings

## ADR 0011: ASIO Strategy
- Decided on direct asio-sys over CPAL to enable native DSD and control panel access.
- Steinberg's Oct 2025 GPLv3 license update is a key enabler for open-source redistribution.
- LLVM/Clang dependency is accepted as a trade-off for full ASIO control.

## Task 2: Add asio-sys dependency
- Added `asio-sys` (v0.2.5) as a Windows-only dependency in `audio-engine`.
- Used `[target.'cfg(windows)'.dependencies]` to isolate the dependency, ensuring cross-platform build stability for non-Windows environments.

## Task 3: ASIO driver enumeration
- Created `asio_device.rs` with `AsioDriverInfo` struct and `list_asio_drivers()` function.
- API pattern: `Asio::new()` then `asio.driver_names()` returns `Vec<String>` of driver names.
- Module gated with `#[cfg(windows)]` in lib.rs - won't compile on non-Windows.
- Enumeration only reads registry, doesn't initialize/load drivers (cheap operation).
- Returns empty vec if no ASIO drivers installed - no panic.

## Task 5: DoP pipeline integration with AsioOutput
- AsioOutput uses f32 ring buffer, so DoP u32 samples must be converted.
- Conversion formula: `(dop_sample << 8) as i32` shifts 24-bit DoP to top bits, then normalize by dividing by 2^31.
- This preserves DoP marker bytes (0x05/0xFA) through the f32 representation.
- Reverse conversion: `(f32 * 2^31) as i32` then `>> 8` recovers original 24-bit DoP value.
- Test verifies round-trip marker and DSD byte preservation without ASIO driver dependency.
