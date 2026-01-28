# Gapless Playback Test Fixtures

Test fixtures for gapless playback are **programmatically generated**, not stored as binary blobs.

## Why Programmatic Generation?

1. **Reproducibility**: Tests generate fixtures with known, exact sample counts
2. **No binary blobs**: Keeps repository clean, avoids large file storage
3. **Flexibility**: Can generate fixtures with specific characteristics (sample rates, durations, encoder delays)
4. **Verification**: Generated fixtures have known properties that can be mathematically verified

## Fixture Types

### Track Pair Fixtures

Generated fixtures simulate consecutive tracks for testing gapless transitions:

- `track_a`: First track with known sample count and optional padding
- `track_b`: Second track with known encoder delay to trim

### Encoder Delay Fixtures

Fixtures for testing encoder delay handling:

- MP3-style: Simulated LAME encoder delay (typically 576 samples)
- AAC-style: Simulated iTunSMPB encoder delay

## Generation

Fixtures are generated at test runtime using `MemoryAudioSource` (to be implemented).

Example:
```rust
let track_a = MemoryAudioSource::sine_wave(44100, 1.0, 440.0); // 1 second, 440Hz
let track_b = MemoryAudioSource::sine_wave(44100, 1.0, 880.0); // 1 second, 880Hz
```

## Contract Requirements

All fixtures must satisfy the `GaplessContract`:
- Zero sample gap between tracks
- Zero sample overlap
- Exact sample count at transitions
