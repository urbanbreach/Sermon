export interface TrackRow {
  id: number;
  library_folder_id: number;
  path: string;
  path_display?: string;
  title?: string;
  artist?: string;
  album?: string;
  album_artist?: string;
  track_no?: number;
  disc_no?: number;
  year?: number;
  genre?: string;
  codec?: string;
  container?: string;
  sample_rate?: number;
  bit_depth?: number;
  channels?: number;
  duration_ms?: number;
  is_missing: boolean;
}

export interface LibraryFolder {
  id: number;
  path: string;
  enabled: boolean;
}

export interface ScanProgress {
  scanned: number;
  total: number;
}

export interface ScanComplete {
  scan_id: number;
  scanned: number;
  total: number;
  skipped: number;
  errors: number;
  elapsed_ms: number;
}

export type SortBy = 'title' | 'artist' | 'album';
export type SortDirection = 'asc' | 'desc';
