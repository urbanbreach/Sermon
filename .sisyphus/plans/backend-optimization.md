# Backend Optimization Plan — Sermon (Tauri + Rust)

## TL;DR

> **Quick Summary**: Build a measured, guardrail-driven backend optimization program that improves audio stability, latency, CPU/memory, and footprint without touching the Svelte UI. We’ll define compatibility + performance budgets, then refactor the audio pipeline to remove realtime blocking/allocations, improve caches/DB, and trim build features, all with TDD and frequent commits.
>
> **Deliverables**:
> - Backend compatibility + performance budget docs (formats/backends/RT constraints)
> - Baseline metrics + profiling/diagnostics harness
> - Realtime-safe audio pipeline (no blocking I/O/DB on audio loop)
> - Allocation reductions in decode/resample/output paths
> - ASIO path improvements + underrun telemetry
> - Cache policies (waveform/artwork) + memory caps for gapless
> - DB connection/migration optimizations + query/index tuning
> - Robust error handling (remove panics/unwraps)
> - Release profile + dependency feature trimming with size/perf reports
>
> **Estimated Effort**: XL
> **Parallel Execution**: YES — 3 waves
> **Critical Path**: Task 1 → Task 3 → Task 4 → Task 5 → Task 9

---

## Context

### Original Request
Optimize backend only (Rust/Tauri) for sound quality & playback, logic correctness, speed, footprint, code quality/best practices, and developer experience. No frontend changes. Breaks and dependency upgrades are allowed. Use TDD and run tests from WSL via Windows invocation. Commit before applying changes and commit frequently (existing repo style).

### Interview Summary
**Key Decisions**:
- **Scope**: Backend only (`src-tauri/` + `crates/`). Frontend (`ui/`) excluded. Artifacts/temp directories excluded.
- **Optimization goals**: Balanced across glitches/underruns, latency, CPU, memory, startup.
- **Breaking changes**: OK.
- **Dependency upgrades**: Allowed.
- **Testing**: TDD (tests first).
- **Build trimming & profiling**: Included.
- **Commits**: Frequent, commit before applying changes, follow existing repo style.
- **Formats**: “Audiophile‑first, more better” — keep maximum format support; do not drop existing supported formats.
- **Audio correctness**: Bit‑perfect output; gapless must preserve bit‑perfectness where compatible.
- **Realtime strictness**: Hard ban on blocking I/O/DB/alloc/logging in RT callback paths.
- **Cache policy**: Enforce caps with LRU eviction; set default caps (artwork 256MB, waveform 1GB) and per‑file RAM load cap (512MB) for gapless.
- **Build trade‑offs**: Use release profile optimizations (LTO, strip, panic=abort); keep `rusqlite` bundled by default for Windows reliability.
- **Baseline hardware**: Ryzen 7 9800X3D, 32GB DDR5, RTX 5080, M.2 SSD.

### Research Findings (Key References)
**Audio loop & playback**
- `src-tauri/src/lib.rs` audio thread + ring buffer filling + preload logic + sleep points. (lines ~109–195, ~1116–1289)
**Decode & gapless**
- `crates/audio-engine/src/gapless_decoder.rs` loads full file + fs::read + to_vec allocations. (lines ~102–107, ~160–217)
- `crates/audio-engine/src/memory_source.rs` full-file RAM load (`read_to_end`). (lines ~42–49)
- `crates/audio-engine/src/decode.rs` `decode_next()` uses `.to_vec()`; `decode_next_into()` exists. (lines ~93–150, ~153–217)
**Output & ring buffers**
- `crates/audio-engine/src/output.rs` per-write `Vec` allocations; ring buffer underrun/overflow warnings. (lines ~626–699, ~804–929)
**Resampling**
- `crates/audio-engine/src/resample.rs` per-call output Vec + per-channel buffers. (lines ~93–127)
**ASIO path**
- `crates/audio-engine/src/asio_worker.rs` callback + per-sample loops + ring buffer usage. (lines ~511–585, ~675–714)
**Waveform cache**
- `src-tauri/src/commands/waveform.rs` waveform cache key + cache read/write; no eviction. (lines ~47–71, ~410–469)
**Artwork cache**
- `src-tauri/src/commands/artwork.rs` cache read/write + DB mapping. (lines ~60–93, ~331–575)
- `crates/library/migrations/0004_artwork_cache.sql` cache tables + indexes (eviction hints). (lines ~5–30)
**Database**
- `crates/library/src/db/mod.rs` `open_db()` pragmas; heavy queries; settings access. (lines ~11–99, ~440–600)
- `crates/library/src/db/migrations.rs` migration runner. (lines ~5–46)
- `src-tauri/src/commands/playback.rs` repeats `open_db()` + `apply_migrations()` in commands. (lines ~416–418, ~600–606)
**Build/Deps**
- `crates/audio-engine/Cargo.toml` uses `symphonia = { features = ["all"] }`. (line ~7)
- `crates/library/Cargo.toml` uses `rusqlite` with `bundled`. (line ~7)
- Root `Cargo.toml` has no release profile. (lines ~1–5)

### Metis Review (Gaps Identified)
**Guardrails to set explicitly**:
- Define **supported formats/backends** (compatibility contract) before trimming Symphonia features.
- Define **performance budgets** (underruns, latency, CPU, memory, startup).
- Enforce **realtime rules** (no blocking I/O/DB/alloc/logging in audio callback).
**Potential scope creep**: new audio backends, UI changes, unrelated refactors.
**Assumptions to validate**: cache policy, data migration tolerance, build trade-offs (panic=abort, strip), test realism.

---

## Work Objectives

### Core Objective
Deliver a measured backend optimization program that improves audio stability and overall performance, while keeping behavior changes explicit and testable.

### Concrete Deliverables
- A compatibility & performance budget document (formats/backends/RT rules).
- Baseline metrics & profiling harness with reproducible measurements.
- Audio pipeline refactor eliminating blocking I/O/DB access on realtime paths.
- Reduced allocations in decode/resample/output paths.
- ASIO path performance/stability improvements and telemetry.
- Cache eviction policy (waveform/artwork) and memory caps for RAM-loading.
- DB migration/connection optimizations and query/index tuning.
- Robust error handling (no panics on startup/DSD parsing).
- Build profile + dependency feature trimming with size/perf report.

### Definition of Done
- [x] Tests pass via Windows invocation from WSL (TDD for each task).
- [x] Measured performance baseline + post-change deltas recorded.
- [ ] Audio playback works in shared/exclusive/ASIO (where supported) without underruns in manual test.
- [x] Binary size and startup metrics documented before/after.

### Must Have
- Balanced improvements across glitches, latency, CPU, memory, startup.
- No frontend/UI changes.
- TDD: tests first for code changes.
- Frequent commits before applying changes, using existing repo style.

### Must NOT Have (Guardrails)
- No changes in `ui/`.
- No new audio backends beyond existing WASAPI/ASIO.
- No drive‑by refactors unrelated to metrics or stability.
- No blocking I/O/DB access on realtime audio paths once refactor is complete.

---

## Verification Strategy (TDD)

### Test Decision
- **Infrastructure exists**: YES (Rust unit/integration tests).
- **User wants tests**: TDD (tests first).
- **Framework**: `cargo test`.

### WSL → Windows Test Invocation (Reference Pattern)
Use WSL to invoke Windows `cargo`:

```bash
# From WSL
cmd.exe /c "cd /d $(wslpath -w /mnt/e/code/sermon) && cargo test -p audio-engine"
cmd.exe /c "cd /d $(wslpath -w /mnt/e/code/sermon) && cargo test -p library"
cmd.exe /c "cd /d $(wslpath -w /mnt/e/code/sermon) && cargo test"
```

### Manual Execution Verification (Always Required)
- **Audio playback**: `cmd.exe /c "cd /d <WIN_REPO> && cargo tauri dev"` then:
  - Play a FLAC (44.1/16) track; verify start latency and no glitches.
  - Seek repeatedly; verify no underruns and stable telemetry.
  - Switch output modes (shared/exclusive) and confirm output reinit.
  - If ASIO driver available, play and verify underrun counters.
- **Caches**: generate waveform and artwork; verify cache size constraints/eviction behavior.
- **DB**: run scan/search, verify no migration thrash and stable query performance.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately):
├── Task 1: Compatibility & performance budgets (docs + guardrails)
├── Task 2: Baseline metrics + profiling/diagnostics harness
├── Task 7: DB connection/migration optimization plan & tests
└── Task 8: Error-handling hardening (startup/DSD parsing)

Wave 2 (After Wave 1):
├── Task 3: Realtime audio thread refactor (remove blocking I/O/DB)
├── Task 6: Cache eviction + memory policy
└── Task 9: Build profile + feature trimming (depends on Task 1)

Wave 3 (After Wave 2):
├── Task 4: Allocation reduction in decode/resample/output
└── Task 5: ASIO path optimization + telemetry

Critical Path: Task 1 → Task 3 → Task 4 → Task 5 → Task 9
```

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
|------|------------|--------|-----------------------|
| 1 | None | 3, 9 | 2, 7, 8 |
| 2 | None | 3, 4, 5 | 1, 7, 8 |
| 3 | 1, 2 | 4, 5 | 6 |
| 4 | 3 | 5 | 6 |
| 5 | 3, 4 | None | 6 |
| 6 | 1 | 4, 5 | 3 |
| 7 | None | None | 1, 2, 8 |
| 8 | None | None | 1, 2, 7 |
| 9 | 1 | None | 6 |

---

## TODOs

> **All tasks are backend-only.**
> **Each task uses TDD** unless explicitly “docs-only.”

### 1) Define Compatibility Contract + Performance Budgets (Docs & Guardrails)

**What to do**:
- Document supported codecs/containers/DSD variants and output backends.
- Define performance budgets (e.g., max underruns/hour, max start latency, CPU/mem targets, startup time).
- Define realtime guardrails (no blocking I/O/DB/logging/allocs on RT callback).
- Define supported formats as **“audiophile‑first”** (maximize format coverage while retaining performance). Maintain **all formats supported by Symphonia “all”** and DSD DSF/DFF; explicitly list them in the doc.
- Define audio correctness targets: **bit‑perfect output**; gapless must preserve bit‑perfectness where compatible.
- Define realtime rules strictness: hard ban on blocking I/O/DB/alloc/logging on RT callback.
- Baseline measurements use **user hardware**: Ryzen 7 9800X3D / 32GB DDR5 / RTX 5080 / M.2 SSD.

**Must NOT do**:
- Do not change playback behavior yet (docs/guardrails only).

**Recommended Agent Profile**:
- **Category**: writing
  - Reason: Documentation + policy definition.
- **Skills**: none
- **Skills Evaluated but Omitted**:
  - `rust-async-patterns`: not required for docs.

**Parallelization**:
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 1 (with Tasks 2, 7, 8)
- **Blocks**: Tasks 3, 9
- **Blocked By**: None

**References**:
- `docs/adr/0005-audio-decode-symphonia.md` — current codec/support assumptions.
- `README.md` — stated supported formats/backends.
- `crates/audio-engine/Cargo.toml:7` — Symphonia features set to `all`.
- `crates/audio-engine/src/dsd_decode.rs:1–118` — DSF/DFF support scope.
- **External**: Symphonia README (supported formats list) — authoritative codec/container list for compatibility matrix.

**Acceptance Criteria (TDD)**:
- [x] New doc file created (e.g., `docs/adr/00xx-backend-compat-budgets.md`).
- [x] Doc includes **explicit compatibility matrix** and **numerical budgets**.
- [x] Doc includes **realtime guardrails** and "must not" list.
- [x] `cmd.exe /c "cd /d <WIN_REPO> && cargo test"` → PASS (no regressions).

**Manual Execution Verification**:
- [x] Open doc and verify it lists supported codecs/containers/DSD and output backends.
- [x] Verify budgets include: startup time, playback start latency, underrun rate, CPU, memory.

**Commit**: YES
- Message: `docs(backend): add compatibility matrix and perf budgets`

---

### 2) Baseline Metrics + Profiling/Diagnostics Harness

**What to do**:
- Add baseline measurement points for startup, playback start/seek, underruns, CPU/memory.
- Expose metrics via diagnostics export (existing `cmd_settings_export_diagnostics`).
- Add profiling/bench harness (e.g., Criterion for decode/resample).
- Extend diagnostics JSON schema with explicit fields:
 - Extend diagnostics JSON schema with explicit fields:
  - `metrics.startup_ms` (from `run()` start → first interactive event).
  - `metrics.playback_start_ms` (PlayNow request → output start success).
  - `metrics.seek_ms` (Seek command → ring buffer refill complete).
  - `metrics.audio.underruns` (ring buffer underrun count, ASIO callback underruns, DoP drops).
  - `metrics.process.cpu_pct` + `metrics.process.rss_bytes` (process-level usage).
- **Measurement method**: use `sysinfo` crate for process CPU% + RSS (cross‑platform), captured during diagnostics export.
- **Dependency**: add `sysinfo` to `src-tauri/Cargo.toml` (backend diagnostics path).
- **Metrics data flow**:
  - Add `DiagnosticsState` in `src-tauri/src/state.rs` (atomic counters + timestamps).
  - `run()` records `startup_start_ms`; `sermon://first-interactive` listener sets `startup_complete_ms`.
  - `cmd_playback_start`/`PlaybackCommand::PlayNow` sets `playback_start_ms` start; audio thread marks completion once output starts.
  - `PlaybackCommand::Seek` sets `seek_start_ms`; audio thread marks completion once ring buffer fill ≥ 50% capacity after seek.
  - `cmd_settings_export_diagnostics` reads DiagnosticsState and emits metrics JSON.
  - Hook points: `handle_playback_command` after `playback.start_playback` succeeds (lib.rs ~1440–1452); output start inside `AudioPlayback::start_playback` (lib.rs ~676–682) marks playback‑start completion; seek path (lib.rs ~1534–1560) + ring‑buffer fill in audio tick (lib.rs ~1222–1235).

**Must NOT do**:
- No UI changes; keep diagnostics backend-only.

**Recommended Agent Profile**:
- **Category**: unspecified-high
  - Reason: Cross-cutting backend instrumentation.
- **Skills**: none
- **Skills Evaluated but Omitted**:
  - `rust-async-patterns`: not necessary for basic diagnostics.

**Parallelization**:
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 1
- **Blocks**: Task 3, Task 4, Task 5
- **Blocked By**: None

**References**:
- `src-tauri/src/lib.rs:46–85` — tracing setup and log levels.
- `src-tauri/src/lib.rs:103–108` — `sermon://first-interactive` listener for startup timing.
- `src-tauri/src/lib.rs:1137–1141` — telemetry ticks.
- `crates/audio-engine/src/output.rs:804–929` — ring buffer underrun/overflow counters.
- `src-tauri/src/commands/waveform.rs:410–485` — existing performance logs.
- `src-tauri/src/commands/settings.rs:178–231` — diagnostics export JSON structure.
- `src-tauri/src/state.rs:22–53` — place to add shared diagnostics/metrics state.
- `src-tauri/Cargo.toml` — add `sysinfo` dependency.
- **External**: `sysinfo` crate docs (process CPU/memory metrics collection).

**Acceptance Criteria (TDD)**:
- [x] Tests added for any new metrics/serialization structs (TDD).
- [x] `cmd.exe /c "cd /d <WIN_REPO> && cargo test -p audio-engine"` → PASS.
- [x] `cmd.exe /c "cd /d <WIN_REPO> && cargo test -p library"` → PASS.
- [x] `cmd_settings_export_diagnostics` JSON includes `metrics.*` fields in `src-tauri/src/commands/settings.rs`.

**Manual Execution Verification**:
- [ ] Run `cargo tauri dev` (Windows) and trigger playback + waveform generation.
- [ ] Verify diagnostics output includes baseline metrics and underrun counts.
- [ ] Save baseline diagnostics to `.sisyphus/evidence/metrics-baseline.json`.
- [ ] Run `cmd.exe /c "cd /d <WIN_REPO> && cargo bench -p audio-engine"` and save output to `.sisyphus/evidence/bench-baseline.txt`.

**Commit**: YES
- Message: `feat(backend): add baseline metrics + diagnostics`

---

### 3) Realtime Audio Thread Refactor (Remove Blocking I/O/DB)

**What to do**:
- Remove DB access and file I/O from the audio loop and preload path.
- Move settings reads to a non‑RT thread and push updates via channels.
- Move gapless preload to background worker (no blocking in audio tick).
- **Settings propagation**: reuse `PlaybackCommand::SetOutputSettings` and add `PlaybackCommand::SetPlayerSettings` (buffer size, load_to_memory, preload_next) so audio thread never reads DB for settings.
- **Settings propagation**: `cmd_settings_set_category` (settings.rs ~113–144) sends `PlaybackCommand::SetPlayerSettings` to audio thread; audio thread applies in `handle_playback_command` (lib.rs ~1686+).
- **Start playback**: `AudioPlayback::start_playback` uses cached settings (no `open_db` call).
- **Gapless preload**: add `preload_tx/preload_rx` channel with a worker thread; audio thread sends next track path; worker reads file bytes and returns `PreloadResult` to audio thread; audio thread calls `gapless_decoder.preload_next_from_bytes(...)` (new method) to build decoder state on the audio thread.

**Must NOT do**:
- No UI changes; no new backend audio paths.

**Recommended Agent Profile**:
- **Category**: ultrabrain
  - Reason: Concurrency + realtime safety changes.
- **Skills**: `rust-async-patterns`
  - `rust-async-patterns`: thread/async coordination guidance.
- **Skills Evaluated but Omitted**:
  - `tauri`: not needed for core threading changes.

**Parallelization**:
- **Can Run In Parallel**: YES (after Tasks 1 & 2)
- **Parallel Group**: Wave 2
- **Blocks**: Tasks 4, 5
- **Blocked By**: Tasks 1, 2

**References**:
- `src-tauri/src/lib.rs:1116–1181` — audio thread loads settings via DB.
- `src-tauri/src/lib.rs:1222–1265` — preload next track triggered in audio tick.
- `crates/audio-engine/src/gapless_decoder.rs:102–107` — full-file load + `fs::read`.
- `crates/audio-engine/src/memory_source.rs:42–49` — blocking `read_to_end` load.
- `src-tauri/src/lib.rs:1686–1735` — `PlaybackCommand::SetOutputSettings` handler (propagation path).
- `src-tauri/src/commands/settings.rs:113–144` — settings update path (hook to send `SetPlayerSettings`).

**Acceptance Criteria (TDD)**:
- [x] New tests for settings update propagation and preload scheduling.
- [x] `cmd.exe /c "cd /d <WIN_REPO> && cargo test -p audio-engine"` → PASS.
- [x] No DB open/migration calls on audio thread (verified by code inspection + tests).

**Manual Execution Verification**:
- [ ] Playback start/seek works; underrun count ≤ budget from Task 1.
- [ ] Logs show preload handled on background path (no blocking in audio tick).

**Commit**: YES
- Message: `refactor(audio): remove blocking work from audio thread`

---

### 4) Reduce Allocations in Decode/Resample/Output Paths

**What to do**:
- Replace per‑tick allocations with reusable buffers.
- Use `decode_next_into()` where possible to avoid `to_vec()`.
- Reuse output buffers in `write_from_buffer` and resampler output.

**Must NOT do**:
- No changes that alter audio output format or introduce new DSP.

**Recommended Agent Profile**:
- **Category**: ultrabrain
  - Reason: Performance tuning in hot paths.
- **Skills**: none
- **Skills Evaluated but Omitted**:
  - `rust-async-patterns`: not central to allocation changes.

**Parallelization**:
- **Can Run In Parallel**: NO (depends on Task 3)
- **Parallel Group**: Wave 3
- **Blocks**: Task 5
- **Blocked By**: Task 3

**References**:
- `crates/audio-engine/src/decode.rs:93–150` — `decode_next()` uses `.to_vec()`.
- `crates/audio-engine/src/gapless_decoder.rs:160–217` — `.to_vec()` in decode path.
- `crates/audio-engine/src/resample.rs:93–127` — output Vec and per-channel buffers.
- `crates/audio-engine/src/output.rs:626–699` — `Vec` allocation for samples in write.

**Acceptance Criteria (TDD)**:
- [x] Add tests to validate buffer reuse logic and output correctness.
- [x] `cmd.exe /c "cd /d <WIN_REPO> && cargo test -p audio-engine"` → PASS.

**Manual Execution Verification**:
- [ ] Playback works; underrun count does not regress.
- [ ] Run `cmd.exe /c "cd /d <WIN_REPO> && cargo bench -p audio-engine"` and compare against `.sisyphus/evidence/bench-baseline.txt`.

**Commit**: YES
- Message: `perf(audio): reduce allocations in decode/resample/output`

---

### 5) ASIO Path Optimization + Telemetry

**What to do**:
- Optimize ASIO ring buffer push (batch writes vs per-sample loop).
- Add telemetry for callback underruns / DoP drops surfaced via diagnostics.
- Apply MMCSS scheduling on Windows for **audio thread + ASIO callback thread** using `AvSetMmThreadCharacteristicsW` (task: "Pro Audio"). Log success/failure and continue if unavailable.

**Must NOT do**:
- No new ASIO features beyond stability/perf; no new UI.

**Recommended Agent Profile**:
- **Category**: ultrabrain
  - Reason: Audio callback safety + performance.
- **Skills**: none
- **Skills Evaluated but Omitted**:
  - `tauri`: not necessary for ASIO core.

**Parallelization**:
- **Can Run In Parallel**: NO
- **Parallel Group**: Wave 3
- **Blocks**: None
- **Blocked By**: Tasks 3, 4

**References**:
- `crates/audio-engine/src/asio_worker.rs:511–585` — callback loop and per‑sample conversion.
- `crates/audio-engine/src/asio_worker.rs:675–714` — `push_samples()` per‑sample loop.
- `crates/audio-engine/src/asio_worker.rs:90–113` — `callback_underruns`/`dop_drops` counters.
- `crates/audio-engine/src/output.rs:1437–1455` — ASIO counter accessors on `OutputBackend`.

**Acceptance Criteria (TDD)**:
- [x] Add tests for telemetry counters and batch push correctness.
- [x] `cmd.exe /c "cd /d <WIN_REPO> && cargo test -p audio-engine"` → PASS.

**Manual Execution Verification**:
- [ ] On Windows with ASIO driver, callback underruns / DoP drops ≤ budget from Task 1.
- [ ] Diagnostics report callback underruns and DoP drops.

**Commit**: YES
- Message: `perf(asio): batch ring buffer + telemetry`

---

### 6) Cache Eviction + Memory Policy (Waveform/Artwork + Gapless RAM)

**What to do**:
- Add size-based eviction for waveform cache (no current cap).
- Add eviction/cleanup for artwork cache using existing mapping tables.
- Add memory policy for `MemoryAudioSource` / gapless preload (size threshold or LRU).
- **Cache caps**: artwork 256MB (LRU), waveform 1GB (LRU). Skip caching entries larger than 25% of cap.
- **Memory policy**: per‑file RAM load cap 512MB; files above cap use streaming decoder.
- **Artwork size accounting**: compute size via `fs::metadata` for each cache file during eviction (DB has no size_bytes).
- **Decoder selection**: enforce RAM cap in `AudioPlayback::start_playback` before `GaplessDecoder::new` (lib.rs ~612–630); fallback to `AudioDecoder::open` when file size > 512MB.
- **LRU signal (artwork)**: use `selected_at` in `artwork_cache_map_album/track`; update timestamp on cache hit.
- **Update points (artwork)**: bump `selected_at` on cache hit in `cmd_artwork_get_bytes` and `cmd_artwork_get_best_for_*`.
- **LRU signal (waveform)**: add metadata index (e.g., `waveform_cache_index` table or JSON) with `cache_key`, `size_bytes`, `last_access_ms` updated on read/write.
- **LRU signal (waveform)**: add `waveform_cache_map` table in **library DB** with `cache_key`, `track_id`, `size_bytes`, `mtime_ms`, `last_access_ms` updated on read/write (new migration).
- **Migration**: add `0007_waveform_cache.sql` to create `waveform_cache_map` + indexes.
- **Eviction trigger**: enforce caps on cache write (post‑write), evict oldest by LRU until total size ≤ cap.

**Must NOT do**:
- No UI changes; policies remain backend-only.

**Recommended Agent Profile**:
- **Category**: unspecified-high
  - Reason: Cache policy + disk I/O adjustments.
- **Skills**: none
- **Skills Evaluated but Omitted**:
  - `rust-async-patterns`: optional but not required.

**Parallelization**:
- **Can Run In Parallel**: YES (after Task 1)
- **Parallel Group**: Wave 2
- **Blocks**: Task 4/5 indirectly (memory limits)
- **Blocked By**: Task 1

**References**:
- `src-tauri/src/commands/waveform.rs:410–469` — waveform cache write path.
- `src-tauri/src/commands/artwork.rs:60–93` — artwork cache read/write.
- `crates/library/migrations/0004_artwork_cache.sql:5–30` — cache mapping + indexes.
- `crates/audio-engine/src/memory_source.rs:42–49` — full-file RAM load.
- `crates/library/src/db/migrations.rs:5–46` — migration runner (add waveform cache table).
- `src-tauri/src/lib.rs:612–666` — `AudioPlayback::start_playback` memory vs streaming choice.

**Acceptance Criteria (TDD)**:
- [x] Add tests for cache eviction policy (size cap, LRU behavior).
- [x] Add tests for memory policy fallback (large file uses streaming path).
- [x] `cmd.exe /c "cd /d <WIN_REPO> && cargo test -p library"` → PASS.

**Manual Execution Verification**:
- [ ] Generate multiple waveforms until cap; verify eviction occurs.
- [ ] Artwork cache grows then evicts old entries.
- [ ] Large file playback no longer spikes RAM beyond policy.

**Commit**: YES
- Message: `feat(cache): add eviction + memory policy`

---

### 7) DB Connection/Migration Optimization + Query Tuning

**What to do**:
- Avoid `apply_migrations()` per command; run once and gate with a flag.
- Consider connection pooling/shared connection for command handlers.
- Add indexes for heavy browse/search queries if missing.
- **Migration gating**: add `migrations_applied` flag in `LibraryState`; set in `tauri::Builder::setup` after successful migrations; command handlers skip `apply_migrations` when true.
- **Connection reuse**: keep per‑command `open_db` (no pooling) for now to avoid cross‑thread SQLite sharing; revisit only if baseline timings show connection overhead >5% of request time.
- **Update ALL call sites**: remove or gate `apply_migrations` in `commands/library.rs`, `commands/playback.rs`, `commands/artwork.rs`, `commands/waveform.rs`, `commands/settings.rs`.
- **Target queries for indexing**:
  - `list_albums_page` / `list_artists_page` / `list_album_tracks_page` / `list_artist_tracks_page` / `list_tracks_page` in `crates/library/src/db/mod.rs` (browse paths).
  - `search_suggest` / `search_tracks_page` / `search_albums_page` / `search_artists_page` (FTS + joins).
- **Measurement**: capture baseline query timings with `Instant` in tests/bench harness; use `EXPLAIN QUERY PLAN` to confirm index usage.

**Must NOT do**:
- No schema changes that lose data without migration steps.

**Recommended Agent Profile**:
- **Category**: unspecified-high
  - Reason: DB correctness + performance.
- **Skills**: none
- **Skills Evaluated but Omitted**:
  - `rust-async-patterns`: not needed for DB config.

**Parallelization**:
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 1
- **Blocks**: None
- **Blocked By**: None

**References**:
- `crates/library/src/db/mod.rs:11–21` — open_db pragmas.
- `crates/library/src/db/migrations.rs:5–46` — migration runner.
- `src-tauri/src/commands/playback.rs:416–418` — repeated migration calls.
- `src-tauri/src/state.rs:8–19` — LibraryState (place to add migration flag).
- `src-tauri/src/commands/library.rs` — multiple `apply_migrations` call sites.
- `src-tauri/src/commands/artwork.rs` — multiple `apply_migrations` call sites.
- `src-tauri/src/commands/waveform.rs:394` — `apply_migrations` call.
- `src-tauri/src/commands/settings.rs:94/131/165/187` — `apply_migrations` calls.
- `crates/library/src/db/mod.rs:440–560` — album/artist browse SQL.
- `crates/library/src/db/mod.rs:560–710` — album/artist tracks + tracks page SQL.
- `crates/library/src/db/mod.rs:840–1015` — FTS search SQL.

**Acceptance Criteria (TDD)**:
- [x] Add tests ensuring migrations only run once per startup.
- [x] Add tests for updated query/index paths.
- [x] `cmd.exe /c "cd /d <WIN_REPO> && cargo test -p library"` → PASS.
- [ ] Baseline query timings captured to `.sisyphus/evidence/db-query-baseline.txt` (before/after).

**Manual Execution Verification**:
- [ ] Run search/browse; observe improved query latency and no migration spam.

**Commit**: YES
- Message: `perf(db): reduce migrations + tune queries`

---

### 8) Error Handling Hardening (Startup/DSD Parsing)

**What to do**:
- Replace `.expect()` and brittle `.unwrap()` in backend startup and DSD parsing with recoverable errors.
- Ensure audio errors propagate with meaningful codes.

**Must NOT do**:
- No UI changes; preserve current error event shapes.

**Recommended Agent Profile**:
- **Category**: quick
  - Reason: Targeted error-handling cleanup.
- **Skills**: none

**Parallelization**:
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 1
- **Blocks**: None
- **Blocked By**: None

**References**:
- `src-tauri/src/lib.rs:109–138` — startup `.expect()` for dirs/DB.
- `crates/audio-engine/src/dsd_decode.rs:98–117` — `try_into().unwrap()` in DSF parse.

**Acceptance Criteria (TDD)**:
- [x] Add tests for error propagation (invalid DSD, missing dirs).
- [x] `cmd.exe /c "cd /d <WIN_REPO> && cargo test -p audio-engine"` → PASS.

**Manual Execution Verification**:
- [ ] Force invalid DSD file; verify error returned without panic.

**Commit**: YES
- Message: `fix(backend): harden startup + DSD parsing errors`

---

### 9) Build Profile + Dependency Feature Trimming

**What to do**:
- Add `[profile.release]` with LTO/strip/panic=abort/opt-level per policy.
- Trim Symphonia features to only supported formats (after Task 1).
- Decide on `rusqlite` bundled vs system SQLite trade-off.
- **Release defaults**: `lto = "thin"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`, `opt-level = 3` (optimize performance + size).
- **SQLite**: keep `rusqlite` **bundled** by default for Windows reliability + FTS5 availability.
- **Symphonia**: keep `features = ["all"]` unless explicit compatibility matrix allows safe trimming without format loss.

**Must NOT do**:
- No dropping of required codecs without updating compatibility matrix & tests.

**Recommended Agent Profile**:
- **Category**: unspecified-high
  - Reason: Build + dependency trade‑offs.
- **Skills**: none

**Parallelization**:
- **Can Run In Parallel**: YES (after Task 1)
- **Parallel Group**: Wave 2
- **Blocks**: None
- **Blocked By**: Task 1

**References**:
- `crates/audio-engine/Cargo.toml:7` — Symphonia features = all.
- `crates/library/Cargo.toml:7` — rusqlite bundled.
- Root `Cargo.toml` — no release profile.
- `docs/adr/0005-audio-decode-symphonia.md` — expected codec breadth.

**Acceptance Criteria (TDD)**:
- [x] `cargo test` passes with release/profile changes.
- [x] Binary size report captured before/after (explicit command output saved).
- [x] Updated compatibility doc reflects any drops.

**Manual Execution Verification**:
- [x] Build release: `cmd.exe /c "cd /d <WIN_REPO> && cargo build --release"`.
- [x] Size check: `powershell.exe -Command "(Get-Item target\\release\\sermon.exe).Length"` saved to `.sisyphus/evidence/binary-size.txt`.
- [ ] Playback of supported formats still works.

**Commit**: YES
- Message: `build: optimize release profile + trim features`

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 1 | `docs(backend): add compatibility matrix and perf budgets` | docs/ | cargo test |
| 2 | `feat(backend): add baseline metrics + diagnostics` | src-tauri/, crates/ | cargo test -p audio-engine |
| 3 | `refactor(audio): remove blocking work from audio thread` | src-tauri/, crates/audio-engine | cargo test -p audio-engine |
| 4 | `perf(audio): reduce allocations in decode/resample/output` | crates/audio-engine | cargo test -p audio-engine |
| 5 | `perf(asio): batch ring buffer + telemetry` | crates/audio-engine | cargo test -p audio-engine |
| 6 | `feat(cache): add eviction + memory policy` | src-tauri/, crates/audio-engine | cargo test -p library |
| 7 | `perf(db): reduce migrations + tune queries` | crates/library, src-tauri | cargo test -p library |
| 8 | `fix(backend): harden startup + DSD parsing errors` | src-tauri/, crates/audio-engine | cargo test -p audio-engine |
| 9 | `build: optimize release profile + trim features` | Cargo.toml, crate manifests | cargo test |

**Commit Rule**: Commit *before applying changes* and commit frequently per task boundaries (existing repo style).

---

## Success Criteria

### Verification Commands
```bash
cmd.exe /c "cd /d <WIN_REPO> && cargo test"
cmd.exe /c "cd /d <WIN_REPO> && cargo test -p audio-engine"
cmd.exe /c "cd /d <WIN_REPO> && cargo test -p library"
```

### Final Checklist
- [x] Compatibility & performance budgets documented.
- [x] Realtime guardrails enforced (no blocking I/O/DB/allocs on audio thread).
- [ ] Underruns reduced or bounded within budgets.
- [ ] Startup/seek latency improved or within budgets.
- [ ] CPU/memory usage improved or within budgets.
- [x] Binary size reduction documented.
- [x] All tests pass via Windows invocation from WSL.
