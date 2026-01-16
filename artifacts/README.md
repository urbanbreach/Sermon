# Sermon UI Artifacts

This directory contains manual UI snapshots and review packs for each milestone.

## Snapshot Process

1. **Configure Display**:
   - Resolution: Set window size to approximately **1440x900** (or use a monitor of this resolution).
   - Scaling: **100%** (Windows Display Settings > Scale and layout > 100%).
   - *Important*: Snapshots must be taken at 1.0 device scale factor to ensure pixel consistency.

2. **Run Snapshot Mode**:
   ```powershell
   pnpm ui:snapshots -- --milestone 00   # Foundation
   pnpm ui:snapshots -- --milestone 01   # Library DB Scan
   pnpm ui:snapshots -- --milestone 03   # WASAPI Exclusive Bit-Perfect
   ```
   This will launch the application with mocked data (`SERMON_MOCK=1`) and snapshot mode enabled (`SERMON_SNAPSHOT=1`).

3. **Capture Screenshots**:
   - Navigate to the required screens.
   - Use Windows Snipping Tool (Win+Shift+S) or Alt+PrntScrn.
   - Save files to `artifacts/ui/{milestone-name}/`.

## Milestone Directories

### Milestone 00 - Foundation
- Directory: `artifacts/ui/00-foundation/`
- Screenshots:
  - `shell-library.png` - Main library view (Albums)
  - `shell-now-playing.png` - Now Playing view/mode
  - `shell-settings.png` - Settings screen

### Milestone 01 - Library DB Scan
- Directory: `artifacts/ui/01-library-db-scan/`
- Screenshots:
  - `tracks-empty.png` - Tracks view with no library folders
  - `scanning.png` - Tracks view during active scan (progress indicator)
  - `tracks-populated.png` - Tracks view with populated library

### Milestone 03 - WASAPI Exclusive Bit-Perfect
- Directory: `artifacts/ui/03-wasapi-exclusive-bit-perfect/`
- Screenshots:
  - `settings-audio.png` - Settings view showing Audio section with Output Mode, Policy, and Fade controls
  - `diagnostics-playing-44k.png` - Diagnostics view while playing a 44.1 kHz track (showing bit-perfect status)
  - `diagnostics-playing-96k.png` - Diagnostics view while playing a 96 kHz track (showing format switch)

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
│   ├── 01-library-db-scan/
│   │   ├── REVIEW.md
│   │   ├── tracks-empty.png
│   │   ├── scanning.png
│   │   └── tracks-populated.png
│   └── ...
└── README.md
```
