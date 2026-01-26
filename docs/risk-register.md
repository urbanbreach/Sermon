# Risk Register

| ID | Risk | Owner | Mitigation | Status |
|----|------|-------|------------|--------|
| R001 | **WebView2 blur/perf issues on Windows**<br>Windows rendering can sometimes be blurry or sluggish depending on DPI scaling and hardware acceleration. | UI Lead | **Reduced effects toggle**: Implement a setting to disable translucency/blur effects if performance is poor. Ensure high-DPI awareness is enabled in Tauri config. | Open - M01: No issues observed during development |
| R002 | **Non-deterministic screenshots**<br>UI tests may flake due to animations, timestamps, or random data, making visual regression testing unreliable. | QA | **Snapshot mode flags**: Implement `SERMON_SNAPSHOT` env var to freeze animations and mock time. See ADR 0001. | Mitigated - M01: SERMON_MOCK and SERMON_SNAPSHOT flags implemented |
| R003 | **Cold start budget risk**<br>Application startup time might exceed acceptable limits (e.g., >2s) as the database grows. | Perf Lead | **Measure from day 1**: Implement startup timing logs immediately. Set a strict budget. Defer heavy initialization (like library scanning) until after first paint. | Open - M01: DB initialization is synchronous at startup; scan is async. Baseline TBD |
| R004 | **Tagging safe-write vs file identity**<br>Writing tags to files might change their inode/mtime/hash, causing the library scanner to treat them as new files and lose play history. | Backend Lead | **Re-read file identity after write**: The scanner must verify file identity (path + content hash/size) and update the DB record in-place rather than deleting and re-inserting. | Mitigated - M01: File identity uses NTFS File ID (stable across writes) with fallback strategy. See ADR 0004 |
| R005 | **Dependency version drift**<br>Rust crates or NPM packages might update, introducing breaking changes or bugs if not pinned. | DevOps | **Lock files and engine constraints**: Commit `Cargo.lock` and `package-lock.json`. Enforce strict versioning in CI. | Open - Lock files committed |
| R006 | **Network share identity instability**<br>Files on network shares may not have stable file IDs, causing duplicate detection issues. | Backend Lead | **Fallback identity strategy**: Use path+mtime+size+hash when NTFS File ID unavailable. Set folder status to warn user. | Mitigated - M01: Fallback identity implemented in identity.rs |
| R007 | **Large library scan performance**<br>Scanning 100K+ files could take excessive time or memory. | Perf Lead | **Incremental scanning**: Skip unchanged files. Batch DB inserts. Avoid audio decoding during scan. | Mitigated - M01: Incremental scan implemented. Baseline to be measured |
| R008 | **Device Invalidation**<br>WASAPI device becomes invalid during playback. | Backend Lead | **Recovery strategy**: Detect error, attempt recovery, fallback to default. | Open |
| R009 | **Playback Position Drift**<br>UI position may drift from actual playback. | UI Lead | **Throttled updates**: Event-driven updates from engine, throttled to 250ms. | Open |
| R010 | **Device Preference Missing**<br>Saved device no longer exists on startup. | Backend Lead | **Default fallback**: Fallback to system default with warning. | Open |
| R011 | **Exclusive Mode Device In Use**<br>WASAPI Exclusive mode fails when another application holds exclusive access to the audio device. | Backend Lead | **Clear error + Fallback**: Clear error message. Suggest closing other apps. One-click fallback to Compatibility mode. Auto-fallback in Compatibility policy. | Mitigated |
| R012 | **Unsupported Format in Strict Mode**<br>Track format not supported by DAC in Exclusive mode with Strict policy. | Backend Lead | **Error details + Fallback**: Clear error with format mismatch details. Offer switch to Compatibility mode. Log negotiation attempts. | Mitigated |
| R013 | **Silent Quality Degradation**<br>User unaware that audio is not bit-perfect when in Compatibility mode. | UI Lead | **UI Indicators**: Bit-perfect indicator in BottomBar. Diagnostics view details. Reason string explaining why not bit-perfect. | Mitigated |
| R014 | **Tag Write Data Loss**<br>Writing tags could corrupt or truncate audio files if interrupted mid-write. | Backend Lead | **Safe write strategy**: Temp file + atomic commit. Backup before replace. Post-commit verification. See ADR 0007. | Mitigated - M05 |
| R015 | **Unknown Tag Field Loss**<br>Editing known fields might destroy unknown/custom tags in the file. | Backend Lead | **In-place tag modification**: Use Lofty's `remove_others(false)`. Modify existing tags rather than replacing. Document per-container preservation policy. | Mitigated - M05 |
| R016 | **File Lock During Tag Write**<br>Tag write fails when file is locked by another process (media player, indexer). | Backend Lead | **Bounded retry with backoff**: 3 attempts, 75ms/200ms delays. Clear error message with guidance. | Mitigated - M05 |
| R017 | **Identity Churn After Tag Write**<br>Atomic file replacement may change NTFS File ID, causing duplicate detection. | Backend Lead | **Re-read identity post-commit**: Update DB identity fields in place while keeping track ID stable. Preflight duplication hazard check. | Mitigated - M05 |
| R018 | **Provider Rate Limits**<br>iTunes/Deezer APIs may rate-limit or block requests if fetched too frequently. | Backend Lead | **Aggressive caching**: Never re-fetch same album. Cache persists across sessions. Eviction by LRU, not expiry. | Mitigated - M06 |
| R019 | **Provider Unavailability**<br>External artwork providers may be temporarily or permanently unavailable. | Backend Lead | **Graceful fallback**: Embedded artwork preferred. UI shows placeholder if no art. Multiple providers for redundancy. | Mitigated - M06 |
| R020 | **Non-deterministic Theme Colors**<br>Dynamic theme extraction could produce different colors on different runs, breaking snapshot tests. | UI Lead | **Fixed algorithm**: Deterministic sampling grid (48×48, 3px step). Exact math (no floating-point variance). Snapshot mode uses fixture artwork. | Mitigated - M06 |
| R021 | **Settings Migration Data Loss**<br>Future settings migrations could inadvertently reset or lose user preferences. | Backend Lead | **Forward-only migrations**: Never delete existing settings. New keys use INSERT OR IGNORE. user_version PRAGMA tracks schema version. | Mitigated - M07 |
| R022 | **Buffer Size Change Without Restart**<br>User changes buffer size but doesn't restart, leading to confusion about why it's not applied. | UI Lead | **Clear UX**: Warning banner in PlayerPrefs.svelte persists until restart. Hint text explains restart requirement. | Mitigated - M07 |
| R023 | **Scan-on-Startup User Confusion**<br>User disables scan-on-startup but doesn't understand why new files aren't appearing. | UI Lead | **Documentation**: Settings documentation explains behavior. Future: Add manual scan button to Library category. | Open - M07 |
| R024 | **Export Diagnostics Privacy**<br>Diagnostics export could contain sensitive information (file paths, device names). | Backend Lead | **Minimal data**: Export only settings and system info needed for debugging. No track data, no file paths beyond device names. User controls when/where to save. | Mitigated - M07 |
| R025 | **DoP to Non-DoP DAC**<br>Enabling DoP on a DAC that doesn't support it produces static/noise instead of audio. | Backend Lead | **Opt-in + Strict mode**: DoP disabled by default. Strict mode verifies DAC support before playback. Clear error messages guide users. | Mitigated - M08 |
| R026 | **DST Compressed DFF Files**<br>DFF files with DST compression cannot be decoded without a DST decompressor. | Backend Lead | **Clear error**: Detect DST-compressed files and show specific error explaining limitation. Future: Add DST decoder dependency. | Open - M08 |
| R027 | **DSD Rate Mismatch**<br>User's DAC may not support high DoP rates (352.8 kHz for DSD128, 705.6 kHz for DSD256). | Backend Lead | **Format negotiation**: Query DAC for supported rates. Clear error when rate unsupported. Most DACs support at least DSD64. | Open - M08 |
| R028 | **ASIO driver crash/hang during playback**<br>Third-party ASIO drivers can be unstable and cause the audio thread or application to hang. | Backend Lead | **Watchdog timer**: Implement a watchdog to detect unresponsive audio threads. Provide clear error messages and guidance on driver updates. | Open |
| R029 | **Build toolchain complexity (LLVM requirement)**<br>The `asio-sys` crate requires LLVM/Clang to generate bindings, increasing barrier for contributors and CI complexity. | DevOps | **Environment documentation**: Document the LLVM dependency clearly. Ensure CI runners have required toolchains pre-installed. | Open |
| R030 | **ASIO driver in use by another application**<br>ASIO is strictly exclusive. If another app is using the driver, initialization will fail. | Backend Lead | **Device availability check**: Detect busy drivers and show clear error message. Offer fallback to WASAPI. | Open |
| R031 | **DoP over ASIO format incompatibility**<br>Some ASIO drivers require specific sample rates or bit depths to accept DoP data, causing unpredictable behavior if not met. | Backend Lead | **Strict format negotiation**: Verify driver capabilities before starting DoP playback. Implement robust error reporting. | Open |

## Milestone 01 Burn-Down Notes

- **R002 Mitigated**: Snapshot mode with `SERMON_MOCK=1` and `SERMON_SNAPSHOT=1` environment variables implemented and documented in ADR 0001.
- **R004 Mitigated**: NTFS File ID (Volume Serial + File Index) used for stable identity. Fallback to path+mtime+size+hash for non-NTFS. Documented in ADR 0004.
- **R006 Mitigated**: Fallback identity strategy implemented for network shares and non-NTFS volumes.
- **R007 Mitigated**: Scanner skips unchanged files based on identity matching. Performance baseline to be recorded after real-world testing.

## Milestone 05 Burn-Down Notes

- **R014 Mitigated**: Safe write with temp file + atomic commit implemented in `crates/library/src/safe_write.rs`. Backup enabled by default.
- **R015 Mitigated**: Lofty's `WriteOptions::remove_others(false)` preserves non-primary tags. In-place tag modification preserves unknown fields. Policy documented in `/docs/tagging.md`.
- **R016 Mitigated**: Retry logic with 3 attempts and 75ms/200ms backoff handles transient locks. UI shows "Retrying save..." status.
- **R017 Mitigated**: Post-commit identity re-read updates DB fields in place. Track ID remains stable. Preflight duplication check before commit.

## Milestone 06 Burn-Down Notes

- **R018 Mitigated**: Artwork cache persists across sessions with 256MB limit. Cache key is deterministic based on album identity + provider. No re-fetching of already-cached artwork.
- **R019 Mitigated**: Embedded artwork is preferred source. Multiple providers (iTunes, Deezer) provide redundancy. UI shows placeholder gracefully when no artwork available.
- **R020 Mitigated**: Theme extraction uses fixed 48×48 canvas, 3px sampling grid, exact integer math. Snapshot mode uses solid-color fixture artwork for reproducible theme colors.

## Milestone 07 Burn-Down Notes

- **R021 Mitigated**: Forward-only migrations with INSERT OR IGNORE pattern. user_version PRAGMA tracks schema at v5. No existing settings modified.
- **R022 Mitigated**: PlayerPrefs.svelte displays persistent warning banner when buffer size changes. Clear restart instruction provided.
- **R023 Open**: Scan-on-startup toggle works but users may not understand implications. Future milestone should add manual scan button.
- **R024 Mitigated**: Export diagnostics contains only system info, audio device, and settings keys/values. No file paths or track data included.

## Milestone 08 Burn-Down Notes

- **R025 Mitigated**: DoP disabled by default. Strict mode (enabled by default) requires DAC confirmation. UI provides clear enable/disable toggles in Preferences → Devices.
- **R026 Open**: DST-compressed DFF files not supported. Detection not yet implemented - future milestone should add DST detection and clear error message.
- **R027 Open**: High-rate DoP depends on DAC capabilities. Error messages implemented for unsupported rates. Most content is DSD64 which works with most DACs.
