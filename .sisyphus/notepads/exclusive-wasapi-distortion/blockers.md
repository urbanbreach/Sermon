# Blockers - Exclusive WASAPI Distortion Fix

## Status: IMPLEMENTATION COMPLETE - AWAITING HARDWARE QA

All code implementation is done. 5 commits created. 17/34 checkboxes marked.

The remaining 17 checkboxes require physical hardware testing that cannot be automated.

---

## 2026-01-17: Manual Hardware QA Required

### Blocker Type: Hardware Dependency

All remaining checkboxes (19 items) require **manual testing on iFi Zen DAC hardware**:
- Physical DAC connection required
- Audio playback listening test required
- DAC indicator verification required
- Cannot be automated or simulated

### Affected Items

**Definition of Done (4 items):**
- 44.1/16 and 96/24 FLAC play with correct pitch and no distortion
- DAC indicator matches track sample rate
- Bit-perfect indicator behavior
- MP3 strict mode playback

**Task 1 Manual Verification (4 items):**
- Manual QA: logs confirm correct values
- Play test files in exclusive mode
- Confirm logs show correct values
- Save terminal output as evidence

**Task 2 Manual Verification (4 items):**
- Manual QA: polling exclusive clean playback
- Set timing mode to Polling in Settings
- Play test files, confirm no distortion
- Confirm DAC rate indicator

**Task 3 Manual Verification (2 items):**
- Play 24-bit FLAC, confirm bit-perfect "yes"
- Play MP3 in strict mode, confirm reason

**Task 4 Manual Verification (3 items):**
- 24-bit exclusive playback clean with correct pitch
- Play 96kHz/24-bit FLAC, confirm no distortion
- Verify DAC indicator and UI diagnostics

**Final Checklist (2 items):**
- Exclusive playback clean on iFi Zen DAC
- DAC indicator matches track sample rate

### Resolution Path

1. User must perform manual testing with:
   - iFi Zen DAC Signature V2
   - 44.1 kHz / 16-bit FLAC test file
   - 96 kHz / 24-bit FLAC test file
   - MP3 test file

2. Run application: `cargo tauri dev`

3. Test procedure:
   - Settings → Audio → Timing Mode = "Polling (USB Compatible)"
   - Play each test file
   - Listen for distortion/pitch issues
   - Check DAC sample rate indicator
   - Check Diagnostics view for correct values

4. If tests pass: Mark remaining checkboxes and commit

### Implementation Status

**CODE IS COMPLETE** - All 4 implementation tasks finished:
- Task 1: Instrumentation logging ✅
- Task 2: Timing mode (event/polling) ✅
- Task 3: Bit-perfect diagnostics fix ✅
- Task 4: 24-in-32 MSB-aligned packing ✅

Verified:
- `cargo build --release` ✅
- `cargo test` - 28 tests pass ✅
- `npm run check` - 0 errors ✅
