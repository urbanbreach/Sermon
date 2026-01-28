<script lang="ts">
  import { navigate } from '../state/route';
  import { 
    currentTrack, playbackState, togglePlayPause, next, previous, 
    volume, setVolume, playbackError, switchToDefault, progress, audioDebug,
    positionMs, durationMs
  } from '../state/playback';
  import { seek } from '../state/playback';
  import { currentArtworkUrl } from '../state/artwork';
  import { isRailOpen } from '../state/rightRail';
  import { pressScale, hoverScale } from '../utils/animations';
  import { SkipBack, Pause, Play, SkipForward, Volume2, Shuffle, Repeat } from '@lucide/svelte';
  import WaveformSeekbar from './WaveformSeekbar.svelte';
  import { waveformPeaks, loadWaveformPeaks, clearWaveformPeaks } from '../state/waveform';
  
  import { 
    glassMainBlur, glassMainBg, bottomBarWaveformSeekbar
  } from '../state/effects';

  // Load waveform when track changes
  $effect(() => {
    if ($currentTrack?.id && $bottomBarWaveformSeekbar) {
      loadWaveformPeaks($currentTrack.id);
    } else {
      clearWaveformPeaks();
    }
  });

  let isUnity = $derived($audioDebug?.policy === 'strict' && $audioDebug?.output_mode === 'exclusive');

  let isEmpty = $derived(!$currentTrack);

  // Interactive Seek State
  let isDragging = $state(false);
  let dragProgress = $state(0);
  let progressTrackEl: HTMLDivElement | undefined = $state();

  // Visual state derived from dragging or playback
  let visualProgress = $derived(isDragging ? dragProgress : $progress);
  let visualPositionMs = $derived(isDragging ? (dragProgress * $durationMs) : $positionMs);

  // Time formatting for progress display
  function formatTime(ms: number): string {
    if (!ms || ms < 0) return '0:00';
    const mins = Math.floor(ms / 60000);
    const secs = Math.floor((ms % 60000) / 1000);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  let currentTimeDisplay = $derived(formatTime(visualPositionMs));
  let remainingTimeDisplay = $derived($durationMs > 0 ? `-${formatTime($durationMs - visualPositionMs)}` : '-0:00');

  function getProgressFromEvent(e: MouseEvent) {
    if (!progressTrackEl) return 0;
    const rect = progressTrackEl.getBoundingClientRect();
    const x = e.clientX - rect.left;
    return Math.max(0, Math.min(1, x / rect.width));
  }

  function handleTrackMouseDown(e: MouseEvent) {
    if (!progressTrackEl) return;
    isDragging = true;
    dragProgress = getProgressFromEvent(e);
  }

  function handleWindowMouseMove(e: MouseEvent) {
    if (isDragging) {
      dragProgress = getProgressFromEvent(e);
    }
  }

  function handleWindowMouseUp(e: MouseEvent) {
    if (isDragging) {
      const finalProgress = getProgressFromEvent(e);
      seek(finalProgress * $durationMs);
      isDragging = false;
    }
  }

  function openNowPlaying() {
    navigate({ name: 'now-playing' });
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') {
      openNowPlaying();
    }
  }
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
  <div class="bottom-bar" data-testid="glass-panel" style="--bar-blur: {$glassMainBlur}px; --bar-bg: {$glassMainBg};">
    {#if $bottomBarWaveformSeekbar && $waveformPeaks.status === 'ready'}
      <!-- WAVEFORM MODE: Single-row MusicBee-like layout -->
      <div class="waveform-single-row" class:empty={isEmpty}>
        <!-- Compact Now Playing -->
        <div 
          class="compact-now-playing"
          onclick={openNowPlaying}
          role="button"
          tabindex="0"
          onkeypress={handleKey}
        >
          {#if $currentArtworkUrl}
            <img src={$currentArtworkUrl} alt="" class="compact-artwork" />
          {:else}
            <div class="compact-artwork-placeholder"></div>
          {/if}
          <span class="compact-title">{$currentTrack?.title || 'Nothing Playing'}</span>
        </div>

        <!-- Transport Controls (no shuffle/repeat) -->
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
    {:else}
      <!-- DEFAULT MODE: 2-row layout -->
      <!-- ROW 1: Progress Row -->
      <div class="progress-row">
        <span class="time-label current">{currentTimeDisplay}</span>
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div 
          class="progress-track"
          bind:this={progressTrackEl}
          onmousedown={handleTrackMouseDown}
        >
          <div 
            class="progress-fill" 
            style="width: {visualProgress * 100}%; transition: {isDragging ? 'none' : 'width 0.15s linear'}"
          >
            <div class="progress-thumb"></div>
          </div>
        </div>
        <span class="time-label remaining">{remainingTimeDisplay}</span>
      </div>
      
      <!-- ROW 2: Controls Row -->
      <div class="controls-row" class:empty={isEmpty}>
        <!-- LEFT: Transport + Action Icons -->
        <div class="left-cluster">
          <div class="transport-controls">
            <button class="ctrl-btn small" title="Shuffle" use:pressScale={{ scale: 0.95 }}><Shuffle size={16} /></button>
            <button class="ctrl-btn" onclick={previous} title="Previous" use:pressScale={{ scale: 0.95 }}><SkipBack size={20} fill="currentColor" /></button>
            <button class="ctrl-btn play" onclick={togglePlayPause} title={$playbackState === 'playing' ? 'Pause' : 'Play'} use:pressScale>
              {#if $playbackState === 'playing'}
                <Pause size={22} fill="currentColor" />
              {:else}
                <Play size={22} fill="currentColor" />
              {/if}
            </button>
            <button class="ctrl-btn" onclick={next} title="Next" use:pressScale={{ scale: 0.95 }}><SkipForward size={20} fill="currentColor" /></button>
            <button class="ctrl-btn small" title="Repeat" use:pressScale={{ scale: 0.95 }}><Repeat size={16} /></button>
          </div>

        </div>
        
        <!-- CENTER: Now Playing Pill -->
        <div 
          class="now-playing-pill" 
          onclick={openNowPlaying} 
          role="button" 
          tabindex="0" 
          onkeypress={handleKey}
          use:hoverScale={{ scale: 1.02 }}
        >
          {#if $currentArtworkUrl}
            <img src={$currentArtworkUrl} alt="" class="pill-artwork" />
          {:else}
            <div class="pill-artwork-placeholder"></div>
          {/if}
          <div class="pill-info">
            <span class="pill-title">{$currentTrack?.title || 'Nothing Playing'}</span>
            <span class="pill-artist">{$currentTrack?.artist || 'Select a track'}</span>
          </div>
        </div>
        
        <!-- RIGHT: Volume -->
        <div class="right-cluster" onclick={(e) => e.stopPropagation()}>
          <div class="volume-control">
            {#if isUnity}
              <span class="vol-label-unity">Unity</span>
            {:else}
              <Volume2 size={18} />
            {/if}
            <input 
              type="range" 
              min="0" 
              max="1" 
              step="0.01" 
              value={isUnity ? 1.0 : $volume} 
              disabled={isUnity}
              aria-label="Volume"
              aria-valuenow={isUnity ? 1 : $volume}
              aria-valuemin={0}
              aria-valuemax={1}
              oninput={(e) => setVolume(e.currentTarget.valueAsNumber)} 
            />
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<svelte:window onmousemove={handleWindowMouseMove} onmouseup={handleWindowMouseUp} />

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
    height: var(--layout-player-height, 88px);
    z-index: 5;
  }

  /* Glass effect using CSS backdrop-filter */
  .bottom-bar {
    height: 100%;
    width: 100%;
    display: flex;
    flex-direction: column;
    
    /* Glassmorphism effect */
    background: var(--bar-bg, rgba(18, 18, 22, 0.75));
    backdrop-filter: blur(var(--bar-blur, 16px));
    -webkit-backdrop-filter: blur(var(--bar-blur, 16px));
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: 0 -4px 30px rgba(0, 0, 0, 0.15);
  }

  /* ===== ROW 1: PROGRESS ===== */
  .progress-row {
    height: var(--layout-bottom-bar-progress-height, 28px);
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 16px;
  }

  .time-label {
    font-size: 11px;
    color: var(--text-tertiary);
    min-width: 42px;
    font-variant-numeric: tabular-nums;
  }
  .time-label.current { text-align: right; }
  .time-label.remaining { text-align: left; }

  .progress-track {
    flex: 1;
    height: 3px;
    background: var(--surface-1);
    border-radius: 1.5px;
    cursor: pointer;
    position: relative;
    transition: height var(--motion-fast) var(--ease-out);
  }

  .waveform-container {
    flex: 1;
    height: 32px;
    min-height: 32px;
  }
  
  .progress-track:hover {
    height: 5px;
    border-radius: 2.5px;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(
      90deg,
      rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.85),
      rgba(255, 255, 255, 0.95)
    );
    border-radius: 1.5px;
    position: relative;
    box-shadow: 0 0 8px rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.25);
  }

  .progress-track:hover .progress-fill {
    border-radius: 2.5px;
    box-shadow: 0 0 12px rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.4);
  }

  .progress-thumb {
    position: absolute;
    right: -6px;
    top: 50%;
    transform: translateY(-50%) scale(0);
    width: 12px;
    height: 12px;
    background: #fff;
    border-radius: 50%;
    opacity: 0;
    transition: transform var(--motion-fast) var(--ease-out), opacity var(--motion-fast) var(--ease-out);
    box-shadow: var(--shadow-2);
    border: 1px solid rgba(0, 0, 0, 0.15);
  }

  .progress-track:hover .progress-thumb,
  .progress-track:active .progress-thumb {
    opacity: 1;
    transform: translateY(-50%) scale(1);
  }

  /* ===== ROW 2: CONTROLS ===== */
  .controls-row {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    gap: 24px;
  }

  /* LEFT CLUSTER */
  .left-cluster {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-shrink: 0;
  }

  .transport-controls {
    display: flex;
    align-items: center;
    gap: 4px;
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
  .ctrl-btn.small { padding: 4px; opacity: 0.6; }
  .ctrl-btn.small:hover { opacity: 1; }
  .ctrl-btn.play { 
    width: 40px; 
    height: 40px; 
    background: var(--surface-2);
    border-radius: 50%;
    box-shadow: var(--shadow-1);
    color: var(--text-primary);
  }
  .ctrl-btn.play:hover { 
    background: var(--surface-hover); 
    box-shadow: var(--shadow-2);
    transform: scale(1.05);
  }
  .ctrl-btn.play:active {
    transform: scale(0.95);
    box-shadow: var(--shadow-inset);
  }

  .action-btn {
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.5);
    cursor: pointer;
    padding: 6px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }
  .action-btn:hover { color: #fff; background: rgba(255, 255, 255, 0.08); }

  /* CENTER: NOW PLAYING PILL */
  .now-playing-pill {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 16px 6px 6px;
    background: var(--surface-1);
    border: 1px solid var(--glass-border);
    border-radius: 999px;
    cursor: pointer;
    transition: all var(--motion-fast) var(--ease-out);
    min-width: 180px;
    max-width: 320px;
    box-shadow: var(--shadow-1);
  }
  .now-playing-pill:hover { 
    background: var(--surface-hover); 
    box-shadow: var(--shadow-2);
    border-color: rgba(255, 255, 255, 0.12);
  }

  .pill-artwork {
    width: 32px;
    height: 32px;
    border-radius: 6px;
    object-fit: cover;
    flex-shrink: 0;
  }

  .pill-artwork-placeholder {
    width: 32px;
    height: 32px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.1);
    flex-shrink: 0;
  }

  .pill-info {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    gap: 1px;
  }

  .pill-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .pill-artist {
    font-size: 11px;
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* RIGHT CLUSTER */
  .right-cluster {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .volume-control {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-tertiary);
  }

  .volume-control input[type=range] {
    width: 100px;
    height: 4px;
    background: var(--surface-1);
    border-radius: 2px;
    appearance: none;
    cursor: pointer;
    transition: height var(--motion-fast) var(--ease-out);
  }
  .volume-control input[type=range]:hover {
    height: 6px;
  }
  .volume-control input[type=range]::-webkit-slider-thumb {
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #fff;
    box-shadow: var(--shadow-1);
    border: 1px solid rgba(0, 0, 0, 0.15);
    transition: transform var(--motion-fast) var(--ease-out);
  }
  .volume-control input[type=range]:hover::-webkit-slider-thumb {
    transform: scale(1.15);
  }
  .volume-control input[type=range]:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .volume-control input[type=range]:disabled::-webkit-slider-thumb {
    background: var(--text-disabled);
    cursor: not-allowed;
  }

  .vol-label-unity {
    font-size: 10px;
    color: #98989E; /* Cider neutral gray - not accent color */
    text-transform: uppercase;
    font-weight: 600;
    letter-spacing: 0.5px;
  }

  /* Empty state: dim controls when nothing is playing */
  .controls-row.empty .transport-controls {
    opacity: 0.5;
    pointer-events: none;
  }
  .controls-row.empty .now-playing-pill {
    opacity: 0.6;
  }

  /* ===== WAVEFORM SINGLE-ROW MODE ===== */
  .waveform-single-row {
    height: 100%;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 0 16px;
  }

  .waveform-single-row.empty .waveform-transport {
    opacity: 0.5;
    pointer-events: none;
  }

  /* Compact Now Playing */
  .compact-now-playing {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px 4px 4px;
    background: var(--surface-1);
    border: 1px solid var(--glass-border);
    border-radius: 999px;
    cursor: pointer;
    transition: all var(--motion-fast) var(--ease-out);
    min-width: 120px;
    max-width: 180px;
    flex-shrink: 0;
  }
  .compact-now-playing:hover {
    background: var(--surface-hover);
    border-color: rgba(255, 255, 255, 0.12);
  }

  .compact-artwork {
    width: 28px;
    height: 28px;
    border-radius: 4px;
    object-fit: cover;
    flex-shrink: 0;
  }

  .compact-artwork-placeholder {
    width: 28px;
    height: 28px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.1);
    flex-shrink: 0;
  }

  .compact-title {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Waveform Transport */
  .waveform-transport {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .ctrl-btn.waveform-play {
    width: 36px;
    height: 36px;
  }

  /* Waveform Center (flexible) */
  .waveform-center {
    flex: 1;
    min-width: 200px;
    height: 48px;
  }

  /* Waveform Right (time + volume) */
  .waveform-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .time-compact {
    font-size: 11px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
    min-width: 32px;
  }
  .time-compact.remaining {
    color: var(--text-tertiary);
  }

  .time-separator {
    font-size: 10px;
    color: var(--text-tertiary);
    opacity: 0.5;
  }

  .volume-control-compact {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-tertiary);
    margin-left: 8px;
  }

  .volume-control-compact input[type=range] {
    width: 80px;
    height: 4px;
    background: var(--surface-1);
    border-radius: 2px;
    appearance: none;
    cursor: pointer;
  }
  .volume-control-compact input[type=range]::-webkit-slider-thumb {
    appearance: none;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #fff;
    box-shadow: var(--shadow-1);
    border: 1px solid rgba(0, 0, 0, 0.15);
  }
  .volume-control-compact input[type=range]:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
