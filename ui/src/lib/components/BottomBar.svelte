<script lang="ts">
  import { 
    currentTrack, playbackState, togglePlayPause, next, previous, 
    volume, setVolume, playbackError, switchToDefault, progress, audioDebug,
    positionMs, durationMs, queue, currentTrackFull
  } from '../state/playback';
  import { seek } from '../state/playback';
  import { selectedSummary } from '../state/albumSelection';
  import { pressScale } from '../utils/animations';
  import { SkipBack, Pause, Play, SkipForward, Volume2 } from '@lucide/svelte';
  import WaveformSeekbar from './WaveformSeekbar.svelte';
  import { waveformPeaks, loadWaveformPeaks, clearWaveformPeaks } from '../state/waveform';
  import DiagnosticsHoverCard from './DiagnosticsHoverCard.svelte';

  // Load waveform when track changes
  $effect(() => {
    if ($currentTrack?.id) {
      loadWaveformPeaks($currentTrack.id);
    } else {
      clearWaveformPeaks();
    }
  });

  let isUnity = $derived($audioDebug?.policy === 'strict' && ($audioDebug?.output_mode === 'exclusive' || $audioDebug?.output_mode === 'asio'));

  let isEmpty = $derived(!$currentTrack);

  // Visual state for display and waveform rendering
  let visualProgress = $derived($progress);
  let visualPositionMs = $derived($positionMs);

  // Time formatting for progress display
  function formatTime(ms: number): string {
    if (!ms || ms < 0) return '0:00';
    const mins = Math.floor(ms / 60000);
    const secs = Math.floor((ms % 60000) / 1000);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function formatQueueDuration(ms: number): string {
    if (!ms || ms < 0) return '0:00';
    const totalSecs = Math.floor(ms / 1000);
    const hours = Math.floor(totalSecs / 3600);
    const mins = Math.floor((totalSecs % 3600) / 60);
    const secs = totalSecs % 60;
    if (hours > 0) {
      return `${hours}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function formatMegabytes(bytes: number): string {
    if (!bytes || bytes <= 0) return '0,0 MB';
    return `${(bytes / (1024 * 1024)).toFixed(1).replace('.', ',')} MB`;
  }

  function formatNowPlayingLabel(
    title?: string,
    artist?: string,
    selection?: {
      kind: 'album' | 'library' | 'track';
      label: string;
      artist: string;
    } | null
  ): string {
    const normalizedTitle = title?.trim();
    const normalizedArtist = artist?.trim();

    if (normalizedArtist && normalizedTitle) return `${normalizedArtist} - ${normalizedTitle}`;
    if (normalizedTitle) return normalizedTitle;
    if (normalizedArtist) return normalizedArtist;

    if (selection?.kind === 'track' || selection?.kind === 'album') {
      const selectedLabel = selection.label.trim();
      const selectedArtist = selection.artist.trim();
      if (selectedArtist && selectedLabel) return `${selectedArtist} - ${selectedLabel}`;
      if (selectedLabel) return selectedLabel;
    }

    if (selection?.kind === 'library') {
      return 'Library - All Tracks';
    }

    return 'Nothing Playing';
  }

  function buildSelectedDetails(
    selection: {
      kind: 'album' | 'library' | 'track';
      trackCount: number;
      sizeBytes: number;
      durationMs: number;
      loading: boolean;
    } | null,
    fallbackTrackSizeBytes: number,
    fallbackTrackDurationMs: number
  ): string {
    if (selection) {
      const sizeLabel = selection.loading && selection.sizeBytes <= 0
        ? '… MB'
        : formatMegabytes(selection.sizeBytes);
      const durationLabel = selection.loading && selection.durationMs <= 0
        ? '…'
        : formatQueueDuration(selection.durationMs);

      if (selection.kind === 'library') {
        const countLabel = selection.trackCount === 1 ? '1 track' : `${selection.trackCount} tracks`;
        return `selected: ${countLabel}, ${sizeLabel}, ${durationLabel}`;
      }

      if (selection.kind === 'track') {
        return `selected: 1 track, ${sizeLabel}, ${durationLabel}`;
      }

      return `selected: 1 album, ${sizeLabel}, ${durationLabel}`;
    }

    if (fallbackTrackSizeBytes > 0 || fallbackTrackDurationMs > 0) {
      return `selected: 1 track, ${formatMegabytes(fallbackTrackSizeBytes)}, ${formatQueueDuration(fallbackTrackDurationMs)}`;
    }

    return 'selected: none';
  }

  let currentTimeDisplay = $derived(formatTime(visualPositionMs));
  let remainingTimeDisplay = $derived($durationMs > 0 ? `-${formatTime($durationMs - visualPositionMs)}` : '-0:00');
  let queueDurationMs = $derived($queue.reduce((total, item) => total + (item.duration_ms || 0), 0));
  let nowPlayingLabel = $derived(
    formatNowPlayingLabel(
      $currentTrack?.title,
      $currentTrack?.artist,
      $selectedSummary
    )
  );
  let selectedDetails = $derived(
    buildSelectedDetails(
      $selectedSummary,
      $currentTrackFull?.sizeBytes || 0,
      $currentTrackFull?.durationMs || 0
    )
  );
  let queueDetails = $derived(`queued: ${formatQueueDuration(queueDurationMs)}`);
  let rightMetaLabel = $derived(`${selectedDetails} / ${queueDetails}`);
  let detailTrackDisplay = $derived(nowPlayingLabel.trim().length > 0 ? nowPlayingLabel : 'Loading track details…');
  let detailMetaDisplay = $derived(rightMetaLabel.trim().length > 0 ? rightMetaLabel : 'selected: … / queued: …');


</script>

{#if $playbackError}
  <div class="error-banner">
     <span>Error: {$playbackError.message}</span>
     {#if $playbackError.action === 'switch_to_default'}
        <button onclick={switchToDefault}>Switch to Default Device</button>
     {/if}
  </div>
{/if}

<div class="bottom-bar-wrapper">
  <div class="waveform-detail-strip">
    <span class="detail-track">{detailTrackDisplay}</span>
    <span class="detail-meta">{detailMetaDisplay}</span>
  </div>

  <div class="bottom-bar">
    <div class="waveform-single-row" class:empty={isEmpty}>
      <!-- Diagnostics Pill -->
      <DiagnosticsHoverCard />

      <!-- Transport Controls -->
      <div class="waveform-transport">
        <button class="ctrl-btn" onclick={previous} title="Previous" use:pressScale={{ scale: 0.95 }}><SkipBack size={18} fill="currentColor" /></button>
        <button class="ctrl-btn play waveform-play" onclick={togglePlayPause} title={$playbackState === 'playing' ? 'Pause' : 'Play'} use:pressScale>
          {#if $playbackState === 'playing'}
            <Pause size={20} fill="currentColor" />
          {:else}
            <Play size={20} fill="currentColor" />
          {/if}
        </button>
        <button class="ctrl-btn" onclick={next} title="Next" use:pressScale={{ scale: 0.95 }}><SkipForward size={18} fill="currentColor" /></button>
      </div>

      <!-- Waveform Seekbar (center, flexible) -->
      <div class="waveform-center">
        <WaveformSeekbar 
          peaks={$waveformPeaks.peaksU8} 
          trackId={$waveformPeaks.trackId}
          progress={visualProgress} 
          durationMs={$durationMs}
          on:seek={(e) => seek(e.detail.ms)}
        />
      </div>

      <!-- Time Labels + Volume -->
      <div class="waveform-right">
        <span class="time-compact">{currentTimeDisplay}</span>
        <span class="time-separator">/</span>
        <span class="time-compact remaining">{remainingTimeDisplay}</span>
        <div class="volume-control-compact">
          {#if isUnity}
            <span class="vol-label-unity">Unity</span>
          {:else}
            <Volume2 size={16} />
          {/if}
          <input 
            type="range" 
            min="0" 
            max="1" 
            step="0.01" 
            value={isUnity ? 1.0 : $volume} 
            disabled={isUnity}
            aria-label="Volume"
            oninput={(e) => setVolume(e.currentTarget.valueAsNumber)} 
          />
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  /* Error banner - positioned above bottom bar */
  .error-banner {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    transform: translateY(-100%);
    background: #d32f2f;
    color: white;
    padding: 8px 16px;
    font-size: 13px;
    display: flex;
    justify-content: center;
    gap: 12px;
    align-items: center;
    z-index: 10;
  }
  .error-banner button {
    background: white;
    color: #d32f2f;
    border: none;
    padding: 4px 10px;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 600;
    font-size: 12px;
  }

  .bottom-bar-wrapper {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    box-sizing: border-box;
    height: var(--layout-player-height, 64px);
    padding-top: var(--layout-bottom-bar-meta-height, 14px);
    z-index: 40;
    overflow: visible;
    isolation: isolate;
    display: flex;
    flex-direction: column;
    border-top: 0.5px solid rgba(255, 255, 255, 0.06);
    box-shadow: none;

    /* Unified glass material — matches top-bar.np-transparent */
    background: rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(20px) saturate(1.2);
    -webkit-backdrop-filter: blur(20px) saturate(1.2);
  }

  /* Transparent — inherits glass from .bottom-bar-wrapper */
  .bottom-bar {
    height: 100%;
    width: 100%;
    display: flex;
    flex-direction: column;
    background: transparent;
  }

  .ctrl-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px;
    border-radius: 6px;
    transition: all var(--motion-fast) var(--ease-out);
  }
  .ctrl-btn:hover { 
    color: var(--text-primary); 
    background: var(--surface-hover); 
  }
  .ctrl-btn:active { transform: scale(0.95); }
  .ctrl-btn.play { 
    width: 36px; 
    height: 36px; 
    background: rgba(255, 255, 255, 0.12);
    border-radius: 50%;
    box-shadow: var(--shadow-1), 0 0 0 1px rgba(255, 255, 255, 0.06);
    color: var(--text-primary);
  }
  .ctrl-btn.play:hover { 
    background: rgba(255, 255, 255, 0.18); 
    box-shadow: var(--shadow-2), 0 0 0 1px rgba(255, 255, 255, 0.10);
    transform: scale(1.05);
  }
  .ctrl-btn.play:active {
    transform: scale(0.95);
    box-shadow: var(--shadow-inset);
  }

  .vol-label-unity {
    font-size: 10px;
    color: var(--text-tertiary); /* Cider neutral gray - not accent color */
    text-transform: uppercase;
    font-weight: 600;
    letter-spacing: 0.5px;
  }

  .waveform-detail-strip {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: var(--layout-bottom-bar-meta-height, 14px);
    min-height: var(--layout-bottom-bar-meta-height, 14px);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 0 12px;
    font-size: 10px;
    line-height: var(--layout-bottom-bar-meta-height, 14px);
    color: var(--text-tertiary);
    border-bottom: none;
    background: transparent;
    pointer-events: none;
  }

  .detail-track {
    display: block;
    flex: 1;
    min-width: 0;
    transform: translateY(0.5px);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .detail-meta {
    display: block;
    flex-shrink: 0;
    max-width: 60%;
    transform: translateY(0.5px);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    text-align: right;
  }

  /* ===== WAVEFORM SINGLE-ROW MODE ===== */
  .waveform-single-row {
    height: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 12px;
  }

  .waveform-single-row.empty .waveform-transport {
    opacity: 0.5;
    pointer-events: none;
  }

  /* Waveform Transport */
  .waveform-transport {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .ctrl-btn.waveform-play {
    width: 34px;
    height: 34px;
  }

  /* Waveform Center (flexible) */
  .waveform-center {
    flex: 1;
    min-width: 180px;
    height: 30px;
  }

  /* Waveform Right (time + volume) */
  .waveform-right {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .time-compact {
    font-size: 10px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
    min-width: 30px;
  }
  .time-compact.remaining {
    color: var(--text-tertiary);
  }

  .time-separator {
    font-size: 9px;
    color: var(--text-tertiary);
    opacity: 0.5;
  }

  .volume-control-compact {
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--text-tertiary);
    margin-left: 6px;
  }

  .volume-control-compact input[type=range] {
    width: 70px;
    height: 3px;
    background: var(--surface-1);
    border-radius: 2px;
    appearance: none;
    cursor: pointer;
  }
  .volume-control-compact input[type=range]::-webkit-slider-thumb {
    appearance: none;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--text-primary);
    box-shadow: var(--shadow-1);
    border: 1px solid var(--divider-color);
  }
  .volume-control-compact input[type=range]:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
