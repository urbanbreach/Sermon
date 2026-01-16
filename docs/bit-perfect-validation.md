# Bit-Perfect Validation

This document describes methods to validate bit-perfect audio playback in Sermon.

## Overview

Bit-perfect playback means audio data reaches the DAC without any modification:
- No sample rate conversion
- No bit depth conversion  
- No gain/volume changes
- No DSP processing

## Internal Validation (Automated)

### NullSink Backend Tests

Sermon includes a `NullSinkOutput` backend for automated testing of audio invariants.

#### What It Tests
- **No unintended gain changes**: Samples written match samples captured (within floating-point precision)
- **Unity gain enforcement**: In Exclusive+Strict mode, volume is locked to 1.0
- **Format preservation**: Output format matches input format

#### Limitations
- Tests pre-driver audio only (cannot verify actual DAC output)
- Does not test hardware-specific behaviors
- Does not test actual WASAPI exclusive mode initialization

#### Running Tests
```bash
cargo test -p audio-engine
```

### Diagnostics UI

The Diagnostics view (`/diagnostics` route) shows real-time audio pipeline status:
- Bit-Perfect status with reason if not bit-perfect
- Exclusive mode active status
- Conversion type (none, pad_16_to_24, shared_fallback)
- Gain mode (unity vs software)

## External Validation (Manual)

### DAC Sample Rate Indicator

Most external DACs display the current sample rate on their front panel or via LED indicators.

#### Validation Steps
1. Enable Exclusive mode with Strict policy in Settings
2. Play a 44.1 kHz track → Verify DAC shows 44100
3. Play a 96 kHz track → Verify DAC shows 96000
4. Confirm automatic switching without manual intervention

#### Expected Behavior
- DAC indicator should change within ~1 second of track change
- No intermediate sample rates (e.g., no 48000 when playing 44100)

### DTS WAV Passthrough Test

For DACs with DTS decoding capability, a DTS-encoded WAV file can verify bit-perfect transmission.

#### Method
1. Obtain a DTS-encoded WAV file (not provided - user must source)
2. Enable Exclusive mode with Strict policy
3. Play the DTS WAV file
4. If bit-perfect: DAC decodes and outputs multi-channel audio
5. If not bit-perfect: DAC outputs noise or silence

#### Limitations
- Requires DTS-capable DAC
- User must provide test file
- Not all DACs support DTS over WASAPI

### SPDIF Bit-Exactness Tools

For advanced users with SPDIF output, specialized tools can verify bit-exact transmission.

#### Reference Tools
- [spdif-bit-exactness-tools](https://github.com/jwhitham/spdif-bit-exactness-tools) - Tools for verifying SPDIF bit-exactness

## Limitations & Caveats

### What Sermon Cannot Guarantee
1. **Driver behavior**: Audio drivers may apply processing after WASAPI
2. **Hardware DSP**: Some DACs have internal processing that cannot be bypassed
3. **OS interference**: Windows audio enhancements must be disabled at the device level

### Ensuring True Bit-Perfect Playback
1. Use WASAPI Exclusive mode (default in Sermon)
2. Use Strict policy (default in Sermon)
3. Disable Windows audio enhancements for your output device
4. Verify with DAC sample rate indicator
5. Optionally verify with DTS WAV test

## References

1. [MSB Technology - Bit-Perfect Testing](https://msbtechnology.com/support/bit-perfect-testing/)
   - Industry reference for bit-perfect verification methods

2. [Matthew van Eerde - WASAPI Audio Format Negotiation](https://matthewvaneerde.wordpress.com/2017/10/17/how-to-negotiate-an-audio-format-for-a-windows-audio-session-api-wasapi-client/)
   - Technical deep-dive into WASAPI format handling

3. [SPDIF Bit-Exactness Tools](https://github.com/jwhitham/spdif-bit-exactness-tools)
   - Open source tools for verifying SPDIF bit-exactness

## See Also

- [ADR 0006: Audio Output (WASAPI)](adr/0006-audio-output-wasapi.md)
- [Risk Register](risk-register.md)
