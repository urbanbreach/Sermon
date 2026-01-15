# Learnings

## Dependency Versions
- `wasapi` crate version "2" does not exist. Used "0.22" instead which is the latest as of Jan 2026.
- `thiserror` version "2" exists and works.

## Workspace Management
- When excluding a crate from workspace via `exclude = ["crates/audio"]`, it is no longer checked by default workspace commands.

## Database
- SQLite `user_version` PRAGMA is effective for simple linear migrations.
- `rusqlite`'s `execute_batch` handles multiple SQL statements (including PRAGMA updates) in a single call, which simplifies migration scripts.
- Testing migrations with `tempfile` ensures clean state and avoids side effects between tests.

## Audio Engine State Accounting
- Store `last_play_start: Option<Instant>` in the session; on `update_time` add elapsed ms to both `played_ms` and `position_ms` only when `PlaybackState::Playing`.
- Use an internal `update_time_at(now: Instant)` helper for deterministic unit tests without `sleep`.
- Implement `previous` as a pure queue decision (`RestartCurrent` vs `MoveToPrevious`) with the 3s threshold driven by `position_ms`.

## Symphonia Decode
- `FormatReader::default_track()` borrows; use `.cloned()` if you need to store `FormatReader` and track fields together.
- Convert decoded audio to interleaved `f32` via `SampleBuffer::<f32>::copy_interleaved_ref(decoded)`.
- After `FormatReader::seek(...)`, call `Decoder::reset()` before decoding again.

## WASAPI Shared Output
- Prefer `StreamMode::EventsShared` + `AudioClient::set_get_eventhandle()` for event-driven render timing.
- `AudioClient::get_mixformat()` returns the device mix format (safe default for shared mode).
- Device invalidation is typically surfaced as `WasapiError::Windows` with HRESULT `AUDCLNT_E_DEVICE_INVALIDATED` (`0x88890004`).

## Buffering
- A bounded `VecDeque<f32>` works as a simple ring buffer for ~200–500ms between decode and render; on underrun, fill remaining frames with zeros.

## Tauri Playback Wiring
- Use a dedicated audio thread with a `crossbeam_channel` command receiver plus a `tick(Duration::from_millis(250))` channel to throttle `evt_playback_position` emission.
- Validate `track_id` in the command handler via `library::get_track_by_id` before enqueueing playback commands; the audio thread can re-resolve to build `audio_engine::TrackInfo` for `EngineState`.

## UI / State Management
- **Shared State**: Used Svelte stores in `state/playback.ts` to manage playback state globally. This allows `BottomBar`, `NowPlayingView`, and `TracksView` to stay in sync without prop drilling.
- **Event Listeners**: Centralized `initPlaybackListeners` in `state/playback.ts` keeps the `App.svelte` clean and ensures all listeners are set up once.
- **Derived Stores**: Used `derived` stores for `progress` and `isPlaying` to simplify logic in components.
- **Glassmorphism**: Continued using `--glass-bg`, `--glass-blur` variables to maintain the visual consistency.
- **Error Handling**: Placed the "Switch to Default Device" error banner in `BottomBar` as it's the most visible place for playback issues.
- **Accessibility**: Svelte check reports accessibility warnings for click events on divs. Used `svelte-ignore` or proper roles where necessary.
