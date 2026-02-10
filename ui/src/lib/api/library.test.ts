import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
import { Fixtures } from '../data/fixtures';
import { listAlbumsPage } from './library';

function toAlbumSortKeys(items: Array<{ albumArtistSort: string; albumTitleSort: string }>): string[] {
  return items.map((item) => `${item.albumArtistSort}::${item.albumTitleSort}`);
}

describe('library API mock-mode pagination', () => {
  const invokeMock = vi.mocked(invoke);

  beforeEach(() => {
    invokeMock.mockReset();
    vi.stubEnv('SERMON_MOCK', '1');
  });

  afterEach(() => {
    vi.unstubAllEnvs();
  });

  test('listAlbumsPage: returns deterministic ordering in mock mode', async () => {
    const first = await listAlbumsPage(20);
    const second = await listAlbumsPage(20);

    const expectedOrderedKeys = Fixtures.getAlbums()
      .map((album) => {
        const artist = Fixtures.getArtist(album.artistId);
        return {
          albumArtistSort: (artist?.name && artist.name.trim().toLowerCase()) || 'unknown artist',
          albumTitleSort: (album.title && album.title.trim().toLowerCase()) || 'unknown album',
        };
      })
      .sort((a, b) => {
        const byArtist = a.albumArtistSort.localeCompare(b.albumArtistSort);
        if (byArtist !== 0) {
          return byArtist;
        }

        return a.albumTitleSort.localeCompare(b.albumTitleSort);
      });

    expect(first).toEqual(second);
    expect(toAlbumSortKeys(first.items)).toEqual(toAlbumSortKeys(expectedOrderedKeys));
    expect(invokeMock).not.toHaveBeenCalled();
  });

  test('listAlbumsPage: cursor pagination returns stable non-overlapping pages', async () => {
    const page1 = await listAlbumsPage(2);

    expect(page1.items).toHaveLength(2);
    expect(page1.nextCursor).toEqual({
      albumArtistSort: page1.items[1].albumArtistSort,
      albumTitleSort: page1.items[1].albumTitleSort,
    });

    const page2 = await listAlbumsPage(2, page1.nextCursor);

    expect(page2.items).toHaveLength(2);

    const page1Keys = toAlbumSortKeys(page1.items);
    const page2Keys = toAlbumSortKeys(page2.items);
    const overlap = page2Keys.filter((key) => page1Keys.includes(key));
    expect(overlap).toEqual([]);
  });

  test('listAlbumsPage: iterates through all pages and ends with undefined cursor', async () => {
    const allKeys: string[] = [];
    let cursor: { albumArtistSort: string; albumTitleSort: string } | undefined;

    while (true) {
      const page = await listAlbumsPage(2, cursor);
      allKeys.push(...toAlbumSortKeys(page.items));

      if (!page.nextCursor) {
        break;
      }

      cursor = page.nextCursor;
    }

    expect(allKeys).toHaveLength(Fixtures.getAlbums().length);
    expect(new Set(allKeys).size).toBe(allKeys.length);
  });
});
