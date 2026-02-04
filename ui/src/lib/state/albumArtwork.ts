import { writable } from 'svelte/store';
import type { AlbumListItem } from '../types/library';

const negativeCache = new Set<string>();
const artworkVersions = new Map<string, number>();

export const albumArtworkCacheVersion = writable(0);

function bumpCacheVersion(): void {
  albumArtworkCacheVersion.update((value) => value + 1);
}

function normalizeSegment(value: string): string {
  return value.trim();
}

export function getAlbumKey(album: AlbumListItem): string {
  return `${album.albumArtistSort}||${album.albumTitleSort}||${album.trackCount}`;
}

export function getAlbumArtworkVersion(album: AlbumListItem): number | undefined {
  return artworkVersions.get(getAlbumKey(album));
}

export function getAlbumArtworkUrl(
  album: AlbumListItem,
  size: number = 256,
  version?: number
): string | null {
  if (!album.albumArtistSort || !album.albumTitleSort) return null;
  
  // Prefer direct cache_key route when available (no DB lookup needed)
  if (album.artworkCacheKey) {
    const cacheKey = encodeURIComponent(album.artworkCacheKey);
    return `sermon-artwork://localhost/thumb/${cacheKey}?s=${size}`;
  }
  
  // Fallback to legacy album route (requires DB lookup)
  const artist = encodeURIComponent(normalizeSegment(album.albumArtistSort));
  const title = encodeURIComponent(normalizeSegment(album.albumTitleSort));
  let url = `sermon-artwork://localhost/album/${artist}/${title}?s=${size}`;
  if (version) {
    url += `&v=${version}`;
  }
  return url;
}

export function getAlbumArtworkSrc(
  album: AlbumListItem,
  size: number = 256,
  version?: number
): string | null {
  return getAlbumArtworkUrl(album, size, version);
}

export function bumpAlbumArtworkVersion(album: AlbumListItem): void {
  const key = getAlbumKey(album);
  artworkVersions.set(key, Date.now());
  negativeCache.delete(key);
  bumpCacheVersion();
}

export function resetAlbumArtworkCache(): void {
  negativeCache.clear();
  artworkVersions.clear();
  bumpCacheVersion();
}
