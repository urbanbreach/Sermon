import { getArtworkThumbBytes } from '../api/artwork';
import { dropReadyThumb } from './artworkDecodeQueue';

type CacheEntry = {
  url: string | null;
  refCount: number;
  pending: Promise<string> | null;
  lastUsedAt: number;
};

/**
 * Normal idle reclaim window for dev blob URLs.
 * Kept intentionally short so zero-ref entries do not linger after viewport churn.
 */
const DEV_URL_IDLE_TTL_MS = 3_000;

/**
 * Aggressive reclaim window while the album grid is under scroll pressure.
 * During sustained scrolling we prioritize reclaim speed over retention.
 */
const DEV_URL_PRESSURE_IDLE_TTL_MS = 1_000;

/**
 * Recurring prune cadence while scroll pressure is active.
 */
const PRESSURE_PRUNE_INTERVAL_MS = 800;

/**
 * Chooses a pressure-aware cache budget from available device memory.
 * Defaults are conservative in dev to prevent large blob pileups.
 */
function resolveDevUrlMaxEntries(): number {
  if (typeof navigator === 'undefined') {
    return 280;
  }

  const memory = (navigator as Navigator & { deviceMemory?: number }).deviceMemory;
  if (!memory || !Number.isFinite(memory)) {
    return 280;
  }

  if (memory >= 12) return 700;
  if (memory >= 8) return 550;
  if (memory >= 4) return 400;
  return 280;
}

const DEV_URL_MAX_ENTRIES = resolveDevUrlMaxEntries();

// Module-level cache keyed by `${cacheKey}:${size}`
const devArtworkUrlCache = new Map<string, CacheEntry>();
const devArtworkUrlIndex = new Map<string, string>();
let pruneTimer: ReturnType<typeof setTimeout> | null = null;
let pressurePruneTimer: ReturnType<typeof setTimeout> | null = null;
let scrollPressureActive = false;

function buildCacheKey(cacheKey: string, size: number): string {
  return `${cacheKey}:${size}`;
}

function nowMs(): number {
  return performance.now();
}

function base64ToBytes(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

/**
 * Revokes and removes a cache entry once it is fully unreferenced.
 * Safety invariant: object URLs are never revoked while refCount > 0.
 */
function revokeAndDelete(cacheKeyWithSize: string, entry: CacheEntry): void {
  if (entry.refCount > 0) {
    return;
  }

  if (entry.url) {
    dropReadyThumb(entry.url);
    URL.revokeObjectURL(entry.url);
    devArtworkUrlIndex.delete(entry.url);
  }
  devArtworkUrlCache.delete(cacheKeyWithSize);
}

/**
 * Returns the currently active idle TTL used for zero-ref reclamation.
 */
function resolveIdleTtlMs(): number {
  return scrollPressureActive ? DEV_URL_PRESSURE_IDLE_TTL_MS : DEV_URL_IDLE_TTL_MS;
}

/**
 * Reclaims idle zero-ref entries.
 * Under scroll pressure this uses a shorter TTL to reclaim churned blobs sooner.
 */
function pruneIdleEntries(currentTime = nowMs()): void {
  const idleTtlMs = resolveIdleTtlMs();

  for (const [cacheKeyWithSize, entry] of devArtworkUrlCache.entries()) {
    if (entry.refCount > 0) continue;
    if (entry.pending) continue;
    if (
      devArtworkUrlCache.size <= DEV_URL_MAX_ENTRIES &&
      currentTime - entry.lastUsedAt < idleTtlMs
    ) {
      continue;
    }
    revokeAndDelete(cacheKeyWithSize, entry);
  }
}

/**
 * Starts recurring pressure-mode prune passes.
 */
function schedulePressurePrune(): void {
  if (pressurePruneTimer) return;

  pressurePruneTimer = setTimeout(() => {
    pressurePruneTimer = null;
    pruneIdleEntries(nowMs());

    if (scrollPressureActive) {
      schedulePressurePrune();
    }
  }, PRESSURE_PRUNE_INTERVAL_MS);
}

/**
 * Schedules an idle prune pass for zero-ref entries.
 * Pressure mode is driven by notifyScrollPressure().
 */
function schedulePrune(): void {
  if (pruneTimer) return;
  pruneTimer = setTimeout(() => {
    pruneTimer = null;
    const currentTime = nowMs();
    pruneIdleEntries(currentTime);

    for (const entry of devArtworkUrlCache.values()) {
      if (entry.refCount === 0 && !entry.pending) {
        schedulePrune();
        return;
      }
    }
  }, resolveIdleTtlMs() + 128);
}

/**
 * Signals whether scroll pressure is active.
 * - Active: prune immediately and maintain fast recurring prune cadence.
 * - Inactive: stop fast cadence and schedule one trailing idle prune.
 */
export function notifyScrollPressure(active: boolean): void {
  if (active) {
    scrollPressureActive = true;
    pruneIdleEntries(nowMs());
    schedulePressurePrune();
    return;
  }

  if (!scrollPressureActive) return;

  scrollPressureActive = false;
  if (pressurePruneTimer) {
    clearTimeout(pressurePruneTimer);
    pressurePruneTimer = null;
  }
  if (pruneTimer) {
    clearTimeout(pruneTimer);
    pruneTimer = null;
  }
  schedulePrune();
}

function enforceCacheLimit(currentTime = nowMs()): void {
  if (devArtworkUrlCache.size <= DEV_URL_MAX_ENTRIES) return;

  pruneIdleEntries(currentTime);
  if (devArtworkUrlCache.size <= DEV_URL_MAX_ENTRIES) return;

  const candidates = [...devArtworkUrlCache.entries()]
    .filter(([, entry]) => entry.refCount === 0 && !entry.pending)
    .sort((a, b) => a[1].lastUsedAt - b[1].lastUsedAt);

  while (devArtworkUrlCache.size > DEV_URL_MAX_ENTRIES && candidates.length > 0) {
    const [cacheKeyWithSize, entry] = candidates.shift()!;
    revokeAndDelete(cacheKeyWithSize, entry);
  }
}

export async function getDevArtworkUrl(cacheKey: string, size: number): Promise<string> {
  const cacheKeyWithSize = buildCacheKey(cacheKey, size);
  const currentTime = nowMs();
  pruneIdleEntries(currentTime);

  const cached = devArtworkUrlCache.get(cacheKeyWithSize);
  if (cached) {
    cached.refCount += 1;
    cached.lastUsedAt = currentTime;
    if (cached.url) {
      return cached.url;
    }
    if (cached.pending) {
      return cached.pending;
    }
  }

  const entry: CacheEntry = cached ?? {
    url: null,
    refCount: 1,
    pending: null,
    lastUsedAt: currentTime
  };

  if (!cached) {
    devArtworkUrlCache.set(cacheKeyWithSize, entry);
  }

  if (entry.pending) {
    return entry.pending;
  }

  const pending = (async () => {
    const response = await getArtworkThumbBytes(cacheKey, size);
    const bytes = base64ToBytes(response.bytesBase64);
    const safeBytes = Uint8Array.from(bytes);
    const blob = new Blob([safeBytes], { type: response.mime });
    const createdUrl = URL.createObjectURL(blob);

    const liveEntry = devArtworkUrlCache.get(cacheKeyWithSize);
    if (!liveEntry) {
      URL.revokeObjectURL(createdUrl);
      throw new Error(`Dev artwork cache entry missing for ${cacheKeyWithSize}`);
    }

    if (liveEntry.url && liveEntry.url !== createdUrl) {
      URL.revokeObjectURL(createdUrl);
      liveEntry.pending = null;
      liveEntry.lastUsedAt = nowMs();
      return liveEntry.url;
    }

    liveEntry.url = createdUrl;
    liveEntry.pending = null;
    liveEntry.lastUsedAt = nowMs();
    devArtworkUrlIndex.set(createdUrl, cacheKeyWithSize);
    enforceCacheLimit(liveEntry.lastUsedAt);
    return createdUrl;
  })().catch((err) => {
    const liveEntry = devArtworkUrlCache.get(cacheKeyWithSize);
    if (liveEntry && liveEntry.pending === pending) {
      liveEntry.pending = null;
      liveEntry.refCount = 0;
      liveEntry.lastUsedAt = nowMs();
      if (!liveEntry.url) {
        devArtworkUrlCache.delete(cacheKeyWithSize);
      }
    }
    throw err;
  });

  entry.pending = pending;
  return pending;
}

export function retainDevArtworkUrl(url: string): void {
  const cacheKeyWithSize = devArtworkUrlIndex.get(url);
  if (!cacheKeyWithSize) return;
  const entry = devArtworkUrlCache.get(cacheKeyWithSize);
  if (!entry || entry.url !== url) {
    devArtworkUrlIndex.delete(url);
    return;
  }
  entry.refCount += 1;
  entry.lastUsedAt = nowMs();
}

export function releaseDevArtworkUrl(url: string): void {
  const cacheKeyWithSize = devArtworkUrlIndex.get(url);
  if (!cacheKeyWithSize) {
    dropReadyThumb(url);
    URL.revokeObjectURL(url);
    return;
  }

  const entry = devArtworkUrlCache.get(cacheKeyWithSize);
  if (!entry || entry.url !== url) {
    devArtworkUrlIndex.delete(url);
    dropReadyThumb(url);
    URL.revokeObjectURL(url);
    return;
  }

  if (entry.refCount > 0) {
    entry.refCount -= 1;
  }

  entry.lastUsedAt = nowMs();
  if (entry.refCount === 0 && !entry.pending) {
    enforceCacheLimit(entry.lastUsedAt);
    schedulePrune();
  }
}

/**
 * Returns cache size and reference-state distribution for dev diagnostics/MCP probes.
 */
export function getDevUrlCacheStats(): {
  size: number;
  liveRefCount: number;
  zeroRefCount: number;
} {
  let liveRefCount = 0;
  let zeroRefCount = 0;
  for (const entry of devArtworkUrlCache.values()) {
    if (entry.refCount > 0) liveRefCount += 1;
    else zeroRefCount += 1;
  }
  return { size: devArtworkUrlCache.size, liveRefCount, zeroRefCount };
}

if (typeof window !== 'undefined' && import.meta.env.DEV) {
  (window as Window & {
    __devUrlCacheStats?: () => ReturnType<typeof getDevUrlCacheStats>;
  }).__devUrlCacheStats = getDevUrlCacheStats;
}
