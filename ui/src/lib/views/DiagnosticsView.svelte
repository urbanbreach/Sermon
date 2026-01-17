<script lang="ts">
  import { audioDebug } from '../state/playback';
  import { onMount } from 'svelte';
  import { getLibraryStats } from '../api/library';
  import type { LibraryStats } from '../types/library';

  let debug = $derived($audioDebug);
  let stats: LibraryStats | null = $state(null);

  onMount(async () => {
    try {
      stats = await getLibraryStats();
    } catch (e) {
      console.error('Failed to load library stats:', e);
    }
  });

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatDate(ms?: number): string {
    if (!ms) return 'Never';
    return new Date(ms).toLocaleString();
  }
</script>

<div class="view-container">
  <h1>Audio Diagnostics</h1>

  <div class="grid">
    <!-- Status Card -->
    <div class="card status-card">
      <h2>Playback Status</h2>
      <div class="status-indicator" class:bit-perfect={debug?.bit_perfect === 'yes'}>
        <div class="dot"></div>
        <span class="label">Bit-Perfect:</span>
        <span class="value">{debug?.bit_perfect === 'yes' ? 'YES' : 'NO'}</span>
      </div>
      {#if debug?.bit_perfect === 'no'}
        <div class="reason">
          Reason: {debug?.bit_perfect_reason || 'Unknown'}
        </div>
      {/if}
    </div>

    <!-- Configuration -->
    <div class="card">
      <h2>Configuration</h2>
      <div class="row">
        <span class="label">Output Mode</span>
        <span class="value">{debug?.output_mode || '-'}</span>
      </div>
      <div class="row">
        <span class="label">Policy</span>
        <span class="value">{debug?.policy || '-'}</span>
      </div>
      <div class="row">
        <span class="label">Exclusive Active</span>
        <span class="value highlight">{debug?.exclusive_active ? 'YES' : 'NO'}</span>
      </div>
    </div>

    <!-- Processing -->
    <div class="card">
      <h2>Processing</h2>
      <div class="row">
        <span class="label">Conversion</span>
        <span class="value" class:warn={debug?.conversion !== 'none'}>
          {debug?.conversion || 'none'}
        </span>
      </div>
      <div class="row">
        <span class="label">Gain Mode</span>
        <span class="value">{debug?.gain_mode || '-'}</span>
      </div>
      <div class="row">
        <span class="label">Fade Enabled</span>
        <span class="value">{debug?.fade_enabled ? 'Yes' : 'No'}</span>
      </div>
    </div>

    <!-- Formats -->
    <div class="card full-width">
      <h2>Format Pipeline</h2>
      <div class="pipeline">
        <div class="stage">
          <h3>Source</h3>
          <div class="details">
            <div>{debug?.decode_format?.sample_rate || 0} Hz</div>
            <div>{debug?.decode_format?.bit_depth || 0}-bit</div>
            <div>{debug?.decode_format?.channels || 0} ch</div>
            <div class="sub">{debug?.decode_format?.codec || '-'}</div>
          </div>
        </div>

        <div class="arrow">→</div>

        <div class="stage">
          <h3>Output</h3>
          <div class="details">
            <div>{debug?.output_format?.sample_rate || 0} Hz</div>
            <div>
              {#if debug?.output_format?.valid_bits && debug?.output_format?.valid_bits !== debug?.output_format?.bit_depth}
                {debug?.output_format?.valid_bits}-bit (in {debug?.output_format?.bit_depth}-bit container)
              {:else}
                {debug?.output_format?.bit_depth || 0}-bit
              {/if}
            </div>
            <div>{debug?.output_format?.channels || 0} ch</div>
            <div class="sub">WASAPI</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Library Stats -->
    <div class="card">
      <h2>Library Stats</h2>
      {#if stats}
        <div class="row">
          <span class="label">Tracks</span>
          <span class="value">{stats.trackCount.toLocaleString()}</span>
        </div>
        <div class="row">
          <span class="label">Albums</span>
          <span class="value">{stats.albumCount.toLocaleString()}</span>
        </div>
        <div class="row">
          <span class="label">Artists</span>
          <span class="value">{stats.artistCount.toLocaleString()}</span>
        </div>
        <div class="row">
          <span class="label">Database Size</span>
          <span class="value">{formatBytes(stats.dbSizeBytes)}</span>
        </div>
        <div class="row">
          <span class="label">Last Scan</span>
          <span class="value">{formatDate(stats.lastScanCompletedMs)}</span>
        </div>
      {:else}
        <div class="loading">Loading...</div>
      {/if}
    </div>
  </div>
</div>

<style>
  .view-container {
    padding: 2rem;
    color: #fff;
    height: 100%;
    overflow-y: auto;
  }

  h1 {
    margin-bottom: 2rem;
    font-size: 1.5rem;
  }

  h2 {
    font-size: 1rem;
    color: #888;
    margin-bottom: 1rem;
    border-bottom: 1px solid #333;
    padding-bottom: 0.5rem;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1.5rem;
  }

  .card {
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
    border-radius: 8px;
    padding: 1.5rem;
  }

  .full-width {
    grid-column: 1 / -1;
  }

  .row {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.75rem;
    font-size: 0.95rem;
  }

  .label {
    color: #aaa;
  }

  .value {
    font-family: monospace;
    font-weight: bold;
  }

  .value.highlight {
    color: #4af;
  }

  .value.warn {
    color: #fa4;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }

  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #444;
  }

  .status-indicator.bit-perfect .dot {
    background: #4f4;
    box-shadow: 0 0 10px rgba(68, 255, 68, 0.4);
  }

  .status-indicator:not(.bit-perfect) .dot {
    background: #fa4;
  }

  .reason {
    font-size: 0.85rem;
    color: #fa4;
    margin-top: 0.5rem;
    padding: 0.5rem;
    background: rgba(255, 170, 68, 0.1);
    border-radius: 4px;
  }

  .pipeline {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 2rem;
    padding: 1rem 0;
  }

  .stage {
    text-align: center;
    background: rgba(255, 255, 255, 0.05);
    padding: 1rem 2rem;
    border-radius: 8px;
    min-width: 120px;
  }

  .stage h3 {
    font-size: 0.9rem;
    color: #888;
    margin-bottom: 0.5rem;
    text-transform: uppercase;
    letter-spacing: 1px;
  }

  .details {
    font-family: monospace;
    font-size: 1.1rem;
    line-height: 1.4;
  }

  .sub {
    font-size: 0.8rem;
    color: #666;
    margin-top: 0.25rem;
  }

  .arrow {
    font-size: 2rem;
    color: #444;
  }
</style>
