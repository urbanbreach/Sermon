<script lang="ts">
  import { recordLqipLoaded, recordThumbLoaded, recordArtworkError } from '../utils/artworkMetrics';

  interface Props {
    cacheKey: string | null | undefined;
    artistSort?: string;
    titleSort?: string;
    size?: number;
    alt?: string;
    class?: string;
  }

  let {
    cacheKey,
    artistSort = '',
    titleSort = '',
    size = 256,
    alt = 'Album artwork',
    class: className = ''
  }: Props = $props();

  let currentSrc = $state('');
  let stage = $state<'none' | 'lqip' | 'thumb' | 'full'>('none');

  $effect(() => {
    if (!cacheKey) {
      stage = 'none';
      currentSrc = '';
      return;
    }

    let active = true;
    const encodedKey = encodeURIComponent(cacheKey);
    const lqipUrl = `sermon-artwork://localhost/lqip/${encodedKey}`;
    const thumbUrl = `sermon-artwork://localhost/thumb/${encodedKey}?s=${size}`;

    // Start with LQIP
    stage = 'lqip';
    recordLqipLoaded(cacheKey);
    currentSrc = lqipUrl;

    // Preload and decode thumbnail
    const img = new Image();
    img.src = thumbUrl;

    img.decode()
      .then(() => {
        if (!active) return;
        
        // Swap to high-res but keep blur (stage 'thumb')
        currentSrc = thumbUrl;
        stage = 'thumb';
        recordThumbLoaded(cacheKey);

        // Remove blur in next frame (stage 'full')
        requestAnimationFrame(() => {
          if (!active) return;
          stage = 'full';
        });
      })
      .catch((err) => {
        if (!active) return;
        console.warn('Artwork decode failed:', err);
        // On failure, show placeholder
        recordArtworkError(cacheKey);
        stage = 'none';
      });

    return () => {
      active = false;
    };
  });
</script>

<div 
  class="artwork-container {className}" 
  data-artwork-stage={stage} 
  data-cache-key={cacheKey}
>
  {#if stage !== 'none'}
    <img 
      src={currentSrc} 
      {alt}
      class:blur={stage === 'lqip' || stage === 'thumb'}
      draggable="false"
    />
  {:else}
    <div class="placeholder"></div>
  {/if}
</div>

<style>
  .artwork-container {
    width: 100%;
    height: 100%;
    position: relative;
    overflow: hidden;
    background: var(--surface-2);
  }

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    transition: filter 0.3s ease-out;
    will-change: filter;
  }

  img.blur {
    filter: blur(10px);
    transform: scale(1.05); /* Prevent blurred edges from showing background */
  }

  .placeholder {
    width: 100%;
    height: 100%;
    background: var(--surface-2);
  }
</style>
