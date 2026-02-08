<script lang="ts">
  import { getArtworkBestForAlbum } from '../api/artwork';
  import {
    getDevArtworkUrl,
    releaseDevArtworkUrl
  } from '../utils/artworkDevUrls';
  import { enqueueArtworkDecode, hasReadyThumb, markThumbReady } from '../utils/artworkDecodeQueue';
  import { recordLqipLoaded, recordThumbLoaded, recordArtworkError } from '../utils/artworkMetrics';

  interface Props {
    cacheKey?: string | null;
    artistSort?: string;
    titleSort?: string;
    size?: number;
    previewUrl?: string | null;
    deferHighRes?: boolean;
    alt?: string;
    class?: string;
  }

  let {
    cacheKey = null,
    artistSort = '',
    titleSort = '',
    size = 256,
    previewUrl = null,
    deferHighRes = false,
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
    // Prefer direct cache key route when available (fast path)
    if (key) {
      const encodedKey = encodeURIComponent(key);
      return isLqip
        ? `sermon-artwork://localhost/lqip/${encodedKey}`
        : `sermon-artwork://localhost/thumb/${encodedKey}?s=${targetSize}`;
    }

    // Fallback to /album/ route - it can discover and cache artwork on-demand
    if (artist && title) {
      const artistSegment = encodeURIComponent(artist);
      const titleSegment = encodeURIComponent(title);
      return `sermon-artwork://localhost/album/${artistSegment}/${titleSegment}?s=${targetSize}`;
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
    targetSize: number,
    includeThumb: boolean
  ): Promise<{ lqipUrl: string; thumbUrl: string | null } | null> {
    const resolvedKey = await resolveCacheKey(key, artist, title);
    if (!resolvedKey) return null;

    let lqipUrl: string | null = null;
    let thumbUrl: string | null = null;

    try {
      lqipUrl = await getDevArtworkUrl(resolvedKey, LQIP_SIZE);
      if (includeThumb) {
        thumbUrl = await getDevArtworkUrl(resolvedKey, targetSize);
      }

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
    const normalizedPreview = previewUrl?.trim() || '';
    const hasPreview = normalizedPreview.length > 0;

    // Need either artist/title or cacheKey to load artwork
    if (!cacheKey && !canUseAlbumRoute) {
      stage = 'none';
      currentSrc = '';
      return;
    }

    let active = true;
    let devUrls: { lqipUrl: string; thumbUrl: string | null } | null = null;
    let cancelDecode: (() => void) | null = null;
    let pendingDecodeImage: HTMLImageElement | null = null;
    const metricsKey = cacheKey || `album:${normalizedArtist}||${normalizedTitle}`;

    const loadWithUrls = (lqipUrl: string, thumbUrl: string | null) => {
      // Skip redundant work if we're already showing this thumb at full quality
      if (thumbUrl && stage === 'full' && currentSrc === thumbUrl) {
        return;
      }

      if (thumbUrl && hasReadyThumb(thumbUrl)) {
        stage = 'full';
        currentSrc = thumbUrl;
        recordThumbLoaded(metricsKey);
        return;
      }

      if (hasPreview) {
        stage = 'full';
        currentSrc = normalizedPreview;
      } else {
        // Start with LQIP (blurred placeholder)
        stage = 'lqip';
        recordLqipLoaded(metricsKey);
        currentSrc = lqipUrl;
      }

      if (deferHighRes || !thumbUrl) {
        return;
      }

      // Preload and decode thumbnail before swapping (prevents jank)
      // Create the Image inside queued work so cancelled tasks do not allocate/download eagerly.
      cancelDecode = enqueueArtworkDecode(async () => {
        const img = new Image();
        pendingDecodeImage = img;
        img.decoding = 'async';
        img.src = thumbUrl;

        try {
          await img.decode();

          if (!active) return;

          markThumbReady(thumbUrl);
          currentSrc = thumbUrl;
          recordThumbLoaded(metricsKey);

          if (!hasPreview) {
            // Swap to high-res but keep blur momentarily (stage 'thumb')
            stage = 'thumb';

            // Remove blur in next frame (stage 'full')
            requestAnimationFrame(() => {
              if (!active) return;
              stage = 'full';
            });
            return;
          }

          stage = 'full';
        } catch (err) {
          if (!active) return;
          console.warn('Artwork decode failed:', err);
          // On failure, show placeholder
          recordArtworkError(metricsKey);
          stage = 'none';
        } finally {
          if (pendingDecodeImage === img) {
            pendingDecodeImage = null;
          }
          img.src = '';
        }
      });
    };

    if (useDevThumbFallback) {
      loadDevThumbUrls(cacheKey, normalizedArtist, normalizedTitle, size, !deferHighRes)
        .then((urls) => {
          if (!urls) {
            recordArtworkError(metricsKey);
            stage = 'none';
            currentSrc = '';
            return;
          }
          if (!active) {
            releaseDevArtworkUrl(urls.lqipUrl);
            if (urls.thumbUrl) {
              releaseDevArtworkUrl(urls.thumbUrl);
            }
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
      const thumbUrl = deferHighRes
        ? null
        : buildUrl(cacheKey, normalizedArtist, normalizedTitle, size, false);

      if (!lqipUrl) {
        stage = 'none';
        currentSrc = '';
        return;
      }

      loadWithUrls(lqipUrl, thumbUrl);
    }

    return () => {
      active = false;
      if (cancelDecode) {
        cancelDecode();
        cancelDecode = null;
      }
      if (pendingDecodeImage) {
        pendingDecodeImage.src = '';
        pendingDecodeImage = null;
      }
      if (devUrls) {
        releaseDevArtworkUrl(devUrls.lqipUrl);
        if (devUrls.thumbUrl) {
          releaseDevArtworkUrl(devUrls.thumbUrl);
        }
        devUrls = null;
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
      class:soft-loading={stage === 'lqip' || stage === 'thumb'}
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
    transition: opacity 0.18s ease-out;
  }

  img.soft-loading {
    opacity: 0.86;
  }

  .artwork-container[data-artwork-stage='thumb'] img.soft-loading {
    opacity: 0.94;
  }

  .placeholder {
    width: 100%;
    height: 100%;
    background: var(--surface-2);
  }
</style>
