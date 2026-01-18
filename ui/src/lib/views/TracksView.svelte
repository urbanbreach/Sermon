<script lang="ts">
  import { onMount } from 'svelte';
  import { tracks, sortBy, sortDirection, scanStatus, scanProgress, setSortBy, toggleSortDirection, initLibrary } from '../state/library';
  import { playNow, addToQueue } from '../state/playback';
  import type { SortBy, TrackRow } from '../types/library';
  import TagEditor from '../components/TagEditor.svelte';

  // Tag editor state
  let editingTrack = $state<TrackRow | null>(null);
  let tagEditorOpen = $state(false);

  onMount(() => {
    initLibrary();
  });

  function formatDuration(ms?: number): string {
    if (!ms) return '--:--';
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

  function getSortIndicator(field: SortBy): string {
    if ($sortBy !== field) return '';
    return $sortDirection === 'asc' ? ' ▲' : ' ▼';
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
        <th class="sortable" on:click={() => handleSort('title')}>
          Title{getSortIndicator('title')}
        </th>
        <th class="sortable" on:click={() => handleSort('artist')}>
          Artist{getSortIndicator('artist')}
        </th>
        <th class="sortable" on:click={() => handleSort('album')}>
          Album{getSortIndicator('album')}
        </th>
        <th>Duration</th>
        <th>Sample Rate</th>
        <th>Bit Depth</th>
        <th>Actions</th>
      </tr>
    </thead>
    <tbody>
      {#each $tracks as track, i}
        <tr class:missing={track.is_missing} on:dblclick={() => !track.is_missing && playNow(track.id)}>
          <td>{i + 1}</td>
          <td>{track.title || 'Unknown'}</td>
          <td>{track.artist || 'Unknown'}</td>
          <td>{track.album || 'Unknown'}</td>
          <td>{formatDuration(track.duration_ms)}</td>
          <td>{track.sample_rate ? `${track.sample_rate / 1000}kHz` : '--'}</td>
          <td>{track.bit_depth ? `${track.bit_depth}-bit` : '--'}</td>
          <td class="actions">
            <button class="icon-btn" title="Play Now" on:click|stopPropagation={() => playNow(track.id)}>▶</button>
            <button class="icon-btn" title="Add to Queue" on:click|stopPropagation={() => addToQueue(track.id)}>+</button>
            <button class="icon-btn" title="Edit Tags" on:click|stopPropagation={() => openTagEditor(track)}>✎</button>
          </td>
        </tr>
      {:else}
        <tr>
          <td colspan="7" class="empty">No tracks found. Add a library folder in Settings.</td>
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
  }
  .header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
  }
  .scan-progress {
    background: var(--glass-bg);
    padding: 0.5rem 1rem;
    border-radius: var(--glass-radius);
    font-size: 0.85rem;
    color: #4af;
  }
  .tracks-table {
    width: 100%;
    border-collapse: collapse;
    text-align: left;
    font-size: 0.9rem;
  }
  th {
    border-bottom: 1px solid var(--glass-border);
    padding: 0.8rem;
    color: #888;
    font-weight: normal;
  }
  th.sortable {
    cursor: pointer;
  }
  th.sortable:hover {
    color: #fff;
  }
  td {
    padding: 0.8rem;
    border-bottom: 1px solid var(--glass-highlight);
    color: rgba(255,255,255,0.8);
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
  }
  .actions {
    display: flex;
    gap: 0.5rem;
  }
</style>
