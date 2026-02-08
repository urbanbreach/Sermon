type DecodeTask = {
  cancelled: boolean;
  run: () => Promise<void>;
};

/**
 * Maximum parallel decodes when the grid is not actively scrolling.
 * Kept at 6 (existing baseline) to preserve normal-speed visual throughput.
 */
const MAX_CONCURRENT_DECODES = 6;

/**
 * Maximum parallel decodes while the album grid is actively scrolling.
 * Kept at 1 to prevent high-velocity churn from piling up decode work.
 */
const MAX_SCROLL_CONCURRENT_DECODES = 1;

/**
 * Maximum decoded thumb URLs tracked as "ready" (skip-decode-on-revisit).
 * Kept at ~3x a full viewport to balance re-scroll hit rate against
 * transient memory pressure from holding decoded images.
 */
const MAX_READY_THUMBS = 512;

/**
 * Upper bound on queued (not-yet-started) decode tasks.
 * Beyond this, the queue is compacted to shed stale/cancelled work.
 * Set to ~3x viewport capacity to keep throughput high without
 * accumulating excessive stale entries.
 */
const MAX_QUEUED_DECODES = 600;

/**
 * Compact the queue when this many cancelled entries accumulate.
 * Lower value = more frequent compaction = less waste from cancelled tasks.
 */
const QUEUE_COMPACT_THRESHOLD = 48;

let inFlightDecodes = 0;
let albumGridScrolling = false;

const decodeQueue: DecodeTask[] = [];
const readyThumbs = new Set<string>();
const readyThumbOrder: string[] = [];
let cancelledQueuedDecodes = 0;

function compactDecodeQueue(): void {
  if (decodeQueue.length === 0) {
    cancelledQueuedDecodes = 0;
    return;
  }

  let writeIndex = 0;
  for (let readIndex = 0; readIndex < decodeQueue.length; readIndex += 1) {
    const task = decodeQueue[readIndex];
    if (task.cancelled) continue;
    decodeQueue[writeIndex] = task;
    writeIndex += 1;
  }

  decodeQueue.length = writeIndex;
  cancelledQueuedDecodes = 0;
}

function maybeCompactDecodeQueue(): void {
  if (cancelledQueuedDecodes < QUEUE_COMPACT_THRESHOLD) {
    return;
  }
  if (cancelledQueuedDecodes * 2 < decodeQueue.length && decodeQueue.length <= MAX_QUEUED_DECODES) {
    return;
  }
  compactDecodeQueue();
}

function pumpDecodeQueue(): void {
  const concurrentLimit = albumGridScrolling
    ? MAX_SCROLL_CONCURRENT_DECODES
    : MAX_CONCURRENT_DECODES;

  while (inFlightDecodes < concurrentLimit && decodeQueue.length > 0) {
    const task = decodeQueue.shift();
    if (!task) continue;
    if (task.cancelled) {
      cancelledQueuedDecodes = Math.max(0, cancelledQueuedDecodes - 1);
      continue;
    }

    inFlightDecodes += 1;
    task.run()
      .catch(() => {
        // errors are handled by caller
      })
      .finally(() => {
        inFlightDecodes = Math.max(0, inFlightDecodes - 1);
        pumpDecodeQueue();
      });
  }
}

/**
 * Signals whether album-grid scrolling is active.
 * Scrolling mode uses the low-concurrency lane; settled mode restores throughput.
 */
export function setAlbumGridScrolling(scrolling: boolean): void {
  if (albumGridScrolling === scrolling) return;
  albumGridScrolling = scrolling;
  if (!scrolling) {
    pumpDecodeQueue();
  }
}

/**
 * Enqueues an artwork decode task and returns a cancellation callback.
 * Cancelled queued tasks are compacted aggressively to reduce stale memory pressure.
 */
export function enqueueArtworkDecode(run: () => Promise<void>): () => void {
  if (decodeQueue.length >= MAX_QUEUED_DECODES) {
    compactDecodeQueue();
  }

  const task: DecodeTask = {
    cancelled: false,
    run
  };

  decodeQueue.push(task);
  pumpDecodeQueue();

  return () => {
    if (task.cancelled) return;
    task.cancelled = true;
    cancelledQueuedDecodes += 1;
    maybeCompactDecodeQueue();
  };
}

/**
 * Compacts cancelled queued work and resumes draining when scrolling is idle.
 */
export function flushArtworkDecodeQueue(): void {
  compactDecodeQueue();
  if (!albumGridScrolling) {
    pumpDecodeQueue();
  }
}

/**
 * Returns whether a decoded thumb URL is in the ready set (decode skip fast-path).
 */
export function hasReadyThumb(url: string): boolean {
  return readyThumbs.has(url);
}

/**
 * Marks a decoded thumb URL as ready and applies FIFO/LRU-style cap eviction.
 */
export function markThumbReady(url: string): void {
  if (readyThumbs.has(url)) return;

  readyThumbs.add(url);
  readyThumbOrder.push(url);

  if (readyThumbOrder.length <= MAX_READY_THUMBS) return;

  const oldest = readyThumbOrder.shift();
  if (!oldest) return;
  readyThumbs.delete(oldest);
}

/**
 * Removes a URL from ready tracking (called when dev blob URLs are revoked).
 */
export function dropReadyThumb(url: string): void {
  if (!readyThumbs.delete(url)) return;

  const idx = readyThumbOrder.indexOf(url);
  if (idx >= 0) {
    readyThumbOrder.splice(idx, 1);
  }
}

/**
 * Returns decode queue diagnostics for dev/MCP inspection.
 */
export function getDecodeQueueStats(): {
  queueLength: number;
  inFlight: number;
  readyThumbCount: number;
  scrolling: boolean;
} {
  return {
    queueLength: decodeQueue.length,
    inFlight: inFlightDecodes,
    readyThumbCount: readyThumbs.size,
    scrolling: albumGridScrolling
  };
}

if (typeof window !== 'undefined' && import.meta.env.DEV) {
  (window as Window & {
    __decodeQueueStats?: () => ReturnType<typeof getDecodeQueueStats>;
  }).__decodeQueueStats = getDecodeQueueStats;
}
