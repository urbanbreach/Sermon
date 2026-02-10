import { writable } from 'svelte/store';
import { listAlbumTracksPage } from '../api/library';
import type { AlbumListItem, TrackRow } from '../types/library';

export interface SelectedSummary {
  kind: 'album' | 'library' | 'track';
  label: string;
  artist: string;
  trackCount: number;
  sizeBytes: number;
  durationMs: number;
  loading: boolean;
}

type AlbumSummaryInput = Pick<
  AlbumListItem,
  'albumArtistSort' | 'albumTitleSort' | 'albumArtistDisplay' | 'albumTitleDisplay' | 'trackCount'
>;

export const selectedSummary = writable<SelectedSummary | null>(null);
// Backward-compatible alias for existing imports during migration.
export const selectedAlbumSummary = selectedSummary;

const summaryCache = new Map<string, { trackCount: number; sizeBytes: number; durationMs: number }>();
let activeSelectionRequestId = 0;

function buildAlbumKey(albumArtistSort: string, albumTitleSort: string): string {
  return `${albumArtistSort}||${albumTitleSort}`;
}

export function clearSelectedAlbumSummary(): void {
  clearSelectedSummary();
}

export function clearSelectedSummary(): void {
  activeSelectionRequestId += 1;
  selectedSummary.set(null);
}

function sumSizeBytes(tracks: TrackRow[]): number {
  return tracks.reduce((total, track) => total + (track.sizeBytes || 0), 0);
}

function sumDurationMs(tracks: TrackRow[]): number {
  return tracks.reduce((total, track) => total + (track.durationMs || 0), 0);
}

export function selectAlbumSummary(album: AlbumSummaryInput): void {
  const requestId = ++activeSelectionRequestId;
  const albumKey = buildAlbumKey(album.albumArtistSort, album.albumTitleSort);
  const cached = summaryCache.get(albumKey);

  selectedSummary.set({
    kind: 'album',
    label: album.albumTitleDisplay,
    artist: album.albumArtistDisplay,
    trackCount: cached?.trackCount ?? album.trackCount,
    sizeBytes: cached?.sizeBytes ?? 0,
    durationMs: cached?.durationMs ?? 0,
    loading: !cached
  });

  if (cached) return;

  void listAlbumTracksPage(album.albumArtistSort, album.albumTitleSort, 10000)
    .then((page) => {
      if (requestId !== activeSelectionRequestId) return;

      const resolved = {
        trackCount: page.items.length || album.trackCount,
        sizeBytes: sumSizeBytes(page.items),
        durationMs: sumDurationMs(page.items)
      };

      summaryCache.set(albumKey, resolved);

      selectedSummary.update((current) => {
        if (!current) return current;
        if (current.kind !== 'album') {
          return current;
        }
        if (current.label !== album.albumTitleDisplay || current.artist !== album.albumArtistDisplay) {
          return current;
        }

        return {
          ...current,
          ...resolved,
          loading: false
        };
      });
    })
    .catch((error) => {
      if (requestId !== activeSelectionRequestId) return;
      console.warn('Failed to resolve selected album summary:', error);

      selectedSummary.update((current) => {
        if (!current) return current;
        if (current.kind !== 'album') {
          return current;
        }
        if (current.label !== album.albumTitleDisplay || current.artist !== album.albumArtistDisplay) {
          return current;
        }

        return {
          ...current,
          loading: false
        };
      });
    });
}

export function selectLibrarySummaryFromTracks(allTracks: TrackRow[]): void {
  activeSelectionRequestId += 1;
  selectedSummary.set({
    kind: 'library',
    label: 'All Tracks',
    artist: 'Library',
    trackCount: allTracks.length,
    sizeBytes: sumSizeBytes(allTracks),
    durationMs: sumDurationMs(allTracks),
    loading: false
  });
}

export function selectTrackSummary(track: TrackRow): void {
  activeSelectionRequestId += 1;
  selectedSummary.set({
    kind: 'track',
    label: track.title?.trim() || 'Unknown Track',
    artist: track.artist?.trim() || 'Unknown Artist',
    trackCount: 1,
    sizeBytes: track.sizeBytes || 0,
    durationMs: track.durationMs || 0,
    loading: false
  });
}
