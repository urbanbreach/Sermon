<script lang="ts">
  import { onMount } from 'svelte';
  import { currentRoute, goBack, canGoBack } from '../state/route';
  import { playNow, addToQueue } from '../state/playback';
  import { listAlbumTracksPage } from '../api/library';
  import type { TrackRow, AlbumTrackCursor } from '../types/library';

  let tracks = $state<TrackRow[]>([]);
  let loading = $state(false);
  let nextCursor = $state<AlbumTrackCursor | undefined>(undefined);
  
  // Derived state for album info from route
  let albumArtistSort = $derived(
    $currentRoute.name === 'album-detail' ? $currentRoute.albumArtistSort : ''
  );
  let albumTitleSort = $derived(
    $currentRoute.name === 'album-detail' ? $currentRoute.albumTitleSort : ''
  );

  // Derived album info from first track or route
  let albumDisplay = $derived(tracks.length > 0 ? tracks[0].album : albumTitleSort);
  let artistDisplay = $derived(tracks.length > 0 ? tracks[0].album_artist || tracks[0].artist : albumArtistSort);
  let year = $derived(tracks.length > 0 ? tracks[0].year : undefined);
  let totalTracks = $derived(tracks.length);
  let totalDuration = $derived(tracks.reduce((acc, t) => acc + (t.duration_ms || 0), 0));

  async function loadTracks(reset = true) {
    if (reset) {
      tracks = [];
      nextCursor = undefined;
    }
    
    if (!$currentRoute || $currentRoute.name !== 'album-detail') return;

    loading = true;
    try {
      const page = await listAlbumTracksPage(
        albumArtistSort, 
        albumTitleSort, 
        100, // Load enough for most albums
        nextCursor
      );
      
      if (reset) {
        tracks = page.items;
      } else {
        tracks = [...tracks, ...page.items];
      }
      nextCursor = page.nextCursor;
    } catch (e) {
      console.error('Failed to load album tracks:', e);
    } finally {
      loading = false;
    }
  }

  // React to route changes
  $effect(() => {
    if ($currentRoute.name === 'album-detail') {
      loadTracks(true);
    }
  });

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
</script>

<div class="view-container">
  <div class="top-bar">
    <button class="back-btn" onclick={goBack} disabled={!$canGoBack}>
      ← Back
    </button>
  </div>

  <div class="album-header">
    <div class="artwork-placeholder">
      <span class="note-icon">♪</span>
    </div>
    <div class="album-info">
      <h1>{albumDisplay}</h1>
      <div class="meta">
        <span class="artist">{artistDisplay}</span>
        {#if year}
          <span class="bullet">•</span>
          <span class="year">{year}</span>
        {/if}
        <span class="bullet">•</span>
        <span class="stats">{totalTracks} tracks, {formatTotalDuration(totalDuration)}</span>
      </div>
      <div class="album-actions">
         <button class="primary-btn" onclick={() => tracks.length > 0 && playNow(tracks[0].id)}>
            Play
         </button>
      </div>
    </div>
  </div>
  
  <div class="tracks-list">
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
          <tr class:missing={track.is_missing} ondblclick={() => !track.is_missing && playNow(track.id)}>
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
                 <button class="icon-btn" title="Play Now" onclick={(e) => { e.stopPropagation(); playNow(track.id); }}>▶</button>
                 <button class="icon-btn" title="Add to Queue" onclick={(e) => { e.stopPropagation(); addToQueue(track.id); }}>+</button>
               </div>
            </td>
          </tr>
        {:else}
          {#if !loading}
             <tr><td colspan="4" class="empty">No tracks found</td></tr>
          {/if}
        {/each}
        {#if loading}
            <tr><td colspan="4" class="loading">Loading...</td></tr>
        {/if}
      </tbody>
    </table>
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
    background: linear-gradient(135deg, #333 0%, #111 100%);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 8px 24px rgba(0,0,0,0.5);
    flex-shrink: 0;
  }
  
  .note-icon {
    font-size: 4rem;
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
  
  .primary-btn:hover {
    transform: scale(1.05);
  }
  
  .primary-btn:active {
    transform: scale(0.95);
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
  
  .empty, .loading {
    text-align: center;
    padding: 3rem;
    color: #666;
  }
</style>
