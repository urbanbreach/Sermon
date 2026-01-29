# Backend Optimization Decisions

## Key Interview Decisions
- Formats: "Audiophile-first, more better" - maximize format support
- Audio correctness: Bit-perfect output required
- Realtime strictness: Hard ban on blocking I/O/DB/alloc/logging in RT callback
- Cache caps: artwork 256MB LRU, waveform 1GB LRU
- Per-file RAM load cap: 512MB (files above use streaming decoder)
- Build: LTO, strip, panic=abort; keep rusqlite bundled for Windows
- SQLite: Keep bundled by default for Windows reliability + FTS5
- Symphonia: Keep features = ["all"] unless compat matrix allows safe trimming
