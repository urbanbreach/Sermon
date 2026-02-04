import { getArtworkThumbBytes } from '../api/artwork';

type CacheEntry = { url: string; refCount: number };

// Module-level cache: Map<string, { url: string, refCount: number }>
// Key format: `${cacheKey}:${size}`
const devArtworkUrlCache = new Map<string, CacheEntry>();

function buildCacheKey(cacheKey: string, size: number): string {
  return `${cacheKey}:${size}`;
}

function base64ToBytes(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

export async function getDevArtworkUrl(cacheKey: string, size: number): Promise<string> {
  const cacheKeyWithSize = buildCacheKey(cacheKey, size);
  const cached = devArtworkUrlCache.get(cacheKeyWithSize);
  if (cached) return cached.url;

  const response = await getArtworkThumbBytes(cacheKey, size);
  const bytes = base64ToBytes(response.bytesBase64);
  const blob = new Blob([bytes], { type: response.mime });
  const url = URL.createObjectURL(blob);

  devArtworkUrlCache.set(cacheKeyWithSize, { url, refCount: 0 });
  return url;
}

export function retainDevArtworkUrl(url: string): void {
  for (const entry of devArtworkUrlCache.values()) {
    if (entry.url === url) {
      entry.refCount += 1;
      return;
    }
  }
}

export function releaseDevArtworkUrl(url: string): void {
  for (const [key, entry] of devArtworkUrlCache.entries()) {
    if (entry.url === url) {
      if (entry.refCount <= 1) {
        URL.revokeObjectURL(entry.url);
        devArtworkUrlCache.delete(key);
      } else {
        entry.refCount -= 1;
      }
      return;
    }
  }
}
