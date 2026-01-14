ultrawork
Generate the plan.

# Milestone 10 - lastfm-history

## Goal
- Add **Last.fm** integration (only) and a local play history view—no other accounts or streaming.

## Scope (In)
- Last.fm:
    - Auth flow (token/session key)
    - “Now Playing” updates
    - Scrobbling with correct timing thresholds
    - Retry queue when offline
- DB:
    - `play_history` table (track_id, started_at, played_ms, scrobbled_at, lastfm_status)
- UI:
    - Preferences → Internet → Last.fm connect/disconnect
    - History view: Recently played
    - Optional “Scrobble status” in Now Playing

## Non-scope (Out)
- Any other integrations (Spotify, Qobuz, etc.).
- User accounts beyond Last.fm.

## Prerequisites/Dependencies
- Milestone 02 — playback-shared-now-playing

## Key Decisions
- **Store play history locally in SQLite**
    - Why: already have DB; enables UX without remote dependence.

## Deliverables
- Last.fm integration service (backend) + settings storage.
- Play history DB table + queries.
- UI:
    - Last.fm connect UI
    - History page
- `/docs/lastfm.md` (setup, privacy notes, failure modes)
- UI vision:
    - `/artifacts/ui/10-lastfm-history/prefs-lastfm.png`
    - `/artifacts/ui/10-lastfm-history/history.png`
    - `/artifacts/ui/10-lastfm-history/REVIEW.md`
- Risk register update.

## Acceptance Criteria
- User can connect Last.fm and see connection state persist after restart.
- Tracks scrobble after threshold; failures retry.
- History view populates with played tracks.
- Snapshot pack produced.

## Commands (Labeled)
- `cargo tauri dev` (Pre-existing)
- `pnpm ui:snapshots -- --milestone 10` (Introduced)
- `cargo test` (Pre-existing)

## Verification (Tiered)
- **Tier A — Hardware-agnostic**:
    - Mocked API tests for Last.fm auth and scrobble endpoints.
    - DB schema verification for `play_history`.
    - Unit tests for retry queue logic.
- **Tier B — Hardware-dependent**:
    - N/A

## Risks & Mitigations
- **Rate limiting / API errors** → backoff + retry queue.
- **Privacy** → clear setting + ability to disable scrobbling entirely.

## Tweaks (MUST/SHOULD/MAY)
- **T1 (MUST)**: Play session + time accounting.
    - Define `play_id` (UUID/ULID) created when a track starts.
    - Time accounting: `played_ms` accumulates only while rendering audio (exclude pause). Seeking doesn't add time.
    - Engine emits: `PlaybackStarted { play_id... }`, `PlaybackProgress`, `PlaybackStopped`.
- **T4.1 (MUST)**: Scrobble timing rules.
    - Track must be > 30 seconds.
    - Must be played ≥ 50% of duration or 4 minutes (whichever first).
    - "Played time" excludes pauses.
- **T4.2 (SHOULD)**: Last.fm Batch + retry design.
    - Use batch scrobbles (up to 50) for retry queue.
- **T4.3 (MUST)**: Auth + signature mechanics.
    - Implement `api_sig` construction (sort params, concat, append secret, MD5).
    - Handle `sk` (session key) and `api_key`.
- **T4.4 (MUST)**: Secret/session storage.
    - **Do not** store session key in plaintext SQLite.
    - Use `keyring` crate (Windows Credential Manager) or DPAPI via `byte_stream`.
- **T4.5 (MAY)**: Last.fm Terms compliance.
    - Review terms regarding non-commercial use and rate limits.
- **T5 (SHOULD)**: play_history schema additions.
    - Add `play_id` (PK/UUID).
    - Add `lastfm_attempts`, `lastfm_last_error`, `lastfm_next_retry_at` for diagnostics.
    - Index: `(started_at DESC)`, `(lastfm_status, started_at)`.
- **T8.1 (MAY)**: History retention policy.
    - Cap at X plays or Y days.
    - "Clear history" button.

## Suggestions
- Consider launching explore agents for codebase pattern discovery
- Verify all acceptance criteria with lsp_diagnostics before completion

## Deferred/Backlog
- History retention policy (T8.1) - MAY, recommended default: 90 days.

## Failure Recovery / Resume
- Checkpointing: note last completed deliverable after each major task group.
- Resume: if a session id is provided, use `sisyphus_task(resume="<session_id>", prompt="fix: <specific failure>")`.
- If resume fails: rerun the original task without category wrapper.

## Session Prompt Variables
- `milestone_id`: 10
- `milestone_title`: lastfm-history
- `milestone_file`: C:\Obsidian Vaults\Sermon\prompts\Milestone 10 - lastfm-history.md
- `special_emphasis`: none
- `dependencies`: Milestone 02 — playback-shared-now-playing

## Milestone-specific Notes
### Source Additions
- Add a small “Scrobble Diagnostics” section in logs for supportability.
