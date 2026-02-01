<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { currentRoute, navigate } from '../state/route';
  import { playNow, addToQueue, playNowWithQueue } from '../state/playback';
  import { listAlbumTracksPage } from '../api/library';
  import { getArtworkBestForAlbum, getArtworkBytes } from '../api/artwork';
  import type { TrackRow, AlbumTrackCursor } from '../types/library';
  import { Play, Plus, Shuffle, MoreHorizontal, Search, ChevronDown } from '@lucide/svelte';

  let tracks = $state<TrackRow[]>([]);
  let loading = $state(true);
  let loadError = $state<string | null>(null);
  let nextCursor = $state<AlbumTrackCursor | undefined>(undefined);
  let hasLoaded = $state(false);
  let artworkUrl: string | null = $state(null);
  
  // Search and Sort state
  let searchQuery = $state('');
  let debouncedSearchQuery = $state('');
  let sortField = $state<'album' | 'title' | 'artist'>('album');
  let sortDirection = $state<'asc' | 'desc'>('asc');

  // Debounce search query
  let searchTimeout: any;
  $effect(() => {
    searchQuery; // dependency
    if (searchTimeout) clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      debouncedSearchQuery = searchQuery;
    }, 150);
  });
  
  // Get route params once on mount to avoid reactive loops
  let albumArtistSort = '';
  let albumTitleSort = '';
  
  // Derived album info
  let albumDisplay = $derived(tracks.length > 0 ? (tracks[0].album || 'Unknown Album') : 'Loading...');
  let artistDisplay = $derived(tracks.length > 0 ? (tracks[0].albumArtist || tracks[0].artist || 'Unknown Artist') : '');
  let artistInitial = $derived(artistDisplay ? artistDisplay[0].toUpperCase() : '?');
  let year = $derived(tracks.length > 0 ? tracks[0].year : undefined);
  let totalTracks = $derived(tracks.length);
  let totalDuration = $derived(tracks.reduce((acc, t) => acc + (t.durationMs || 0), 0));

  let sortLabel = $derived(sortField === 'album' ? 'Track #' : sortField === 'title' ? 'Title' : 'Artist');
  let directionLabel = $derived(sortDirection === 'asc' ? 'Ascending' : 'Descending');

  // Memoize sorted tracks
  let sortedTracks = $derived.by(() => {
    return [...tracks].sort((a, b) => {
      let cmp = 0;
      if (sortField === 'album') cmp = (a.trackNo || 0) - (b.trackNo || 0);
      else if (sortField === 'title') cmp = (a.title || '').localeCompare(b.title || '');
      else if (sortField === 'artist') cmp = (a.artist || '').localeCompare(b.artist || '');
      return sortDirection === 'asc' ? cmp : -cmp;
    });
  });

  // Derived filtered tracks from memoized sorted list
  let filteredTracks = $derived.by(() => {
    const q = debouncedSearchQuery.toLowerCase().trim();
    if (!q) return sortedTracks;
    return sortedTracks.filter(t => 
      (t.title || '').toLowerCase().includes(q) || 
      (t.artist || '').toLowerCase().includes(q)
    );
  });

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
        return `${hours} HR ${mins} MIN`;
    }
    return `${minutes} MIN`;
  }

  function handlePlayAlbum() {
    const validTracks = filteredTracks.filter(t => t.id && !t.isMissing);
    if (validTracks.length > 0) {
      const trackIds = validTracks.map(t => t.id);
      playNowWithQueue(trackIds, 0);
    }
  }

  function handleTrackDoubleClick(track: TrackRow) {
    if (track.isMissing || !track.id) return;
    const validTracks = filteredTracks.filter(t => t.id && !t.isMissing);
    const trackIds = validTracks.map(t => t.id);
    const startIndex = validTracks.findIndex(t => t.id === track.id);
    if (startIndex >= 0) {
      playNowWithQueue(trackIds, startIndex);
    }
  }

  function handleAddAlbumToQueue() {
    filteredTracks.forEach(track => {
      if (track.id && !track.isMissing) {
        addToQueue(track.id);
      }
    });
  }

  function toggleSortField() {
     if (sortField === 'album') sortField = 'title';
     else if (sortField === 'title') sortField = 'artist';
     else sortField = 'album';
  }

  function toggleSortDirection() {
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
  }
</script>

<div class="view-container">
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
      
      <div class="artist-row">
        <div class="artist-avatar">{artistInitial}</div>
        <span class="artist-name">{artistDisplay}</span>
        <button class="artist-follow-btn" title="Follow">
          <Plus size={14} />
        </button>
      </div>

      <div class="metadata-line">
        {#if year}
          <span class="meta-item">{year}</span>
          <span class="meta-separator">•</span>
        {/if}
        <span class="meta-item">{totalTracks} TRACKS</span>
        <span class="meta-separator">•</span>
        <span class="meta-item">{formatTotalDuration(totalDuration)}</span>
      </div>

      <div class="album-actions">
         <div class="actions-left">
            <button class="play-btn" onclick={handlePlayAlbum} disabled={filteredTracks.length === 0}>
               <Play size={18} fill="currentColor" /> Play
            </button>
            <button class="shuffle-btn" disabled={filteredTracks.length === 0}>
               <Shuffle size={18} /> Shuffle
            </button>
         </div>
         <div class="actions-right">
             <button class="add-btn" onclick={handleAddAlbumToQueue} title="Add to Queue" disabled={filteredTracks.length === 0}>
                <Plus size={20} />
             </button>
             <button class="more-btn" title="More options">
                <MoreHorizontal size={20} />
             </button>
         </div>
      </div>
    </div>
  </div>
  
  <div class="track-controls-row">
    <div class="search-field">
      <Search size={14} />
      <input type="text" placeholder="Search in album..." bind:value={searchQuery} />
    </div>
    <div class="sort-controls">
      <button class="sort-pill" onclick={toggleSortField}>{sortLabel} <ChevronDown size={12} /></button>
      <button class="sort-pill" onclick={toggleSortDirection}>{directionLabel} <ChevronDown size={12} /></button>
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
    {:else if filteredTracks.length === 0}
      <div class="empty-state">No tracks found</div>
    {:else}
      <table>
        <tbody>
          {#each filteredTracks as track}
            <tr class:missing={track.isMissing} ondblclick={() => handleTrackDoubleClick(track)}>
              <td class="col-num">{track.trackNo || '-'}</td>
              <td class="col-title">
                  <div class="title-cell">
                      {track.title || '—'}
                      {#if track.artist && track.artist !== artistDisplay}
                          <span class="track-artist">{track.artist}</span>
                      {/if}
                  </div>
              </td>
              <td class="col-duration">{formatDuration(track.durationMs)}</td>
              <td class="col-actions">
                 <button class="row-more-btn" title="Track options" onclick={(e) => { e.stopPropagation(); }}>
                    <MoreHorizontal size={16} />
                 </button>
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
    color: var(--text-primary);
    height: 100%;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2rem;
    background: transparent;
  }
  
  .album-header {
    display: flex;
    gap: 2rem;
    align-items: flex-end;
  }
  
  .artwork {
    width: 280px;
    height: 280px;
    border-radius: var(--artwork-radius-album-detail, 12px);
    box-shadow: none;
    flex-shrink: 0;
    overflow: hidden;
    background: var(--surface-2);
  }
  
  .artwork img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  
  .artwork-placeholder {
    width: 280px;
    height: 280px;
    background: var(--surface-2);
    border-radius: var(--artwork-radius-album-detail, 12px);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: none;
    flex-shrink: 0;
    color: var(--text-disabled);
  }
  
  .album-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding-bottom: 0.5rem;
  }
  
  h1 {
    font-size: 38px;
    font-weight: 700;
    margin: 0;
    line-height: 1.1;
    text-shadow: none;
    letter-spacing: -0.02em;
    color: var(--text-primary);
  }
  
  .artist-row {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 0.5rem 0;
  }

  .artist-avatar {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--surface-2);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .artist-name {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .artist-follow-btn {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--surface-2);
    border: none;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .artist-follow-btn:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  /* Metadata Line */
  .metadata-line {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 0.25rem;
  }

  .meta-item {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .meta-separator {
    color: var(--text-disabled);
    font-size: 10px;
  }

  .album-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 1.5rem;
  }
  
  .actions-left, .actions-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  /* Play Button (accent filled) */
  .play-btn {
    background: var(--theme-accent);
    color: #000;
    border: none;
    padding: 0 24px;
    height: 38px;
    border-radius: var(--radius-pill, 999px);
    font-weight: 600;
    font-size: 14px;
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    transition: transform 0.2s;
  }
  
  .play-btn:hover:not(:disabled) {
    transform: scale(1.05);
  }
  
  .play-btn:active:not(:disabled) {
    transform: scale(0.95);
  }

  /* Shuffle Button (outlined) */
  .shuffle-btn {
    background: var(--surface-2);
    color: var(--text-primary);
    border: none;
    padding: 0 20px;
    height: 38px;
    border-radius: var(--radius-pill, 999px);
    font-weight: 500;
    font-size: 14px;
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    transition: background 0.2s;
  }
  
  .shuffle-btn:hover:not(:disabled) {
    background: var(--surface-hover);
  }

  /* Circle buttons */
  .add-btn, .more-btn {
    width: 38px;
    height: 38px;
    border-radius: 50%;
    background: var(--surface-2);
    border: none;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background 0.2s;
  }
  
  .add-btn:hover, .more-btn:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  .play-btn:disabled, .shuffle-btn:disabled, .add-btn:disabled {
    opacity: 0.5;
    cursor: default;
    transform: none;
  }

  /* Track controls row */
  .track-controls-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border-dim);
    margin-bottom: 0.5rem;
  }

  .search-field {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--surface-2);
    border: none;
    border-radius: 6px;
    padding: 0 12px;
    height: 34px;
    width: 220px;
    color: var(--text-secondary);
  }

  .search-field input {
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-size: 13px;
    width: 100%;
  }
  
  .search-field input::placeholder {
    color: var(--text-disabled);
  }

  .sort-controls {
    display: flex;
    gap: 8px;
  }

  .sort-pill {
    display: flex;
    align-items: center;
    gap: 6px;
    background: var(--surface-2);
    border: none;
    border-radius: var(--radius-pill, 999px);
    padding: 0 14px;
    height: 30px;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s;
  }
  
  .sort-pill:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  .tracks-list {
    flex: 1;
  }
  
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-body, 14px);
  }
  
  td {
    padding: 0 var(--table-cell-gap, 12px);
    height: 56px; /* Increased from 36px */
    border-bottom: 1px solid var(--border-dim);
    color: var(--text-secondary);
    vertical-align: middle;
  }
  
  tr:hover {
    background: var(--surface-hover);
  }
  
  tr.missing {
    opacity: 0.5;
  }
  
  .col-num {
    width: 50px;
    text-align: right;
    color: var(--text-tertiary);
    font-size: 13px;
    font-weight: 500;
  }
  
  .col-title {
    color: var(--text-primary);
  }
  
  .title-cell {
    display: flex;
    flex-direction: column;
    justify-content: center;
    line-height: 1.3;
  }
  
  .track-artist {
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .col-duration {
    width: 80px;
    text-align: right;
    font-variant-numeric: tabular-nums;
    font-size: 13px;
    color: var(--text-tertiary);
  }
  
  .col-actions {
    width: 60px;
    text-align: right;
  }
  
  /* Row ellipsis button */
  .row-more-btn {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    width: 32px;
    height: 32px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s, background 0.2s, color 0.2s;
  }
  
  .row-more-btn:hover {
    background: var(--surface-2);
    color: var(--text-primary);
  }

  tr:hover .row-more-btn {
    opacity: 1;
  }

  .loading-state, .empty-state, .error-state {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    height: 200px;
    font-size: 1.2rem;
    color: var(--text-tertiary);
    gap: 1rem;
  }

  .error-state {
    color: #f66;
  }

  .error-detail {
    font-size: 0.9rem;
    color: var(--text-tertiary);
  }

  .error-state button {
    background: var(--surface-2);
    border: none;
    color: var(--text-primary);
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
  }

  .error-state button:hover {
    background: var(--surface-hover);
  }
</style>
