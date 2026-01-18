import { invoke } from '@tauri-apps/api/core';
import type { TrackRow, LibraryFolder, SortBy, SortDirection, SearchSuggestResponse, AlbumListItem, AlbumCursor, Page, ArtistListItem, ArtistCursor, AlbumTrackCursor, OffsetCursor, LibraryStats, UpdateTrackTagsRequest, RawTagsResult } from '../types/library';

export async function addFolder(path: string): Promise<LibraryFolder> {
  return invoke('cmd_library_add_folder', { path });
}

export async function listFolders(): Promise<LibraryFolder[]> {
  return invoke('cmd_library_list_folders');
}

export async function listTracks(sortBy: SortBy, direction: SortDirection): Promise<TrackRow[]> {
  return invoke('cmd_library_list_tracks', { sortBy, direction });
}

export async function startScan(path: string): Promise<{ scan_id: number }> {
  return invoke('cmd_scan_start', { path });
}

export async function searchSuggest(query: string, limit?: number): Promise<SearchSuggestResponse> {
  return invoke('cmd_library_search_suggest', { request: { query, limit } });
}

export async function listAlbumsPage(limit: number, cursor?: AlbumCursor): Promise<Page<AlbumListItem, AlbumCursor>> {
  return invoke('cmd_library_list_albums_page', { request: { limit, cursor } });
}

export async function listArtistsPage(limit: number, cursor?: ArtistCursor): Promise<Page<ArtistListItem, ArtistCursor>> {
  return invoke('cmd_library_list_artists_page', { request: { limit, cursor } });
}

export async function listAlbumTracksPage(
  albumArtistSort: string,
  albumTitleSort: string,
  limit: number,
  cursor?: AlbumTrackCursor
): Promise<Page<TrackRow, AlbumTrackCursor>> {
  return invoke('cmd_library_list_album_tracks_page', { 
    request: { albumArtistSort, albumTitleSort, limit, cursor } 
  });
}

export async function searchTracksPage(
  query: string,
  sortBy: string,
  direction: string,
  limit: number,
  cursor?: OffsetCursor
): Promise<Page<TrackRow, OffsetCursor>> {
  return invoke('cmd_library_search_tracks_page', { 
    request: { query, sortBy, direction, limit, cursor } 
  });
}

export async function searchAlbumsPage(
  query: string,
  limit: number,
  cursor?: AlbumCursor
): Promise<Page<AlbumListItem, AlbumCursor>> {
  return invoke('cmd_library_search_albums_page', { 
    request: { query, limit, cursor } 
  });
}

export async function searchArtistsPage(
  query: string,
  limit: number,
  cursor?: ArtistCursor
): Promise<Page<ArtistListItem, ArtistCursor>> {
  return invoke('cmd_library_search_artists_page', { 
    request: { query, limit, cursor } 
  });
}

export async function getLibraryStats(): Promise<LibraryStats> {
  return invoke('cmd_library_get_stats');
}

// ============================================================================
// Tag Editing API (Milestone 05)
// ============================================================================

/**
 * Update tags for a track on disk and in the database.
 * 
 * @param request - The update request containing track ID, backup preference, and tag patches
 * @returns The updated track row after successful write
 * @throws Error if the write fails (file locked, permission denied, etc.)
 */
export async function updateTrackTags(request: UpdateTrackTagsRequest): Promise<TrackRow> {
  return invoke('cmd_library_update_track_tags', { request });
}

/**
 * Get all raw tags from a track file for debugging/inspection.
 * 
 * @param trackId - The track ID to read raw tags from
 * @returns All raw tag items from all tag types in the file
 */
export async function getRawTags(trackId: number): Promise<RawTagsResult> {
  return invoke('cmd_library_get_raw_tags', { trackId });
}
