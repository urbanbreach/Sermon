# Learnings: Preferences View Implementation

## Approach
- Created a new view `PreferencesView.svelte` separate from `SettingsView`.
- Followed the MusicBee-style layout: left sidebar for categories, main content area for settings.
- Used CSS Grid/Flexbox for layout.
- Reused global glass styling variables (`--glass-bg`, etc.) to maintain consistency.
- Used `data-category` attributes on buttons to facilitate potential future automation or testing.
- Implemented "mock mode" guards for future functionality.

## Route Integration
- Updated `Route` union type in `route.ts`.
- Updated `LeftNav.svelte` to include the new navigation item.
- Updated `App.svelte` to render the view based on the route name.

## Styles
- Glassmorphism is central to the app's aesthetic.
- Sidebar: 200px width, glass background.
- Content: Flex-grow, transparent/darker background for contrast.
- Active states: highlighted with distinct visual cues.

## Future Work
- Implement actual settings logic for each category.
- Connect "Reset to Defaults" to backend.
- Implement "Export Diagnostics" (Task 9).

## Category Components Implementation
- Split preferences into 7 granular components in `ui/src/lib/components/preferences/`.
- Reused existing stores from `playback.ts`, `library.ts`, `effects.ts` instead of duplicating logic.
- Implemented real functionality for:
  - Player: Device selection, Output Mode, Buffer Size.
  - Library: Folder list, Add Folder, Scan on Startup.
  - Internet: Provider toggles.
- Used placeholders for future backend features (DoP, Last.fm, etc.) with consistent `[Coming Soon]` or "Takes effect in future update" labeling.
- Leveraged Svelte 5 runes (`$state`) for local component state while maintaining store subscriptions for global state.
