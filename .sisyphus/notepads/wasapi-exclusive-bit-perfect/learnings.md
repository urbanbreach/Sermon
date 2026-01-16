
## Task 3: PCM Conversion and NullSink

- **Pattern**: Used `OutputBackend` enum to wrap `WasapiOutput` and `NullSinkOutput` instead of trait object. This simplifies ownership and `Send` bound management, as `WasapiOutput` contains raw pointers (not Send) but is used within a single thread. Note: `AudioOutput` trait does not require `Send` if used within the same thread.
- **Testing**: Added `NullSinkOutput` to capture samples for verification. Added unit tests for PCM conversion logic (f32 to i16/i24/i32).
- **Gotcha**: `WasapiOutput` is not `Send` (due to COM pointers). When implementing traits for it, avoid `Send` bound if the object is confined to a single thread (like the audio thread).
- **Borrow Checker**: Calculating effective volume before borrowing `output` mutably was necessary to satisfy borrow checker in `process_audio`.

## Task 5: UI Settings & Diagnostics

- **Svelte & TypeScript Interaction**: The LSP can be stale; `pnpm check` is the authority. Adding types to union definitions might take time to propagate in IDE features.
- **UI Components**: 
    - `DiagnosticsView`: Visualizes audio pipeline (Source -> Output) with conversion warnings.
    - `SettingsView`: Added Exclusive Mode controls, wired to backend API.
    - `BottomBar`: Implemented "Unity" volume mode (disabled slider) when strictly bit-perfect.
- **State**: `audioDebug` store effectively drives real-time UI updates for bit-perfect status without polling.

## Task 8: Risk Register

- **Process**: Risk register format is Markdown table. Added risks R011, R012, R013 for WASAPI Exclusive mode covering device-in-use, format mismatch, and silent degradation.
