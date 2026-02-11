<script lang="ts">
  import { onMount } from 'svelte';
  import { currentTrack, currentTrackFull } from '../state/playback';
  import { currentLyrics, currentLineIndex, lyricsStatus } from '../state/lyrics';
  import { currentArtworkUrl } from '../state/artwork';
  import { clearAlphabetSelector } from '../state/alphabetSelector';
  import { reduceEffects } from '../state/effects';
  import ArtworkImage from '../components/ArtworkImage.svelte';
  import { Disc } from '@lucide/svelte';

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
</script>

<div
  class="now-playing-view"
  data-testid="now-playing-view"
  data-reduce-effects={$reduceEffects}
  style:--album-art={$currentArtworkUrl ? `url(${$currentArtworkUrl})` : 'none'}
>
  {#if $currentTrack}
    <div class="np-content">
      <!-- Left: Artwork + Track Info -->
      <div class="np-artwork-pane" data-testid="now-playing-artwork">
        <div class="np-artwork-frame">
          <ArtworkImage
            artistSort={albumContext.artistSort}
            titleSort={albumContext.titleSort}
            size={512}
            alt="Album artwork for {$currentTrack.title || 'Unknown'}"
            class="np-art-img"
          />
        </div>
        <div class="np-track-info">
          <h1>{$currentTrack.title || 'Untitled'}</h1>
          <h2>{$currentTrack.artist || 'Unknown Artist'}</h2>
          <h3>{$currentTrack.album || 'Unknown Album'}</h3>
        </div>
      </div>

      <!-- Right: Lyrics -->
      <div class="np-lyrics-pane" data-testid="now-playing-lyrics">
        {#if $lyricsStatus === 'loading'}
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
  }

  /* Blurred album art background */
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

  /* Gradient overlay for readability */
  .now-playing-view::after {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(120deg, rgba(10, 10, 10, 0.35), rgba(10, 10, 10, 0.6));
    z-index: 0;
  }

  /* Reduce effects: matte fallback */
  .now-playing-view[data-reduce-effects='true']::before {
    filter: none;
    opacity: 0.15;
  }

  /* Two-column content */
  .np-content {
    position: relative;
    z-index: 1;
    flex: 1;
    display: flex;
    gap: 3rem;
    padding: 2.5rem;
    min-height: 0;
    align-items: center;
  }

  /* Left pane: Artwork + Track Info */
  .np-artwork-pane {
    flex: 0 0 auto;
    width: 45%;
    max-width: 500px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.75rem;
  }

  .np-artwork-frame {
    width: 100%;
    aspect-ratio: 1;
    border-radius: 6px;
    overflow: hidden;
    box-shadow:
      0 24px 64px rgba(0, 0, 0, 0.55),
      0 4px 16px rgba(0, 0, 0, 0.35);
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
    padding: 0 0.5rem;
  }

  .np-track-info h1 {
    font-size: clamp(1.5rem, 2.5vw, 2.25rem);
    font-weight: 800;
    line-height: 1.15;
    margin: 0 0 0.35rem 0;
    letter-spacing: -0.025em;
    text-shadow: 0 2px 12px rgba(0, 0, 0, 0.5);
    color: #fff;
  }

  .np-track-info h2 {
    font-size: clamp(1rem, 1.5vw, 1.3rem);
    font-weight: 500;
    margin: 0 0 0.2rem 0;
    color: rgba(255, 255, 255, 0.75);
    text-shadow: 0 1px 8px rgba(0, 0, 0, 0.4);
  }

  .np-track-info h3 {
    font-size: clamp(0.85rem, 1.2vw, 1.05rem);
    font-weight: 400;
    margin: 0;
    color: rgba(255, 255, 255, 0.5);
    text-shadow: 0 1px 6px rgba(0, 0, 0, 0.3);
  }

  /* Right pane: Lyrics */
  .np-lyrics-pane {
    flex: 1;
    min-width: 0;
    min-height: 0;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    position: relative;
    mask-image: linear-gradient(
      to bottom,
      transparent 0%,
      black 10%,
      black 90%,
      transparent 100%
    );
    -webkit-mask-image: linear-gradient(
      to bottom,
      transparent 0%,
      black 10%,
      black 90%,
      transparent 100%
    );
  }

  .np-lyrics-scroll {
    max-height: 100%;
    width: 100%;
    overflow-y: auto;
    text-align: center;
    scrollbar-width: none;
    scroll-behavior: smooth;
    padding: 0 1.5rem;
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
    font-size: clamp(1.5rem, 2.2vw, 2rem);
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

  /* Responsive: stack vertically below 900px */
  @media (max-width: 900px) {
    .np-content {
      flex-direction: column;
      padding: 1.5rem;
      gap: 2rem;
      align-items: stretch;
    }

    .np-artwork-pane {
      width: 100%;
      max-width: 320px;
      align-self: center;
    }

    .np-lyrics-pane {
      flex: 1;
    }

    .lyric-line {
      font-size: 1.25rem;
    }
  }

  /* Accessibility: prefers-reduced-motion */
  @media (prefers-reduced-motion: reduce) {
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
