<script lang="ts">
  import { X } from '@lucide/svelte';
  import { getArtworkBestForAlbum } from '../api/artwork';
  import { listAlbumTracksPage } from '../api/library';
  import { getDevArtworkUrl, retainDevArtworkUrl, releaseDevArtworkUrl } from '../utils/artworkDevUrls';
  import type { AlbumListItem, TrackRow } from '../types/library';
  import { getAlbumKey, getAlbumArtworkUrl } from '../state/albumArtwork';
  import { playNowWithQueue, addToQueue, addToQueueNext } from '../state/playback';
  import * as ContextMenu from './primitives/ContextMenu.svelte';
  import ArtworkImage from './ArtworkImage.svelte';

  interface Props {
    album: AlbumListItem;
    onClose?: () => void;
    onedit?: (trackIds: number[]) => void;
  }

  let { album, onClose, onedit }: Props = $props();

  let tracks = $state<TrackRow[]>([]);
  let loading = $state(false);
  let loadError = $state<string | null>(null);
  let activeAlbumKey = $state('');
  let selectedTrackId = $state<number | null>(null);

  let backgroundArtUrl = $state('');
  let previewArtUrl = $state<string | null>(null);

  const PREVIEW_ART_SIZE = 256;
  const DETAIL_ART_SIZE = 256;
  const LQIP_ART_SIZE = 32;

  async function resolveArtworkCacheKey(targetAlbum: AlbumListItem): Promise<string | null> {
    if (targetAlbum.artworkCacheKey) return targetAlbum.artworkCacheKey;
    if (!targetAlbum.albumArtistSort || !targetAlbum.albumTitleSort) return null;

    try {
      const best = await getArtworkBestForAlbum(
        targetAlbum.albumArtistSort,
        targetAlbum.albumTitleSort
      );
      return best.cacheKey ?? null;
    } catch (err) {
      console.warn('Artwork lookup failed:', err);
      return null;
    }
  }

  $effect(() => {
    const fallbackUrl = getAlbumArtworkUrl(album, PREVIEW_ART_SIZE);
    previewArtUrl = fallbackUrl;
    backgroundArtUrl = fallbackUrl || '';

    let active = true;
    let devPreviewUrl: string | null = null;
    let devBackgroundUrl: string | null = null;

    const useDevFallback = import.meta.env.DEV;

    const applyResolvedArtwork = async () => {
      const cacheKey = await resolveArtworkCacheKey(album);
      if (!active || !cacheKey) return;

      const encodedKey = encodeURIComponent(cacheKey);

      if (useDevFallback) {
        try {
          const [previewUrl, backgroundUrl] = await Promise.all([
            getDevArtworkUrl(cacheKey, PREVIEW_ART_SIZE),
            getDevArtworkUrl(cacheKey, LQIP_ART_SIZE)
          ]);

          if (!active) {
            releaseDevArtworkUrl(previewUrl);
            releaseDevArtworkUrl(backgroundUrl);
            return;
          }

          devPreviewUrl = previewUrl;
          devBackgroundUrl = backgroundUrl;
          retainDevArtworkUrl(previewUrl);
          retainDevArtworkUrl(backgroundUrl);

          previewArtUrl = previewUrl;
          backgroundArtUrl = backgroundUrl;
        } catch (err) {
          console.warn('Failed to load background artwork:', err);
          if (active) {
            previewArtUrl = fallbackUrl;
            backgroundArtUrl = fallbackUrl || '';
          }
        }
        return;
      }

      previewArtUrl = `sermon-artwork://localhost/thumb/${encodedKey}?s=${PREVIEW_ART_SIZE}`;
      backgroundArtUrl = `sermon-artwork://localhost/lqip/${encodedKey}`;
    };

    void applyResolvedArtwork();

    return () => {
      active = false;
      if (devPreviewUrl) releaseDevArtworkUrl(devPreviewUrl);
      if (devBackgroundUrl) releaseDevArtworkUrl(devBackgroundUrl);
    };
  });

  let trackColumnsEl = $state<HTMLDivElement | null>(null);
  let trackColumnsWidth = $state(0);
  let trackRowHeight = $state(0);

  const COLUMN_GAP = 24;
  const MIN_COLUMN_WIDTH = 240;
  const MAX_COLUMNS = 4;
  const DEFAULT_ROW_HEIGHT = 22;
  const TARGET_MAX_ROWS = 6;

  type TrackLayout = {
    columns: number;
    height: string;
    columnCounts: number[];
  };

  let trackLayout = $derived.by<TrackLayout>(() => {
    const trackCount = tracks.length;
    if (trackCount === 0) {
      return { columns: 1, height: 'auto', columnCounts: [] };
    }

    const availableWidth = trackColumnsWidth;
    const effectiveRowHeight = trackRowHeight || DEFAULT_ROW_HEIGHT;
    const maxColumnsByWidth = Math.max(
      1,
      Math.floor((availableWidth + COLUMN_GAP) / (MIN_COLUMN_WIDTH + COLUMN_GAP))
    );
    const maxColumns = Math.min(MAX_COLUMNS, maxColumnsByWidth);
    const columnsTarget = Math.ceil(trackCount / TARGET_MAX_ROWS);
    const columns = Math.max(1, Math.min(columnsTarget, maxColumns));
    const baseRows = Math.floor(trackCount / columns);
    const remainder = trackCount % columns;
    const columnCounts = Array.from({ length: columns }, (_, index) =>
      baseRows + (index < remainder ? 1 : 0)
    );
    const rowsPerColumn = baseRows + (remainder > 0 ? 1 : 0);
    const height = rowsPerColumn * effectiveRowHeight;

    return { columns, height: `${Math.round(height)}px`, columnCounts };
  });

  let trackColumns = $derived.by<TrackRow[][]>(() => {
    if (tracks.length === 0) {
      return [];
    }

    const { columns, columnCounts } = trackLayout;
    if (columns <= 1) {
      return [tracks];
    }

    const groups: TrackRow[][] = [];
    let startIndex = 0;
    for (const count of columnCounts) {
      groups.push(tracks.slice(startIndex, startIndex + count));
      startIndex += count;
    }
    return groups;
  });

  let albumTitle = $derived(album.albumTitleDisplay || 'Unknown Album');
  let albumArtist = $derived(album.albumArtistDisplay || 'Unknown Artist');
  let totalDuration = $derived(tracks.reduce((acc, t) => acc + (t.durationMs || 0), 0));

  $effect(() => {
    if (!selectedTrackId) return;
    const stillExists = tracks.some(track => track.id === selectedTrackId);
    if (!stillExists) {
      selectedTrackId = null;
    }
  });

  $effect(() => {
    const key = getAlbumKey(album);
    if (key === activeAlbumKey) return;
    activeAlbumKey = key;
    tracks = [];
    loadTracks(key);
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

  function handleTrackActivate(track: TrackRow) {
    if (track.isMissing || !track.id) return;
    const validTracks = tracks.filter(t => t.id && !t.isMissing);
    const trackIds = validTracks.map(t => t.id);
    const startIndex = validTracks.findIndex(t => t.id === track.id);
    if (startIndex >= 0) {
      playNowWithQueue(trackIds, startIndex);
    }
  }

  function handleTrackSelect(track: TrackRow) {
    if (track.isMissing || !track.id) return;
    selectedTrackId = track.id;
  }

  function handleTrackKeydown(event: KeyboardEvent, track: TrackRow) {
    if (event.key === 'Enter') {
      handleTrackActivate(track);
    }
  }

  function handleQueueNext(track: TrackRow) {
    if (track.isMissing || !track.id) return;
    addToQueueNext([track.id]);
  }

  function handleQueueLast(track: TrackRow) {
    if (track.isMissing || !track.id) return;
    addToQueue(track.id);
  }

  function handleEditTrack(track: TrackRow) {
    if (track.isMissing || !track.id) return;
    onedit?.([track.id]);
  }
</script>

<div
  class="inline-detail"
  style={backgroundArtUrl ? `--album-art: url('${backgroundArtUrl}')` : ''}
>
  <button class="close-btn" onclick={handleClose} aria-label="Close album details">
    <X size={16} />
  </button>

  <div class="detail-grid">
    <div class="artwork-column">
      {#if album.artworkCacheKey || (album.albumArtistSort && album.albumTitleSort)}
        <div class="artwork">
          <ArtworkImage 
            cacheKey={album.artworkCacheKey}
            artistSort={album.albumArtistSort}
            titleSort={album.albumTitleSort}
            size={DETAIL_ART_SIZE}
            previewUrl={previewArtUrl || undefined}
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
      <div class="header-row">
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
            {#each trackColumns as column, columnIndex (columnIndex)}
              <div class="track-column">
                {#each column as track (track.id)}
                  <ContextMenu.Root>
                    <ContextMenu.Trigger asChild>
                      {#snippet child({ props })}
                        <div
                          {...props}
                          class="track-row"
                          class:missing={track.isMissing}
                          class:selected={track.id === selectedTrackId}
                          role="button"
                          tabindex={track.isMissing ? -1 : 0}
                          aria-disabled={track.isMissing}
                          onclick={() => handleTrackSelect(track)}
                          ondblclick={() => handleTrackActivate(track)}
                          onfocus={() => handleTrackSelect(track)}
                          onkeydown={(event) => handleTrackKeydown(event, track)}
                        >
                          <div class="track-number">{track.trackNo || '-'}</div>
                          <div class="track-title">{track.title || '—'}</div>
                          <div class="track-duration">{formatDuration(track.durationMs)}</div>
                        </div>
                      {/snippet}
                    </ContextMenu.Trigger>
                    <ContextMenu.Portal>
                      <ContextMenu.Content class="dropdown-content" data-testid="track-context-menu">
                        <ContextMenu.Item class="dropdown-item" onclick={() => handleTrackActivate(track)}>Play Now</ContextMenu.Item>
                        <ContextMenu.Item class="dropdown-item" onclick={() => handleQueueNext(track)}>Queue Next</ContextMenu.Item>
                        <ContextMenu.Item class="dropdown-item" onclick={() => handleQueueLast(track)}>Queue Last</ContextMenu.Item>
                        <ContextMenu.Separator class="dropdown-separator" />
                        <ContextMenu.Item class="dropdown-item" onclick={() => handleEditTrack(track)}>Edit</ContextMenu.Item>
                      </ContextMenu.Content>
                    </ContextMenu.Portal>
                  </ContextMenu.Root>
                {/each}
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
    min-height: 0;
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
    display: grid;
    grid-template-columns: repeat(var(--track-columns, 2), minmax(0, 1fr));
    column-gap: 24px;
    height: var(--track-columns-height, auto);
    min-height: 0;
    width: 100%;
  }

  .track-column {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .track-row {
    display: grid;
    grid-template-columns: 22px 1fr auto;
    gap: 10px;
    padding: 4px 0;
    color: var(--text-secondary);
    font-size: 12.5px;
    cursor: pointer;
    border-radius: 6px;
    transition: background var(--motion-fast) var(--ease-out), color var(--motion-fast) var(--ease-out);
  }

  .track-row:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  .track-row:focus-visible {
    background: var(--surface-2);
    color: var(--text-primary);
    outline: none;
    box-shadow: inset 0 0 0 1px var(--divider-color);
  }

  .track-row.selected {
    background: var(--surface-2);
    color: var(--text-primary);
    box-shadow: inset 0 0 0 1px var(--divider-color);
  }

  .track-row.missing {
    cursor: default;
    color: var(--text-disabled);
    background: transparent;
    box-shadow: none;
  }

  .track-row.missing:hover,
  .track-row.missing:focus-visible,
  .track-row.missing.selected {
    background: transparent;
    color: var(--text-disabled);
    box-shadow: none;
  }

  .track-row.missing .track-title {
    color: var(--text-disabled);
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
