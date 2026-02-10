<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { VList } from 'virtua/svelte';
  import { tracks, sortBy, scanStatus, scanProgress, setSortBy, toggleSortDirection, initLibrary, loadTracks } from '../state/library';
  import { addToQueue, addToQueueNext, currentTrack, playNowWithQueue } from '../state/playback';
  import { selectLibrarySummaryFromTracks, selectTrackSummary } from '../state/albumSelection';
  import { setViewTitle } from '../state/viewTitle';
  import type { SortBy, TrackRow } from '../types/library';
  import { openTagEditorWindow } from '../state/tagEditorWindow';
  import SkeletonRow from '../components/SkeletonRow.svelte';
  import * as ContextMenu from '../components/primitives/ContextMenu.svelte';
  import { ChevronUp, Play, ListMusic, Check, Square } from '@lucide/svelte';
  import { fadeIn } from '../utils/animations';

  // Multi-select state
  let selectionMode = $state(false);
  let selectedIds = $state<Set<number>>(new Set());
  let hasTrackSelection = $state(false);

  // Right-click context menu state
  let rightClickTrackId = $state<number | null>(null);

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

  $effect(() => {
    if ($tracks.length === 0) return;
    if (hasTrackSelection) return;
    selectLibrarySummaryFromTracks($tracks);
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

  async function openTagEditor(ids: number[]) {
    await openTagEditorWindow(ids);
  }

  function handleTrackDoubleClick(track: TrackRow, trackIndex: number) {
    if (track.isMissing) return;
    const validTracks = $tracks.filter(t => !t.isMissing);
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

  // Selection functions
  function enterSelectionMode() {
    selectionMode = true;
  }

  function toggleSelection(id: number) {
    const newSet = new Set(selectedIds);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    selectedIds = newSet;
  }

  function handleRowClick(e: MouseEvent, track: TrackRow) {
    selectTrackSummary(track);
    hasTrackSelection = true;

    if (selectionMode) {
      e.preventDefault();
      toggleSelection(track.id);
    }
  }

  function playSelected() {
    if (selectedIds.size === 0) return;
    const ids = Array.from(selectedIds);
    playNowWithQueue(ids, 0);
    clearSelection();
  }

  function queueSelected() {
    if (selectedIds.size === 0) return;
    for (const id of selectedIds) {
      addToQueue(id);
    }
    clearSelection();
  }

  function clearSelection() {
    selectedIds = new Set();
    selectionMode = false;
  }

  // Context menu handlers
  function handleContextMenu(e: MouseEvent, track: TrackRow) {
    e.preventDefault();
    
    // Right-click selection rule:
    // If right-clicked row is NOT selected → replace selection with that row
    // If right-clicked row IS selected → keep current selection (multi-select)
    if (!selectedIds.has(track.id)) {
      selectedIds = new Set([track.id]);
    }
    
    rightClickTrackId = track.id;
  }

  function getOrderedSelectedIds(): number[] {
    // Return selected IDs in the order they appear in $tracks
    return $tracks.filter(t => selectedIds.has(t.id)).map(t => t.id);
  }

  async function handlePlayNowSelected() {
    const orderedIds = getOrderedSelectedIds();
    if (orderedIds.length > 0) {
      await playNowWithQueue(orderedIds, 0);
    }
  }

  async function handleQueueNextSelected() {
    const orderedIds = getOrderedSelectedIds();
    if (orderedIds.length > 0) {
      await addToQueueNext(orderedIds);
    }
  }

  async function handleQueueLastSelected() {
    const orderedIds = getOrderedSelectedIds();
    for (const id of orderedIds) {
      await addToQueue(id);
    }
  }

  async function handleEditSelected() {
    const ids = Array.from(selectedIds);
    if (ids.length > 0) {
      await openTagEditor(ids);
    }
  }
</script>

<div class="view-container" use:fadeIn={{ duration: 300 }}>
  {#if $scanStatus === 'scanning'}
    <div class="scan-progress">
      Scanning... {$scanProgress.scanned}/{$scanProgress.total}
    </div>
  {/if}

  {#if selectionMode && selectedIds.size > 0}
    <div class="selection-bar" data-testid="tracks-selection-bar">
      <span class="selection-count">{selectedIds.size} selected</span>
      <div class="selection-actions">
        <button class="selection-btn primary" onclick={playSelected}>Play Now</button>
        <button class="selection-btn" onclick={queueSelected}>Add to Queue</button>
        <button class="selection-btn" onclick={() => openTagEditor(Array.from(selectedIds))}>Edit Tags</button>
        <button class="selection-btn cancel" onclick={clearSelection}>Cancel</button>
      </div>
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
      <div class="col-actions header-cell">
        {#if !selectionMode}
          <button class="select-btn" onclick={enterSelectionMode}>Select</button>
        {:else}
          <button class="select-btn active" onclick={clearSelection}>Done</button>
        {/if}
      </div>
    </div>

    {#if !initialLoadComplete && $tracks.length === 0}
      <div class="loading-state">
        {#each Array(10) as _, idx (idx)}
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
            <ContextMenu.Root>
              <ContextMenu.Trigger>
                {#snippet child({ props })}
                  <div 
                    {...props}
                    class="track-row"
                    class:missing={track.isMissing} 
                    class:playing={$currentTrack?.id === track.id}
                    class:selected={selectedIds.has(track.id)}
                    class:selection-mode={selectionMode}
                    ondblclick={() => handleTrackDoubleClick(track, i)}
                    onclick={(e) => handleRowClick(e, track)}
                    oncontextmenu={(e) => handleContextMenu(e, track)}
                    role="row"
                    tabindex="0"
                    aria-rowindex={i + 1}
                    aria-selected={selectedIds.has(track.id)}
                  >
                    <div class="col-index cell">
                      {#if selectionMode}
                        <button 
                          class="checkbox-btn"
                          onclick={(e) => { e.stopPropagation(); toggleSelection(track.id); }}
                        >
                          {#if selectedIds.has(track.id)}
                            <Check size={14} />
                          {:else}
                            <Square size={14} />
                          {/if}
                        </button>
                      {:else}
                        <div class="row-index">
                          <span class="number">{i + 1}</span>
                          <button class="play-icon" onclick={(e) => handlePlayIconClick(e, track, i)}>
                            <Play size={12} fill="currentColor" />
                          </button>
                        </div>
                      {/if}
                    </div>
                    <div class="col-name cell">{track.title || '—'}</div>
                    <div class="col-artist cell">{track.artist || '—'}</div>
                    <div class="col-album cell">{track.album || '—'}</div>
                    <div class="col-genre cell">{track.genre || '—'}</div>
                    <div class="col-time cell">{formatDuration(track.durationMs)}</div>
                  </div>
                {/snippet}
              </ContextMenu.Trigger>
              <ContextMenu.Portal>
                <ContextMenu.Content class="dropdown-content" data-testid="tracks-context-menu">
                  <ContextMenu.Item class="dropdown-item" onclick={handlePlayNowSelected}>Play Now</ContextMenu.Item>
                  <ContextMenu.Item class="dropdown-item" onclick={handleQueueNextSelected}>Queue Next</ContextMenu.Item>
                  <ContextMenu.Item class="dropdown-item" onclick={handleQueueLastSelected}>Queue Last</ContextMenu.Item>
                  <ContextMenu.Separator class="dropdown-separator" />
                  <ContextMenu.Item class="dropdown-item" onclick={handleEditSelected}>Edit</ContextMenu.Item>
                </ContextMenu.Content>
              </ContextMenu.Portal>
            </ContextMenu.Root>
          {/snippet}
        </VList>
      </div>
    {/if}
  </div>
</div>

<style>
  .view-container {
    padding: 1rem;
    padding-top: 12px;
    padding-right: 0;
    padding-bottom: 0;
    color: #fff;
    height: 100%;
    overflow-y: hidden;
    background: transparent;
    display: flex;
    flex-direction: column;
  }
  .scan-progress {
    background: var(--surface-1);
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    padding: 0.5rem 1rem;
    border-radius: var(--radius-md);
    font-size: 0.85rem;
    color: #d61e30;
    border: 1px solid var(--divider-color);
    box-shadow: var(--shadow-2);
    margin-bottom: 1rem;
  }

  /* Selection Bar */
  .selection-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--surface-2);
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    padding: 8px 16px;
    border-radius: var(--radius-md, 8px);
    border: 1px solid var(--border-subtle);
    box-shadow: var(--shadow-sm);
    margin-bottom: 12px;
    margin-right: 1rem;
  }

  .selection-count {
    font-size: 13px;
    color: var(--text-primary);
    font-weight: 500;
  }

  .selection-actions {
    display: flex;
    gap: 8px;
  }

  .selection-btn {
    background: var(--surface-1);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 12px;
    cursor: pointer;
    transition: all var(--motion-fast) var(--ease-out);
  }

  .selection-btn:hover {
    background: var(--surface-hover);
    border-color: var(--border-hover);
  }

  .selection-btn.primary {
    background: var(--text-primary);
    border-color: var(--text-primary);
    color: var(--bg-root);
  }

  .selection-btn.primary:hover {
    background: rgba(255, 255, 255, 0.9);
    border-color: rgba(255, 255, 255, 0.9);
  }

  .selection-btn.cancel {
    background: transparent;
    border-color: transparent;
    color: var(--text-secondary);
  }

  .selection-btn.cancel:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  /* Select button in header */
  .select-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 11px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    transition: all var(--motion-fast) var(--ease-out);
  }

  .select-btn:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .select-btn.active {
    color: var(--text-primary);
    background: var(--surface-2);
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
    border-bottom: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.06));
    padding-bottom: 8px;
    padding-right: 1rem;
    margin-bottom: 0;
    font-size: 13px;
    color: var(--text-secondary);
    font-weight: 500;
    letter-spacing: 0.01em;
  }

  .header-cell {
    padding: 0 var(--table-cell-gap, 12px);
    display: flex;
    align-items: center;
    height: 32px;
    position: relative;
  }

  .sortable {
    cursor: pointer;
    transition: color 0.2s ease;
  }
  .sortable:hover {
    color: var(--text-primary);
  }
  
  .sortable.sorted {
    color: var(--text-primary);
  }
  .sortable.sorted::after {
    content: '';
    position: absolute;
    bottom: -8px;
    left: 12px;
    right: 12px;
    height: 1px;
    background: var(--text-primary);
    opacity: 0.5;
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
    grid-template-columns: 40px minmax(200px, 1.4fr) 1fr 1fr 1fr 75px;
    height: 44px;
    border-bottom: 1px solid transparent;
    font-size: 14px;
    color: var(--text-secondary);
    transition: background 0.15s ease;
    align-items: center;
    padding-right: 1rem;
  }

  .track-row:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }
  
  .track-row:focus-visible {
    background: var(--surface-hover);
    box-shadow: inset 0 0 0 1px var(--text-secondary);
    outline: none;
  }
  
  .track-row.missing .cell {
    color: var(--text-disabled);
    font-style: italic;
  }

  /* Selection mode styling */
  .track-row.selection-mode {
    cursor: pointer;
  }

  .track-row.selected {
    background: var(--surface-2, rgba(255, 255, 255, 0.08));
    color: var(--text-primary);
  }

  .track-row.selected:hover {
    background: var(--surface-hover, rgba(255, 255, 255, 0.12));
  }

  .checkbox-btn {
    background: transparent;
    border: none;
    padding: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary);
    border-radius: 4px;
    transition: all var(--motion-fast) var(--ease-out);
  }

  .checkbox-btn:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .track-row.selected .checkbox-btn {
    color: var(--text-primary);
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
    font-family: var(--font-mono, monospace);
    font-size: 13px;
    opacity: 0.7;
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
    color: var(--text-tertiary);
    flex: 1;
  }

  .empty-state :global(svg) {
    opacity: 0.3;
  }

  .empty-title {
    font-size: 16px;
    font-weight: 500;
    margin: 0;
    color: var(--text-secondary);
  }

  .empty-hint {
    font-size: 13px;
    margin: 0;
    color: var(--text-tertiary);
  }
  
  /* Row number / play column */
  .col-index {
    text-align: center;
    color: var(--text-tertiary);
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
    color: var(--text-primary);
  }

  .track-row:hover .row-index .number {
    display: none;
  }

  .track-row:hover .play-icon {
    display: flex;
  }

  /* Currently playing indicator */
  .track-row.playing {
    background: transparent;
    color: var(--text-primary);
  }

  .track-row.playing .col-index {
    position: relative;
    color: var(--text-primary);
  }
  .track-row.playing .col-index::before {
    content: '';
    position: absolute;
    left: 4px;
    top: 50%;
    transform: translateY(-50%);
    width: 4px;
    height: 4px;
    background: var(--text-primary);
    border-radius: 50%;
    box-shadow: 0 0 8px var(--text-primary);
  }


</style>
