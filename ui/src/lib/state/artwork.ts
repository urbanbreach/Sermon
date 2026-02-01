import { writable, get } from 'svelte/store';
import { currentTrack } from './playback';
import { getArtworkBestForTrack, getArtworkBytes } from '../api/artwork';
import { Fixtures } from '../data/fixtures';

export const currentArtworkUrl = writable<string | null>(null);

let lastTrackId: number | null = null;
let fallbackTimer: ReturnType<typeof setTimeout> | null = null;

export function initArtworkStore() {
  // Subscribe to currentTrack changes
  currentTrack.subscribe(async (track) => {
    // Clear any pending fallback timer if we have a new track
    if (fallbackTimer) {
      clearTimeout(fallbackTimer);
      fallbackTimer = null;
    }

    if (!track) {
      handleNoTrack();
      return;
    }

    // Skip if same track (optimization)
    if (track.id === lastTrackId && get(currentArtworkUrl)) {
      return;
    }
    
    lastTrackId = track.id;

    const url = await resolveArtworkUrl(track);
    
    if (url) {
      currentArtworkUrl.set(url);
    } else {
      // If we have a track but no artwork, we might want to clear immediately or keep previous?
      // Usually better to clear or show placeholder. For now, clear to allow placeholder UI.
      currentArtworkUrl.set(null);
    }
  });
}

function handleNoTrack() {
  // If track becomes null (stopped/cleared), keep last artwork for 10s then fade/clear
  if (get(currentArtworkUrl)) {
    if (!fallbackTimer) {
      fallbackTimer = setTimeout(() => {
        currentArtworkUrl.set(null);
        lastTrackId = null;
        fallbackTimer = null;
      }, 10000);
    }
  } else {
    // Already empty, just ensure clean state
    lastTrackId = null;
  }
}

async function resolveArtworkUrl(track: any): Promise<string | null> {
  // Mock/Snapshot mode
  if (import.meta.env.SERMON_MOCK === '1') {
    const fixtureAlbums = Fixtures.getAlbums();
    const fixtureArtists = Fixtures.getArtists();

    const albumTitleSort = (track.album?.trim().toLowerCase()) || 'unknown album';
    const albumArtistSort = (track.artist?.trim().toLowerCase()) || 'unknown artist';

    for (const fixtureAlbum of fixtureAlbums) {
      const artist = fixtureArtists.find(a => a.id === fixtureAlbum.artistId);
      const artistSort = (artist?.name?.trim().toLowerCase()) || 'unknown artist';
      const titleSort = (fixtureAlbum.title?.trim().toLowerCase()) || 'unknown album';

      if (artistSort === albumArtistSort && titleSort === albumTitleSort) {
        const url = Fixtures.getArtworkPath(fixtureAlbum.artworkFile);
        return url || null;
      }
    }
    return null;
  }

  // Runtime mode
  try {
    const best = await getArtworkBestForTrack(track.id);
    if (best.source !== 'none' && best.cacheKey && best.mime) {
      const bytes = await getArtworkBytes(best.cacheKey, best.mime);
      return `data:${bytes.mime};base64,${bytes.bytesBase64}`;
    }
  } catch (e) {
    console.error('Failed to resolve artwork URL:', e);
  }
  
  return null;
}
