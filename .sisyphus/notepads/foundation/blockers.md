# Blockers - Foundation Milestone

## 2026-01-14

### BLOCKED: Manual Screenshots (DOCUMENTED - CANNOT BE AUTOMATED)

**Task**: Capture manual screenshots and save with exact filenames
**Status**: BLOCKED - Requires human interaction
**Reason**: Screenshots must be captured using Windows Snipping Tool while the Tauri app is running. This cannot be automated in M00 (Playwright deferred per ADR 0001).

**This is an expected blocker** - the plan explicitly states this is a manual step.

**Instructions for human to complete**:
1. Run `pnpm ui:snapshots -- --milestone 00` from repo root
2. Wait for Tauri app to launch (1440×900 window)
3. Ensure Windows display scaling is set to 100%
4. Navigate to Albums view → capture → save as `artifacts/ui/00-foundation/shell-library.png`
5. Click bottom bar to open Now Playing → capture → save as `artifacts/ui/00-foundation/shell-now-playing.png`
6. Click Settings in left nav → capture → save as `artifacts/ui/00-foundation/shell-settings.png`
7. Verify each screenshot is 1440×900 pixels
8. Mark `[ ] Manual screenshots saved with exact filenames.` as `[x]` in `.sisyphus/plans/foundation.md`

**All other implementation tasks are COMPLETE.**

---

## Resolution

This blocker is **expected and documented** in the plan. Per the plan's "Manual QA Only" section:
> "Use `pnpm ui:snapshots -- --milestone 00` to set snapshot flags, launch the app, and **manually** capture screenshots into `/artifacts/ui/00-foundation/`."

The implementation is complete. This final step awaits human action.
