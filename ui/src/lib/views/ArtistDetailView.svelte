<script lang="ts">
  import { onMount } from 'svelte';
  import { currentRoute, goBack, canGoBack, navigate } from '../state/route';
  import { playNow, addToQueue } from '../state/playback';
  import { listArtistsPage } from '../api/library';
  import { invoke } from '@tauri-apps/api/core';
  import type { TrackRow, ArtistCursor } from '../types/library';

  interface ArtistAlbum {
    albumTitleDisplay: string;
    albumArtistDisplay: string;
    albumTitleSort: string;
    albumArtistSort: string;
    year?: number;
    trackCount: number;
    tracks: TrackRow[];
  }

  let artistSort = '';
  let artistDisplay = $state('Loading...');
  let albums = $state<ArtistAlbum[]>([]);
  let loading = $state(true);
  let loadError = $state<string | null>(null);

  let totalTracks = $derived(albums.reduce((acc, a) => acc + a.tracks.length, 0));
  let totalAlbums = $derived(albums.length);

  onMount(() => {
    const route = $currentRoute;
    if (route.name === 'artist-detail') {
      artistSort = route.artistSort;
      loadArtistTracks();
    }
  });

  async function loadArtistTracks() {
    if (!artistSort) return;
    
    loading = true;
    loadError = null;
    
    try {
      // Load all tracks for this artist
      const response = await invoke<{ items: TrackRow[], nextCursor?: any }>('cmd_library_list_artist_tracks_page', {
        request: {
          artistSort: artistSort,
          limit: 500,
          cursor: null
        }
      });

      const tracks = response.items;
      
      if (tracks.length > 0) {
        // Get artist display name from first track
        artistDisplay = tracks[0].artist || 'Unknown Artist';
        
        // Group tracks by album
        const albumMap = new Map<string, ArtistAlbum>();
        
        for (const track of tracks) {
          const albumKey = `${(track.album_artist || track.artist || '').toLowerCase()}|${(track.album || '').toLowerCase()}`;
          
          if (!albumMap.has(albumKey)) {
            albumMap.set(albumKey, {
              albumTitleDisplay: track.album || 'Unknown Album',
              albumArtistDisplay: track.album_artist || track.artist || 'Unknown Artist',
              albumTitleSort: (track.album || 'unknown album').toLowerCase(),
              albumArtistSort: (track.album_artist || track.artist || 'unknown artist').toLowerCase(),
              year: track.year,
              trackCount: 0,
              tracks: []
            });
          }
          
          const album = albumMap.get(albumKey)!;
          album.tracks.push(track);
          album.trackCount = album.tracks.length;
          if (track.year && (!album.year || track.year < album.year)) {
            album.year = track.year;
          }
        }
        
        // Sort albums by year (newest first), then by name
        albums = Array.from(albumMap.values()).sort((a, b) => {
          if (a.year && b.year) return b.year - a.year;
          if (a.year) return -1;
          if (b.year) return 1;
          return a.albumTitleDisplay.localeCompare(b.albumTitleDisplay);
        });
      } else {
        artistDisplay = artistSort;
        albums = [];
      }
    } catch (e) {
      console.error('Failed to load artist tracks:', e);
      loadError = e instanceof Error ? e.message : 'Failed to load artist';
      artistDisplay = artistSort;
    } finally {
      loading = false;
    }
  }

  function handleAlbumClick(album: ArtistAlbum) {
    navigate({
      name: 'album-detail',
      albumArtistSort: album.albumArtistSort,
      albumTitleSort: album.albumTitleSort
    });
  }

  function handlePlayArtist() {
    const allTracks = albums.flatMap(a => a.tracks);
    if (allTracks.length > 0 && allTracks[0].id) {
      playNow(allTracks[0].id);
    }
  }

  function formatDuration(ms?: number): string {
    if (!ms) return '--:--';
    const minutes = Math.floor(ms / 60000);
    const seconds = ((ms % 60000) / 1000).toFixed(0);
    return minutes + ":" + (Number(seconds) < 10 ? '0' : '') + seconds;
  }
</script>

<div class="view-container">
  <div class="top-bar">
    <button class="back-btn" onclick={goBack} disabled={!$canGoBack}>
      ← Back
    </button>
  </div>

  <div class="artist-header">
    <div class="artist-icon">
      <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="8" r="4"/>
        <path d="M4 20c0-4 4-6 8-6s8 2 8 6"/>
      </svg>
    </div>
    <div class="artist-info">
      <h1>{artistDisplay}</h1>
      {#if !loading}
        <div class="meta">
          <span>{totalAlbums} albums</span>
          <span class="bullet">•</span>
          <span>{totalTracks} tracks</span>
        </div>
      {/if}
      <div class="artist-actions">
        <button class="primary-btn" onclick={handlePlayArtist} disabled={albums.length === 0}>
          Play All
        </button>
      </div>
    </div>
  </div>

  <div class="content">
    {#if loading}
      <div class="loading-state">Loading...</div>
    {:else if loadError}
      <div class="error-state">
        <p>Failed to load artist</p>
        <p class="error-detail">{loadError}</p>
        <button onclick={loadArtistTracks}>Retry</button>
      </div>
    {:else if albums.length === 0}
      <div class="empty-state">No albums found for this artist</div>
    {:else}
      <h2>Discography</h2>
      <div class="albums-grid">
        {#each albums as album}
          <div 
            class="album-card"
            role="button"
            tabindex="0"
            onclick={() => handleAlbumClick(album)}
            onkeydown={(e) => e.key === 'Enter' && handleAlbumClick(album)}
          >
            <div class="album-artwork">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="12" cy="12" r="10"/>
                <circle cx="12" cy="12" r="3"/>
              </svg>
            </div>
            <div class="album-info">
              <div class="album-title" title={album.albumTitleDisplay}>{album.albumTitleDisplay}</div>
              <div class="album-meta">
                {#if album.year}<span>{album.year}</span>{/if}
                <span>{album.trackCount} tracks</span>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .view-container {
    padding: 2rem;
    color: #fff;
    height: 100%;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  .top-bar {
    margin-bottom: 0.5rem;
  }

  .back-btn {
    background: transparent;
    border: none;
    color: #888;
    cursor: pointer;
    font-size: 1rem;
    padding: 0;
    transition: color 0.2s;
  }

  .back-btn:hover {
    color: #fff;
  }

  .back-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .artist-header {
    display: flex;
    gap: 2rem;
    align-items: center;
  }

  .artist-icon {
    width: 150px;
    height: 150px;
    background: linear-gradient(135deg, #2a2a2a 0%, #1a1a1a 100%);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #444;
    flex-shrink: 0;
  }

  .artist-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  h1 {
    font-size: 2.5rem;
    font-weight: 700;
    margin: 0;
    line-height: 1.1;
  }

  h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin: 0 0 1rem 0;
    color: #fff;
  }

  .meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: #aaa;
    font-size: 1rem;
  }

  .bullet {
    color: #666;
  }

  .artist-actions {
    margin-top: 1rem;
  }

  .primary-btn {
    background: #fff;
    color: #000;
    border: none;
    padding: 0.8rem 2rem;
    border-radius: 30px;
    font-weight: 600;
    font-size: 1rem;
    cursor: pointer;
    transition: transform 0.1s;
  }

  .primary-btn:hover:not(:disabled) {
    transform: scale(1.05);
  }

  .primary-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .content {
    flex: 1;
  }

  .albums-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 1.5rem;
  }

  .album-card {
    background: var(--glass-highlight);
    border-radius: var(--glass-radius);
    border: 1px solid var(--glass-border);
    overflow: hidden;
    cursor: pointer;
    transition: transform 0.2s, background-color 0.2s;
  }

  .album-card:hover {
    transform: translateY(-4px);
    background: var(--glass-border);
  }

  .album-artwork {
    width: 100%;
    aspect-ratio: 1;
    background: linear-gradient(135deg, #2a2a2a 0%, #1a1a1a 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #555;
  }

  .album-info {
    padding: 0.75rem;
  }

  .album-title {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 0.9rem;
    margin-bottom: 0.25rem;
  }

  .album-meta {
    font-size: 0.8rem;
    color: #888;
    display: flex;
    gap: 0.5rem;
  }

  .loading-state, .empty-state, .error-state {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    height: 200px;
    font-size: 1.2rem;
    color: #888;
    gap: 1rem;
  }

  .error-state {
    color: #f66;
  }

  .error-detail {
    font-size: 0.9rem;
    color: #888;
  }

  .error-state button {
    background: rgba(255,255,255,0.1);
    border: 1px solid rgba(255,255,255,0.2);
    color: #fff;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
  }
</style>
