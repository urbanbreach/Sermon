import { get } from 'svelte/store';
import { beforeEach, describe, expect, test, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
import {
  fetchLyricsForTrack,
  findActiveLineIndex,
  lyricsData,
  lyricsStatus,
  parseLrc,
} from './lyrics';

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

describe('lyrics parser', () => {
  test('parseLrc: parses standard [mm:ss.xx] timestamps', () => {
    const lines = parseLrc('[00:01.50]Hello\n[01:02.34]World');

    expect(lines).toEqual([
      { timeMs: 1500, text: 'Hello' },
      { timeMs: 62_340, text: 'World' },
    ]);
  });

  test('parseLrc: parses [mm:ss.xxx] timestamps', () => {
    const lines = parseLrc('[00:01.234]Hello');

    expect(lines).toEqual([{ timeMs: 1234, text: 'Hello' }]);
  });

  test('parseLrc: handles multi-timestamp lines', () => {
    const lines = parseLrc('[00:15.00][00:45.00]Some lyrics');

    expect(lines).toEqual([
      { timeMs: 15_000, text: 'Some lyrics' },
      { timeMs: 45_000, text: 'Some lyrics' },
    ]);
  });

  test('parseLrc: applies offset tag', () => {
    const lines = parseLrc('[offset:+500]\n[00:01.00]Hello');

    expect(lines).toEqual([{ timeMs: 1500, text: 'Hello' }]);
  });

  test('parseLrc: skips metadata tags', () => {
    const lines = parseLrc([
      '[ar:Artist]',
      '[ti:Title]',
      '[al:Album]',
      '[length:03:00]',
      '[00:10.00]Line',
    ].join('\n'));

    expect(lines).toEqual([{ timeMs: 10_000, text: 'Line' }]);
  });

  test('parseLrc: returns empty for empty input', () => {
    expect(parseLrc('')).toEqual([]);
    expect(parseLrc('   \n\n')).toEqual([]);
  });

  test('parseLrc: sorts by timestamp', () => {
    const lines = parseLrc('[00:20.00]Second\n[00:10.00]First');

    expect(lines).toEqual([
      { timeMs: 10_000, text: 'First' },
      { timeMs: 20_000, text: 'Second' },
    ]);
  });
});

describe('active line binary search', () => {
  const timedLines = [
    { timeMs: 10_000, text: 'Line 1' },
    { timeMs: 20_000, text: 'Line 2' },
    { timeMs: 30_000, text: 'Line 3' },
  ];

  test('findActiveLineIndex: returns 0 before first line', () => {
    expect(findActiveLineIndex(timedLines, 5000)).toBe(0);
  });

  test('findActiveLineIndex: returns correct index mid-song', () => {
    expect(findActiveLineIndex(timedLines, 25_000)).toBe(1);
  });

  test('findActiveLineIndex: returns last index past end', () => {
    expect(findActiveLineIndex(timedLines, 99_000)).toBe(2);
  });

  test('findActiveLineIndex: empty lines returns 0', () => {
    expect(findActiveLineIndex([], 25_000)).toBe(0);
  });
});

describe('lyrics fetch lifecycle', () => {
  const invokeMock = vi.mocked(invoke);

  beforeEach(() => {
    invokeMock.mockReset();
    lyricsData.set(null);
    lyricsStatus.set('idle');
  });

  test('fetchLyricsForTrack: sets status to loading then ready', async () => {
    invokeMock.mockResolvedValue({
      trackId: 42,
      syncedLyrics: '[00:01.00]Hello',
      plainLyrics: null,
      source: 'embedded',
    });

    const fetchPromise = fetchLyricsForTrack(42);

    expect(get(lyricsStatus)).toBe('loading');

    await fetchPromise;

    expect(get(lyricsStatus)).toBe('ready');
    expect(get(lyricsData)).toEqual({
      trackId: 42,
      lines: [{ timeMs: 1000, text: 'Hello' }],
      plainLines: [],
      isSynced: true,
      source: 'embedded',
    });
  });

  test('fetchLyricsForTrack: stale requests are discarded', async () => {
    const first = createDeferred<{
      trackId: number;
      syncedLyrics: string | null;
      plainLyrics: string | null;
      source: string;
    }>();
    const second = createDeferred<{
      trackId: number;
      syncedLyrics: string | null;
      plainLyrics: string | null;
      source: string;
    }>();

    invokeMock.mockImplementation(((_command: string, payload?: unknown) => {
      const req = payload as { request: { trackId: number } } | undefined;
      if (req?.request.trackId === 1) {
        return first.promise;
      }
      return second.promise;
    }) as typeof invokeMock);

    const firstRequest = fetchLyricsForTrack(1);
    const secondRequest = fetchLyricsForTrack(2);

    second.resolve({
      trackId: 2,
      syncedLyrics: '[00:02.00]Second track line',
      plainLyrics: null,
      source: 'cache',
    });
    await secondRequest;

    expect(get(lyricsStatus)).toBe('ready');
    expect(get(lyricsData)?.trackId).toBe(2);

    first.resolve({
      trackId: 1,
      syncedLyrics: '[00:01.00]First track line',
      plainLyrics: null,
      source: 'cache',
    });
    await firstRequest;

    expect(get(lyricsStatus)).toBe('ready');
    expect(get(lyricsData)?.trackId).toBe(2);
  });
});
