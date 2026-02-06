<script lang="ts">
  import { getArtworkBestForAlbum } from '../api/artwork';
  import {
    getDevArtworkUrl,
    retainDevArtworkUrl,
    releaseDevArtworkUrl
  } from '../utils/artworkDevUrls';
  import { enqueueArtworkDecode, hasReadyThumb, markThumbReady } from '../utils/artworkDecodeQueue';
  import { recordLqipLoaded, recordThumbLoaded, recordArtworkError } from '../utils/artworkMetrics';

  interface Props {
    cacheKey?: string | null;
    artistSort?: string;
    titleSort?: string;
    size?: number;
    deferHighRes?: boolean;
    alt?: string;
    class?: string;
  }

  let {
    cacheKey = null,
    artistSort = '',
    titleSort = '',
    size = 256,
    deferHighRes = false,
    alt = 'Album artwork',
    class: className = ''
  }: Props = $props();

  let canvasEl = $state<HTMLCanvasElement | null>(null);
  let currentSrc = $state('');
  let stage = $state<'none' | 'lqip' | 'full'>('none');

  const LQIP_SIZE = 32;
  const useDevThumbFallback = import.meta.env.DEV;
  const MAX_BITMAP_CACHE = 128;

  type CachedBitmap = ImageBitmap | HTMLImageElement;
  const bitmapCache = new Map<string, CachedBitmap>();
  const bitmapOrder: string[] = [];

  function trackBitmapUsage(url: string) {
    const index = bitmapOrder.indexOf(url);
    if (index !== -1) {
      bitmapOrder.splice(index, 1);
    }
    bitmapOrder.push(url);
  }

  function cacheBitmap(url: string, bitmap: CachedBitmap) {
    if (!bitmapCache.has(url)) {
      bitmapCache.set(url, bitmap);
    }
    trackBitmapUsage(url);

    while (bitmapOrder.length > MAX_BITMAP_CACHE) {
      const oldest = bitmapOrder.shift();
      if (!oldest) break;
      const evicted = bitmapCache.get(oldest);
      if (evicted && 'close' in evicted && typeof evicted.close === 'function') {
        evicted.close();
      }
      bitmapCache.delete(oldest);
    }
  }

  async function loadBitmap(url: string, preferOnload = false): Promise<CachedBitmap> {
    const cached = bitmapCache.get(url);
    if (cached) {
      trackBitmapUsage(url);
      return cached;
    }

    const img = new Image();
    img.src = url;

    if (preferOnload || url.includes('/lqip/')) {
      await new Promise<void>((resolve, reject) => {
        img.onload = () => resolve();
        img.onerror = () => reject(new Error(`Failed to load image: ${url}`));
      });
    } else {
      img.decoding = 'async';
      await img.decode();
    }

    if (typeof createImageBitmap === 'function') {
      try {
        const bitmap = await createImageBitmap(img);
        cacheBitmap(url, bitmap);
        return bitmap;
      } catch {
        // fall through to HTMLImageElement cache
      }
    }

    cacheBitmap(url, img);
    return img;
  }

  function getBitmapSize(bitmap: CachedBitmap): { width: number; height: number } {
    if ('naturalWidth' in bitmap) {
      return {
        width: bitmap.naturalWidth || bitmap.width,
        height: bitmap.naturalHeight || bitmap.height
      };
    }

    return { width: bitmap.width, height: bitmap.height };
  }

  async function drawSource(src: string, preferOnload = false): Promise<void> {
    const canvas = canvasEl;
    if (!canvas) return;

    const bitmap = await loadBitmap(src, preferOnload);

    const rect = canvas.getBoundingClientRect();
    const cssWidth = Math.max(1, Math.round(rect.width));
    const cssHeight = Math.max(1, Math.round(rect.height));
    const dpr = window.devicePixelRatio || 1;
    const drawWidth = Math.max(1, Math.round(cssWidth * dpr));
    const drawHeight = Math.max(1, Math.round(cssHeight * dpr));

    if (canvas.width !== drawWidth || canvas.height !== drawHeight) {
      canvas.width = drawWidth;
      canvas.height = drawHeight;
    }

    const ctx = canvas.getContext('2d', { alpha: false, desynchronized: true });
    if (!ctx) return;

    const { width: sourceWidth, height: sourceHeight } = getBitmapSize(bitmap);
    if (!sourceWidth || !sourceHeight) return;

    const sourceAspect = sourceWidth / sourceHeight;
    const targetAspect = drawWidth / drawHeight;

    let sx = 0;
    let sy = 0;
    let sw = sourceWidth;
    let sh = sourceHeight;

    if (sourceAspect > targetAspect) {
      sw = sourceHeight * targetAspect;
      sx = (sourceWidth - sw) / 2;
    } else {
      sh = sourceWidth / targetAspect;
      sy = (sourceHeight - sh) / 2;
    }

    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = 'high';
    ctx.clearRect(0, 0, drawWidth, drawHeight);
    ctx.drawImage(bitmap, sx, sy, sw, sh, 0, 0, drawWidth, drawHeight);
  }

  function buildUrl(
    key: string | null | undefined,
    artist: string,
    title: string,
    targetSize: number,
    isLqip: boolean
  ): string | null {
    if (key) {
      const encodedKey = encodeURIComponent(key);
      return isLqip
        ? `sermon-artwork://localhost/lqip/${encodedKey}`
        : `sermon-artwork://localhost/thumb/${encodedKey}?s=${targetSize}`;
    }

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

    if (!cacheKey && !canUseAlbumRoute) {
      stage = 'none';
      currentSrc = '';
      return;
    }

    let active = true;
    let devUrls: { lqipUrl: string; thumbUrl: string } | null = null;
    let cancelDecode: (() => void) | null = null;
    const metricsKey = cacheKey || `album:${normalizedArtist}||${normalizedTitle}`;

    const protocolLqipUrl = buildUrl(cacheKey, normalizedArtist, normalizedTitle, LQIP_SIZE, true);
    const protocolThumbUrl = buildUrl(cacheKey, normalizedArtist, normalizedTitle, size, false);

    if (!protocolLqipUrl || !protocolThumbUrl) {
      stage = 'none';
      currentSrc = '';
      return;
    }

    stage = 'lqip';
    recordLqipLoaded(metricsKey);
    currentSrc = protocolLqipUrl;

    const queueThumb = (thumbUrl: string) => {
      if (hasReadyThumb(thumbUrl)) {
        stage = 'full';
        currentSrc = thumbUrl;
        recordThumbLoaded(metricsKey);
        return;
      }

      if (deferHighRes) {
        return;
      }

      cancelDecode = enqueueArtworkDecode(async () => {
        await loadBitmap(thumbUrl);
        if (!active) return;

        markThumbReady(thumbUrl);
        currentSrc = thumbUrl;
        stage = 'full';
        recordThumbLoaded(metricsKey);
      });
    };

    if (useDevThumbFallback) {
      loadDevThumbUrls(cacheKey, normalizedArtist, normalizedTitle, size)
        .then((urls) => {
          if (!active) {
            if (urls) {
              releaseDevArtworkUrl(urls.lqipUrl);
              releaseDevArtworkUrl(urls.thumbUrl);
            }
            return;
          }

          if (!urls) {
            queueThumb(protocolThumbUrl);
            return;
          }

          devUrls = urls;
          currentSrc = urls.lqipUrl;
          queueThumb(urls.thumbUrl);
        })
        .catch((err) => {
          if (!active) return;
          console.warn('Artwork decode failed:', err);
          queueThumb(protocolThumbUrl);
        });
    } else {
      queueThumb(protocolThumbUrl);
    }

    return () => {
      active = false;
      cancelDecode?.();
      if (devUrls) {
        releaseDevArtworkUrl(devUrls.lqipUrl);
        releaseDevArtworkUrl(devUrls.thumbUrl);
      }
    };
  });

  $effect(() => {
    if (!canvasEl || stage === 'none' || !currentSrc) return;

    let active = true;
    drawSource(currentSrc, stage === 'lqip').catch((err) => {
      if (!active) return;
      console.warn('Artwork canvas draw failed:', err);
      recordArtworkError(cacheKey || `album:${artistSort.trim()}||${titleSort.trim()}`);
    });

    return () => {
      active = false;
    };
  });
</script>

<div class="artwork-container {className}" data-artwork-stage={stage} data-cache-key={cacheKey}>
  <canvas
    bind:this={canvasEl}
    class:soft-loading={stage === 'lqip'}
    aria-label={alt}
  ></canvas>
</div>

<style>
  .artwork-container {
    width: 100%;
    height: 100%;
    position: relative;
    overflow: hidden;
    background: var(--surface-2);
  }

  canvas {
    width: 100%;
    height: 100%;
    display: block;
    transition: opacity 0.16s ease-out;
  }

  canvas.soft-loading {
    opacity: 0.86;
  }
</style>
