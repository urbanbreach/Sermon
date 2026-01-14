# UI Review: Milestone 00 (Foundation)

**Status:** COMPLETE
**Date:** 2026-01-14

## Screenshots

- [x] `shell-library.png`
- [x] `shell-now-playing.png`
- [x] `shell-settings.png`

## What Changed

- Initial shell layout implementation (LeftNav, TopBar, BottomBar, content area)
- Store-based routing (Albums/Artists/Tracks/Settings/Now Playing)
- Liquid-glass CSS tokens applied (--glass-bg, --glass-blur, --glass-border, etc.)
- Dark-only theme
- Mock fixture data support (SERMON_MOCK=1)
- Snapshot mode support (SERMON_SNAPSHOT=1)

## Known Issues

- [x] Glass blur effect subtle on solid dark backgrounds (expected behavior)
- [x] Placeholder icons used for window controls
- [ ] Settings view has minor a11y warnings (label associations)

## Next UI Focus

- Milestone 01: Library DB Scan - real data integration
