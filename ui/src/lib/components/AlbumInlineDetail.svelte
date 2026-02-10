<script lang="ts">
  import { X } from '@lucide/svelte';
  import { getArtworkBestForAlbum } from '../api/artwork';
  import { listAlbumTracksPage } from '../api/library';
  import { getDevArtworkUrl, retainDevArtworkUrl, releaseDevArtworkUrl } from '../utils/artworkDevUrls';
  import type { AlbumListItem, TrackRow } from '../types/library';
  import { getAlbumKey, getAlbumArtworkUrl } from '../state/albumArtwork';
  import { shouldAnimateAlbumInlineEnter } from '../state/albumInlineAnimation';
  import { playNowWithQueue, addToQueue, addToQueueNext } from '../state/playback';
  import * as ContextMenu from './primitives/ContextMenu.svelte';
  import ArtworkImage from './ArtworkImage.svelte';

  interface Props {
    album: AlbumListItem;
    closing?: boolean;
    animationNonce?: number;
    selectedColumnIndex?: number;
    totalColumns?: number;
    sameRowSwitch?: boolean;
    onClose?: () => void;
    onedit?: (trackIds: number[]) => void;
  }

  let { album, closing = false, animationNonce = 0, selectedColumnIndex = 0, totalColumns = 1, sameRowSwitch = false, onClose, onedit }: Props = $props();

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

  let shouldAnimateEnter = $state(true);
  let contentTransitioning = $state(false);
  let animateTrackEntrance = $state(false);

  $effect(() => {
    if (closing) {
      shouldAnimateEnter = false;
      return;
    }

    shouldAnimateEnter = shouldAnimateAlbumInlineEnter(animationNonce);
  });

  let wrapperEl = $state<HTMLDivElement | null>(null);
  let wrapperWidth = $state(0);

  let caretX = $derived.by(() => {
    if (totalColumns <= 0 || wrapperWidth <= 0) return 0;
    return ((2 * selectedColumnIndex + 1) / (2 * totalColumns)) * (wrapperWidth + 16) - 8;
  });

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

  interface DiscGroup {
    discNo: number;
    tracks: TrackRow[];
  }

  interface DiscSection {
    discNo: number;
    columns: TrackRow[][];
    height: string;
    showLabel: boolean;
  }

  function hasMultipleDiscs(tracks: TrackRow[]): boolean {
    const discs = new Set(tracks.map((track) => track.discNo ?? 1));
    return discs.size > 1 || (discs.size === 1 && !discs.has(1));
  }

  function groupTracksByDisc(tracks: TrackRow[]): DiscGroup[] {
    const groups = new Map<number, TrackRow[]>();

    for (const track of tracks) {
      const discNo = track.discNo ?? 1;
      if (!groups.has(discNo)) {
        groups.set(discNo, []);
      }
      groups.get(discNo)!.push(track);
    }

    const sortedDiscs = Array.from(groups.keys()).sort((a, b) => a - b);
    return sortedDiscs.map((discNo) => ({
      discNo,
      tracks: groups.get(discNo)!
    }));
  }

  let discGroups = $derived.by(() => groupTracksByDisc(tracks));
  let showDiscDividers = $derived.by(() => hasMultipleDiscs(tracks));

  function estimateTrackColumnsWidth(width: number): number {
    if (width <= 0) return 0;

    const horizontalPadding = 40;
    if (width <= 900) {
      return Math.max(MIN_COLUMN_WIDTH, width - horizontalPadding);
    }

    const artworkWidth = width <= 1200 ? 180 : 200;
    return Math.max(
      MIN_COLUMN_WIDTH,
      width - artworkWidth - COLUMN_GAP - horizontalPadding
    );
  }

  let sectionColumnCount = $derived.by(() => {
    const totalTracks = tracks.length;
    if (totalTracks === 0) return 1;

    const availableWidth = trackColumnsWidth > 0
      ? trackColumnsWidth
      : estimateTrackColumnsWidth(wrapperWidth);
    const maxColumnsByWidth = Math.max(
      1,
      Math.floor((availableWidth + COLUMN_GAP) / (MIN_COLUMN_WIDTH + COLUMN_GAP))
    );
    const maxColumns = Math.min(MAX_COLUMNS, maxColumnsByWidth);
    const columnsTarget = Math.ceil(totalTracks / TARGET_MAX_ROWS);
    return Math.max(1, Math.min(columnsTarget, maxColumns));
  });

  let discSections = $derived.by<DiscSection[]>(() => {
    if (tracks.length === 0) return [];

    const columns = sectionColumnCount;
    const effectiveRowHeight = trackRowHeight || DEFAULT_ROW_HEIGHT;
    const multiDisc = showDiscDividers;
    const groups = multiDisc ? discGroups : [{ discNo: 1, tracks }];

    return groups.map((group) => {
      const trackCount = group.tracks.length;
      const cols = Math.min(columns, trackCount);

      if (cols <= 1) {
        const height = trackCount * effectiveRowHeight;
        return {
          discNo: group.discNo,
          columns: [group.tracks],
          height: `${Math.round(height)}px`,
          showLabel: multiDisc
        };
      }

      const baseRows = Math.floor(trackCount / cols);
      const remainder = trackCount % cols;
      const columnCounts = Array.from({ length: cols }, (_, i) =>
        baseRows + (i < remainder ? 1 : 0)
      );

      const sectionColumns: TrackRow[][] = [];
      let startIndex = 0;
      for (const count of columnCounts) {
        sectionColumns.push(group.tracks.slice(startIndex, startIndex + count));
        startIndex += count;
      }

      const rowsPerColumn = baseRows + (remainder > 0 ? 1 : 0);
      const height = rowsPerColumn * effectiveRowHeight;

      return {
        discNo: group.discNo,
        columns: sectionColumns,
        height: `${Math.round(height)}px`,
        showLabel: multiDisc
      };
    });
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
    const hadPreviousAlbum = activeAlbumKey !== '';
    activeAlbumKey = key;

    if (!hadPreviousAlbum) {
      tracks = [];
    }

    loadTracks(key, hadPreviousAlbum);

    if (hadPreviousAlbum) {
      contentTransitioning = true;
      setTimeout(() => { contentTransitioning = false; }, 200);
    }
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
    if (!wrapperEl) return;
    const observer = new ResizeObserver((entries) => {
      wrapperWidth = entries[0]?.contentRect.width ?? 0;
    });
    observer.observe(wrapperEl);
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


  async function loadTracks(albumKey: string, skipLoadingState: boolean = false) {
    if (!album.albumArtistSort || !album.albumTitleSort) return;
    if (!skipLoadingState) {
      loading = true;
    }
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
      requestAnimationFrame(() => {
        animateTrackEntrance = true;
        setTimeout(() => { animateTrackEntrance = false; }, 300);
      });
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
  class="inline-detail-wrapper"
  class:animate-in={shouldAnimateEnter && !sameRowSwitch}
  class:closing
  bind:this={wrapperEl}
>
  <div class="caret-indicator" style="transform: translateX({caretX}px)"></div>
  <div
    class="inline-detail"
    class:animate-enter={shouldAnimateEnter && !sameRowSwitch}
    class:content-switch={contentTransitioning}
    class:closing
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
        {#if loading && tracks.length === 0}
          <div class="track-columns-skeleton">
            {#each Array(6) as _, i}
              <div class="track-row-skeleton" style="--skel-index: {i}">
                <div class="skel skel-number"></div>
                <div class="skel skel-title" style="width: {55 + (i * 7) % 30}%"></div>
                <div class="skel skel-duration"></div>
              </div>
            {/each}
          </div>
        {:else if loadError}
          <div class="error">{loadError}</div>
        {:else if tracks.length === 0}
          <div class="empty">No tracks found</div>
        {:else}
          <div
            class="disc-sections"
            class:animate-track-entrance={animateTrackEntrance}
            bind:this={trackColumnsEl}
          >
            {#each discSections as section, sectionIndex (section.discNo)}
              {#if section.showLabel}
                <div class="disc-divider" aria-hidden="true">
                  <span class="disc-label">Disc {section.discNo}</span>
                </div>
              {/if}
              <div
                class="track-columns"
                style={`--track-columns: ${section.columns.length}; --track-columns-height: ${section.height}; --section-index: ${sectionIndex}`}
              >
                {#each section.columns as column, columnIndex (columnIndex)}
                  <div class="track-column" style="--col-index: {columnIndex}">
                    {#each column as track, trackIndex (track.id ?? `${sectionIndex}-${columnIndex}-${trackIndex}`)}
                      <ContextMenu.Root>
                        <ContextMenu.Trigger>
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
            {/each}
          </div>
        {/if}
      </div>
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
    --inline-detail-enter-duration: 200ms;
    --inline-detail-exit-duration: 130ms;
    --inline-detail-exit-ease: cubic-bezier(0.5, 0, 1, 1);
    transform-origin: center top;
  }

  .inline-detail.animate-enter {
    animation: inline-detail-enter var(--inline-detail-enter-duration) var(--ease-out) both;
    will-change: transform, opacity;
  }

  .inline-detail.closing {
    animation: inline-detail-exit var(--inline-detail-exit-duration) var(--inline-detail-exit-ease) both;
    pointer-events: none;
    will-change: transform, opacity;
  }

  @keyframes inline-detail-enter {
    from {
      opacity: 0;
      transform: translateY(-8px) scale(0.988);
    }

    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @keyframes inline-detail-exit {
    from {
      opacity: 1;
      transform: translateY(0) scale(1);
    }

    to {
      opacity: 0;
      transform: translateY(-5px) scale(0.99);
    }
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

  .inline-detail.animate-enter::before {
    animation: inline-detail-bg-enter 220ms var(--ease-out) 40ms both;
    will-change: opacity;
  }

  .inline-detail.closing::before {
    animation: none;
  }

  @keyframes inline-detail-bg-enter {
    from {
      opacity: 0;
    }

    to {
      opacity: 0.5;
    }
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

  .inline-detail.animate-enter .close-btn {
    animation: inline-detail-close-btn-enter 130ms var(--ease-out) 100ms both;
    will-change: transform, opacity;
  }

  .inline-detail.closing .close-btn {
    animation: none;
  }

  @keyframes inline-detail-close-btn-enter {
    from {
      opacity: 0;
      transform: scale(0.82);
    }

    to {
      opacity: 1;
      transform: scale(1);
    }
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

  .inline-detail.animate-enter .artwork-column {
    animation: inline-detail-artwork-enter 170ms var(--ease-out) 30ms both;
    transform-origin: center;
    will-change: transform, opacity;
  }

  .inline-detail.closing .artwork-column {
    animation: none;
  }

  @keyframes inline-detail-artwork-enter {
    from {
      opacity: 0;
      transform: scale(0.94);
    }

    to {
      opacity: 1;
      transform: scale(1);
    }
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

  .inline-detail.animate-enter .header-row {
    animation: inline-detail-content-enter 140ms var(--ease-out) 50ms both;
    will-change: transform, opacity;
  }

  .inline-detail.closing .header-row {
    animation: none;
  }

  @keyframes inline-detail-content-enter {
    from {
      opacity: 0;
      transform: translateY(4px);
    }

    to {
      opacity: 1;
      transform: translateY(0);
    }
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

  .inline-detail.animate-enter .track-list {
    animation: inline-detail-tracks-enter 150ms var(--ease-out) 80ms both;
    will-change: transform, opacity;
  }

  .inline-detail.closing .track-list {
    animation: none;
  }

  @keyframes inline-detail-tracks-enter {
    from {
      opacity: 0;
      transform: translateY(3px);
    }

    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .disc-sections {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .track-columns {
    display: grid;
    grid-template-columns: repeat(var(--track-columns, 2), minmax(0, 1fr));
    column-gap: 24px;
    height: var(--track-columns-height, auto);
    min-height: 0;
    width: 100%;
    transition: height 180ms var(--ease-out);
  }

  .track-column {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .disc-divider {
    display: flex;
    align-items: center;
    padding: 2px 0;
    margin-top: 4px;
    user-select: none;
  }

  .disc-divider:first-child {
    margin-top: 0;
  }

  .disc-label {
    font-size: 9.5px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    white-space: nowrap;
    color: var(--text-tertiary);
    opacity: 0.55;
    padding-left: 2px;
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

  @media (prefers-reduced-motion: reduce) {
    .inline-detail,
    .inline-detail::before,
    .inline-detail .close-btn,
    .inline-detail .artwork-column,
    .inline-detail .header-row,
    .inline-detail .track-list,
    .caret-indicator,
    .track-row-skeleton,
    .skel,
    .inline-detail.content-switch .artwork-column,
    .inline-detail.content-switch .header-row,
    .inline-detail.content-switch .track-list {
      animation-duration: 1ms !important;
      animation-delay: 0ms !important;
    }

    .disc-sections.animate-track-entrance .track-column {
      animation: none !important;
    }

    .caret-indicator {
      transition: none !important;
    }
  }

  /* ===== WRAPPER ===== */
  .inline-detail-wrapper {
    position: relative;
    padding-top: 10px;
  }

  /* ===== CARET INDICATOR ===== */
  .caret-indicator {
    position: absolute;
    top: 0;
    left: -9px;
    z-index: 3;
    width: 18px;
    height: 10px;
    will-change: transform, opacity;
    pointer-events: none;
    opacity: 0;
  }

  .caret-indicator::after {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    width: 0;
    height: 0;
    border-left: 9px solid transparent;
    border-right: 9px solid transparent;
    border-bottom: 10px solid rgba(255, 255, 255, 0.18);
  }

  .inline-detail-wrapper.animate-in .caret-indicator {
    animation: caret-enter 160ms var(--ease-out) 60ms both;
  }

  .inline-detail-wrapper:not(.animate-in):not(.closing) .caret-indicator {
    opacity: 1;
    transition: transform 150ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .inline-detail-wrapper.closing .caret-indicator {
    animation: caret-exit 130ms cubic-bezier(0.5, 0, 1, 1) both;
  }

  @keyframes caret-enter {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes caret-exit {
    from { opacity: 1; }
    to { opacity: 0; }
  }

  /* ===== CONTENT SWITCH (same-row album change) ===== */
  .inline-detail.content-switch .artwork-column {
    animation: inline-content-artwork 170ms var(--ease-out) both;
    will-change: transform, opacity;
  }

  .inline-detail.content-switch .header-row {
    animation: inline-content-header 150ms var(--ease-out) 25ms both;
    will-change: transform, opacity;
  }

  .inline-detail.content-switch .track-list {
    animation: inline-content-tracks 150ms var(--ease-out) 40ms both;
    will-change: transform, opacity;
  }

  @keyframes inline-content-artwork {
    from { opacity: 0.3; transform: scale(0.97); }
    to { opacity: 1; transform: scale(1); }
  }

  @keyframes inline-content-header {
    from { opacity: 0.2; transform: translateY(3px); }
    to { opacity: 1; transform: translateY(0); }
  }

  @keyframes inline-content-tracks {
    from { opacity: 0.2; transform: translateY(2px); }
    to { opacity: 1; transform: translateY(0); }
  }

  /* ===== SKELETON LOADING ===== */
  .track-columns-skeleton {
    display: flex;
    flex-direction: column;
  }

  .track-row-skeleton {
    display: grid;
    grid-template-columns: 22px 1fr auto;
    gap: 10px;
    padding: 4px 0;
    animation: skeleton-fade-in 150ms var(--ease-out) both;
    animation-delay: calc(var(--skel-index, 0) * 20ms);
  }

  .skel {
    border-radius: 4px;
    height: 12px;
    background: linear-gradient(
      90deg,
      rgba(255, 255, 255, 0.04) 0%,
      rgba(255, 255, 255, 0.08) 50%,
      rgba(255, 255, 255, 0.04) 100%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s ease-in-out infinite;
  }

  .skel-number {
    width: 16px;
    justify-self: end;
  }

  .skel-duration {
    width: 32px;
  }

  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  @keyframes skeleton-fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  /* ===== TRACK LIST COLUMN CASCADE ENTRANCE ===== */
  .disc-sections.animate-track-entrance .track-column {
    animation: column-cascade-enter 120ms cubic-bezier(0.25, 1, 0.5, 1) both;
    animation-delay: calc(var(--col-index, 0) * 35ms);
  }

  @keyframes column-cascade-enter {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
