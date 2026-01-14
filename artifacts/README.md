# Sermon UI Artifacts

This directory contains manual UI snapshots and review packs for each milestone.

## Snapshot Process

1. **Configure Display**:
   - Resolution: Set window size to approximately **1440x900** (or use a monitor of this resolution).
   - Scaling: **100%** (Windows Display Settings > Scale and layout > 100%).
   - *Important*: Snapshots must be taken at 1.0 device scale factor to ensure pixel consistency.

2. **Run Snapshot Mode**:
   ```powershell
   pnpm ui:snapshots -- --milestone 00
   ```
   This will launch the application with mocked data (`SERMON_MOCK=1`) and snapshot mode enabled (`SERMON_SNAPSHOT=1`).

3. **Capture Screenshots**:
   - Navigate to the required screens.
   - Use Windows Snipping Tool (Win+Shift+S) or Alt+PrntScrn.
   - Save files to `artifacts/ui/{milestone}-foundation/`.

## Naming Conventions

Save screenshots with the following standard names (PNG format):

- `shell-library.png` - Main library view (Albums)
- `shell-now-playing.png` - Now Playing view/mode
- `shell-settings.png` - Settings screen

## Review Packs

Each milestone directory contains a `REVIEW.md` file. This file serves as the sign-off document for the UI implementation, listing the captured artifacts, changes, and any known issues.

## Directory Structure

```
artifacts/
├── ui/
│   ├── 00-foundation/
│   │   ├── REVIEW.md
│   │   ├── shell-library.png
│   │   └── ...
│   └── ...
└── README.md
```
