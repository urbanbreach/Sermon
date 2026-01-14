# Product Backlog

This backlog contains "MAY" items extracted from milestone prompts during normalization. These features are deferred to keep the core milestones lean but may be pulled in based on user feedback or specific triggers.

| Source Milestone | Feature / Item | Category | Recommended Default | Pull-in Trigger |
|------------------|----------------|----------|---------------------|-----------------|
| 01, 04 | T8.2 - DB pragmas (WAL, synchronous) | Perf | Default SQLite safety | Scanning speed issues or UI stutter |
| 02 | T8.3 - Bit-perfect indicator | UX | Hidden | User feedback regarding audio quality confidence |
| 10 | T4.5 - Last.fm Terms compliance | Docs/Legal | Standard Notice | App Store submission or public release |
| 10 | T8.1 - History retention policy | UX | Keep Forever | Database size exceeds 100MB |
| 11 | T6.5 - Single instance enforcement | UX | Enforced | Users launching multiple instances by mistake |
| 06 | T8.4 - Artwork attribution note | Docs | README Credit | Using CC-licensed placeholder art |
| 04 | Advanced filtering / smart playlists | UX | Basic Search | Demand for boolean logic or metadata criteria |
| 05 | Bulk tag editing | UX | Single File Edit | User requests for album-level fixes |
| 06 | Lyrics fetching | UX | None | Feature parity requests |
| 07 | Full hotkey customization | UX | Hardcoded Common Keys | Accessibility needs or key conflicts |
| 00 | Play session/time accounting spec | Architecture | Defer | When implementing play history |
| 00 | Gapless playback decision | Architecture | Defer | When implementing audio queue |
| 00 | Tagging safe-write vs file identity | Architecture | Defer | When implementing tag editor |
| 00 | Last.fm scrobble rules + secret storage | Docs/Legal | Defer | When implementing Last.fm |
| 00 | Windows productization (installer, updater, SMTC) | Productization | Defer | Before first release |
| 00 | History retention policy | UX | Defer | When DB exceeds 100MB |
| 00 | DB pragmas (WAL, synchronous) | Perf | Defer | When scanning speed issues |
| 00 | Bit-perfect indicator | UX | Defer | User feedback on audio quality |
| 00 | Artwork attribution | Docs | Defer | Using CC-licensed art |
| 00 | ASIO licensing checklist | Docs/Legal | Defer | Before ASIO release |
