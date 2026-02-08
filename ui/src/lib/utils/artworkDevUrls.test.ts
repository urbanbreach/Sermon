import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

vi.mock('../api/artwork', () => ({
  getArtworkThumbBytes: vi.fn()
}));

vi.mock('./artworkDecodeQueue', () => ({
  dropReadyThumb: vi.fn()
}));

type Deferred<T> = {
  promise: Promise<T>;
  resolve: (value: T) => void;
};

function createDeferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((res) => {
    resolve = res;
  });
  return { promise, resolve };
}

let nowMs = 0;
let blobCounter = 0;
let createObjectURLMock: ReturnType<typeof vi.fn>;
let revokeObjectURLMock: ReturnType<typeof vi.fn>;
let originalCreateObjectURL: typeof URL.createObjectURL;
let originalRevokeObjectURL: typeof URL.revokeObjectURL;

function advance(ms: number): void {
  nowMs += ms;
  vi.advanceTimersByTime(ms);
}

async function loadArtworkDevUrls() {
  const module = await import('./artworkDevUrls');
  const artworkApi = await import('../api/artwork');
  const decodeQueue = await import('./artworkDecodeQueue');

  return {
    ...module,
    getArtworkThumbBytesMock: vi.mocked(artworkApi.getArtworkThumbBytes),
    dropReadyThumbMock: vi.mocked(decodeQueue.dropReadyThumb)
  };
}

describe('artworkDevUrls cache lifecycle', () => {
  beforeEach(() => {
    vi.resetModules();
    vi.useFakeTimers();
    vi.clearAllMocks();

    nowMs = 0;
    blobCounter = 0;

    originalCreateObjectURL = URL.createObjectURL;
    originalRevokeObjectURL = URL.revokeObjectURL;

    createObjectURLMock = vi.fn(() => {
      blobCounter += 1;
      return `blob:test-${blobCounter}`;
    });
    revokeObjectURLMock = vi.fn();

    Object.defineProperty(URL, 'createObjectURL', {
      configurable: true,
      writable: true,
      value: createObjectURLMock
    });
    Object.defineProperty(URL, 'revokeObjectURL', {
      configurable: true,
      writable: true,
      value: revokeObjectURLMock
    });

    Object.defineProperty(navigator, 'deviceMemory', {
      configurable: true,
      value: 1
    });

    vi.spyOn(performance, 'now').mockImplementation(() => nowMs);
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
    vi.restoreAllMocks();

    Object.defineProperty(URL, 'createObjectURL', {
      configurable: true,
      writable: true,
      value: originalCreateObjectURL
    });
    Object.defineProperty(URL, 'revokeObjectURL', {
      configurable: true,
      writable: true,
      value: originalRevokeObjectURL
    });
  });

  test('getDevArtworkUrl: creates blob URL and increments refCount', async () => {
    const {
      getDevArtworkUrl,
      getDevUrlCacheStats,
      getArtworkThumbBytesMock
    } = await loadArtworkDevUrls();

    getArtworkThumbBytesMock.mockResolvedValue({
      bytesBase64: btoa('fake-image-data'),
      mime: 'image/jpeg'
    });

    const url = await getDevArtworkUrl('album-1', 256);

    expect(url).toBe('blob:test-1');
    expect(createObjectURLMock).toHaveBeenCalledTimes(1);
    expect(getArtworkThumbBytesMock).toHaveBeenCalledTimes(1);
    expect(getDevUrlCacheStats()).toEqual({ size: 1, liveRefCount: 1, zeroRefCount: 0 });
  });

  test('getDevArtworkUrl: returns cached URL on second acquire for same key', async () => {
    const { getDevArtworkUrl, getArtworkThumbBytesMock } = await loadArtworkDevUrls();

    getArtworkThumbBytesMock.mockResolvedValue({
      bytesBase64: btoa('fake-image-data'),
      mime: 'image/jpeg'
    });

    const first = await getDevArtworkUrl('album-1', 256);
    const second = await getDevArtworkUrl('album-1', 256);

    expect(first).toBe(second);
    expect(createObjectURLMock).toHaveBeenCalledTimes(1);
    expect(getArtworkThumbBytesMock).toHaveBeenCalledTimes(1);
  });

  test('getDevArtworkUrl: increments refCount on re-acquire', async () => {
    const { getDevArtworkUrl, releaseDevArtworkUrl, getDevUrlCacheStats, getArtworkThumbBytesMock } =
      await loadArtworkDevUrls();

    getArtworkThumbBytesMock.mockResolvedValue({
      bytesBase64: btoa('fake-image-data'),
      mime: 'image/jpeg'
    });

    const url = await getDevArtworkUrl('album-1', 256);
    await getDevArtworkUrl('album-1', 256);
    releaseDevArtworkUrl(url);

    expect(getDevUrlCacheStats()).toEqual({ size: 1, liveRefCount: 1, zeroRefCount: 0 });
    expect(revokeObjectURLMock).not.toHaveBeenCalled();
  });

  test('releaseDevArtworkUrl: decrements refCount without revoking when > 0', async () => {
    const { getDevArtworkUrl, releaseDevArtworkUrl, getArtworkThumbBytesMock } =
      await loadArtworkDevUrls();

    getArtworkThumbBytesMock.mockResolvedValue({
      bytesBase64: btoa('fake-image-data'),
      mime: 'image/jpeg'
    });

    const url = await getDevArtworkUrl('album-1', 256);
    await getDevArtworkUrl('album-1', 256);
    releaseDevArtworkUrl(url);

    expect(revokeObjectURLMock).not.toHaveBeenCalled();
  });

  test('releaseDevArtworkUrl: does not revoke URL while refCount > 0', async () => {
    const { getDevArtworkUrl, releaseDevArtworkUrl, getArtworkThumbBytesMock } =
      await loadArtworkDevUrls();

    getArtworkThumbBytesMock.mockResolvedValue({
      bytesBase64: btoa('fake-image-data'),
      mime: 'image/jpeg'
    });

    const url = await getDevArtworkUrl('album-1', 256);
    await getDevArtworkUrl('album-1', 256);
    releaseDevArtworkUrl(url);

    advance(20_000);

    expect(revokeObjectURLMock).not.toHaveBeenCalled();
  });

  test('releaseDevArtworkUrl: revokes after TTL expiry when refCount hits 0', async () => {
    const { getDevArtworkUrl, releaseDevArtworkUrl, dropReadyThumbMock, getArtworkThumbBytesMock } =
      await loadArtworkDevUrls();

    getArtworkThumbBytesMock.mockResolvedValue({
      bytesBase64: btoa('fake-image-data'),
      mime: 'image/jpeg'
    });

    const url = await getDevArtworkUrl('album-1', 256);
    releaseDevArtworkUrl(url);

    advance(3_128);

    expect(dropReadyThumbMock).toHaveBeenCalledWith(url);
    expect(revokeObjectURLMock).toHaveBeenCalledWith(url);
  });

  test('retainDevArtworkUrl: increments refCount for known URL', async () => {
    const {
      getDevArtworkUrl,
      releaseDevArtworkUrl,
      retainDevArtworkUrl,
      getDevUrlCacheStats,
      getArtworkThumbBytesMock
    } = await loadArtworkDevUrls();

    getArtworkThumbBytesMock.mockResolvedValue({
      bytesBase64: btoa('fake-image-data'),
      mime: 'image/jpeg'
    });

    const url = await getDevArtworkUrl('album-1', 256);
    releaseDevArtworkUrl(url);
    retainDevArtworkUrl(url);

    advance(10_000);

    expect(getDevUrlCacheStats()).toEqual({ size: 1, liveRefCount: 1, zeroRefCount: 0 });
    expect(revokeObjectURLMock).not.toHaveBeenCalled();
  });

  test('retainDevArtworkUrl: no-op for unknown URL', async () => {
    const { retainDevArtworkUrl, getDevUrlCacheStats } = await loadArtworkDevUrls();

    retainDevArtworkUrl('blob:unknown');

    expect(getDevUrlCacheStats()).toEqual({ size: 0, liveRefCount: 0, zeroRefCount: 0 });
    expect(revokeObjectURLMock).not.toHaveBeenCalled();
  });

  describe('pressure-aware behavior', () => {
    test('notifyScrollPressure(true): triggers immediate prune of zero-ref entries', async () => {
      const {
        getDevArtworkUrl,
        releaseDevArtworkUrl,
        notifyScrollPressure,
        getArtworkThumbBytesMock
      } = await loadArtworkDevUrls();

      getArtworkThumbBytesMock.mockResolvedValue({
        bytesBase64: btoa('fake-image-data'),
        mime: 'image/jpeg'
      });

      const url = await getDevArtworkUrl('album-1', 256);
      releaseDevArtworkUrl(url);
      advance(1_100);

      notifyScrollPressure(true);

      expect(revokeObjectURLMock).toHaveBeenCalledWith(url);
    });

    test('notifyScrollPressure(true): uses shorter TTL for prune', async () => {
      const {
        getDevArtworkUrl,
        releaseDevArtworkUrl,
        notifyScrollPressure,
        getArtworkThumbBytesMock
      } = await loadArtworkDevUrls();

      getArtworkThumbBytesMock.mockResolvedValue({
        bytesBase64: btoa('fake-image-data'),
        mime: 'image/jpeg'
      });

      const url = await getDevArtworkUrl('album-1', 256);
      releaseDevArtworkUrl(url);

      advance(1_100);
      expect(revokeObjectURLMock).not.toHaveBeenCalled();

      notifyScrollPressure(true);

      expect(revokeObjectURLMock).toHaveBeenCalledWith(url);
    });

    test('notifyScrollPressure(false): clears pressure timer', async () => {
      const { notifyScrollPressure } = await loadArtworkDevUrls();

      notifyScrollPressure(true);
      expect(vi.getTimerCount()).toBe(1);

      notifyScrollPressure(false);

      expect(vi.getTimerCount()).toBe(1);
    });
  });

  describe('cache limit enforcement', () => {
    test('enforceCacheLimit evicts LRU zero-ref entries when over limit', async () => {
      const {
        getDevArtworkUrl,
        releaseDevArtworkUrl,
        getDevUrlCacheStats,
        getArtworkThumbBytesMock
      } = await loadArtworkDevUrls();

      getArtworkThumbBytesMock.mockResolvedValue({
        bytesBase64: btoa('fake-image-data'),
        mime: 'image/jpeg'
      });

      const maxEntries = 280;
      const urls: string[] = [];
      for (let i = 0; i < maxEntries; i += 1) {
        urls.push(await getDevArtworkUrl(`album-${i}`, 256));
      }

      releaseDevArtworkUrl(urls[0]);
      advance(1);
      releaseDevArtworkUrl(urls[1]);

      await getDevArtworkUrl('album-overflow', 256);

      expect(revokeObjectURLMock).toHaveBeenCalled();
      expect(revokeObjectURLMock.mock.calls[0]?.[0]).toBe(urls[0]);
      expect(getDevUrlCacheStats().size).toBeLessThanOrEqual(maxEntries);
    });
  });

  describe('negative paths', () => {
    test('does not revoke URL when refCount > 0 (safety invariant)', async () => {
      const { getDevArtworkUrl, releaseDevArtworkUrl, getArtworkThumbBytesMock } =
        await loadArtworkDevUrls();

      getArtworkThumbBytesMock.mockResolvedValue({
        bytesBase64: btoa('fake-image-data'),
        mime: 'image/jpeg'
      });

      const url = await getDevArtworkUrl('album-1', 256);
      await getDevArtworkUrl('album-1', 256);
      releaseDevArtworkUrl(url);
      advance(15_000);

      expect(revokeObjectURLMock).not.toHaveBeenCalled();
    });

    test('handles release of unknown URL gracefully', async () => {
      const { releaseDevArtworkUrl, dropReadyThumbMock } = await loadArtworkDevUrls();

      expect(() => releaseDevArtworkUrl('blob:unknown')).not.toThrow();
      expect(dropReadyThumbMock).toHaveBeenCalledWith('blob:unknown');
      expect(revokeObjectURLMock).toHaveBeenCalledWith('blob:unknown');
    });

    test('handles concurrent acquire of same key without duplication', async () => {
      const { getDevArtworkUrl, getArtworkThumbBytesMock } = await loadArtworkDevUrls();

      const deferred = createDeferred<{ bytesBase64: string; mime: string }>();
      getArtworkThumbBytesMock.mockImplementation(() => deferred.promise);

      const first = getDevArtworkUrl('album-1', 256);
      const second = getDevArtworkUrl('album-1', 256);

      expect(getArtworkThumbBytesMock).toHaveBeenCalledTimes(1);

      deferred.resolve({
        bytesBase64: btoa('fake-image-data'),
        mime: 'image/jpeg'
      });

      const [firstUrl, secondUrl] = await Promise.all([first, second]);
      expect(firstUrl).toBe(secondUrl);
      expect(createObjectURLMock).toHaveBeenCalledTimes(1);
    });

    test('prevents refCount underflow on repeated release', async () => {
      const {
        getDevArtworkUrl,
        releaseDevArtworkUrl,
        getDevUrlCacheStats,
        getArtworkThumbBytesMock
      } = await loadArtworkDevUrls();

      getArtworkThumbBytesMock.mockResolvedValue({
        bytesBase64: btoa('fake-image-data'),
        mime: 'image/jpeg'
      });

      const url = await getDevArtworkUrl('album-1', 256);
      releaseDevArtworkUrl(url);
      releaseDevArtworkUrl(url);

      expect(getDevUrlCacheStats()).toEqual({ size: 1, liveRefCount: 0, zeroRefCount: 1 });

      advance(3_128);

      expect(revokeObjectURLMock).toHaveBeenCalledTimes(1);
      expect(revokeObjectURLMock).toHaveBeenCalledWith(url);
    });
  });

  describe('debug stats', () => {
    test('getDevUrlCacheStats returns correct size/liveRef/zeroRef counts', async () => {
      const { getDevArtworkUrl, releaseDevArtworkUrl, getDevUrlCacheStats, getArtworkThumbBytesMock } =
        await loadArtworkDevUrls();

      getArtworkThumbBytesMock.mockResolvedValue({
        bytesBase64: btoa('fake-image-data'),
        mime: 'image/jpeg'
      });

      const liveUrl = await getDevArtworkUrl('album-live', 256);
      await getDevArtworkUrl('album-zero', 256);
      const zeroUrl = await getDevArtworkUrl('album-zero', 256);

      releaseDevArtworkUrl(zeroUrl);
      releaseDevArtworkUrl(zeroUrl);

      expect(getDevUrlCacheStats()).toEqual({ size: 2, liveRefCount: 1, zeroRefCount: 1 });

      releaseDevArtworkUrl(liveUrl);
      advance(3_128);
    });
  });
});
