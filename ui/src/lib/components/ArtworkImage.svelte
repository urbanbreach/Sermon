<script lang="ts">
  import { getArtworkBestForAlbum } from '../api/artwork';
  import {
    getDevArtworkUrl,
    retainDevArtworkUrl,
    releaseDevArtworkUrl
  } from '../utils/artworkDevUrls';
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
  const LQIP_SIZE = 32;
  const useDevThumbFallback = import.meta.env.DEV;

  /**
   * Build a protocol URL for artwork.
   * - If artist/title available: use /album/ route (can discover artwork on-demand)
   * - If only cacheKey available: use /thumb/ or /lqip/ route (requires file to exist)
   */
  function buildUrl(
    key: string | null | undefined,
    artist: string,
    title: string,
    targetSize: number,
    isLqip: boolean
  ): string | null {
    // Prefer /album/ route - it can discover and cache artwork on-demand
    if (artist && title) {
      const artistSegment = encodeURIComponent(artist);
      const titleSegment = encodeURIComponent(title);
      return `sermon-artwork://localhost/album/${artistSegment}/${titleSegment}?s=${targetSize}`;
    }

    // Fallback to direct cache key route (requires file to exist)
    if (key) {
      const encodedKey = encodeURIComponent(key);
      return isLqip
        ? `sermon-artwork://localhost/lqip/${encodedKey}`
        : `sermon-artwork://localhost/thumb/${encodedKey}?s=${targetSize}`;
    }

    return null;
  }

  async function resolveCacheKey(
    key: string | null | undefined,
    artist: string,
    title: string
  ): Promise<string | null> {
    if (key) return key;
    if (!artist || !title) return null;

    try {
      const best = await getArtworkBestForAlbum(artist, title);
      return best.cacheKey ?? null;
    } catch (err) {
      console.warn('Artwork lookup failed:', err);
      return null;
    }
  }

  async function loadDevThumbUrls(
    key: string | null | undefined,
    artist: string,
    title: string,
    targetSize: number
  ): Promise<{ lqipUrl: string; thumbUrl: string } | null> {
    const resolvedKey = await resolveCacheKey(key, artist, title);
    if (!resolvedKey) return null;

    let lqipUrl: string | null = null;
    let thumbUrl: string | null = null;

    try {
      lqipUrl = await getDevArtworkUrl(resolvedKey, LQIP_SIZE);
      retainDevArtworkUrl(lqipUrl);
      thumbUrl = await getDevArtworkUrl(resolvedKey, targetSize);
      retainDevArtworkUrl(thumbUrl);

      return { lqipUrl, thumbUrl };
    } catch (err) {
      if (lqipUrl) {
        releaseDevArtworkUrl(lqipUrl);
      }
      if (thumbUrl) {
        releaseDevArtworkUrl(thumbUrl);
      }
      throw err;
    }
  }


  $effect(() => {
    const normalizedArtist = artistSort.trim();
    const normalizedTitle = titleSort.trim();
    const canUseAlbumRoute = normalizedArtist.length > 0 && normalizedTitle.length > 0;

    // Need either artist/title or cacheKey to load artwork
    if (!cacheKey && !canUseAlbumRoute) {
      stage = 'none';
      currentSrc = '';
      return;
    }

    let active = true;
    let devUrls: { lqipUrl: string; thumbUrl: string } | null = null;
    const metricsKey = cacheKey || `album:${normalizedArtist}||${normalizedTitle}`;

    const loadWithUrls = (lqipUrl: string, thumbUrl: string) => {
      // Start with LQIP (blurred placeholder)
      stage = 'lqip';
      recordLqipLoaded(metricsKey);
      currentSrc = lqipUrl;

      // Preload and decode thumbnail before swapping (prevents jank)
      const img = new Image();
      img.src = thumbUrl;

      img.decode()
        .then(() => {
          if (!active) return;

          // Swap to high-res but keep blur momentarily (stage 'thumb')
          currentSrc = thumbUrl;
          stage = 'thumb';
          recordThumbLoaded(metricsKey);

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
          recordArtworkError(metricsKey);
          stage = 'none';
        });
    };

    if (useDevThumbFallback) {
      loadDevThumbUrls(cacheKey, normalizedArtist, normalizedTitle, size)
        .then((urls) => {
          if (!urls) {
            recordArtworkError(metricsKey);
            stage = 'none';
            currentSrc = '';
            return;
          }
          if (!active) {
            releaseDevArtworkUrl(urls.lqipUrl);
            releaseDevArtworkUrl(urls.thumbUrl);
            return;
          }
          devUrls = urls;
          loadWithUrls(urls.lqipUrl, urls.thumbUrl);
        })
        .catch((err) => {
          if (!active) return;
          console.warn('Artwork decode failed:', err);
          recordArtworkError(metricsKey);
          stage = 'none';
        });
    } else {
      // Build URLs using custom protocol
      const lqipUrl = buildUrl(cacheKey, normalizedArtist, normalizedTitle, LQIP_SIZE, true);
      const thumbUrl = buildUrl(cacheKey, normalizedArtist, normalizedTitle, size, false);

      if (!lqipUrl || !thumbUrl) {
        stage = 'none';
        currentSrc = '';
        return;
      }

      loadWithUrls(lqipUrl, thumbUrl);
    }

    return () => {
      active = false;
      if (devUrls) {
        releaseDevArtworkUrl(devUrls.lqipUrl);
        releaseDevArtworkUrl(devUrls.thumbUrl);
      }
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
