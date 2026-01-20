<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { currentRoute, goBack, canGoBack, navigate } from '../state/route';
  import { playNow, addToQueue } from '../state/playback';
  import { listAlbumTracksPage } from '../api/library';
  import { getArtworkBestForAlbum, getArtworkBytes } from '../api/artwork';
  import type { TrackRow, AlbumTrackCursor } from '../types/library';
  import { ChevronLeft, Play, Plus } from '@lucide/svelte';

  let tracks = $state<TrackRow[]>([]);
  let loading = $state(true);
  let loadError = $state<string | null>(null);
  let nextCursor = $state<AlbumTrackCursor | undefined>(undefined);
  let hasLoaded = $state(false);
  let artworkUrl: string | null = $state(null);
  
  // Get route params once on mount to avoid reactive loops
  let albumArtistSort = '';
  let albumTitleSort = '';
  
  // Derived album info from first track (with proper casing)
  let albumDisplay = $derived(tracks.length > 0 ? (tracks[0].album || 'Unknown Album') : 'Loading...');
  let artistDisplay = $derived(tracks.length > 0 ? (tracks[0].album_artist || tracks[0].artist || 'Unknown Artist') : '');
  let year = $derived(tracks.length > 0 ? tracks[0].year : undefined);
  let totalTracks = $derived(tracks.length);
  let totalDuration = $derived(tracks.reduce((acc, t) => acc + (t.duration_ms || 0), 0));

  // Handler for track marked missing event
  function handleTrackMarkedMissing(event: CustomEvent) {
    const { track_id } = event.detail;
    // Check if the missing track is in our current list
    const trackIndex = tracks.findIndex(t => t.id === track_id);
    if (trackIndex !== -1) {
      // Reload tracks - the backend will filter out missing tracks
      loadTracks();
    }
  }

  onMount(() => {
    // Extract route params on mount only
    const route = $currentRoute;
    if (route.name === 'album-detail') {
      albumArtistSort = route.albumArtistSort;
      albumTitleSort = route.albumTitleSort;
      loadTracks();
      loadArtwork();
    }
    
    // Listen for track marked missing events
    window.addEventListener('sermon:track-marked-missing', handleTrackMarkedMissing as EventListener);
  });

  onDestroy(() => {
    window.removeEventListener('sermon:track-marked-missing', handleTrackMarkedMissing as EventListener);
  });

  async function loadArtwork() {
    if (!albumArtistSort || !albumTitleSort) return;

    try {
      const best = await getArtworkBestForAlbum(albumArtistSort, albumTitleSort);
      if (best.source !== 'none' && best.cacheKey && best.mime) {
        const bytes = await getArtworkBytes(best.cacheKey, best.mime);
        artworkUrl = `data:${bytes.mime};base64,${bytes.bytesBase64}`;
      } else {
        artworkUrl = null;
      }
    } catch (e) {
      console.error('Failed to load album artwork:', e);
      artworkUrl = null;
    }
  }

  async function loadTracks() {
    if (!albumArtistSort || !albumTitleSort) return;
    
    loading = true;
    loadError = null;
    
    try {
      const page = await listAlbumTracksPage(
        albumArtistSort, 
        albumTitleSort, 
        100,
        undefined
      );
      
      tracks = page.items;
      nextCursor = page.nextCursor;
      hasLoaded = true;
      
      // If no tracks remain (all marked missing), navigate back to albums
      if (tracks.length === 0 && hasLoaded) {
        navigate({ name: 'albums' });
      }
    } catch (e) {
      console.error('Failed to load album tracks:', e);
      loadError = e instanceof Error ? e.message : 'Failed to load tracks';
    } finally {
      loading = false;
    }
  }

  function formatDuration(ms?: number): string {
    if (!ms) return '—';
    const minutes = Math.floor(ms / 60000);
    const seconds = ((ms % 60000) / 1000).toFixed(0);
    return minutes + ":" + (Number(seconds) < 10 ? '0' : '') + seconds;
  }
  
  function formatTotalDuration(ms: number): string {
    const minutes = Math.floor(ms / 60000);
    if (minutes > 60) {
        const hours = Math.floor(minutes / 60);
        const mins = minutes % 60;
        return `${hours} hr ${mins} min`;
    }
    return `${minutes} min`;
  }

  function handlePlayAlbum() {
    if (tracks.length > 0 && tracks[0].id) {
      playNow(tracks[0].id);
    }
  }

  function handleAddAlbumToQueue() {
    tracks.forEach(track => {
      if (track.id && !track.is_missing) {
        addToQueue(track.id);
      }
    });
  }
</script>

<div class="view-container">
  <div class="top-bar">
    <button class="back-btn" onclick={goBack} disabled={!$canGoBack}>
      <ChevronLeft size={16} /> Back
    </button>
  </div>

  <div class="album-header">
    {#if artworkUrl}
      <div class="artwork">
        <img src={artworkUrl} alt="Album artwork" />
      </div>
    {:else}
      <div class="artwork-placeholder"></div>
    {/if}
    <div class="album-info">
      <h1>{albumDisplay}</h1>
      <div class="meta">
        <span class="artist">{artistDisplay}</span>
        {#if year}
          <span class="bullet">•</span>
          <span class="year">{year}</span>
        {/if}
        {#if hasLoaded}
          <span class="bullet">•</span>
          <span class="stats">{totalTracks} tracks, {formatTotalDuration(totalDuration)}</span>
        {/if}
      </div>
      <div class="album-actions">
         <button class="primary-btn" onclick={handlePlayAlbum} disabled={tracks.length === 0}>
            Play
         </button>
         <button class="secondary-btn" onclick={handleAddAlbumToQueue} disabled={tracks.length === 0}>
            Add to Queue
         </button>
      </div>
    </div>
  </div>
  
  <div class="tracks-list">
    {#if loading && !hasLoaded}
      <div class="loading-state">Loading tracks...</div>
    {:else if loadError}
      <div class="error-state">
        <p>Failed to load tracks</p>
        <p class="error-detail">{loadError}</p>
        <button onclick={loadTracks}>Retry</button>
      </div>
    {:else if tracks.length === 0}
      <div class="empty-state">No tracks found for this album</div>
    {:else}
      <table>
        <thead>
          <tr>
            <th class="col-num">#</th>
            <th class="col-title">Title</th>
            <th class="col-duration">Duration</th>
            <th class="col-actions"></th>
          </tr>
        </thead>
        <tbody>
          {#each tracks as track}
            <tr class:missing={track.is_missing} ondblclick={() => !track.is_missing && track.id && playNow(track.id)}>
              <td class="col-num">{track.track_no || '-'}</td>
              <td class="col-title">
                  <div class="title-cell">
                      {track.title || '—'}
                      {#if track.artist && track.artist !== artistDisplay}
                          <span class="track-artist">{track.artist}</span>
                      {/if}
                  </div>
              </td>
              <td class="col-duration">{formatDuration(track.duration_ms)}</td>
              <td class="col-actions">
                 <div class="row-actions">
                   <button class="icon-btn" title="Play Now" onclick={(e) => { e.stopPropagation(); if (track.id) playNow(track.id); }}>
                     <Play size={14} fill="currentColor" />
                   </button>
                   <button class="icon-btn" title="Add to Queue" onclick={(e) => { e.stopPropagation(); if (track.id) addToQueue(track.id); }}>
                     <Plus size={14} />
                   </button>
                 </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
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
    background: transparent;
  }
  
  .top-bar {
    margin-bottom: 0.5rem;
  }
  
  .back-btn {
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    color: #ccc;
    cursor: pointer;
    font-size: 0.9rem;
    padding: 0.5rem 1rem;
    border-radius: 20px;
    transition: all 0.2s;
    box-shadow: var(--glass-shadow);
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .back-btn:hover {
    color: #fff;
    background: var(--glass-border);
    transform: translateX(-2px);
  }
  
  .back-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
  
  .album-header {
    display: flex;
    gap: 2rem;
    align-items: flex-end;
  }
  
  .artwork {
    width: 220px;
    height: 220px;
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.5);
    flex-shrink: 0;
    overflow: hidden;
  }
  
  .artwork img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  
  .artwork-placeholder {
    width: 220px;
    height: 220px;
    background: linear-gradient(135deg, rgba(255,255,255,0.05) 0%, rgba(255,255,255,0.02) 100%);
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 8px 32px rgba(0,0,0,0.5);
    flex-shrink: 0;
    color: #444;
  }
  
  .album-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding-bottom: 1rem;
  }
  
  h1 {
    font-size: var(--text-album-title, 28px);
    font-weight: 700;
    margin: 0;
    line-height: 1.1;
    text-shadow: 0 2px 8px rgba(0,0,0,0.6);
  }
  
  .meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: #aaa;
    font-size: var(--text-body, 14px);
  }
  
  .artist {
    color: #fff;
    font-weight: 600;
    text-shadow: 0 1px 3px rgba(0,0,0,0.5);
    font-size: var(--text-section, 18px);
  }
  
  .bullet {
    color: #666;
  }
  
  .album-actions {
    margin-top: 1rem;
    display: flex;
    gap: 1rem;
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
    transition: transform 0.2s cubic-bezier(0.25, 0.46, 0.45, 0.94), box-shadow 0.2s;
    box-shadow: 0 4px 12px rgba(255,255,255,0.2);
  }
  
  .primary-btn:hover:not(:disabled) {
    transform: scale(1.05);
    box-shadow: 0 6px 16px rgba(255,255,255,0.3);
  }
  
  .primary-btn:active:not(:disabled) {
    transform: scale(0.95);
  }

  .primary-btn:disabled, .secondary-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .secondary-btn {
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    color: #fff;
    border: 1px solid var(--glass-border);
    padding: 0.8rem 2rem;
    border-radius: 30px;
    font-weight: 600;
    font-size: 1rem;
    cursor: pointer;
    transition: background 0.2s, border-color 0.2s, transform 0.2s;
  }

  .secondary-btn:hover:not(:disabled) {
    background: var(--glass-border);
    border-color: rgba(255,255,255,0.5);
    transform: scale(1.05);
  }

  .tracks-list {
    flex: 1;
  }
  
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-body, 14px);
  }
  
  th {
    text-align: left;
    color: #aaa;
    font-weight: 500;
    padding: 0 var(--table-cell-gap, 12px);
    height: var(--table-header-height, 28px);
    font-size: var(--text-table-header, 13px);
    border-bottom: 1px solid var(--glass-border);
  }
  
  td {
    padding: 0 var(--table-cell-gap, 12px);
    height: var(--table-row-height, 36px);
    border-bottom: 1px solid rgba(255,255,255,0.05);
    color: #ddd;
    vertical-align: middle;
  }
  
  tr:hover {
    background: var(--glass-highlight);
  }
  
  tr.missing {
    opacity: 0.5;
  }
  
  .col-num {
    width: 50px;
    text-align: right;
    color: #666;
  }
  
  .col-title {
    color: #fff;
  }
  
  .title-cell {
    display: flex;
    flex-direction: column;
    justify-content: center;
    line-height: 1.2;
  }
  
  .track-artist {
    font-size: 0.85em;
    color: #888;
  }

  .col-duration {
    width: 80px;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  
  .col-actions {
    width: 80px;
    text-align: right;
  }
  
  .row-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    opacity: 0;
    transition: opacity 0.2s;
    align-items: center;
    height: 100%;
  }
  
  tr:hover .row-actions {
    opacity: 1;
  }
  
  .icon-btn {
    background: transparent;
    border: 1px solid rgba(255,255,255,0.2);
    color: #fff;
    width: 28px;
    height: 28px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    font-size: 0.8rem;
    transition: all 0.2s;
  }
  
  .icon-btn:hover {
    background: rgba(255,255,255,0.1);
    border-color: #fff;
    transform: scale(1.1);
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

  .error-state button:hover {
    background: rgba(255,255,255,0.2);
  }
</style>
