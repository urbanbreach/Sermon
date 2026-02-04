// Artwork loading metrics collection
// Used for QA verification of LQIP pipeline performance

interface ArtworkTiming {
  cacheKey: string;
  lqipLoadedAt?: number;
  thumbLoadedAt?: number;
  fullLoadedAt?: number;
  errorAt?: number;
}

const timings = new Map<string, ArtworkTiming>();
let startTime = 0;

export function resetArtworkMetrics(): void {
  timings.clear();
  startTime = performance.now();
}

export function recordLqipLoaded(cacheKey: string): void {
  const t = timings.get(cacheKey) || { cacheKey };
  t.lqipLoadedAt = performance.now() - startTime;
  timings.set(cacheKey, t);
}

export function recordThumbLoaded(cacheKey: string): void {
  const t = timings.get(cacheKey) || { cacheKey };
  t.thumbLoadedAt = performance.now() - startTime;
  timings.set(cacheKey, t);
}

export function recordArtworkError(cacheKey: string): void {
  const t = timings.get(cacheKey) || { cacheKey };
  t.errorAt = performance.now() - startTime;
  timings.set(cacheKey, t);
}

export function getArtworkMetrics(): {
  count: number;
  errorCount: number;
  p95ThumbMs: number;
  timings: ArtworkTiming[];
} {
  const all = [...timings.values()];
  const withThumb = all.filter(t => t.thumbLoadedAt !== undefined);
  const errors = all.filter(t => t.errorAt !== undefined);

  // Calculate p95
  const thumbTimes = withThumb
    .map(t => t.thumbLoadedAt!)
    .sort((a, b) => a - b);
  const p95Index = Math.floor(thumbTimes.length * 0.95);
  const p95ThumbMs = thumbTimes[p95Index] || 0;

  return {
    count: all.length,
    errorCount: errors.length,
    p95ThumbMs,
    timings: all
  };
}

// Expose globally for MCP access
if (typeof window !== 'undefined') {
  (window as any).__artworkMetrics = {
    reset: resetArtworkMetrics,
    get: getArtworkMetrics
  };
}
