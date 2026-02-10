import type { AlbumListItem, LibraryStats } from '../types/library';

type AlbumsViewCacheState = {
  hydrated: boolean;
  items: AlbumListItem[];
  albumCount: number;
  lastScanCompletedMs?: number;
};

const albumsViewCache: AlbumsViewCacheState = {
  hydrated: false,
  items: [],
  albumCount: 0,
  lastScanCompletedMs: undefined
};

export function readAlbumsViewCache(): Readonly<AlbumsViewCacheState> {
  return albumsViewCache;
}

export function writeAlbumsViewCache(
  items: AlbumListItem[],
  stats: Pick<LibraryStats, 'albumCount' | 'lastScanCompletedMs'>
): void {
  albumsViewCache.hydrated = true;
  albumsViewCache.items = items;
  albumsViewCache.albumCount = stats.albumCount;
  albumsViewCache.lastScanCompletedMs = stats.lastScanCompletedMs;
}

export function clearAlbumsViewCache(): void {
  albumsViewCache.hydrated = false;
  albumsViewCache.items = [];
  albumsViewCache.albumCount = 0;
  albumsViewCache.lastScanCompletedMs = undefined;
}

export function isAlbumsViewCacheFresh(
  stats: Pick<LibraryStats, 'albumCount' | 'lastScanCompletedMs'>
): boolean {
  if (!albumsViewCache.hydrated) {
    return false;
  }

  if (stats.albumCount !== albumsViewCache.albumCount) {
    return false;
  }

  const incomingScan = stats.lastScanCompletedMs ?? null;
  const cachedScan = albumsViewCache.lastScanCompletedMs ?? null;
  return incomingScan === cachedScan;
}
