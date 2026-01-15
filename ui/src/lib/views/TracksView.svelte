<script lang="ts">
  import { onMount } from 'svelte';
  import { tracks, sortBy, sortDirection, scanStatus, scanProgress, setSortBy, toggleSortDirection, initLibrary } from '../state/library';
  import type { SortBy } from '../types/library';

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
      </tr>
    </thead>
    <tbody>
      {#each $tracks as track, i}
        <tr class:missing={track.is_missing}>
          <td>{i + 1}</td>
          <td>{track.title || 'Unknown'}</td>
          <td>{track.artist || 'Unknown'}</td>
          <td>{track.album || 'Unknown'}</td>
          <td>{formatDuration(track.duration_ms)}</td>
          <td>{track.sample_rate ? `${track.sample_rate / 1000}kHz` : '--'}</td>
          <td>{track.bit_depth ? `${track.bit_depth}-bit` : '--'}</td>
        </tr>
      {:else}
        <tr>
          <td colspan="7" class="empty">No tracks found. Add a library folder in Settings.</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

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
</style>
