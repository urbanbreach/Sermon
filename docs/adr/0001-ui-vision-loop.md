# ADR 0001: UI Vision Loop

## Context
We need a way to verify UI changes and prevent regressions. While Playwright is a standard tool for end-to-end testing, setting up full automated visual regression testing in Milestone 00 introduces significant complexity and maintenance overhead for a rapidly changing UI.

## Decision
**DEFER** automated Playwright visual testing for Milestone 00.
**ADOPT** a manual UI Vision Loop using snapshot flags and human review.

We will not build an automated screenshot pipeline yet. Instead, we will rely on manual capture and review, facilitated by application flags that ensure deterministic rendering.

## Snapshot Flags
To ensure screenshots are consistent (deterministic) and comparable, the application will support the following environment variables:

- `SERMON_MOCK=1`: Forces the backend to return mock data instead of real database/filesystem queries. Ensures data consistency.
- `SERMON_SNAPSHOT=1`: Freezes UI animations, sets fixed dates/times, and hides non-deterministic elements (like blinking cursors or random tips).

## Manual Capture Steps
1. Launch the application with flags:
   ```bash
   SERMON_MOCK=1 SERMON_SNAPSHOT=1 npm run dev
   ```
2. Navigate to the screen/state to be verified.
3. Manually capture a screenshot of the window.
4. Save the screenshot to `test/snapshots/<feature_name>.png`.

## Review Pack Format
When submitting UI changes, include a "Review Pack" in the PR description or a separate Markdown file:

```markdown
### UI Vision Review
| State | Before | After |
|-------|--------|-------|
| Main Library | <img src="..." width="300" /> | <img src="..." width="300" /> |
| Player Bar | <img src="..." width="300" /> | <img src="..." width="300" /> |
```
