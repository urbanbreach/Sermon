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

export type SearchHit = 
  | { type: 'track'; trackId: number; title?: string; artist?: string; album?: string }
  | { type: 'album'; albumArtistSort: string; albumTitleSort: string; albumArtistDisplay: string; albumTitleDisplay: string; year?: number }
  | { type: 'artist'; artistSort: string; artistDisplay: string };

export interface SearchSuggestResponse {
  results: SearchHit[];
}

export interface AlbumListItem {
  albumTitleDisplay: string;
  albumArtistDisplay: string;
  albumTitleSort: string;
  albumArtistSort: string;
  year?: number;
  trackCount: number;
}

export interface AlbumCursor {
  albumArtistSort: string;
  albumTitleSort: string;
}

export interface Page<T, C> {
  items: T[];
  nextCursor?: C;
}

export interface ArtistListItem {
  artistDisplay: string;
  artistSort: string;
  trackCount: number;
  albumCount: number;
}

export interface ArtistCursor {
  artistSort: string;
}

export interface AlbumTrackCursor {
  discNo: number;
  trackNo: number;
  titleSort: string;
  id: number;
}

export interface OffsetCursor {
  offset: number;
}

export interface LibraryStats {
  trackCount: number;
  albumCount: number;
  artistCount: number;
  dbSizeBytes: number;
  lastScanCompletedMs?: number;
}
