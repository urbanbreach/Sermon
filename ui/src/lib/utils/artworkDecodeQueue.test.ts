import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

type Deferred = {
  promise: Promise<void>;
  resolve: () => void;
  reject: (reason?: unknown) => void;
};

function createDeferred(): Deferred {
  let resolve!: () => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<void>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

async function flushMicrotasks(passes = 6): Promise<void> {
  for (let i = 0; i < passes; i += 1) {
    await Promise.resolve();
  }
}

async function loadDecodeQueueModule() {
  return import('./artworkDecodeQueue');
}

describe('artworkDecodeQueue', () => {
  beforeEach(() => {
    vi.resetModules();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  test('enqueueArtworkDecode: executes tasks up to concurrency limit', async () => {
    const { enqueueArtworkDecode, getDecodeQueueStats } = await loadDecodeQueueModule();

    const started: number[] = [];
    const deferredTasks = Array.from({ length: 8 }, () => createDeferred());

    for (let i = 0; i < deferredTasks.length; i += 1) {
      enqueueArtworkDecode(() => {
        started.push(i);
        return deferredTasks[i].promise;
      });
    }

    expect(started).toHaveLength(6);
    expect(getDecodeQueueStats()).toMatchObject({ queueLength: 2, inFlight: 6 });

    deferredTasks[0].resolve();
    await flushMicrotasks();

    expect(started).toHaveLength(7);
    expect(getDecodeQueueStats()).toMatchObject({ queueLength: 1, inFlight: 6 });

    for (const deferred of deferredTasks) {
      deferred.resolve();
    }
    await flushMicrotasks();

    expect(getDecodeQueueStats()).toMatchObject({ queueLength: 0, inFlight: 0 });
  });

  test('enqueueArtworkDecode: respects scroll concurrency limit (MAX_SCROLL_CONCURRENT_DECODES=1)', async () => {
    const { enqueueArtworkDecode, getDecodeQueueStats, setAlbumGridScrolling } =
      await loadDecodeQueueModule();

    setAlbumGridScrolling(true);

    const started: number[] = [];
    const deferredTasks = Array.from({ length: 3 }, () => createDeferred());

    for (let i = 0; i < deferredTasks.length; i += 1) {
      enqueueArtworkDecode(() => {
        started.push(i);
        return deferredTasks[i].promise;
      });
    }

    expect(started).toHaveLength(1);
    expect(getDecodeQueueStats()).toMatchObject({ queueLength: 2, inFlight: 1, scrolling: true });

    deferredTasks[0].resolve();
    await flushMicrotasks();

    expect(started).toHaveLength(2);

    deferredTasks[1].resolve();
    deferredTasks[2].resolve();
    await flushMicrotasks();
  });

  test('cancel callback: marks task as cancelled', async () => {
    const { enqueueArtworkDecode, setAlbumGridScrolling, getDecodeQueueStats } =
      await loadDecodeQueueModule();

    setAlbumGridScrolling(true);

    const blocker = createDeferred();
    enqueueArtworkDecode(() => blocker.promise);

    let ranCancelledTask = false;
    const cancel = enqueueArtworkDecode(async () => {
      ranCancelledTask = true;
    });

    cancel();
    blocker.resolve();
    await flushMicrotasks();

    expect(ranCancelledTask).toBe(false);
    expect(getDecodeQueueStats()).toMatchObject({ queueLength: 0, inFlight: 0 });
  });

  test('cancel callback: is idempotent (double-cancel safe)', async () => {
    const { enqueueArtworkDecode, setAlbumGridScrolling } = await loadDecodeQueueModule();

    setAlbumGridScrolling(true);

    const blocker = createDeferred();
    enqueueArtworkDecode(() => blocker.promise);

    let cancelledTaskRan = false;
    const cancel = enqueueArtworkDecode(async () => {
      cancelledTaskRan = true;
    });

    expect(() => {
      cancel();
      cancel();
    }).not.toThrow();

    blocker.resolve();
    await flushMicrotasks();

    expect(cancelledTaskRan).toBe(false);
  });

  test('compaction: removes cancelled tasks from queue', async () => {
    const {
      enqueueArtworkDecode,
      flushArtworkDecodeQueue,
      getDecodeQueueStats,
      setAlbumGridScrolling
    } = await loadDecodeQueueModule();

    setAlbumGridScrolling(true);

    const blocker = createDeferred();
    enqueueArtworkDecode(() => blocker.promise);

    const cancels: Array<() => void> = [];
    for (let i = 0; i < 10; i += 1) {
      cancels.push(
        enqueueArtworkDecode(async () => {
          // no-op
        })
      );
    }

    for (let i = 0; i < 6; i += 1) {
      cancels[i]();
    }

    expect(getDecodeQueueStats()).toMatchObject({ queueLength: 10, inFlight: 1 });

    flushArtworkDecodeQueue();

    expect(getDecodeQueueStats()).toMatchObject({ queueLength: 4, inFlight: 1 });

    blocker.resolve();
    await flushMicrotasks();
  });

  test('compaction triggers at threshold', async () => {
    const { enqueueArtworkDecode, getDecodeQueueStats, setAlbumGridScrolling } =
      await loadDecodeQueueModule();

    setAlbumGridScrolling(true);

    const blocker = createDeferred();
    enqueueArtworkDecode(() => blocker.promise);

    const cancels: Array<() => void> = [];
    for (let i = 0; i < 60; i += 1) {
      cancels.push(
        enqueueArtworkDecode(async () => {
          // no-op
        })
      );
    }

    for (let i = 0; i < 48; i += 1) {
      cancels[i]();
    }

    expect(getDecodeQueueStats()).toMatchObject({ queueLength: 12, inFlight: 1 });

    blocker.resolve();
    await flushMicrotasks();
  });

  describe('ready thumb tracking', () => {
    test('markThumbReady: adds URL to ready set', async () => {
      const { hasReadyThumb, markThumbReady } = await loadDecodeQueueModule();

      markThumbReady('blob:one');

      expect(hasReadyThumb('blob:one')).toBe(true);
    });

    test('hasReadyThumb: returns true for ready URLs', async () => {
      const { hasReadyThumb, markThumbReady } = await loadDecodeQueueModule();

      markThumbReady('blob:ready');

      expect(hasReadyThumb('blob:ready')).toBe(true);
      expect(hasReadyThumb('blob:missing')).toBe(false);
    });

    test('markThumbReady: evicts oldest when exceeding MAX_READY_THUMBS', async () => {
      const { getDecodeQueueStats, hasReadyThumb, markThumbReady } = await loadDecodeQueueModule();

      for (let i = 0; i <= 512; i += 1) {
        markThumbReady(`blob:${i}`);
      }

      expect(hasReadyThumb('blob:0')).toBe(false);
      expect(hasReadyThumb('blob:1')).toBe(true);
      expect(hasReadyThumb('blob:512')).toBe(true);
      expect(getDecodeQueueStats().readyThumbCount).toBe(512);
    });

    test('dropReadyThumb: removes URL from tracking', async () => {
      const { dropReadyThumb, hasReadyThumb, markThumbReady } = await loadDecodeQueueModule();

      markThumbReady('blob:drop-me');
      expect(hasReadyThumb('blob:drop-me')).toBe(true);

      dropReadyThumb('blob:drop-me');

      expect(hasReadyThumb('blob:drop-me')).toBe(false);
    });
  });

  describe('scroll mode', () => {
    test('setAlbumGridScrolling(true): limits concurrency', async () => {
      const { enqueueArtworkDecode, setAlbumGridScrolling } = await loadDecodeQueueModule();

      setAlbumGridScrolling(true);

      const started: number[] = [];
      const deferredTasks = Array.from({ length: 4 }, () => createDeferred());

      for (let i = 0; i < deferredTasks.length; i += 1) {
        enqueueArtworkDecode(() => {
          started.push(i);
          return deferredTasks[i].promise;
        });
      }

      expect(started).toEqual([0]);

      for (const deferred of deferredTasks) {
        deferred.resolve();
      }
      await flushMicrotasks();
    });

    test('setAlbumGridScrolling(false): resumes full concurrency', async () => {
      const { enqueueArtworkDecode, getDecodeQueueStats, setAlbumGridScrolling } =
        await loadDecodeQueueModule();

      setAlbumGridScrolling(true);

      const started: number[] = [];
      const deferredTasks = Array.from({ length: 8 }, () => createDeferred());

      for (let i = 0; i < deferredTasks.length; i += 1) {
        enqueueArtworkDecode(() => {
          started.push(i);
          return deferredTasks[i].promise;
        });
      }

      expect(started).toHaveLength(1);

      setAlbumGridScrolling(false);

      expect(started).toHaveLength(6);
      expect(getDecodeQueueStats()).toMatchObject({ queueLength: 2, inFlight: 6, scrolling: false });

      for (const deferred of deferredTasks) {
        deferred.resolve();
      }
      await flushMicrotasks();
    });
  });

  describe('debug stats', () => {
    test('getDecodeQueueStats: returns correct queue/inflight/ready counts', async () => {
      const { enqueueArtworkDecode, getDecodeQueueStats, markThumbReady, setAlbumGridScrolling } =
        await loadDecodeQueueModule();

      setAlbumGridScrolling(true);

      const blocker = createDeferred();
      enqueueArtworkDecode(() => blocker.promise);
      enqueueArtworkDecode(async () => {
        // queued until blocker resolves
      });

      markThumbReady('blob:stats-ready');

      expect(getDecodeQueueStats()).toEqual({
        queueLength: 1,
        inFlight: 1,
        readyThumbCount: 1,
        scrolling: true
      });

      blocker.resolve();
      await flushMicrotasks();
    });
  });
});
