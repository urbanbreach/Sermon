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

export interface QuickScanComplete {
  foldersChecked: number;
  filesChecked: number;
  filesAdded: number;
  filesMarkedMissing: number;
  filesRestored: number;
  elapsedMs: number;
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

// ============================================================================
// Tag Editing Types (Milestone 05)
// ============================================================================

/** Patch operation for string tag fields */
export type TagPatch =
  | { op: 'leave' }
  | { op: 'set'; value: string }
  | { op: 'clear' };

/** Patch operation for numeric tag fields */
export type NumberPatch =
  | { op: 'leave' }
  | { op: 'set'; value: number }
  | { op: 'clear' };

/** Request to update track tags */
export interface UpdateTrackTagsRequest {
  trackId: number;
  createBackup: boolean;
  title: TagPatch;
  artist: TagPatch;
  album: TagPatch;
  albumArtist: TagPatch;
  genre: TagPatch;
  trackNo: NumberPatch;
  discNo: NumberPatch;
  year: NumberPatch;
}

/** Status event for tag write progress */
export interface TagWriteStatusEvent {
  trackId: number;
  phase: 'retry' | 'success' | 'error';
  attempt: number;
  maxAttempts: number;
  lastErrorCode?: number;
}

/** Helper to create a "leave" patch */
export const leavePatch = (): TagPatch => ({ op: 'leave' });

/** Helper to create a "set" patch for strings */
export const setTagPatch = (value: string): TagPatch => ({ op: 'set', value });

/** Helper to create a "clear" patch */
export const clearPatch = (): TagPatch => ({ op: 'clear' });

/** Helper to create a "leave" patch for numbers */
export const leaveNumberPatch = (): NumberPatch => ({ op: 'leave' });

/** Helper to create a "set" patch for numbers */
export const setNumberPatch = (value: number): NumberPatch => ({ op: 'set', value });

/** Helper to create a "clear" patch for numbers */
export const clearNumberPatch = (): NumberPatch => ({ op: 'clear' });

// ============================================================================
// Raw Tags Types (Milestone 05 - Task 7)
// ============================================================================

/** A single raw tag item */
export interface RawTagItem {
  key: string;
  value: string;
}

/** Raw tags from a single tag type */
export interface RawTags {
  tagType: string;
  items: RawTagItem[];
}

/** All raw tags from a file */
export interface RawTagsResult {
  tags: RawTags[];
}
