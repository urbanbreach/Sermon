<script lang="ts">
  import { onMount } from 'svelte';
  import { currentRoute, goBack, canGoBack } from '../state/route';
  import { playNow, addToQueue } from '../state/playback';
  import { listAlbumTracksPage } from '../api/library';
  import type { TrackRow, AlbumTrackCursor } from '../types/library';

  let tracks = $state<TrackRow[]>([]);
  let loading = $state(true);
  let loadError = $state<string | null>(null);
  let nextCursor = $state<AlbumTrackCursor | undefined>(undefined);
  let hasLoaded = $state(false);
  
  // Get route params once on mount to avoid reactive loops
  let albumArtistSort = '';
  let albumTitleSort = '';
  
  // Derived album info from first track (with proper casing)
  let albumDisplay = $derived(tracks.length > 0 ? (tracks[0].album || 'Unknown Album') : 'Loading...');
  let artistDisplay = $derived(tracks.length > 0 ? (tracks[0].album_artist || tracks[0].artist || 'Unknown Artist') : '');
  let year = $derived(tracks.length > 0 ? tracks[0].year : undefined);
  let totalTracks = $derived(tracks.length);
  let totalDuration = $derived(tracks.reduce((acc, t) => acc + (t.duration_ms || 0), 0));

  onMount(() => {
    // Extract route params on mount only
    const route = $currentRoute;
    if (route.name === 'album-detail') {
      albumArtistSort = route.albumArtistSort;
      albumTitleSort = route.albumTitleSort;
      loadTracks();
    }
  });

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
    } catch (e) {
      console.error('Failed to load album tracks:', e);
      loadError = e instanceof Error ? e.message : 'Failed to load tracks';
    } finally {
      loading = false;
    }
  }

  function formatDuration(ms?: number): string {
    if (!ms) return '--:--';
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
      ← Back
    </button>
  </div>

  <div class="album-header">
    <div class="artwork-placeholder">
      <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10"/>
        <circle cx="12" cy="12" r="3"/>
      </svg>
    </div>
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
                      {track.title || 'Unknown Title'}
                      {#if track.artist && track.artist !== artistDisplay}
                          <span class="track-artist">{track.artist}</span>
                      {/if}
                  </div>
              </td>
              <td class="col-duration">{formatDuration(track.duration_ms)}</td>
              <td class="col-actions">
                 <div class="row-actions">
                   <button class="icon-btn" title="Play Now" onclick={(e) => { e.stopPropagation(); if (track.id) playNow(track.id); }}>▶</button>
                   <button class="icon-btn" title="Add to Queue" onclick={(e) => { e.stopPropagation(); if (track.id) addToQueue(track.id); }}>+</button>
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
  
  .album-header {
    display: flex;
    gap: 2rem;
    align-items: flex-end;
  }
  
  .artwork-placeholder {
    width: 200px;
    height: 200px;
    background: linear-gradient(135deg, #2a2a2a 0%, #1a1a1a 100%);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 8px 24px rgba(0,0,0,0.5);
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
    font-size: 2.5rem;
    font-weight: 700;
    margin: 0;
    line-height: 1.1;
  }
  
  .meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: #aaa;
    font-size: 1rem;
  }
  
  .artist {
    color: #fff;
    font-weight: 600;
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
    transition: transform 0.1s;
  }
  
  .primary-btn:hover:not(:disabled) {
    transform: scale(1.05);
  }
  
  .primary-btn:active:not(:disabled) {
    transform: scale(0.95);
  }

  .primary-btn:disabled, .secondary-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .secondary-btn {
    background: transparent;
    color: #fff;
    border: 1px solid rgba(255,255,255,0.3);
    padding: 0.8rem 2rem;
    border-radius: 30px;
    font-weight: 600;
    font-size: 1rem;
    cursor: pointer;
    transition: background-color 0.2s, border-color 0.2s;
  }

  .secondary-btn:hover:not(:disabled) {
    background: rgba(255,255,255,0.1);
    border-color: rgba(255,255,255,0.5);
  }

  .tracks-list {
    flex: 1;
  }
  
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.95rem;
  }
  
  th {
    text-align: left;
    color: #888;
    font-weight: normal;
    padding: 0.8rem;
    border-bottom: 1px solid rgba(255,255,255,0.1);
  }
  
  td {
    padding: 0.8rem;
    border-bottom: 1px solid rgba(255,255,255,0.05);
    color: #ddd;
  }
  
  tr:hover {
    background: rgba(255,255,255,0.05);
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
  }
  
  .track-artist {
    font-size: 0.8em;
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
  }
  
  .icon-btn:hover {
    background: rgba(255,255,255,0.1);
    border-color: #fff;
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
