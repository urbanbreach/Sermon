<script lang="ts">
  import { onMount } from 'svelte';
  import { currentTrack, currentTrackFull, queue, playNow, audioTelemetry } from '../state/playback';
  import { currentLyrics, currentLineIndex, lyricsStatus, fetchLyricsForTrack } from '../state/lyrics';
  import { currentArtworkUrl } from '../state/artwork';
  import { clearAlphabetSelector } from '../state/alphabetSelector';
  import { reduceEffects } from '../state/effects';
  import ArtworkImage from '../components/ArtworkImage.svelte';
  import { Disc, ListMusic, Volume2 } from '@lucide/svelte';

  let lyricsScrollEl: HTMLDivElement | undefined = $state(undefined);

  onMount(() => {
    clearAlphabetSelector();
  });

  // Auto-scroll to active lyric line
  $effect(() => {
    const idx = $currentLineIndex;
    if (!lyricsScrollEl || idx < 0) return;
    const activeLine = lyricsScrollEl.querySelector('.lyric-line.active');
    if (activeLine) {
      const reduced = $reduceEffects || window.matchMedia('(prefers-reduced-motion: reduce)').matches;
      activeLine.scrollIntoView({
        behavior: reduced ? 'auto' : 'smooth',
        block: 'center',
      });
    }
  });

  // Fallback: if lyrics state is idle with an active track, trigger a fetch.
  $effect(() => {
    if (!$currentTrack || $lyricsStatus !== 'idle') {
      return;
    }

    void fetchLyricsForTrack($currentTrack.id);
  });

  // Derive album context for ArtworkImage props
  const albumContext = $derived.by(() => {
    const full = $currentTrackFull;
    const basic = $currentTrack;
    if (!full && !basic) return { artistSort: '', titleSort: '' };
    const artistRaw = full?.albumArtist || full?.artist || basic?.artist || 'unknown artist';
    const titleRaw = full?.album || basic?.album || 'unknown album';
    return {
      artistSort: artistRaw.trim().toLowerCase(),
      titleSort: titleRaw.trim().toLowerCase(),
    };
  });

  // Build quality-details string from best available source
  const qualityLine = $derived.by(() => {
    const tel = $audioTelemetry;
    const track = $currentTrack;
    const full = $currentTrackFull;

    // Source priority: telemetry decode > track event > full track row
    const codec = tel?.format?.decode?.codec || track?.codec || full?.codec;
    const sampleRate = tel?.format?.decode?.sample_rate || track?.sample_rate || full?.sampleRate;
    const bitDepth = tel?.format?.decode?.bit_depth || track?.bit_depth || full?.bitDepth;
    const channels = tel?.format?.decode?.channels || track?.channels || full?.channels;
    const isDsd = tel?.format?.decode?.is_dsd;
    const dsdRate = tel?.format?.decode?.dsd_rate_hz || track?.dsd_rate_hz || full?.dsdRateHz;

    if (!codec && !sampleRate && !bitDepth) return '';

    const parts: string[] = [];

    if (isDsd && dsdRate) {
      // DSD: show multiplier label + rate
      const multiplier = Math.round(dsdRate / 44100);
      parts.push(`DSD${multiplier}`);
      const mhz = dsdRate / 1_000_000;
      parts.push(`${mhz.toFixed(1)} MHz`);
    } else {
      // PCM: codec, sample rate, bit depth
      if (codec) parts.push(codec.toUpperCase());
      if (sampleRate) {
        const khz = sampleRate / 1000;
        parts.push(Number.isInteger(khz) ? `${khz} kHz` : `${khz.toFixed(1)} kHz`);
      }
      if (bitDepth) parts.push(`${bitDepth}-bit`);
    }

    // Channels
    if (channels) {
      if (channels === 1) parts.push('Mono');
      else if (channels === 2) parts.push('Stereo');
      else parts.push(`${channels}ch`);
    }

    // Bitrate (calculated from file size + duration)
    const sizeBytes = full?.sizeBytes;
    const durationMs = full?.durationMs;
    if (sizeBytes && durationMs && durationMs > 0) {
      const kbps = Math.round((sizeBytes * 8) / durationMs);
      parts.push(`${kbps} kbps`);
    }

    return parts.join(' · ');
  });

  // Combined album + artist meta line with graceful fallbacks
  const metaLine = $derived.by(() => {
    const album = $currentTrack?.album;
    const artist = $currentTrack?.artist;
    if (album && artist) return `${album} — ${artist}`;
    if (album) return album;
    if (artist) return artist;
    return 'Unknown Artist';
  });

  function handleQueueKeydown(e: KeyboardEvent, trackId: number) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      playNow(trackId);
    }
  }

  // ── Draggable split control ───────────────────────────────────
  let splitRatio = $state(0.5);
  let isDragging = $state(false);
  let panelBodyEl: HTMLDivElement | undefined = $state(undefined);

  function handleSplitPointerDown(e: PointerEvent) {
    if (e.button !== 0 || !e.isPrimary) return;
    isDragging = true;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function handleSplitPointerMove(e: PointerEvent) {
    if (!isDragging || !panelBodyEl) return;
    const rect = panelBodyEl.getBoundingClientRect();
    if (rect.height <= 0) return;
    splitRatio = Math.max(0, Math.min(1, (e.clientY - rect.top) / rect.height));
  }

  function handleSplitLost() {
    isDragging = false;
  }

  function handleSplitKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      splitRatio = Math.max(0, splitRatio - 0.05);
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      splitRatio = Math.min(1, splitRatio + 0.05);
    } else if (e.key === 'Home') {
      e.preventDefault();
      splitRatio = 0;
    } else if (e.key === 'End') {
      e.preventDefault();
      splitRatio = 1;
    }
  }
</script>

<div
  class="now-playing-view"
  data-testid="now-playing-view"
  data-reduce-effects={$reduceEffects}
  style:--album-art={$currentArtworkUrl ? `url(${$currentArtworkUrl})` : 'none'}
>
  {#if $currentTrack}
    <div class="np-content">
      <div class="np-artwork-pane" data-testid="now-playing-artwork">
        <div class="np-artwork-frame">
          <ArtworkImage
            artistSort={albumContext.artistSort}
            titleSort={albumContext.titleSort}
            size={0}
            alt="Album artwork for {$currentTrack.title || 'Unknown'}"
            class="np-art-img"
          />
        </div>
        <div class="np-track-info">
          <h1>{$currentTrack.title || 'Untitled'}</h1>
          <h2>{metaLine}</h2>
          {#if qualityLine}
            <p class="np-quality-line">{qualityLine}</p>
          {/if}
        </div>
      </div>

      <div
        class="np-panel-pane"
        id="now-playing-panel-pane"
        data-testid="now-playing-right"
      >
        <div class="np-card-body" bind:this={panelBodyEl}>
          <div
            class="np-lyrics-pane"
            id="np-lyrics-content"
            data-testid="now-playing-lyrics"
            style={`flex: ${splitRatio} 1 0; min-height: 0;`}
          >
            {#if $lyricsStatus === 'loading' || $lyricsStatus === 'idle'}
              <div class="np-lyrics-state">
                <div class="lyrics-loading-pulse"></div>
                <p>Loading lyrics&hellip;</p>
              </div>
            {:else if $lyricsStatus === 'error'}
              <div class="np-lyrics-state">
                <p>Failed to load lyrics.</p>
              </div>
            {:else if $currentLyrics && $currentLyrics.length > 0}
              <div class="np-lyrics-scroll" bind:this={lyricsScrollEl}>
                <div class="np-lyrics-spacer"></div>
                {#each $currentLyrics as line, i (i)}
                  <p
                    class="lyric-line"
                    class:active={i === $currentLineIndex}
                    class:before={i < $currentLineIndex}
                    class:after={i > $currentLineIndex}
                  >
                    {line || '\u00A0'}
                  </p>
                {/each}
                <div class="np-lyrics-spacer"></div>
              </div>
            {:else}
              <div class="np-lyrics-state">
                <Disc size={40} strokeWidth={1.5} />
                <p>No lyrics available</p>
              </div>
            {/if}
          </div>

          <button
            type="button"
            class="np-split-handle"
            class:dragging={isDragging}
            role="slider"
            aria-orientation="vertical"
            aria-label="Resize lyrics and queue panels"
            aria-controls="np-lyrics-content np-queue-content"
            aria-valuenow={Math.round(splitRatio * 100)}
            aria-valuemin={0}
            aria-valuemax={100}
            onpointerdown={handleSplitPointerDown}
            onpointermove={handleSplitPointerMove}
            onpointerup={handleSplitLost}
            onpointercancel={handleSplitLost}
            onlostpointercapture={handleSplitLost}
            onkeydown={handleSplitKeydown}
          >
            <span class="np-split-handle-line"></span>
          </button>

          <div
            class="np-queue-pane"
            id="np-queue-content"
            data-testid="now-playing-queue"
            style={`flex: ${1 - splitRatio} 1 0; min-height: 0;`}
          >
            {#if $queue.length > 0}
              <div class="np-queue-header">
                <span class="queue-count">{$queue.length} {$queue.length === 1 ? 'track' : 'tracks'}</span>
              </div>
              <div class="np-queue-scroll">
                {#each $queue as item, i (item.track_id + '-' + i)}
                  {@const isPlaying = item.track_id === $currentTrack?.id}
                  <div
                    class="queue-item"
                    class:playing={isPlaying}
                    role="button"
                    tabindex="0"
                    onclick={() => playNow(item.track_id)}
                    onkeydown={(e) => handleQueueKeydown(e, item.track_id)}
                  >
                    <span class="queue-index">
                      {#if isPlaying}
                        <Volume2 size={13} strokeWidth={2} />
                      {:else}
                        {i + 1}
                      {/if}
                    </span>
                    <div class="queue-item-info">
                      <span class="queue-item-title">{item.title || 'Untitled'}</span>
                      <span class="queue-item-artist">{item.artist || 'Unknown'}</span>
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="np-lyrics-state">
                <ListMusic size={40} strokeWidth={1.5} />
                <p>Queue is empty</p>
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  {:else}
    <!-- Empty: nothing playing -->
    <div class="np-empty" data-testid="now-playing-empty">
      <Disc size={64} strokeWidth={1} />
      <h2>Nothing Playing</h2>
      <p>Select a track to get started</p>
    </div>
  {/if}
</div>

<style>
  .now-playing-view {
    flex: 1;
    position: relative;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    color: #fff;
    background: var(--surface-0, #0a0a0a);
    animation: np-fade-in 0.35s ease-out;
  }

  @keyframes np-fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .now-playing-view::before {
    content: '';
    position: absolute;
    inset: 0;
    background-image: var(--album-art, none);
    background-size: cover;
    background-position: center;
    filter: blur(36px) saturate(0.95);
    opacity: 0.5;
    transform: scale(1.1);
    z-index: 0;
  }

  .now-playing-view::after {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(120deg, rgba(10, 10, 10, 0.35), rgba(10, 10, 10, 0.6));
    z-index: 0;
  }

  .now-playing-view[data-reduce-effects='true']::before {
    filter: none;
    opacity: 0.15;
  }

  .np-content {
    position: relative;
    z-index: 1;
    flex: 1;
    display: flex;
    gap: 2.5rem;
    padding: 2rem 2.5rem;
    min-height: 0;
    align-items: center;
    justify-content: space-between;
  }

  .np-artwork-pane {
    flex: 1 1 0%;
    max-width: 600px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    order: 1;
  }

  .np-artwork-frame {
    width: 100%;
    max-width: 560px;
    aspect-ratio: 1;
    border-radius: var(--artwork-radius-album-detail, 12px);
    overflow: hidden;
    box-shadow:
      0 32px 80px rgba(0, 0, 0, 0.6),
      0 6px 20px rgba(0, 0, 0, 0.4);
    background: rgba(255, 255, 255, 0.03);
  }

  :global(.np-art-img) {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .np-track-info {
    text-align: center;
    width: 100%;
    padding: 0 0.25rem;
  }

  .np-track-info h1 {
    font-size: clamp(1.25rem, 2vw, 1.625rem);
    font-weight: 700;
    line-height: 1.2;
    margin: 0 0 0.25rem 0;
    letter-spacing: -0.02em;
    text-shadow: 0 1px 8px rgba(0, 0, 0, 0.4);
    color: rgba(255, 255, 255, 0.92);
    display: -webkit-box;
    line-clamp: 2;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .np-track-info h2 {
    font-size: clamp(0.875rem, 1.2vw, 1rem);
    font-weight: 500;
    margin: 0;
    color: rgba(255, 255, 255, 0.55);
    text-shadow: 0 1px 6px rgba(0, 0, 0, 0.3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .np-quality-line {
    margin: 0.375rem 0 0 0;
    font-size: 12px;
    font-weight: 500;
    letter-spacing: 0.04em;
    color: rgba(255, 255, 255, 0.45);
    text-shadow: 0 1px 4px rgba(0, 0, 0, 0.25);
    font-variant-numeric: tabular-nums;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .np-panel-pane {
    flex: 0 0 clamp(380px, 41vw, 590px);
    width: clamp(380px, 41vw, 590px);
    max-width: 590px;
    min-width: 340px;
    min-height: 0;
    height: min(84vh, 860px);
    max-height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    align-self: center;
    order: 2;
    background: rgba(10, 10, 10, 0.28);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 16px;
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.04),
      0 8px 32px rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
  }

  .np-card-body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding-top: 0;
  }

  .np-card-body .np-lyrics-spacer {
    height: 18vh;
  }

  .np-card-body .lyric-line {
    font-size: clamp(1rem, 1.3vw, 1.25rem);
  }

  .np-split-handle {
    flex: 0 0 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: row-resize;
    touch-action: none;
    user-select: none;
    position: relative;
    z-index: 2;
    background: transparent;
    border: none;
    padding: 0;
    margin: 0;
    width: 100%;
    appearance: none;
    color: inherit;
    font: inherit;
  }

  .np-split-handle::before {
    content: '';
    position: absolute;
    inset: -4px 0;
  }

  .np-split-handle-line {
    width: 32px;
    height: 3px;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.12);
    transition: background 0.15s ease, width 0.15s ease;
  }

  .np-split-handle:hover .np-split-handle-line,
  .np-split-handle.dragging .np-split-handle-line {
    background: rgba(255, 255, 255, 0.3);
    width: 48px;
  }

  .np-split-handle:focus-visible {
    outline: none;
  }

  .np-split-handle:focus-visible .np-split-handle-line {
    background: rgba(255, 255, 255, 0.4);
    box-shadow: var(--focus-ring, 0 0 0 2px rgba(255, 255, 255, 0.25));
  }

  .np-lyrics-pane {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    position: relative;
    mask-image: linear-gradient(
      to bottom,
      transparent 0%,
      black 8%,
      black 92%,
      transparent 100%
    );
    -webkit-mask-image: linear-gradient(
      to bottom,
      transparent 0%,
      black 8%,
      black 92%,
      transparent 100%
    );
  }

  /* Queue Pane */
  .np-queue-pane {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    mask-image: linear-gradient(
      to bottom,
      black 0%,
      black 94%,
      transparent 100%
    );
    -webkit-mask-image: linear-gradient(
      to bottom,
      black 0%,
      black 94%,
      transparent 100%
    );
  }

  .np-queue-header {
    flex-shrink: 0;
    padding: 16px 16px 8px;
    text-align: center;
  }

  .queue-count {
    font-size: 11px;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.35);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .np-queue-scroll {
    flex: 1;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.1) transparent;
    padding: 0 8px 8px;
  }

  .np-queue-scroll::-webkit-scrollbar {
    width: 4px;
  }

  .np-queue-scroll::-webkit-scrollbar-track {
    background: transparent;
  }

  .np-queue-scroll::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
  }

  .queue-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 10px;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .queue-item:hover {
    background: rgba(255, 255, 255, 0.06);
  }

  .queue-item:focus {
    outline: none;
  }

  .queue-item:focus-visible {
    background: rgba(255, 255, 255, 0.08);
    box-shadow: var(--focus-ring);
  }

  .queue-item.playing {
    background: rgba(255, 255, 255, 0.08);
  }

  .queue-index {
    width: 22px;
    flex-shrink: 0;
    text-align: center;
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    color: rgba(255, 255, 255, 0.3);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .queue-item.playing .queue-index {
    color: #fff;
  }

  .queue-item-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .queue-item-title {
    font-size: 13px;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.8);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .queue-item.playing .queue-item-title {
    color: #fff;
    font-weight: 600;
  }

  .queue-item-artist {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.35);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .np-lyrics-scroll {
    max-height: 100%;
    width: 100%;
    overflow-y: auto;
    text-align: center;
    scrollbar-width: none;
    scroll-behavior: smooth;
    padding: 0 1rem;
  }

  .np-lyrics-scroll::-webkit-scrollbar {
    display: none;
  }

  .np-lyrics-spacer {
    height: 40vh;
  }

  /* Lyric lines */
  .lyric-line {
    margin: 0;
    padding: 0.5rem 0;
    font-size: clamp(1.25rem, 2vw, 1.75rem);
    font-weight: 700;
    line-height: 1.2;
    transition: all 0.3s ease;
    color: rgba(255, 255, 255, 0.3);
    cursor: default;
  }

  .lyric-line.active {
    color: #fff;
    transform: scale(1.05);
    text-shadow: 0 0 40px rgba(255, 255, 255, 0.3);
  }

  .lyric-line.before {
    color: rgba(255, 255, 255, 0.25);
  }

  .lyric-line.after {
    color: rgba(255, 255, 255, 0.35);
  }

  /* Lyrics empty / loading / error states */
  .np-lyrics-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    color: rgba(255, 255, 255, 0.4);
    text-align: center;
    padding: 2rem;
  }

  .np-lyrics-state p {
    margin: 0;
    font-size: 1rem;
    font-weight: 500;
  }

  .lyrics-loading-pulse {
    width: 36px;
    height: 36px;
    border: 2px solid rgba(255, 255, 255, 0.15);
    border-top-color: rgba(255, 255, 255, 0.6);
    border-radius: 50%;
    animation: lyric-spin 0.8s linear infinite;
  }

  @keyframes lyric-spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* Empty state: nothing playing */
  .np-empty {
    position: relative;
    z-index: 1;
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    color: rgba(255, 255, 255, 0.3);
  }

  .np-empty h2 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.5);
  }

  .np-empty p {
    margin: 0;
    font-size: 0.95rem;
    color: rgba(255, 255, 255, 0.3);
  }

  @media (max-width: 1200px) {
    .np-panel-pane {
      flex: 0 0 clamp(340px, 43vw, 530px);
      width: clamp(340px, 43vw, 530px);
      max-width: 530px;
      min-width: 300px;
      height: min(78vh, 740px);
    }

    .np-artwork-pane {
      max-width: 480px;
    }

    .np-artwork-frame {
      max-width: 460px;
    }
  }

  @media (max-width: 900px) {
    .np-content {
      flex-direction: column;
      padding: 1.25rem;
      gap: 1.25rem;
      align-items: stretch;
    }

    .np-artwork-pane {
      order: 1;
      width: 100%;
      max-width: 320px;
      align-self: center;
      gap: 0.75rem;
    }

    .np-panel-pane {
      order: 2;
      flex: 1;
      width: 100%;
      max-width: none;
      min-width: 0;
      height: min(62vh, 640px);
      max-height: none;
      align-self: stretch;
    }

    .lyric-line {
      font-size: 1.25rem;
    }

    .np-split-handle {
      flex-basis: 20px;
    }

    .np-split-handle::before {
      inset: -8px 0;
    }
  }

  /* Accessibility: prefers-reduced-motion */
  @media (prefers-reduced-motion: reduce) {
    .now-playing-view {
      animation: none;
    }

    .lyric-line {
      transition: none;
      transform: none !important;
    }

    .np-lyrics-scroll {
      scroll-behavior: auto;
    }

    .lyrics-loading-pulse {
      animation: none;
      border-color: rgba(255, 255, 255, 0.3);
    }
  }
</style>
