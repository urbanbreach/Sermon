# Learnings - Exclusive WASAPI Distortion Fix

## 2026-01-17 Task 1: Instrumentation

### Discovered Patterns
- WasapiOutput struct stores both `bit_depth` (container size) and `valid_bits` (actual precision)
- AudioOutput trait is implemented by WasapiOutput, NullSinkOutput, and OutputBackend
- Logging uses `tracing` crate with structured fields (info!, trace!, warn!)
- UI types in `ui/src/lib/types/playback.ts` must mirror Rust structs in `src-tauri/src/commands/playback.rs`

### Successful Approaches
- Added `valid_bits()` method to trait and all implementations
- Used Option<u16> for valid_bits in AudioFormatData since shared mode may not have this
- Svelte conditional rendering for "X-bit (in Y-bit container)" display

### Technical Details
- WaveFormat methods available: get_blockalign(), get_avgbytespersec(), get_bitspersample(), get_validbitspersample()
- Exclusive mode uses EventsExclusive with period_hns parameter
- Ring buffer has available_frames() method for fill stats

## 2026-01-17 Task 2: Timing Mode

### Implementation
- Added `timing_mode` field to WasapiOutput struct
- StreamMode selection: PollingExclusive when timing="polling", EventsExclusive when timing="event"
- Write loop only waits for event handle when timing_mode == "event"
- Default timing mode is "polling" for USB DAC compatibility

### Settings Flow
- DB: `audio.output.timing` stored via set_setting/get_setting
- Rust: AudioOutputSettings.timing field
- UI: SettingsView dropdown with "Polling (USB Compatible)" and "Event-Driven" options

## 2026-01-17 Task 3: Bit-Perfect Diagnostics

### Fix Applied
- Changed `determine_bit_perfect` to compare track bit_depth to output.valid_bits() instead of output.bit_depth()
- This correctly handles 24-in-32 container format (valid_bits=24, bit_depth=32)
- Removed strict mode blocking for unknown bit depth - MP3 can now play, just marked as not bit-perfect

## 2026-01-17 Task 4: PCM Packing Alignment

### Critical Fix
- WAVEFORMATEXTENSIBLE spec requires valid bits to be LEFT-ALIGNED (MSB-aligned) with zero-padded LSBs
- Previous implementation was LSB-aligned (right-justified) - INCORRECT
- Fixed `f32_to_i24_in_i32_le` to shift 24-bit value left by 8 bits: `let i32_val = i24_val << 8;`
- Bit layout: [31:8] = 24-bit audio sample, [7:0] = zero padding
- This fix should resolve the audio distortion on USB DACs

## 2026-01-17 Bug Fix: PollingExclusive Buffer Size

### Issue
- Error `0x88890011` (AUDCLNT_E_BUFFER_SIZE_NOT_ALIGNED) when using PollingExclusive mode
- Playback failed immediately with "exclusive_unavailable" error

### Root Cause
- For `PollingExclusive`, `buffer_duration_hns` must be LARGER than `period_hns`
- Original code set both to the same value (desired_period)
- wasapi-rs examples use 16x period for buffer duration

### Fix Applied
- Changed `buffer_duration_hns` from `desired_period` to `4 * desired_period`
- This gives enough buffer time to refill before underrun

### Reference
- wasapi-rs example: `examples/playnoise_exclusive_poll.rs:58`
- Uses: `buffer_duration_hns: 16 * desired_period`
