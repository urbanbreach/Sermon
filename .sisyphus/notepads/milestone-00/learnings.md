# Learnings

## Rust Tracing
- `tracing-subscriber` requires the `Layer` trait to be in scope (`use tracing_subscriber::Layer`) to use `.with_filter()`.
- `tracing_appender::non_blocking` returns a guard that must be kept alive (e.g., bound to a variable in `main` or `run` that lives for the duration of the program).
- `tracing_appender::rolling::never` is used for single file logging (no rotation).

## Tauri
- `tauri::Builder::setup` allows registering event listeners using `app.listen`.
- `app.listen` takes a closure.

## Sermon Implementation
- Logging is initialized in `src-tauri/src/lib.rs` inside `init_tracing` called by `run`.
- Logs are stored in `artifacts/logs/sermon.log` relative to the CWD (repo root).
- `SERMON_DEBUG` env var controls log level (INFO vs DEBUG).
