# Testing Strategy

## Command Taxonomy (Local ↔ CI)

| Local command | CI equivalent | What it validates |
|---|---|---|
| `pnpm run check:ui` | `pnpm run check:ui` | Frontend static validation (`svelte-check` + TypeScript project checks) |
| `pnpm run test:ui` | `pnpm run test:ui` | Frontend unit/integration tests via Vitest |
| `pnpm run test:rust` | `pnpm run test:rust` | Rust workspace unit/integration tests across crates and `src-tauri` |
| `pnpm run test:all` | `pnpm run test:all` | Full local pre-merge gate (frontend checks + frontend tests + Rust tests) |
| `pnpm run ci:local` | CI job entrypoint calling `pnpm run test:all` | Local reproduction of CI quality gate |

## Parity Policy

CI commands MUST mirror local scripts.

This repo treats root npm scripts as the source of truth for verification. CI should call the same script names used by developers locally to avoid environment drift and false-green pipelines.

## Test Layers

1. **Frontend checks**: `pnpm run check:ui`
2. **Frontend tests**: `pnpm run test:ui`
3. **Rust tests**: `pnpm run test:rust`

Recommended full validation before merge: `pnpm run ci:local`.

## Coverage Baseline

See [Testing Baseline](testing-baseline.md) for current coverage metrics and next milestones.

## Flake Governance

See [Testing Flake Policy](testing-flake-policy.md) for flake definitions, quarantine protocol, and SLA requirements.
