import { writable, get } from 'svelte/store';

const artworkCacheInternal = writable<Map<number, string | null>>(new Map());

export const queueArtworkCache = {
  subscribe: artworkCacheInternal.subscribe,
};

export function setQueueArtworkCache(trackId: number, url: string | null) {
  artworkCacheInternal.update((cache) => {
    cache.set(trackId, url);
    return cache;
  });
}

export function getQueueArtworkFromCache(trackId: number): string | null | undefined {
  return get(artworkCacheInternal).get(trackId);
}

export function clearQueueArtworkCache() {
  artworkCacheInternal.set(new Map());
}

const MAX_CACHE_SIZE = 200;

export function pruneQueueArtworkCache() {
  artworkCacheInternal.update((cache) => {
    if (cache.size > MAX_CACHE_SIZE) {
      const entries = Array.from(cache.entries());
      const toRemove = entries.slice(0, cache.size - MAX_CACHE_SIZE);
      for (const [key] of toRemove) {
        cache.delete(key);
      }
    }
    return cache;
  });
}
