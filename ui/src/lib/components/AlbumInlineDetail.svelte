<script lang="ts">
  import { X } from '@lucide/svelte';
  import { listAlbumTracksPage } from '../api/library';
  import type { AlbumListItem, TrackRow } from '../types/library';
  import { getAlbumKey } from '../state/albumArtwork';
  import ArtworkImage from './ArtworkImage.svelte';

  interface Props {
    album: AlbumListItem;
    onClose?: () => void;
  }

  let { album, onClose }: Props = $props();

  let tracks = $state<TrackRow[]>([]);
  let loading = $state(false);
  let loadError = $state<string | null>(null);
  let activeAlbumKey = $state('');

  let backgroundArtUrl = $derived.by(() => {
    if (!album.artworkCacheKey) return '';
    const key = encodeURIComponent(album.artworkCacheKey);
    return `sermon-artwork://localhost/thumb/${key}?s=512`;
  });

  let detailEl = $state<HTMLDivElement | null>(null);
  let headerRowEl = $state<HTMLDivElement | null>(null);
  let trackColumnsEl = $state<HTMLDivElement | null>(null);
  let trackColumnsWidth = $state(0);
  let trackRowHeight = $state(0);
  let artworkSize = $state(0);
  let headerRowHeight = $state(0);

  const COLUMN_GAP = 24;
  const MIN_COLUMN_WIDTH = 240;
  const MAX_COLUMNS = 3;
  const DEFAULT_ROW_HEIGHT = 22;
  const DEFAULT_ARTWORK_SIZE = 200;
  const DEFAULT_HEADER_HEIGHT = 48;
  const HEADER_GAP = 10;
  const DEFAULT_MAX_ROWS = 6;

  type TrackLayout = {
    columns: number;
    height: string;
  };

  let trackLayout = $derived.by<TrackLayout>(() => {
    const trackCount = tracks.length;
    if (trackCount === 0) {
      return { columns: 1, height: 'auto' };
    }

    const availableWidth = trackColumnsWidth;
    const effectiveArtworkSize = artworkSize || DEFAULT_ARTWORK_SIZE;
    const effectiveRowHeight = trackRowHeight || DEFAULT_ROW_HEIGHT;
    const effectiveHeaderHeight = headerRowHeight || DEFAULT_HEADER_HEIGHT;
    const targetTrackHeight = Math.max(
      0,
      effectiveArtworkSize - effectiveHeaderHeight - HEADER_GAP
    );
    const maxColumnsByWidth = Math.max(
      1,
      Math.floor((availableWidth + COLUMN_GAP) / (MIN_COLUMN_WIDTH + COLUMN_GAP))
    );
    const maxColumns = Math.min(MAX_COLUMNS, maxColumnsByWidth);
    const maxRowsByHeight = Math.floor(targetTrackHeight / effectiveRowHeight);
    const maxRowsPerColumn = maxRowsByHeight > 0 ? maxRowsByHeight : DEFAULT_MAX_ROWS;
    const columnsNeeded = Math.ceil(trackCount / maxRowsPerColumn);
    const columns = Math.max(1, Math.min(columnsNeeded, maxColumns));
    const rowsPerColumn = Math.ceil(trackCount / columns);
    const neededHeight = rowsPerColumn * effectiveRowHeight;
    const height = Math.max(targetTrackHeight, neededHeight);

    return { columns, height: `${Math.round(height)}px` };
  });

  let albumTitle = $derived(album.albumTitleDisplay || 'Unknown Album');
  let albumArtist = $derived(album.albumArtistDisplay || 'Unknown Artist');
  let totalDuration = $derived(tracks.reduce((acc, t) => acc + (t.durationMs || 0), 0));

  $effect(() => {
    const key = getAlbumKey(album);
    if (key === activeAlbumKey) return;
    activeAlbumKey = key;
    tracks = [];
    loadTracks(key);
  });

  $effect(() => {
    const el = detailEl;
    if (!el) return;

    const updateArtworkSize = () => {
      const style = getComputedStyle(el);
      const size = Number.parseFloat(style.getPropertyValue('--artwork-size'));
      if (!Number.isNaN(size) && size > 0) {
        artworkSize = size;
      }
    };

    updateArtworkSize();
    const observer = new ResizeObserver(updateArtworkSize);
    observer.observe(el);
    return () => observer.disconnect();
  });

  $effect(() => {
    const el = headerRowEl;
    if (!el) return;

    const updateHeaderHeight = () => {
      const height = el.getBoundingClientRect().height;
      if (height > 0) {
        headerRowHeight = height;
      }
    };

    updateHeaderHeight();
    const observer = new ResizeObserver(updateHeaderHeight);
    observer.observe(el);
    return () => observer.disconnect();
  });

  $effect(() => {
    if (!trackColumnsEl) return;

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (entry) {
        trackColumnsWidth = entry.contentRect.width;
      }
    });
    observer.observe(trackColumnsEl);
    return () => observer.disconnect();
  });

  $effect(() => {
    if (!trackColumnsEl || tracks.length === 0) return;
    const firstRow = trackColumnsEl.querySelector('.track-row');
    if (!(firstRow instanceof HTMLElement)) return;

    const updateRowHeight = () => {
      const height = firstRow.getBoundingClientRect().height;
      if (height > 0) {
        trackRowHeight = height;
      }
    };

    updateRowHeight();
    const observer = new ResizeObserver(updateRowHeight);
    observer.observe(firstRow);
    return () => observer.disconnect();
  });


  async function loadTracks(albumKey: string) {
    if (!album.albumArtistSort || !album.albumTitleSort) return;
    loading = true;
    loadError = null;

    try {
      const page = await listAlbumTracksPage(
        album.albumArtistSort,
        album.albumTitleSort,
        200,
        undefined
      );

      if (albumKey !== activeAlbumKey) return;
      tracks = page.items;
    } catch (e) {
      console.error('Failed to load album tracks:', e);
      if (albumKey === activeAlbumKey) {
        loadError = e instanceof Error ? e.message : 'Failed to load tracks';
      }
    } finally {
      if (albumKey === activeAlbumKey) {
        loading = false;
      }
    }
  }

  function formatDuration(ms?: number): string {
    if (!ms) return '—';
    const minutes = Math.floor(ms / 60000);
    const seconds = ((ms % 60000) / 1000).toFixed(0);
    return minutes + ":" + (Number(seconds) < 10 ? '0' : '') + seconds;
  }

  function formatTotalDuration(ms: number): string {
    const minutes = Math.floor(ms / 60000);
    if (minutes > 60) {
      const hours = Math.floor(minutes / 60);
      const mins = minutes % 60;
      return `${hours} HR ${mins} MIN`;
    }
    return `${minutes} MIN`;
  }

  function handleClose() {
    onClose?.();
  }
</script>

<div
  class="inline-detail"
  bind:this={detailEl}
  style={backgroundArtUrl ? `--album-art: url('${backgroundArtUrl}')` : ''}
>
  <button class="close-btn" onclick={handleClose} aria-label="Close album details">
    <X size={16} />
  </button>

  <div class="detail-grid">
    <div class="artwork-column">
      {#if album.artworkCacheKey}
        <div class="artwork">
          <ArtworkImage 
            cacheKey={album.artworkCacheKey}
            artistSort={album.albumArtistSort}
            titleSort={album.albumTitleSort}
            size={512}
            alt="{albumTitle} artwork"
          />
        </div>
      {:else}
        <div class="artwork-placeholder">
          <span>{albumArtist?.[0] ?? '?'}</span>
        </div>
      {/if}
    </div>

    <div class="info-column">
      <div class="header-row" bind:this={headerRowEl}>
        <div class="title-block">
          <div class="album-title">{albumTitle}</div>
          <div class="meta-line">
            <span class="meta-artist">{albumArtist}</span>
            <span class="dot">•</span>
            <span>{album.trackCount} TRACKS</span>
            <span class="dot">•</span>
            <span>{formatTotalDuration(totalDuration)}</span>
          </div>
        </div>
      </div>

      <div class="track-list">
        {#if loading}
          <div class="loading">Loading tracks...</div>
        {:else if loadError}
          <div class="error">{loadError}</div>
        {:else if tracks.length === 0}
          <div class="empty">No tracks found</div>
        {:else}
          <div
            class="track-columns"
            bind:this={trackColumnsEl}
            style={`--track-columns: ${trackLayout.columns}; --track-columns-height: ${trackLayout.height}`}
          >
            {#each tracks as track (track.id)}
              <div class="track-row">
                <div class="track-number">{track.trackNo || '-'}</div>
                <div class="track-title">{track.title || '—'}</div>
                <div class="track-duration">{formatDuration(track.durationMs)}</div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .inline-detail {
    position: relative;
    border-radius: var(--radius-md);
    background: var(--surface-1);
    border: 1px solid var(--divider-color);
    overflow: hidden;
    box-shadow: var(--shadow-3);
    color: var(--text-primary);
    --artwork-size: 200px;
  }

  .inline-detail::before {
    content: '';
    position: absolute;
    inset: 0;
    background-image: var(--album-art, none);
    background-size: cover;
    background-position: center;
    filter: blur(36px) saturate(0.95);
    opacity: 0.5;
    transform: scale(1.1);
  }

  .inline-detail::after {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(120deg, rgba(10, 10, 10, 0.35), rgba(10, 10, 10, 0.6));
  }

  .close-btn {
    position: absolute;
    top: 10px;
    right: 10px;
    z-index: 2;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 1px solid var(--divider-color);
    background: rgba(0, 0, 0, 0.4);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }

  .close-btn:hover {
    color: var(--text-primary);
    background: rgba(0, 0, 0, 0.6);
  }

  .detail-grid {
    position: relative;
    z-index: 1;
    display: grid;
    grid-template-columns: var(--artwork-size) 1fr;
    gap: 18px;
    padding: 18px 20px 20px;
    align-items: start;
  }

  .artwork-column {
    display: flex;
    align-items: flex-start;
  }

  .artwork,
  .artwork-placeholder {
    width: var(--artwork-size);
    height: var(--artwork-size);
    border-radius: var(--artwork-radius-album-detail, var(--radius-md));
    background: var(--surface-2);
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary);
    font-size: 28px;
    font-weight: 600;
  }

  .info-column {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
    min-height: var(--artwork-size);
  }

  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 16px;
  }

  .album-title {
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }

  .meta-line {
    margin-top: 8px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-tertiary);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .meta-artist {
    color: var(--text-secondary);
    text-transform: none;
    letter-spacing: 0.02em;
  }

  .dot {
    opacity: 0.4;
  }

  .track-list {
    background: transparent;
    border: none;
    border-radius: 0;
    padding: 0;
    flex: 1;
    min-height: 0;
    overflow: visible;
  }

  .track-columns {
    column-count: var(--track-columns, 2);
    column-gap: 24px;
    column-fill: auto;
    height: var(--track-columns-height, auto);
    min-height: var(--artwork-size);
    width: 100%;
  }

  .track-row {
    display: grid;
    grid-template-columns: 22px 1fr auto;
    gap: 10px;
    padding: 4px 0;
    break-inside: avoid;
    color: var(--text-secondary);
    font-size: 12.5px;
  }

  .track-number {
    text-align: right;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .track-title {
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-duration {
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .loading,
  .error,
  .empty {
    color: var(--text-tertiary);
    font-size: 13px;
    padding: 10px 0;
  }

  @media (max-width: 1200px) {
    .inline-detail {
      --artwork-size: 180px;
    }
  }

  @media (max-width: 900px) {
    .detail-grid {
      grid-template-columns: 1fr;
    }

    .artwork-column {
      justify-content: center;
    }

    .info-column {
      height: auto;
    }
  }
</style>
