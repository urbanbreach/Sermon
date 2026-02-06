type DecodeTask = {
  cancelled: boolean;
  run: () => Promise<void>;
};

const MAX_CONCURRENT_DECODES = 6;
const MAX_SCROLL_CONCURRENT_DECODES = 1;
const MAX_READY_THUMBS = 4096;

let inFlightDecodes = 0;
let albumGridScrolling = false;

const decodeQueue: DecodeTask[] = [];
const readyThumbs = new Set<string>();
const readyThumbOrder: string[] = [];

function pumpDecodeQueue(): void {
  const concurrentLimit = albumGridScrolling
    ? MAX_SCROLL_CONCURRENT_DECODES
    : MAX_CONCURRENT_DECODES;

  while (inFlightDecodes < concurrentLimit && decodeQueue.length > 0) {
    const task = decodeQueue.shift();
    if (!task || task.cancelled) continue;

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

export function setAlbumGridScrolling(scrolling: boolean): void {
  if (albumGridScrolling === scrolling) return;
  albumGridScrolling = scrolling;
  if (!scrolling) {
    pumpDecodeQueue();
  }
}

export function enqueueArtworkDecode(run: () => Promise<void>): () => void {
  const task: DecodeTask = {
    cancelled: false,
    run
  };

  decodeQueue.push(task);
  pumpDecodeQueue();

  return () => {
    task.cancelled = true;
  };
}

export function hasReadyThumb(url: string): boolean {
  return readyThumbs.has(url);
}

export function markThumbReady(url: string): void {
  if (readyThumbs.has(url)) return;

  readyThumbs.add(url);
  readyThumbOrder.push(url);

  if (readyThumbOrder.length <= MAX_READY_THUMBS) return;

  const oldest = readyThumbOrder.shift();
  if (!oldest) return;
  readyThumbs.delete(oldest);
}
