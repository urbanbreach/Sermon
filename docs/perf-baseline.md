# Performance Baseline

This document tracks performance metrics for the Sermon library scanner.

## Measurement Methodology

### Scan Performance
- **Metric**: `tracks/minute` = (track_count / scan_duration_minutes)
- **Data Source**: `evt_scan_complete` event payload
  - `scanned`: Number of files processed
  - `elapsed_ms`: Total scan duration in milliseconds
- **Calculation**: `tracks_per_minute = (scanned * 60000) / elapsed_ms`

### How to Measure
1. Start with an empty database (delete `library.db` from app data dir)
2. Add a library folder with known track count
3. Run full scan via Settings > Add Folder
4. Record values from `evt_scan_complete` event (visible in logs at `artifacts/logs/sermon.log`)

## Baseline Results

| Date | Dataset | Track Count | Scan Time (s) | Tracks/Min | Storage | Notes |
|------|---------|-------------|---------------|------------|---------|-------|
| TBD  | Initial | TBD         | TBD           | TBD        | SSD     | First measurement after M01 implementation |

## Target Performance

- **Goal**: ≥1000 tracks/minute on SSD storage
- **Acceptable**: ≥500 tracks/minute on HDD storage
- **Warning threshold**: <200 tracks/minute (investigate bottlenecks)

## Environment

Record these details when measuring:
- **Machine**: CPU, RAM, Storage type (SSD/HDD/NVMe)
- **OS**: Windows version
- **Dataset**: File formats (FLAC/MP3/etc), average file size
- **Library location**: Local drive vs network share

## Notes

- Scan performance should be measured with cold cache (restart app before scan)
- Initial scan includes metadata extraction; rescan should be faster (skip unchanged)
- Performance logs are written to `artifacts/logs/sermon.log` with format:
  ```
  scan_complete scanned=<n> total=<n> skipped=<n> errors=<n> elapsed_ms=<n>
  ```

## Milestone 04 - Library Browse & Search Polish

### Metrics Added

| Metric | Description | How to Measure |
|--------|-------------|----------------|
| Cold Start Time | Time from app launch to first interactive | Measure from process start to `first_interactive` log |
| Search Latency | Time from typing to dropdown visible | Stopwatch from keystroke to results appearing |
| Idle Memory | Memory usage with library loaded, idle | Windows Task Manager after app stabilizes |

### Baseline Results (Milestone 04)

| Date | Metric | Value | Dataset | Notes |
|------|--------|-------|---------|-------|
| TBD | Cold Start Time | TBD ms | TBD tracks | First measurement |
| TBD | Search Latency | TBD ms | TBD tracks | Typical query |
| TBD | Idle Memory | TBD MB | TBD tracks | After library load |

### Target Performance

- **Cold Start**: <2000ms to first interactive
- **Search Latency**: <100ms for dropdown to appear (perceived instant)
- **Idle Memory**: <150MB with 10k track library loaded

### How to Measure

#### Cold Start Time
1. Close the application completely
2. Start the app with logging enabled
3. Look for `first_interactive` in logs
4. Calculate: timestamp of `first_interactive` - process start time

#### Search Latency
1. Load a library with 1000+ tracks
2. Focus the search input
3. Type a query (e.g., "beatles")
4. Measure time until dropdown appears with results
5. Repeat 3-5 times and average

#### Idle Memory
1. Start the app and let it load the library
2. Navigate to Albums view, wait for initial load
3. Let the app sit idle for 30 seconds
4. Check Windows Task Manager > Details > sermon.exe
5. Record "Memory (private working set)"

