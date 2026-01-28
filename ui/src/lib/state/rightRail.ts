import { writable, derived, get } from 'svelte/store';
import { currentTrackFull } from './playback';
import { listAlbumTracksPage } from '../api/library';
import type { TrackRow } from '../types/library';

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
  
  const handleResize = () => {
    const width = window.innerWidth;
    // Requirement: persistent >= 1280px, overlay/scrim 1100-1279px
    // At 1280px+, it should be open by default (persistent)
    // Below 1280px, it should be closed by default unless user opened it?
    // The plan says: "Responsive behavior: persistent >=1280px, overlay with scrim 1100-1279px"
    // Usually "persistent" implies it *forces* open or at least defaults open.
    // Let's stick to the logic provided in the prompt's MUST DO section:
    if (width >= 1280) {
      isRailOpen.set(true);
    } else {
      // Below 1280, we default to closed when resizing *into* this range?
      // Or just let user control it?
      // The prompt code snippet says:
      // if (width >= 1280) isRailOpen.set(true);
      // else if (width < 1280) isRailOpen.set(false);
      // This forces it closed when resizing down, which is standard for responsive rails.
      isRailOpen.set(false);
    }
  };
  
  window.addEventListener('resize', handleResize);
  // We don't call handleResize() immediately here because we set the initial value 
  // in the writable declaration, and we don't want to override user state on init 
  // if we can avoid it, but strictly following the prompt instructions:
  // "window.addEventListener('resize', handleResize); handleResize(); // Initial check"
  // So I will include the initial check.
  handleResize();
}

// Toggle rail open/close
export function toggleRail(): void {
  isRailOpen.update(v => !v);
}

// Set rail mode
export function setRailMode(mode: RailMode): void {
  railMode.set(mode);
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
