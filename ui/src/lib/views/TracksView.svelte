<script lang="ts">
  import { onMount } from 'svelte';
  import { tracks, sortBy, sortDirection, scanStatus, scanProgress, setSortBy, toggleSortDirection, initLibrary } from '../state/library';
  import { playNow, addToQueue } from '../state/playback';
  import type { SortBy, TrackRow } from '../types/library';
  import TagEditor from '../components/TagEditor.svelte';
  import { Play, Plus, Pencil, ChevronUp, ChevronDown } from '@lucide/svelte';

  // Tag editor state
  let editingTrack = $state<TrackRow | null>(null);
  let tagEditorOpen = $state(false);

  onMount(() => {
    initLibrary();
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
</script>

<div class="view-container">
  <div class="header">
    <h1>Tracks</h1>
    {#if $scanStatus === 'scanning'}
      <div class="scan-progress">
        Scanning... {$scanProgress.scanned}/{$scanProgress.total}
      </div>
    {/if}
  </div>
  
  <table class="tracks-table">
    <thead>
      <tr>
        <th>#</th>
        <th class="sortable" onclick={() => handleSort('title')}>
          <div class="header-cell">
            Title 
            {#if $sortBy === 'title'}
              {#if $sortDirection === 'asc'}
                <ChevronUp size={12} />
              {:else}
                <ChevronDown size={12} />
              {/if}
            {/if}
          </div>
        </th>
        <th class="sortable" onclick={() => handleSort('artist')}>
          <div class="header-cell">
            Artist 
            {#if $sortBy === 'artist'}
              {#if $sortDirection === 'asc'}
                <ChevronUp size={12} />
              {:else}
                <ChevronDown size={12} />
              {/if}
            {/if}
          </div>
        </th>
        <th class="sortable" onclick={() => handleSort('album')}>
          <div class="header-cell">
            Album 
            {#if $sortBy === 'album'}
              {#if $sortDirection === 'asc'}
                <ChevronUp size={12} />
              {:else}
                <ChevronDown size={12} />
              {/if}
            {/if}
          </div>
        </th>
        <th class="col-numeric">Duration</th>
        <th class="col-numeric">Sample Rate</th>
        <th class="col-numeric">Bit Depth</th>
        <th>Actions</th>
      </tr>
    </thead>
    <tbody>
      {#each $tracks as track, i}
        <tr class:missing={track.is_missing} ondblclick={() => !track.is_missing && playNow(track.id)}>
          <td>{i + 1}</td>
          <td>{track.title || '—'}</td>
          <td>{track.artist || '—'}</td>
          <td>{track.album || '—'}</td>
          <td class="col-numeric">{formatDuration(track.duration_ms)}</td>
          <td class="col-numeric">{track.sample_rate ? `${track.sample_rate / 1000}kHz` : '—'}</td>
          <td class="col-numeric">{track.bit_depth ? `${track.bit_depth}-bit` : '—'}</td>
          <td class="actions">
            <button class="icon-btn" title="Play Now" onclick={(e) => { e.stopPropagation(); playNow(track.id); }}>
              <Play size={14} fill="currentColor" />
            </button>
            <button class="icon-btn" title="Add to Queue" onclick={(e) => { e.stopPropagation(); addToQueue(track.id); }}>
              <Plus size={14} />
            </button>
            <button class="icon-btn" title="Edit Tags" onclick={(e) => { e.stopPropagation(); openTagEditor(track); }}>
              <Pencil size={14} />
            </button>
          </td>
        </tr>
      {:else}
        <tr>
          <td colspan="8" class="empty">No tracks found. Add a library folder in Settings.</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<TagEditor track={editingTrack} open={tagEditorOpen} onclose={closeTagEditor} />

<style>
  .view-container {
    padding: 2rem;
    color: #fff;
    height: 100%;
    overflow-y: auto;
    background: transparent;
  }
  .header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
  }
  .header h1 {
    font-size: 24px;
    font-weight: 600;
    text-shadow: 0 2px 4px rgba(0,0,0,0.5);
    margin: 0;
  }
  .scan-progress {
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    padding: 0.5rem 1rem;
    border-radius: var(--glass-radius);
    font-size: 0.85rem;
    color: #4af;
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
  }
  .tracks-table {
    width: 100%;
    border-collapse: collapse;
    text-align: left;
    font-size: var(--text-body, 14px);
  }
  th {
    border-bottom: 1px solid var(--glass-border);
    padding: 0 var(--table-cell-gap, 12px);
    height: var(--table-header-height, 28px);
    color: #aaa;
    font-weight: 500;
    font-size: var(--text-table-header, 13px);
  }
  th.sortable {
    cursor: pointer;
  }
  th.sortable:hover {
    color: #fff;
  }
  .header-cell {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  td {
    padding: 0 var(--table-cell-gap, 12px);
    height: var(--table-row-height, 36px);
    border-bottom: 1px solid rgba(255,255,255,0.05);
    color: rgba(255,255,255,0.9);
    font-size: var(--text-body, 14px);
  }
  .col-numeric {
    font-variant-numeric: tabular-nums;
  }
  tr {
    transition: background 0.15s ease;
  }
  tr:hover {
    background: var(--glass-highlight);
  }
  tr.missing td {
    color: rgba(255,255,255,0.4);
    font-style: italic;
  }
  .empty {
    text-align: center;
    color: #666;
    padding: 2rem;
  }
  .icon-btn {
    background: transparent;
    border: 1px solid var(--glass-border);
    color: #fff;
    border-radius: 4px;
    width: 24px;
    height: 24px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    font-size: 0.8rem;
    transition: all 0.2s;
  }
  .icon-btn:hover {
    background: rgba(255,255,255,0.2);
    border-color: #fff;
    transform: scale(1.1);
  }
  .icon-btn:focus-visible {
    outline: 2px solid #4af;
    outline-offset: 2px;
  }
  .actions {
    display: flex;
    gap: 0.5rem;
  }
</style>
