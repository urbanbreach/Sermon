# Testing Baseline

This baseline captures what is currently measured in the repository and defines the next realistic milestones for expanding coverage without introducing unrealistic immediate gates.

## What is Measured

- **Frontend tests (Vitest)** via `pnpm run test:ui`
- **Rust workspace tests** via `cargo test --workspace` (`pnpm run test:rust`)
- **Component smoke checks** in frontend + app-level command validation
- **Desktop harness coverage** through QA automation scripts under `scripts/`

## Current Baseline Values

- **Frontend**: 11 test files, 89 tests (all passing)
- **Rust crates with test coverage**:
  - `crates/library`
  - `crates/tags`
  - `crates/audio-engine`
  - `src-tauri` (`sermon` crate)
- **Desktop harness**: 4 QA scripts
  - `scripts/qa-artwork.mjs`
  - `scripts/qa-playback.mjs`
  - `scripts/qa-library.mjs`
  - `scripts/qa-navigation.mjs`
- **CI baseline**: GitHub Actions workflow with distinct frontend and Rust jobs (`.github/workflows/ci.yml`)

## Target Next Milestones

- **Frontend**: add `@testing-library/svelte` for DOM rendering and user-flow assertions
- **Rust**: integrate coverage tooling (`cargo-tarpaulin` or `llvm-cov`) for measurable line/function coverage reports
- **Desktop**: move QA harness toward automated MCP-driven CI execution for repeatable desktop regression checks

## Coverage Gaps

- Tag editing flow end-to-end validation (safe-write + UI roundtrip)
- ASIO device switching and invalidation scenarios (hardware-dependent)
- DSD playback path (DoP/format negotiation edge paths)

## Risk Alignment

| Risk ID | Theme | Baseline alignment |
|---|---|---|
| **R004** | Tagging safe-write vs file identity | Rust crate tests and existing scanning/tagging checks provide partial regression protection; end-to-end tag editor flow remains a known gap. |
| **R006** | Network share identity instability | Library identity fallback behavior is covered in Rust test layers; CI keeps this in the standard Rust gate. |
| **R007** | Large library scan performance | Library scanning logic is covered functionally; performance baselines are tracked separately and should be expanded with load-oriented checks. |
| **R008** | Device invalidation during playback | Playback logic has baseline Rust/QA coverage; true device-failure behavior still needs hardware-backed automation scenarios. |
| **R011** | Exclusive mode device in use | Baseline validates command paths and playback behavior, but exclusive-device contention requires targeted environment tests. |
| **R029** | LLVM/toolchain complexity for ASIO | CI Rust job on Windows with LLVM setup is the current control point for detecting toolchain regressions early. |

## Scope Boundary Notes

- Gapless verification remains bounded by deterministic test data and manual/harness checks where hardware behavior is required.
- Desktop/manual checks are part of coverage observability, not an immediate merge-blocking threshold.
- Flake quarantine and retry governance follow [Testing Flake Policy](testing-flake-policy.md).
- Gapless boundary and manual verification details are captured in [Gapless Playback Testing Guide](gapless-testing.md).
