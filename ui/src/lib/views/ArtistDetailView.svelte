<script lang="ts">
  import { onMount } from 'svelte';
  import { currentRoute, goBack, canGoBack, navigate } from '../state/route';
  import { playNow, addToQueue } from '../state/playback';
  import { listArtistsPage } from '../api/library';
  import { invoke } from '@tauri-apps/api/core';
  import type { TrackRow, ArtistCursor } from '../types/library';
  import { ArrowLeft } from '@lucide/svelte';
  import { openAlbumInline } from '../state/albumInline';
  import { selectAlbumSummary } from '../state/albumSelection';
  import { getArtworkBestForAlbum } from '../api/artwork';
  import ArtworkImage from '../components/ArtworkImage.svelte';

  interface ArtistAlbum {
    albumTitleDisplay: string;
    albumArtistDisplay: string;
    albumTitleSort: string;
    albumArtistSort: string;
    year?: number;
    trackCount: number;
    tracks: TrackRow[];
    artworkCacheKey?: string;
  }

  let artistSort = '';
  let artistDisplay = $state('Loading...');
  let albumMap = $state<Map<string, ArtistAlbum>>(new Map());
  let albums = $derived(Array.from(albumMap.values()).sort((a, b) => {
    if (a.year && b.year) return b.year - a.year;
    if (a.year) return -1;
    if (b.year) return 1;
    return a.albumTitleDisplay.localeCompare(b.albumTitleDisplay);
  }));
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
        const map = new Map<string, ArtistAlbum>();
        
        for (const track of tracks) {
          const albumKey = `${(track.albumArtist || track.artist || '').toLowerCase()}|${(track.album || '').toLowerCase()}`;
          
          if (!map.has(albumKey)) {
            map.set(albumKey, {
              albumTitleDisplay: track.album || 'Unknown Album',
              albumArtistDisplay: track.albumArtist || track.artist || 'Unknown Artist',
              albumTitleSort: (track.album || 'unknown album').toLowerCase(),
              albumArtistSort: (track.albumArtist || track.artist || 'unknown artist').toLowerCase(),
              year: track.year,
              trackCount: 0,
              tracks: []
            });
          }
          
          const album = map.get(albumKey)!;
          album.tracks.push(track);
          album.trackCount = album.tracks.length;
          if (track.year && (!album.year || track.year < album.year)) {
            album.year = track.year;
          }
        }
        
        albumMap = map;
        
        // Load artwork keys in background
        loadArtworkKeys(Array.from(map.values()));
      } else {
        artistDisplay = artistSort;
        albumMap = new Map();
      }

    } catch (e) {
      console.error('Failed to load artist tracks:', e);
      loadError = e instanceof Error ? e.message : 'Failed to load artist';
      artistDisplay = artistSort;
    } finally {
      loading = false;
    }
  }

  async function loadArtworkKeys(albumsToLoad: ArtistAlbum[]) {
    let updated = false;
    for (const album of albumsToLoad) {
      if (album.artworkCacheKey) continue;
      try {
        const best = await getArtworkBestForAlbum(album.albumArtistSort, album.albumTitleSort);
        if (best.cacheKey) {
          album.artworkCacheKey = best.cacheKey;
          updated = true;
        }
      } catch (e) {
        // Ignore errors
      }
    }
    
    if (updated) {
      // Trigger reactivity
      albumMap = new Map(albumMap);
    }
  }

  function handleAlbumClick(album: ArtistAlbum) {
    selectAlbumSummary(album);
    openAlbumInline({
      albumArtistSort: album.albumArtistSort,
      albumTitleSort: album.albumTitleSort
    });
    navigate({ name: 'albums' });
  }

  function handlePlayArtist() {
    const allTracks = albums.flatMap(a => a.tracks);
    if (allTracks.length > 0 && allTracks[0].id) {
      playNow(allTracks[0].id);
    }
  }

  function formatDuration(ms?: number): string {
    if (!ms) return '—';
    const minutes = Math.floor(ms / 60000);
    const seconds = ((ms % 60000) / 1000).toFixed(0);
    return minutes + ":" + (Number(seconds) < 10 ? '0' : '') + seconds;
  }
</script>

<div class="view-container">
  <div class="top-bar">
    <button class="back-btn" onclick={goBack} disabled={!$canGoBack}>
      <ArrowLeft size={16} /> Back
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
        {#each albums as album (album.albumArtistSort + album.albumTitleSort)}
          <div 
            class="album-card"
            role="button"
            tabindex="0"
            onclick={() => handleAlbumClick(album)}
            onkeydown={(e) => e.key === 'Enter' && handleAlbumClick(album)}
          >
            <div class="album-artwork-container">
              <ArtworkImage
                cacheKey={album.artworkCacheKey}
                artistSort={album.albumArtistSort}
                titleSort={album.albumTitleSort}
                size={256}
                alt="{album.albumTitleDisplay} artwork"
              />
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
    color: var(--text-primary);
    height: 100%;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 3rem;
    background: var(--surface-0);
  }

  .top-bar {
    margin-bottom: 0.5rem;
  }

  .back-btn {
    background: transparent;
    border: 1px solid var(--divider-color);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 0.9rem;
    padding: 0.5rem 1.2rem;
    border-radius: 0;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-mono);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .back-btn:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
    border-color: var(--text-primary);
  }

  .back-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .artist-header {
    display: flex;
    gap: 2.5rem;
    align-items: flex-end;
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--divider-color);
  }

  .artist-icon {
    width: 180px;
    height: 180px;
    background: var(--surface-2);
    border: 1px solid var(--divider-color);
    border-radius: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .artist-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  h1 {
    font-size: 4rem;
    font-weight: 800;
    margin: 0;
    line-height: 0.9;
    letter-spacing: -0.03em;
    color: var(--text-primary);
    text-transform: uppercase;
  }

  h2 {
    font-size: 1.2rem;
    font-weight: 600;
    margin: 0 0 1.5rem 0;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-bottom: 1px solid var(--divider-color);
    padding-bottom: 0.5rem;
    display: inline-block;
  }

  .meta {
    display: flex;
    align-items: center;
    gap: 1rem;
    color: var(--text-secondary);
    font-size: 0.9rem;
    font-family: var(--font-mono);
  }

  .bullet {
    color: var(--divider-color);
  }

  .artist-actions {
    margin-top: 1.5rem;
  }

  .primary-btn {
    background: var(--text-primary);
    color: var(--surface-0);
    border: none;
    padding: 1rem 2.5rem;
    border-radius: 0;
    font-weight: 700;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .primary-btn:hover:not(:disabled) {
    background: var(--theme-accent);
    color: #fff;
    transform: translateY(-2px);
  }

  .primary-btn:disabled {
    opacity: 0.5;
    cursor: default;
    background: var(--surface-2);
    color: var(--text-tertiary);
  }

  .content {
    flex: 1;
  }

  .albums-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 2rem;
  }

  .album-card {
    background: transparent;
    border-radius: 0;
    border: 1px solid transparent;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.2s ease-out;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .album-card:hover {
    transform: translateY(-4px);
  }

  .album-card:hover .album-artwork-container {
    border-color: var(--text-primary);
  }

  .album-artwork-container {
    width: 100%;
    aspect-ratio: 1;
    background: var(--surface-2);
    border: 1px solid var(--divider-color);
    transition: border-color 0.2s;
  }

  .album-info {
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .album-title {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 1rem;
    color: var(--text-primary);
  }

  .album-meta {
    font-size: 0.8rem;
    color: var(--text-secondary);
    display: flex;
    gap: 0.5rem;
    font-family: var(--font-mono);
  }

  .loading-state, .empty-state, .error-state {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    height: 300px;
    font-size: 1.2rem;
    color: var(--text-secondary);
    gap: 1rem;
    font-family: var(--font-mono);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .error-state {
    color: var(--error);
  }

  .error-detail {
    font-size: 0.9rem;
    color: var(--text-tertiary);
    text-transform: none;
  }

  .error-state button {
    background: transparent;
    border: 1px solid var(--text-secondary);
    color: var(--text-primary);
    padding: 0.5rem 1.5rem;
    border-radius: 0;
    cursor: pointer;
    font-family: var(--font-mono);
    text-transform: uppercase;
  }
  
  .error-state button:hover {
    background: var(--text-primary);
    color: var(--surface-0);
  }
</style>
