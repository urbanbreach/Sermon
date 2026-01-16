# Audio Issues Debug Handoff

## Project Context

**Sermon** is a high-fidelity music player for Windows built with Tauri (Rust + Svelte). The audio backend uses WASAPI via the `wasapi` crate (v0.22) with both Shared and Exclusive modes.

### Relevant Files
- `crates/audio-engine/src/output.rs` - WASAPI output implementation, ring buffer, PCM conversion
- `crates/audio-engine/src/decode.rs` - Symphonia-based audio decoder
- `src-tauri/src/lib.rs` - Audio thread, playback loop, format switching
- `src-tauri/src/state.rs` - PlaybackCommand enum, AudioState
- `docs/adr/0006-audio-output-wasapi.md` - Architecture decision record for WASAPI

---

## ACTIVE Audio Issues (UNRESOLVED)

**STATUS: These issues persist despite attempted fixes. The root cause has NOT been identified.**

### Issue 1: Audio Distortion and Speed Issues

**Symptoms (STILL OCCURRING):**
- Audio played too fast or too slow
- Crackling/popping sounds
- Distorted playback

**Attempted Fix (DID NOT RESOLVE):**
- Added ring buffer between decoder and WASAPI output (~500ms capacity)
- Producer: Decoder fills ring buffer with decoded f32 samples
- Consumer: WASAPI drains ring buffer, filling its entire available space
- Clear ring buffer on seek to avoid stale audio

**Commit:** `2f2f9b5 fix(audio): use ring buffer for proper WASAPI playback`

**Hypothesis that was tested:** WASAPI expects the application to fill the *entire* available buffer space when signaled. Writing partial buffers causes audio distortion and playback speed issues.

**Result:** Fix did not resolve the issue. Root cause remains unknown.

### Issue 2: Partial Buffer Writes

**Symptoms (STILL OCCURRING):**
- Audio distortion
- Playback speed anomalies

**Attempted Fix (DID NOT RESOLVE):**
- Changed `write_samples` to only write actual samples available instead of padding with silence

**Commit:** `8ce9672 fix(audio): correct WASAPI write_samples to only write available samples`

**Hypothesis that was tested:** Padding with silence when decoded samples were less than WASAPI's available buffer space caused distortion.

**Result:** Fix did not resolve the issue. Root cause remains unknown.

---

## Investigation Areas

The following areas should be investigated to find the actual root cause:

1. **Timing/Synchronization** - Is the audio thread timing correct? Event-driven vs polling?
2. **Sample Rate Mismatch** - Is there a mismatch between source sample rate and output sample rate?
3. **Buffer Size Calculation** - Are buffer sizes calculated correctly for the device period?
4. **Format Conversion** - Is the f32 to PCM conversion introducing artifacts?
5. **Ring Buffer Logic** - Is the ring buffer push/pop logic correct? Race conditions?
6. **WASAPI Event Handling** - Is the event handle being used correctly?
7. **Decoder Output** - Is Symphonia decoding correctly? Sample format issues?
8. **Channel Layout** - Mono/stereo conversion issues?

---

## Current Architecture

### Audio Pipeline
```
┌──────────┐    ┌─────────────┐    ┌─────────────┐    ┌──────────┐
│ Decoder  │───>│ Ring Buffer │───>│ WASAPI Out  │───>│ Speakers │
│(Symphonia)│    │  (500ms)    │    │ (Excl/Shrd) │    │          │
└──────────┘    └─────────────┘    └─────────────┘    └──────────┘
```

### Ring Buffer Implementation
- Location: `crates/audio-engine/src/output.rs:677-766`
- Type: `VecDeque<f32>` wrapped in `AudioRingBuffer`
- Capacity: ~500ms of audio
- Methods: `push()`, `pop_into()`, `clear()`, `len()`, `capacity_frames()`
- Overflow handling: Drops oldest samples with warning
- Underrun handling: Fills with silence and emits warning

### WASAPI Modes
1. **Shared Mode** (`open_with_format`)
   - Uses `StreamMode::EventsShared` with `autoconvert: true`
   - Windows handles sample rate/format conversion
   - Not bit-perfect

2. **Exclusive Mode** (`open_device_exclusive_with_format`)
   - Uses `StreamMode::EventsExclusive`
   - Format negotiation tries multiple formats (24-in-32, native 24, 32-float)
   - Bit-perfect when exact format match achieved

### PCM Conversion Functions
- `f32_to_i16_le()` - 16-bit PCM
- `f32_to_i24_in_i32_le()` - 24-bit in 32-bit container
- `f32_to_i24_native_le()` - Native 24-bit (3 bytes/sample)
- `f32_to_i32_le()` - 32-bit PCM

---

## Potential Remaining Issues (UNVERIFIED)

### 1. Sample Rate Switch Pops
**Source:** Milestone 03 risk register, ADR mentions
**Description:** When switching between tracks with different sample rates (e.g., 44.1kHz to 96kHz), there may be audible pops/clicks during the transition.
**Current Mitigation:** Optional fade toggle (default off to preserve bit-perfect)
**Location:** `src-tauri/src/lib.rs:340-348` - FadeState implementation

### 2. Ring Buffer Underrun During High Load
**Source:** Code analysis
**Description:** If decode can't keep up with playback, ring buffer underruns cause silence insertion.
**Evidence:** Warning at `output.rs:755-759`: "Ring buffer underrun: writing silence"
**Potential Impact:** Momentary dropouts during high CPU load

### 3. Device Invalidation Handling
**Source:** ADR 0006, code comments
**Description:** Device disconnection/invalidation needs graceful handling.
**Error Code:** `AUDCLNT_E_DEVICE_INVALIDATED` (0x88890004)
**Location:** `output.rs:10`, `output.rs:617-631`

### 4. Exclusive Mode Format Negotiation Edge Cases
**Source:** Code analysis
**Description:** Some DACs may not support any of the attempted format combinations.
**Current Behavior:** Falls back to shared mode in compatibility policy, fails in strict policy.
**Potential Issue:** Error messages may not be clear enough for users.

### 5. Channel Conversion
**Source:** Code at `output.rs:600-675`
**Description:** `convert_channels_interleaved_f32()` handles mono-to-stereo and stereo-to-mono conversion.
**Potential Issue:** Multi-channel (>2) audio handling is untested.

---

## Diagnostic Information Available

### Logging
- Log file: `artifacts/logs/sermon.log`
- Debug mode: Set `SERMON_DEBUG=1` environment variable
- Key log patterns:
  - "Ring buffer underrun" - Decode not keeping up
  - "Ring buffer overflow" - Decode too fast (unlikely)
  - "Opened WASAPI output" - Format/mode info
  - "Exclusive mode failed" - Fallback triggered

### UI Diagnostics
- Route: `/diagnostics` in the app
- Shows: Source format, output format, exclusive active, conversion status, bit-perfect status
- Event: `AudioDebugEvent` emitted to frontend

---

## Test Commands

```bash
# Run audio engine tests
cargo test -p audio-engine

# Run with debug logging
SERMON_DEBUG=1 cargo tauri dev

# Check for audio-related warnings in logs
grep -i "underrun\|overflow\|distort\|error" artifacts/logs/sermon.log
```

---

## References

- [wasapi-rs examples](https://github.com/HEnquist/wasapi-rs/tree/master/examples)
- [CamillaDSP WASAPI implementation](https://github.com/HEnquist/camilladsp/blob/master/src/wasapidevice.rs)
- [WASAPI format negotiation blog](https://matthewvaneerde.wordpress.com/2017/10/17/how-to-negotiate-an-audio-format-for-a-windows-audio-session-api-wasapi-client/)
- [Bit-perfect testing](https://msbtechnology.com/support/bit-perfect-testing/)

---

## Session History

### Milestone 02 (Playback Shared)
- Initial WASAPI implementation
- Discovered ring buffer requirement through debugging distortion issues
- Two fix commits applied (8ce9672, 2f2f9b5)

### Milestone 03 (WASAPI Exclusive / Bit-Perfect)
- Added exclusive mode with format negotiation
- Added PCM integer conversion (f32 to i16/i24/i32)
- Added fade toggle for format switches
- Added diagnostics UI
- Commits: f4293cb through 52907ed (8 commits)

---

## Git History (Audio-Related)

```
52907ed docs: mark milestone 03 complete, update README
5dffd91 chore: update milestone plan and orchestration state
14ec006 docs: add bit-perfect validation guide and update risk register
dc7c816 feat(ui): add audio settings state and API
f2c579c feat(ui): add audio settings and diagnostics UI
d9a1244 feat(library): add audio output settings persistence
8418deb feat(audio): add audio output settings and playback commands
f4293cb feat(audio): add WASAPI Exclusive mode with format negotiation
f0a2637 docs: update README and ADR 0006 with architecture and learnings
2f2f9b5 fix(audio): use ring buffer for proper WASAPI playback
8ce9672 fix(audio): correct WASAPI write_samples to only write available samples
3dc4e07 feat(milestone-02): implement WASAPI Shared audio playback
```

---

## Build Status

**TypeScript Check:** PASSING (0 errors, 8 accessibility warnings)

```bash
# Verified with:
cd ui && pnpm check
# Result: svelte-check found 0 errors and 8 warnings in 2 files
```

The accessibility warnings are non-blocking (missing ARIA roles on clickable divs, labels not associated with controls).

---

*Handoff created: 2026-01-17*
*Last milestone completed: 03 (WASAPI Exclusive / Bit-Perfect)*
