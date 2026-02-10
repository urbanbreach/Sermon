<script lang="ts">
  import type { TrackTagSnapshot } from '../../types/library';

  interface Props {
    tracks: TrackTagSnapshot[];
  }

  let { tracks }: Props = $props();

  let isMulti = $derived(tracks.length > 1);
  let firstTrack = $derived(tracks[0]);

  function formatSize(sizeBytes?: number): string {
    if (!sizeBytes || sizeBytes <= 0) {
      return 'N/A';
    }

    const mb = sizeBytes / (1024 * 1024);
    return `${mb.toFixed(1)} MB`;
  }

  function formatDuration(durationMs?: number): string {
    if (!durationMs || durationMs <= 0) {
      return 'N/A';
    }

    const totalSeconds = Math.floor(durationMs / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes}:${String(seconds).padStart(2, '0')}`;
  }

  function formatDateTime(epochMs?: number): string {
    if (!epochMs || epochMs <= 0) {
      return 'N/A';
    }

    const date = new Date(epochMs);
    return date.toLocaleString();
  }

  function formatLoudnessDb(value?: number): string {
    if (value == null || !Number.isFinite(value)) {
      return 'N/A';
    }

    const sign = value > 0 ? '+' : '';
    return `${sign}${value.toFixed(2).replace('.', ',')} dB`;
  }

  const codecLabel = $derived(firstTrack?.codec ? String(firstTrack.codec) : 'N/A');
  const fileTypeLabel = $derived(
    firstTrack?.container ? `${String(firstTrack.container).toUpperCase()} audio file` : 'N/A',
  );
  const channelsLabel = $derived(firstTrack?.channels ? String(firstTrack.channels) : 'N/A');
  const sampleRateLabel = $derived(
    firstTrack?.sampleRate ? `${firstTrack.sampleRate.toLocaleString()} Hz` : 'N/A',
  );
  const volumeLevelingLabel = $derived(formatLoudnessDb(firstTrack?.loudnessDb));
  const locationValue = $derived(firstTrack?.path ?? 'N/A');
</script>

<div class="inspector-tab">
  {#if isMulti}
    <div class="multi-summary">{tracks.length} tracks selected</div>
  {/if}

  <div class="properties-grid">
    <div class="column">
      <div class="row"><span class="label">type:</span><div class="value">{fileTypeLabel}</div></div>
      <div class="row"><span class="label">tag version:</span><div class="value">N/A</div></div>
      <div class="row"><span class="label">size:</span><div class="value">{formatSize(firstTrack?.sizeBytes)}</div></div>
      <div class="row"><span class="label">duration:</span><div class="value">{formatDuration(firstTrack?.durationMs)}</div></div>
      <div class="row with-check">
        <input type="checkbox" aria-label="date added" disabled />
        <span class="label">date added:</span>
        <div class="value">N/A</div>
      </div>
      <div class="row"><span class="label">date modified:</span><div class="value">{formatDateTime(firstTrack?.mtimeMs)}</div></div>
      <div class="row"><span class="label">last played:</span><div class="value">Unknown</div></div>
      <div class="row"><span class="label">play count:</span><div class="value">0</div></div>
      <div class="row"><span class="label">skipped count:</span><div class="value">0</div></div>
    </div>

    <div class="column">
      <div class="row"><span class="label">encoded with:</span><div class="value">{codecLabel}</div></div>
      <div class="row"><span class="label">channels:</span><div class="value">{channelsLabel}</div></div>
      <div class="row"><span class="label">bitrate:</span><div class="value">N/A</div></div>
      <div class="row"><span class="label">sample rate:</span><div class="value">{sampleRateLabel}</div></div>
      <div class="row"><span class="label">volume leveling:</span><div class="value">{volumeLevelingLabel}</div></div>
    </div>
  </div>

  <div class="origin-block">
    <label class="origin-label">
      <input type="checkbox" aria-label="origin" disabled />
      <span>origin:</span>
    </label>
    <div class="origin-value"></div>
  </div>

  <div class="location-block">
    <div class="location-label">location:</div>
    <div class="location-row">
      <div class="location-value" title={locationValue}>{locationValue}</div>
      <button type="button" class="ellipsis" aria-label="browse location" disabled>...</button>
    </div>
  </div>
</div>

<style>
  .inspector-tab {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 10px 0;
    overflow: auto;
  }

  .multi-summary {
    color: var(--text-secondary);
    font-size: 13px;
  }

  .properties-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 18px;
  }

  .column {
    display: grid;
    gap: 4px;
    align-content: start;
  }

  .row {
    display: grid;
    grid-template-columns: 118px minmax(0, 1fr);
    gap: 6px;
    align-items: center;
  }

  .row.with-check {
    grid-template-columns: 14px 98px minmax(0, 1fr);
  }

  .row.with-check input {
    width: 14px;
    height: 14px;
    accent-color: var(--theme-accent);
  }

  .label {
    color: var(--text-secondary);
    font-size: 13px;
    text-transform: lowercase;
  }

  .value {
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-tertiary);
    min-height: 28px;
    padding: 5px 8px;
    font-size: 13px;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .origin-block,
  .location-block {
    border-top: 1px solid var(--divider-color);
    padding-top: 8px;
  }

  .origin-label {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-secondary);
    font-size: 13px;
    margin-bottom: 6px;
  }

  .origin-label input {
    width: 14px;
    height: 14px;
    accent-color: var(--theme-accent);
  }

  .origin-value {
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    min-height: 28px;
  }

  .location-label {
    color: var(--text-secondary);
    font-size: 13px;
    margin-bottom: 6px;
    text-transform: lowercase;
  }

  .location-row {
    display: grid;
    grid-template-columns: 1fr 28px;
    gap: 6px;
  }

  .location-value {
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    min-height: 28px;
    color: var(--text-secondary);
    padding: 5px 8px;
    font-size: 13px;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .ellipsis {
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-secondary);
    cursor: not-allowed;
  }
</style>
