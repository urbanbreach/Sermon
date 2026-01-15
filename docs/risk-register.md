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

## Milestone 01 Burn-Down Notes

- **R002 Mitigated**: Snapshot mode with `SERMON_MOCK=1` and `SERMON_SNAPSHOT=1` environment variables implemented and documented in ADR 0001.
- **R004 Mitigated**: NTFS File ID (Volume Serial + File Index) used for stable identity. Fallback to path+mtime+size+hash for non-NTFS. Documented in ADR 0004.
- **R006 Mitigated**: Fallback identity strategy implemented for network shares and non-NTFS volumes.
- **R007 Mitigated**: Scanner skips unchanged files based on identity matching. Performance baseline to be recorded after real-world testing.
