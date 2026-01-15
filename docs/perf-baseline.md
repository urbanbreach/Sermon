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
