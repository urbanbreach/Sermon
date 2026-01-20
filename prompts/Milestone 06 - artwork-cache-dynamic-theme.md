# Milestone 06 - artwork-cache-dynamic-theme

## Goal

- Implement album art pipeline (embedded + fetch + cache + optional embed) and the Apple-like **dynamic liquid-glass theming** that reacts to album art.

## Scope (In)

- Artwork sources:
  - Read embedded art from file tags (via tagging pipeline).
  - Fetch art from iTunes Search API, fanart.tv and Deezer API (no accounts).
- Caching:
  - Local cache directory with stable keys (album/artist/title match + provider id).
  - DB mapping from album/track → cached art key.
  - Cache size policy (simple; user-configurable later).
- Optional: “Embed art into file” (uses milestone 05 write-back).
- UI:
  - Album grid and Now Playing show artwork.
  - Artwork picker modal (choose among results).
  - **Dynamic theme engine**:
    - Derive accent color + gradient background from current album art.
    - Apply to glass panels (subtle glow/border highlight).
    - Ensure deterministic snapshot behavior (fixed fixture images → fixed theme outputs).

## Non-scope (Out)

- Lyrics fetching (not requested; lyrics UI can remain placeholder).
- Streaming services.

## Prerequisites/Dependencies

- Milestone 01 — library-db-scan
- Milestone 04 — library-browse-search-polish

## Key Decisions

- **Avoid heavy image processing in Rust initially**
  - Prefer doing palette extraction in frontend canvas (small, fast, avoids `image` crate weight).
- **Cache on disk, not in DB blobs**
  - Why: avoids DB bloat; easy eviction.

## Deliverables

- Artwork service (fetch + cache + map to albums).
- UI:
  - artwork picker modal
  - dynamic theming applied across shell
- `/docs/artwork.md` (providers, caching, embedding options, attribution notes)
- UI vision:
  - `/artifacts/ui/06-artwork-cache-dynamic-theme/albums-themed.png`
  - `/artifacts/ui/06-artwork-cache-dynamic-theme/now-playing-themed.png`
  - `/artifacts/ui/06-artwork-cache-dynamic-theme/artwork-picker.png`
  - `/artifacts/ui/06-artwork-cache-dynamic-theme/REVIEW.md`
- ADR:
  - `/docs/adr/0008-artwork-providers-cache.md`
- Risk register update.

## Acceptance Criteria

- Embedded art displays when available.
- Fetching art works for a known album and is cached for subsequent loads.
- Dynamic theme changes when changing tracks/albums.
- Snapshot pack produced and deterministic (re-running produces identical images).

## Commands (Labeled)

- `cargo tauri dev` (Pre-existing)
- `pnpm ui:snapshots -- --milestone 06` (Introduced)
- `cargo test` (Pre-existing)

## Verification (Tiered)

- **Tier A — Hardware-agnostic**:
  1. Load track with known embedded art, verify display.
  2. Load track without art, use "Fetch" to find artwork from provider.
  3. Verify application theme colors update to match displayed artwork.
- **Tier B — Hardware-dependent**: N/A — not hardware dependent

## Risks & Mitigations

- **Provider rate limits / availability** → caching + graceful fallback; allow provider toggles.
- **Visual nondeterminism from palette extraction** → snapshot mode forces fixed sampling parameters.

## Tweaks (MUST/SHOULD/MAY)

- **T8.4** (MAY): Artwork attribution note. Ensure UI or docs credit the artwork providers (iTunes/Deezer) if their terms require it.

## Suggestions

- Consider launching explore agents for codebase pattern discovery
- Verify all acceptance criteria with lsp_diagnostics before completion
- Use frontend UI/UX engineer subagent to design and polish the UI/UX.
- Use the installed Tauri MCP server to view and edit the UI/UX.

## Deferred/Backlog

- Lyrics fetching (MAY).

## Failure Recovery / Resume

- Checkpointing: note last completed deliverable after each major task group.
- Resume: if a session id is provided, use `sisyphus_task(resume="<session_id>", prompt="fix: <specific failure>")`.
- If resume fails: rerun the original task without category wrapper.

## Session Prompt Variables

- `milestone_id`: 06
- `milestone_title`: artwork-cache-dynamic-theme
- `milestone_file`: C:\Obsidian Vaults\Sermon\prompts\Milestone 06 - artwork-cache-dynamic-theme.md
- `special_emphasis`: none
- `dependencies`: Milestone 01 — library-db-scan; Milestone 04 — library-browse-search-polish

## Milestone-specific Notes

### Source Additions

- Add a “Reduce Motion / Reduce Transparency” toggle (helps perf and accessibility).
- Toggleable effects for the dynamic theme (e.g. blur, glow, border highlight).
