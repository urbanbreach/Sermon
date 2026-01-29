# Backend Optimization Issues

## Known Issues from Research
- `open_db()` + `apply_migrations()` called repeatedly in commands (lib.rs, playback.rs)
- `fs::read` + `to_vec()` allocations in gapless decoder (blocking on audio thread)
- No waveform cache eviction (unbounded growth)
- No size-based eviction for artwork cache
- Per-write Vec allocations in output.rs
- Per-sample loops in ASIO callback (could batch)
- `.expect()` and `.unwrap()` in startup and DSD parsing (panic risk)
- No release profile in root Cargo.toml

## 2026-01-29 Remaining Manual Verification Items

The following items require running the application interactively and cannot be automated:

1. **Playback verification** - Run `cargo tauri dev` and test playback in shared/exclusive/ASIO modes
2. **Diagnostics baseline** - Export diagnostics JSON and save to `.sisyphus/evidence/metrics-baseline.json`
3. **Benchmark baseline** - Run `cargo bench -p audio-engine` and save to `.sisyphus/evidence/bench-baseline.txt`
4. **ASIO underrun testing** - Requires ASIO driver hardware
5. **Cache eviction testing** - Generate large waveforms to trigger eviction
6. **DSD error testing** - Test invalid DSD file handling

These items are blocked pending interactive application testing.

## 2026-01-29 Benchmark Infrastructure

**FINDING**: No Criterion benchmarks are configured in `audio-engine` crate.
- `cargo bench -p audio-engine` runs but all tests are ignored (no actual benchmarks)
- The plan mentioned adding a profiling/bench harness, but this was not implemented
- Benchmark baseline capture is blocked until benchmarks are added

**RECOMMENDATION**: Add Criterion benchmarks for:
- `decode_next()` / `decode_next_into()` throughput
- Resampler throughput  
- Ring buffer fill/drain rates
