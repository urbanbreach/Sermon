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
