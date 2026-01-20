<script lang="ts">
  import { currentLyrics, currentLineIndex } from '../state/lyrics';
  import { currentTrack, progress, positionMs, durationMs } from '../state/playback';
  import { currentArtworkUrl } from '../state/artwork';
  import { goBack, navigate } from '../state/route';
  import { ArrowLeft, Maximize2 } from '@lucide/svelte';
  import { onMount, onDestroy } from 'svelte';

  function formatTime(ms: number): string {
    const minutes = Math.floor(ms / 60000);
    const seconds = Math.floor((ms % 60000) / 1000);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      goBack();
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<div class="lyrics-fullscreen">
  <!-- Background with artwork blur -->
  <div class="lyrics-bg">
    {#if $currentArtworkUrl}
      <img src={$currentArtworkUrl} alt="" class="bg-artwork" />
    {/if}
    <div class="bg-overlay"></div>
  </div>

  <!-- Header -->
  <header class="lyrics-header">
    <button class="back-btn" onclick={() => goBack()} title="Back (Esc)">
      <ArrowLeft size={24} />
    </button>
    <div class="track-info">
      <span class="track-title">{$currentTrack?.title || '—'}</span>
      <span class="track-artist">{$currentTrack?.artist || '—'}</span>
    </div>
    <div class="header-spacer"></div>
  </header>

  <!-- Lyrics content -->
  <main class="lyrics-main">
    {#if $currentLyrics && $currentLyrics.length > 0}
      <div class="lyrics-scroll">
        {#each $currentLyrics as line, i}
          <p 
            class="lyric-line"
            class:active={i === $currentLineIndex}
            class:before={i < $currentLineIndex}
            class:after={i > $currentLineIndex}
          >
            {line || '\u00A0'}
          </p>
        {/each}
      </div>
    {:else}
      <div class="no-lyrics">
        <p>Lyrics not available for this track.</p>
      </div>
    {/if}
  </main>

  <!-- Footer with progress -->
  <footer class="lyrics-footer">
    <div class="progress-bar">
      <div class="progress-fill" style="width: {$progress * 100}%"></div>
    </div>
    <div class="time-display">
      <span>{formatTime($positionMs)}</span>
      <span>{formatTime($durationMs)}</span>
    </div>
  </footer>
</div>

<style>
  .lyrics-fullscreen {
    position: fixed;
    inset: 0;
    z-index: 200;
    display: flex;
    flex-direction: column;
    background: #000;
    color: #fff;
    font-family: 'Inter Variable', Inter, sans-serif;
  }

  /* Background */
  .lyrics-bg {
    position: absolute;
    inset: 0;
    z-index: 0;
    overflow: hidden;
  }

  .bg-artwork {
    width: 100%;
    height: 100%;
    object-fit: cover;
    filter: blur(80px) saturate(1.2);
    transform: scale(1.2);
    opacity: 0.4;
  }

  .bg-overlay {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      180deg,
      rgba(0, 0, 0, 0.7) 0%,
      rgba(0, 0, 0, 0.5) 50%,
      rgba(0, 0, 0, 0.8) 100%
    );
  }

  /* Header */
  .lyrics-header {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    padding: 16px 24px;
    gap: 16px;
    -webkit-app-region: drag;
  }

  .back-btn {
    -webkit-app-region: no-drag;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    color: #fff;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background 0.2s;
  }

  .back-btn:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .track-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    text-align: center;
  }

  .track-title {
    font-size: 18px;
    font-weight: 600;
  }

  .track-artist {
    font-size: 14px;
    color: rgba(255, 255, 255, 0.7);
  }

  .header-spacer {
    width: 40px; /* Balance the back button */
  }

  /* Main lyrics area */
  .lyrics-main {
    position: relative;
    z-index: 1;
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    padding: 0 48px;
  }

  .lyrics-scroll {
    max-height: 100%;
    overflow-y: auto;
    text-align: center;
    padding: 48px 0;
    scrollbar-width: none;
  }

  .lyrics-scroll::-webkit-scrollbar {
    display: none;
  }

  .lyric-line {
    margin: 0;
    padding: 8px 0;
    font-size: 44px;
    font-weight: 700;
    line-height: 1.15;
    transition: all 0.3s ease;
    color: rgba(255, 255, 255, 0.3);
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

  .no-lyrics {
    text-align: center;
    color: rgba(255, 255, 255, 0.5);
    font-size: 18px;
  }

  /* Footer */
  .lyrics-footer {
    position: relative;
    z-index: 1;
    padding: 16px 48px 32px;
  }

  .progress-bar {
    height: 4px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 2px;
    overflow: hidden;
    margin-bottom: 8px;
  }

  .progress-fill {
    height: 100%;
    background: #fff;
    border-radius: 2px;
    transition: width 0.1s linear;
  }

  .time-display {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.6);
    font-variant-numeric: tabular-nums;
  }
</style>
