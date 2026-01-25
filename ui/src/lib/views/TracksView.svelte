<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { VList } from 'virtua/svelte';
  import { tracks, sortBy, sortDirection, scanStatus, scanProgress, setSortBy, toggleSortDirection, initLibrary, loadTracks } from '../state/library';
  import { playNow, addToQueue, currentTrack, playNowWithQueue } from '../state/playback';
  import { setViewTitle } from '../state/viewTitle';
  import type { SortBy, TrackRow } from '../types/library';
  import TagEditor from '../components/TagEditor.svelte';
  import SkeletonRow from '../components/SkeletonRow.svelte';
  import { ChevronUp, Play, ListMusic } from '@lucide/svelte';
  import { fadeIn } from '../utils/animations';

  // Tag editor state
  let editingTrack = $state<TrackRow | null>(null);
  let tagEditorOpen = $state(false);

  // Ellipsis menu state
  let openMenuTrackId = $state<number | null>(null);
  let initialLoadComplete = $state(false);

  onMount(async () => {
    setViewTitle('Tracks');
    initLibrary();
    if ($tracks.length === 0) {
      await loadTracks();
    }
    initialLoadComplete = true;
  });

  onDestroy(() => {
    setViewTitle('');
  });

  function formatDuration(ms?: number): string {
    if (!ms) return '—';
    const minutes = Math.floor(ms / 60000);
    const seconds = ((ms % 60000) / 1000).toFixed(0);
    return minutes + ":" + (Number(seconds) < 10 ? '0' : '') + seconds;
  }

  function handleSort(field: SortBy) {
    if ($sortBy === field) {
      toggleSortDirection();
    } else {
      setSortBy(field);
    }
  }

  function openTagEditor(track: TrackRow) {
    editingTrack = track;
    tagEditorOpen = true;
  }

  function closeTagEditor() {
    tagEditorOpen = false;
    editingTrack = null;
  }

  function toggleMenu(trackId: number, e: MouseEvent) {
    e.stopPropagation();
    openMenuTrackId = openMenuTrackId === trackId ? null : trackId;
  }

  function closeMenu() {
    openMenuTrackId = null;
  }

  function handleTrackDoubleClick(track: TrackRow, trackIndex: number) {
    if (track.is_missing) return;
    const validTracks = $tracks.filter(t => !t.is_missing);
    const trackIds = validTracks.map(t => t.id);
    const startIndex = validTracks.findIndex(t => t.id === track.id);
    if (startIndex >= 0 && trackIds.length > 0) {
      playNowWithQueue(trackIds, startIndex);
    }
  }

  function handlePlayIconClick(e: MouseEvent, track: TrackRow, trackIndex: number) {
    e.stopPropagation();
    handleTrackDoubleClick(track, trackIndex);
  }
</script>

<svelte:window onclick={closeMenu} />

<div class="view-container" use:fadeIn={{ duration: 300 }}>
  {#if $scanStatus === 'scanning'}
    <div class="scan-progress">
      Scanning... {$scanProgress.scanned}/{$scanProgress.total}
    </div>
  {/if}
  
  <div class="tracks-list-container">
    <div class="tracks-header">
      <div class="col-index header-cell">#</div>
      <div class="sortable col-name header-cell" class:sorted={$sortBy === 'title'} onclick={() => handleSort('title')} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && handleSort('title')}>
        <div class="cell-content">
          Name
          {#if $sortBy === 'title'}
            <ChevronUp size={10} />
          {/if}
        </div>
      </div>
      <div class="sortable col-artist header-cell" class:sorted={$sortBy === 'artist'} onclick={() => handleSort('artist')} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && handleSort('artist')}>
        <div class="cell-content">
          Artist
          {#if $sortBy === 'artist'}
            <ChevronUp size={10} />
          {/if}
        </div>
      </div>
      <div class="sortable col-album header-cell" class:sorted={$sortBy === 'album'} onclick={() => handleSort('album')} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && handleSort('album')}>
        <div class="cell-content">
          Album
          {#if $sortBy === 'album'}
            <ChevronUp size={10} />
          {/if}
        </div>
      </div>
      <div class="col-genre header-cell">Genre</div>
      <div class="col-time header-cell">Time</div>
      <div class="col-actions header-cell"></div>
    </div>

    {#if !initialLoadComplete && $tracks.length === 0}
      <div class="loading-state">
        {#each Array(10) as _}
          <SkeletonRow />
        {/each}
      </div>
    {:else if $tracks.length === 0}
      <div class="empty-state" use:fadeIn={{ duration: 300 }}>
        <ListMusic size={48} strokeWidth={1} />
        <p class="empty-title">No tracks found</p>
        <p class="empty-hint">Add a library folder in Preferences to see your music</p>
      </div>
    {:else}
      <div class="list-wrapper">
        <VList data={$tracks} getKey={(t) => t.id} itemSize={40} bufferSize={200}>
          {#snippet children(track: TrackRow, i: number)}
            <div 
              class="track-row"
              class:missing={track.is_missing} 
              class:playing={$currentTrack?.id === track.id}
              ondblclick={() => handleTrackDoubleClick(track, i)}
              role="row"
              tabindex="0"
              aria-rowindex={i + 1}
            >
              <div class="col-index cell">
                <div class="row-index">
                  <span class="number">{i + 1}</span>
                  <button class="play-icon" onclick={(e) => handlePlayIconClick(e, track, i)}>
                    <Play size={12} fill="currentColor" />
                  </button>
                </div>
              </div>
              <div class="col-name cell">{track.title || '—'}</div>
              <div class="col-artist cell">{track.artist || '—'}</div>
              <div class="col-album cell">{track.album || '—'}</div>
              <div class="col-genre cell">{track.genre || '—'}</div>
              <div class="col-time cell">{formatDuration(track.duration_ms)}</div>
              <div class="col-actions cell">
                <button 
                  class="ellipsis-btn" 
                  onclick={(e) => toggleMenu(track.id, e)}
                  aria-haspopup="true"
                  aria-expanded={openMenuTrackId === track.id}
                >
                  ⋯
                </button>
                {#if openMenuTrackId === track.id}
                  <div class="row-menu" onclick={(e) => e.stopPropagation()} role="menu" tabindex="0">
                    <button onclick={() => { playNow(track.id); closeMenu(); }}>Play Now</button>
                    <button onclick={() => { addToQueue(track.id); closeMenu(); }}>Add to Queue</button>
                    <button onclick={() => { openTagEditor(track); closeMenu(); }}>Edit Tags...</button>
                  </div>
                {/if}
              </div>
            </div>
          {/snippet}
        </VList>
      </div>
    {/if}
  </div>
</div>

<TagEditor track={editingTrack} open={tagEditorOpen} onclose={closeTagEditor} />

<style>
  .view-container {
    padding: 1rem;
    padding-top: 12px;
    padding-right: 0;
    color: #fff;
    height: 100%;
    overflow-y: hidden;
    background: transparent;
    display: flex;
    flex-direction: column;
  }
  .scan-progress {
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    padding: 0.5rem 1rem;
    border-radius: var(--glass-radius);
    font-size: 0.85rem;
    color: #d61e30;
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    margin-bottom: 1rem;
  }
  
  .tracks-list-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .tracks-header {
    display: grid;
    grid-template-columns: 40px minmax(200px, 1.4fr) 1fr 1fr 1fr 75px 40px;
    border-bottom: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
    padding-bottom: 4px;
    padding-right: 1rem;
    margin-bottom: 4px;
    font-size: var(--text-table-header, 13px);
    color: #989898;
    font-weight: 400;
  }

  .header-cell {
    padding: 0 var(--table-cell-gap, 12px);
    display: flex;
    align-items: center;
    height: var(--table-header-height, 28px);
    position: relative;
  }

  .sortable {
    cursor: pointer;
  }
  .sortable:hover {
    color: #bbb;
  }
  
  .sortable.sorted {
    color: var(--text-primary);
  }
  .sortable.sorted::after {
    content: '';
    position: absolute;
    bottom: -5px;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--theme-accent);
    border-radius: 1px;
  }
  
  .cell-content {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  
  .list-wrapper {
    flex: 1;
    min-height: 0;
  }

  /* Row styling */
  .track-row {
    display: grid;
    grid-template-columns: 40px minmax(200px, 1.4fr) 1fr 1fr 1fr 75px 40px;
    height: 40px;
    border-bottom: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
    font-size: var(--text-body, 14px);
    color: var(--text-primary);
    transition: background var(--motion-fast) var(--ease-out);
    align-items: center;
    padding-right: 1rem;
  }

  .track-row:hover {
    background: var(--surface-hover);
  }
  
  .track-row:focus-visible {
    background: var(--surface-hover);
    box-shadow: inset 0 0 0 1px var(--accent-medium);
    outline: none;
  }
  
  .track-row.missing .cell {
    color: var(--text-disabled);
    font-style: italic;
  }
  
  .cell {
    padding: 0 var(--table-cell-gap, 12px);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Column adjustments */
  .col-time {
    text-align: right;
    justify-content: flex-end;
    font-variant-numeric: tabular-nums;
  }
  .header-cell.col-time {
    justify-content: flex-end;
  }
  
  .col-actions {
    text-align: center;
    justify-content: center;
    position: relative;
    overflow: visible; /* For dropdown */
  }
  
  /* Loading/Empty States */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 64px 48px;
    color: rgba(255, 255, 255, 0.4);
    flex: 1;
  }

  .empty-state :global(svg) {
    opacity: 0.3;
  }

  .empty-title {
    font-size: 16px;
    font-weight: 500;
    margin: 0;
    color: rgba(255, 255, 255, 0.5);
  }

  .empty-hint {
    font-size: 13px;
    margin: 0;
    color: rgba(255, 255, 255, 0.35);
  }
  
  /* Ellipsis button */
  .ellipsis-btn {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    font-size: 18px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    letter-spacing: 2px;
    opacity: 0;
    transition: all var(--motion-fast) var(--ease-out);
  }
  .track-row:hover .ellipsis-btn {
    opacity: 1;
  }
  .ellipsis-btn[aria-expanded="true"],
  .ellipsis-btn:focus-visible {
    opacity: 1;
  }
  .ellipsis-btn:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }
  .ellipsis-btn:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
  }
  
  /* Row number / play column */
  .col-index {
    text-align: center;
    color: rgba(255,255,255,0.4);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    justify-content: center;
    display: flex;
    align-items: center;
  }

  .row-index {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
  }

  .play-icon {
    background: transparent;
    border: none;
    padding: 0;
    cursor: pointer;
    display: none;
    align-items: center;
    justify-content: center;
    color: #fff;
  }

  .track-row:hover .row-index .number {
    display: none;
  }

  .track-row:hover .play-icon {
    display: flex;
  }

  /* Currently playing indicator */
  .track-row.playing {
    background: var(--accent-weak);
  }

  .track-row.playing .col-index {
    position: relative;
  }
  .track-row.playing .col-index::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
    background: var(--theme-accent);
    border-radius: 0 2px 2px 0;
  }

  /* Dropdown menu */
  .row-menu {
    position: absolute;
    right: 0;
    top: 100%;
    background: rgba(30, 30, 34, 0.95);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid var(--glass-border);
    border-radius: 8px;
    padding: 4px 0;
    min-width: 140px;
    z-index: 100;
    box-shadow: var(--shadow-3);
  }
  .row-menu button {
    display: block;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    border: none;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
    font-size: 13px;
    transition: background var(--motion-fast) var(--ease-out);
  }
  .row-menu button:hover {
    background: var(--surface-hover);
  }
</style>
