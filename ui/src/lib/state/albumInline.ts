import { writable } from 'svelte/store';
import type { AlbumListItem } from '../types/library';

export const ALBUM_INLINE_CLOSE_DURATION_MS = 180;

export type AlbumInlineTarget = {
  albumArtistSort: string;
  albumTitleSort: string;
  isClosing?: boolean;
  animationNonce?: number;
};

export const expandedAlbum = writable<AlbumInlineTarget | null>(null);

let closeTimeoutId: ReturnType<typeof setTimeout> | null = null;
let animationNonceSeed = 0;

function isSameAlbum(a: Pick<AlbumInlineTarget, 'albumArtistSort' | 'albumTitleSort'>, b: Pick<AlbumInlineTarget, 'albumArtistSort' | 'albumTitleSort'>): boolean {
  return a.albumArtistSort === b.albumArtistSort && a.albumTitleSort === b.albumTitleSort;
}

function clearScheduledClose(): void {
  if (!closeTimeoutId) return;
  clearTimeout(closeTimeoutId);
  closeTimeoutId = null;
}

function getCloseDurationMs(): number {
  if (typeof window === 'undefined' || !window.matchMedia) {
    return ALBUM_INLINE_CLOSE_DURATION_MS;
  }
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches
    ? 0
    : ALBUM_INLINE_CLOSE_DURATION_MS;
}

function createOpenState(
  album: Pick<AlbumInlineTarget, 'albumArtistSort' | 'albumTitleSort'>,
  animationNonce = ++animationNonceSeed
): AlbumInlineTarget {
  return {
    albumArtistSort: album.albumArtistSort,
    albumTitleSort: album.albumTitleSort,
    isClosing: false,
    animationNonce
  };
}

function createClosingState(current: AlbumInlineTarget): AlbumInlineTarget | null {
  if (current.isClosing) {
    return current;
  }

  const closeDurationMs = getCloseDurationMs();
  if (closeDurationMs === 0) {
    clearScheduledClose();
    return null;
  }

  const closingState: AlbumInlineTarget = {
    ...current,
    isClosing: true
  };
  const closingTarget = {
    albumArtistSort: current.albumArtistSort,
    albumTitleSort: current.albumTitleSort
  };

  clearScheduledClose();
  closeTimeoutId = setTimeout(() => {
    closeTimeoutId = null;
    expandedAlbum.update((active) => {
      if (!active || !active.isClosing) return active;
      if (!isSameAlbum(active, closingTarget)) return active;
      return null;
    });
  }, closeDurationMs);

  return closingState;
}

function openAlbumInlineTarget(album: Pick<AlbumInlineTarget, 'albumArtistSort' | 'albumTitleSort'>): void {
  clearScheduledClose();
  expandedAlbum.update((current) => {
    if (!current) {
      return createOpenState(album);
    }

    if (!isSameAlbum(current, album)) {
      return createOpenState(album);
    }

    if (current.isClosing) {
      return createOpenState(current, current.animationNonce);
    }

    return current;
  });
}

export function openAlbumInline(album: AlbumInlineTarget): void {
  openAlbumInlineTarget(album);
}

export function openAlbumInlineFromItem(album: AlbumListItem): void {
  openAlbumInlineTarget({
    albumArtistSort: album.albumArtistSort,
    albumTitleSort: album.albumTitleSort
  });
}

export function toggleAlbumInline(album: AlbumInlineTarget): void {
  expandedAlbum.update((current) => {
    if (!current) {
      clearScheduledClose();
      return createOpenState(album);
    }

    if (isSameAlbum(current, album)) {
      if (current.isClosing) {
        clearScheduledClose();
        return createOpenState(current, current.animationNonce);
      }
      return createClosingState(current);
    }

    clearScheduledClose();
    return createOpenState(album);
  });
}

export function clearAlbumInline(): void {
  expandedAlbum.update((current) => {
    if (!current) return null;
    return createClosingState(current);
  });
}
