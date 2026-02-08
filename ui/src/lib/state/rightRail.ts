import { writable, derived, get } from 'svelte/store';
import { currentTrackFull } from './playback';
import { listAlbumTracksPage } from '../api/library';
import type { TrackRow } from '../types/library';
import { persistRailOpen, persistRailMode } from './effects';

export type RailMode = 'now-playing' | 'lyrics';

// Rail mode state
export const railMode = writable<RailMode>('now-playing');

// Rail open state - responsive default
// Default to true if server-side (though Svelte is client-side here), 
// or check window width if available.
const initialOpen = typeof window !== 'undefined' ? window.innerWidth >= 1280 : true;
export const isRailOpen = writable<boolean>(initialOpen);

// Initialize responsive behavior
export function initRailResponsive(): void {
  if (typeof window === 'undefined') return;

  let initialized = false;

  const handleResize = () => {
    if (!initialized) {
      initialized = true;
      return;
    }

    const width = window.innerWidth;
    if (width >= 1280) {
      isRailOpen.set(true);
      persistRailOpen(true);
    } else {
      isRailOpen.set(false);
      persistRailOpen(false);
    }
  };

  window.addEventListener('resize', handleResize);
  handleResize();
}

// Toggle rail open/close
export function toggleRail(): void {
  isRailOpen.update(v => {
    const newVal = !v;
    persistRailOpen(newVal);
    return newVal;
  });
}

// Set rail mode
export function setRailMode(mode: RailMode): void {
  railMode.set(mode);
  persistRailMode(mode);
}

// Album tracks for "Playing Tracks" section
export const albumTracks = writable<TrackRow[]>([]);
export const albumTracksLoading = writable<boolean>(false);

// Track the current album context to avoid refetching
let currentAlbumKey = '';

// Reactively load album tracks when the current track's album changes
export function loadAlbumTracksIfNeeded(): void {
  const track = get(currentTrackFull);
  if (!track) {
    albumTracks.set([]);
    currentAlbumKey = '';
    return;
  }

  const artist = (track.albumArtist || track.artist || 'unknown artist').trim().toLowerCase();
  const album = (track.album || 'unknown album').trim().toLowerCase();
  const newKey = `${artist}||${album}`;

  if (newKey === currentAlbumKey) {
    return; // Already loaded this album
  }

  currentAlbumKey = newKey;
  albumTracksLoading.set(true);

  // Fetch all tracks for this album (limit 200 should cover most albums)
  listAlbumTracksPage(artist, album, 200)
    .then(page => {
      albumTracks.set(page.items);
    })
    .catch(err => {
      console.warn('Failed to load album tracks:', err);
      albumTracks.set([]);
    })
    .finally(() => {
      albumTracksLoading.set(false);
    });
}
