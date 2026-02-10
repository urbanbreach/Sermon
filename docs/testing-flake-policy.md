# Testing Flake Policy

This document defines the governance and quarantine protocol for non-deterministic (flaky) tests. Our goal is a zero-flake CI pipeline where every failure is actionable.

## Definition

A **flaky test** is defined as any test that exhibits non-deterministic behavior (alternating pass/fail results) across identical code, environment, and dependencies.

In this repository, a single observed flake in CI is treated as a bug in the test or the system under test, not an environmental anomaly.

## Retry Policy

Retries are a tool for pipeline stability, not a permanent fix for flakiness.

| Environment | Policy | Configuration |
|---|---|---|
| **Local** | **Forbidden** | Retries MUST be disabled. Local failures must be immediate and consistent. |
| **CI (PR)** | **Allowed (Limited)** | Max 2 retries allowed. Flaky runs MUST be annotated. |
| **CI (Main)** | **Allowed (Limited)** | Max 2 retries allowed. Failures trigger immediate quarantine investigation. |

*Note: All retries in CI must be logged and visible in the test report.*

## Quarantine Criteria

Tests that continue to flake despite the retry policy must be quarantined to protect pipeline velocity.

**Quarantine triggers:**
1. A test fails >2 consecutive CI runs (after retries).
2. A test is identified as non-deterministic during local development but cannot be fixed immediately.
3. A test failure is linked to an external dependency instability that cannot be resolved within 24 hours.

## Quarantine Metadata Schema

Quarantined tests MUST be annotated with specific metadata. Silently skipping or commenting out tests is strictly forbidden.

Required annotation fields:
- `@quarantine`: Explicit marker for the test runner/grepping.
- `Issue ID`: Link to the GitHub issue tracking the fix.
- `Expiry Date`: Maximum 14 days from quarantine date (YYYY-MM-DD).
- `Owner`: The individual or team responsible for the fix.

### Rust Example (Attribute)
```rust
#[test]
#[ignore = "quarantine: https://github.com/owner/repo/issues/123 | expiry: 2026-02-24 | owner: @username"]
fn test_flaky_audio_buffer() { ... }
```

### Frontend Example (Vitest)
```typescript
test.skipIf(isQuarantined)('flaky ui transition', {
  // @quarantine: https://github.com/owner/repo/issues/456
  // @expiry: 2026-02-24
  // @owner: @username
}, () => { ... });
```

## Quarantine Lifecycle

1. **Detection**: Test flakes in CI or local.
2. **Quarantine**: Developer applies metadata and opens a tracking issue.
3. **Investigation**: Owner investigates root cause (e.g., race conditions, time-dependence).
4. **Resolution**: 
   - **Fix**: Code or test updated; quarantine removed.
   - **Remove**: If the test is redundant or impossible to make deterministic, it is deleted.
   - **Extend**: A ONE-TIME extension of 7 days may be granted if the fix is in progress.

## Expiry and SLA

- **SLA**: Quarantined tests MUST be resolved within **14 days**.
- **No Perpetual Quarantines**: Tests that exceed their expiry date without a fix or valid extension will be automatically flagged for removal.
- **Merge Block**: PRs that introduce new flaky tests without quarantine metadata will be rejected.

## Owner Responsibilities

The assigned owner is responsible for:
1. Providing a clear reproduction case in the tracking issue.
2. Fixing the test/system within the 14-day SLA.
3. Communicating blockers if the SLA cannot be met.

## Examples

### ✅ Correct Quarantine (Happy Path)
> "I've quarantined `test_device_enumeration` because it flakes when multiple USB DACs are connected. Issue #88 created, owner assigned, expiry set for 2 weeks."

### ❌ Invalid Quarantine (Negative Path)
> "Skipped this test because it's annoying." (Missing metadata, no issue link, no owner, no expiry).

## Enforcement

1. **CI Grep**: CI jobs check for the `@quarantine` string and validate the presence of required metadata fields.
2. **Audit**: Weekly automated report of all quarantined tests and their SLA status.
3. **Linter**: Custom lint rules (where applicable) to prevent `test.skip` without accompanying documentation.
