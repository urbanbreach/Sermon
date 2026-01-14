# Risk Register

| ID | Risk | Owner | Mitigation | Status |
|----|------|-------|------------|--------|
| R001 | **WebView2 blur/perf issues on Windows**<br>Windows rendering can sometimes be blurry or sluggish depending on DPI scaling and hardware acceleration. | UI Lead | **Reduced effects toggle**: Implement a setting to disable translucency/blur effects if performance is poor. Ensure high-DPI awareness is enabled in Tauri config. | Open |
| R002 | **Non-deterministic screenshots**<br>UI tests may flake due to animations, timestamps, or random data, making visual regression testing unreliable. | QA | **Snapshot mode flags**: Implement `SERMON_SNAPSHOT` env var to freeze animations and mock time. See ADR 0001. | Open |
| R003 | **Cold start budget risk**<br>Application startup time might exceed acceptable limits (e.g., >2s) as the database grows. | Perf Lead | **Measure from day 1**: Implement startup timing logs immediately. Set a strict budget. Defer heavy initialization (like library scanning) until after first paint. | Open |
| R004 | **Tagging safe-write vs file identity**<br>Writing tags to files might change their inode/mtime/hash, causing the library scanner to treat them as new files and lose play history. | Backend Lead | **Re-read file identity after write**: The scanner must verify file identity (path + content hash/size) and update the DB record in-place rather than deleting and re-inserting. | Open |
| R005 | **Dependency version drift**<br>Rust crates or NPM packages might update, introducing breaking changes or bugs if not pinned. | DevOps | **Lock files and engine constraints**: Commit `Cargo.lock` and `package-lock.json`. Enforce strict versioning in CI. | Open |
