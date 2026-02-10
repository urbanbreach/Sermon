<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { getLibraryStats, listAlbumsPage, listAlbumTracksPage } from '../api/library';
  import type { AlbumListItem, LibraryStats } from '../types/library';
  import { setViewTitle } from '../state/viewTitle';
  import { setAlphabetSelector, clearAlphabetSelector } from '../state/alphabetSelector';
  import {
    albumArtworkCacheVersion,
    getAlbumKey,
    bumpAlbumArtworkVersion,
    resetAlbumArtworkCache
  } from '../state/albumArtwork';
  import {
    clearAlbumsViewCache,
    isAlbumsViewCacheFresh,
    readAlbumsViewCache,
    writeAlbumsViewCache
  } from '../state/albumsViewCache';
  import ArtworkPickerModal from '../components/ArtworkPickerModal.svelte';
  import { openTagEditorWindow } from '../state/tagEditorWindow';
  import * as ContextMenu from '../components/primitives/ContextMenu.svelte';
import { playNowWithQueue, addToQueue, addToQueueNext } from '../state/playback';
  import { selectAlbumSummary } from '../state/albumSelection';
  import AlbumInlineDetail from '../components/AlbumInlineDetail.svelte';
  import ArtworkImage from '../components/ArtworkImage.svelte';
  import AlbumsArtworkSurface from '../components/AlbumsArtworkSurface.svelte';
  import {
    ALBUM_INLINE_CLOSE_DURATION_MS,
    expandedAlbum,
    toggleAlbumInline,
    openAlbumInlineFromItem,
    clearAlbumInline
  } from '../state/albumInline';
  import SkeletonCard from '../components/SkeletonCard.svelte';
  import { Disc3 } from '@lucide/svelte';
  import { fadeIn } from '../utils/animations';
  import { setAlbumGridScrolling, flushArtworkDecodeQueue } from '../utils/artworkDecodeQueue';
  import { notifyScrollPressure } from '../utils/artworkDevUrls';
  import { registerArtworkSlot } from '../utils/albumsArtworkSlots';
  import { artworkRoundedAlbums } from '../state/effects';
  import { VList } from 'virtua/svelte';

  let albums: AlbumListItem[] = $state([]);
  let loading = $state(false);
  let initialLoadComplete = $state(false);
  let artworkCacheSeed = $derived($albumArtworkCacheVersion);

  // Artwork picker modal state
  let pickerOpen = $state(false);
  let pickerAlbum: AlbumListItem | null = $state(null);

  let useArtworkSurface = $state(false);

  // Virtualization state
  let containerWidth = $state(0);
  type RowItem =
    | { type: 'row'; albums: AlbumListItem[]; rowIndex: number }
    | { type: 'detail'; album: AlbumListItem; rowIndex: number };

  let vlistRef: VList<RowItem> | undefined = $state();
  let listWrapperEl = $state<HTMLDivElement | null>(null);
  let isScrolling = $state(false);
  let deferHighRes = $state(false);
  let ultraFastScroll = $state(false);
  let artworkRadiusPx = $derived($artworkRoundedAlbums ? 10 : 0);
  let smoothedScrollVelocity = 0;
  let highVelocityStartedAt = 0;
  let lastScrollTop = 0;
  let lastScrollTs = 0;
  let scrollSettlerRaf = 0;
  let lastScrollAt = 0;
  const SCROLL_IDLE_MS = 140;
  const HIGH_VELOCITY_ENTER_PX_PER_S = 14000;
  const ULTRA_FAST_SCROLL_ENTER_PX_PER_S = 26000;
  const ULTRA_FAST_SCROLL_EXIT_PX_PER_S = 18000;
  const HIGH_VELOCITY_CONFIRM_MS = 120;
  let activeBufferSize = $derived(ultraFastScroll ? 220 : deferHighRes ? 300 : 700);
  
  // Compute columns based on container width (min 160px + 16px gap)
  // containerWidth - 32 accounts for 1rem (16px) padding on each side
  let columns = $derived(Math.max(1, Math.floor((containerWidth - 32 + 16) / 176)));
  
  // Chunk albums into rows
  let rows = $derived.by(() => {
    const c = columns;
    const res = [];
    for (let i = 0; i < albums.length; i += c) {
      res.push(albums.slice(i, i + c));
    }
    return res;
  });

  let expandedRowIndex = $derived.by(() => {
    const target = $expandedAlbum;
    if (!target) return -1;
    const index = albums.findIndex(
      (album) =>
        album.albumArtistSort === target.albumArtistSort &&
        album.albumTitleSort === target.albumTitleSort
    );
    if (index === -1) return -1;
    return Math.floor(index / columns);
  });

  let expandedAlbumIndex = $derived.by(() => {
    const target = $expandedAlbum;
    if (!target) return -1;
    return albums.findIndex(
      (album) =>
        album.albumArtistSort === target.albumArtistSort &&
        album.albumTitleSort === target.albumTitleSort
    );
  });

  let expandedColumnIndex = $derived(expandedAlbumIndex >= 0 ? expandedAlbumIndex % columns : -1);

  let expandedAlbumItem = $derived.by(() => {
    const target = $expandedAlbum;
    if (!target) return null;
    return (
      albums.find(
        (album) =>
          album.albumArtistSort === target.albumArtistSort &&
          album.albumTitleSort === target.albumTitleSort
      ) || null
    );
  });

  // Same-row switch detection
  let sameRowSwitch = $state(false);
  let detailRowOpening = $state(false);
  let _prevExpandedRowIndex = -1;
  let _prevExpandedKey = '';
  let _lastDetailOpenNonce = -1;

  $effect(() => {
    const target = $expandedAlbum;
    const currentRow = expandedRowIndex;
    const currentKey = target && !target.isClosing
      ? `${target.albumArtistSort}||${target.albumTitleSort}`
      : '';

    const isSameRow = (
      currentRow >= 0 &&
      currentRow === _prevExpandedRowIndex &&
      currentKey !== '' &&
      _prevExpandedKey !== '' &&
      currentKey !== _prevExpandedKey
    );

    sameRowSwitch = isSameRow;
    _prevExpandedRowIndex = currentRow;
    _prevExpandedKey = currentKey;
  });

  $effect(() => {
    const target = $expandedAlbum;
    if (!target || target.isClosing) {
      detailRowOpening = false;
      return;
    }

    const nonce = target.animationNonce ?? 0;
    if (nonce === _lastDetailOpenNonce) {
      return;
    }

    _lastDetailOpenNonce = nonce;
    detailRowOpening = true;
  });

  function handleDetailRowAnimationEnd(event: AnimationEvent) {
    if (!event.animationName.includes('detail-row-open')) {
      return;
    }
    detailRowOpening = false;
  }


  let displayRows = $derived.by(() => {
    const res: RowItem[] = [];
    rows.forEach((rowAlbums, rowIndex) => {
      res.push({ type: 'row', albums: rowAlbums, rowIndex });
      if (rowIndex === expandedRowIndex && expandedAlbumItem) {
        res.push({ type: 'detail', album: expandedAlbumItem, rowIndex });
      }
    });
    return res;
  });

  let alphabetItems = $derived(albums.map(a => ({ sortKey: a.albumArtistSort })));

  function getDisplayRowIndex(rowIndex: number): number {
    if (expandedRowIndex >= 0 && expandedRowIndex < rowIndex) {
      return rowIndex + 1;
    }
    return rowIndex;
  }

  function handleAlphabetSelect(index: number) {
    const rowIndex = Math.floor(index / columns);
    if (vlistRef) {
      vlistRef.scrollToIndex(getDisplayRowIndex(rowIndex), { align: 'start', smooth: true });
    }
  }

  $effect(() => {
    if (!listWrapperEl) return;
    const scroller = listWrapperEl.firstElementChild;
    if (!(scroller instanceof HTMLElement)) return;

    const handleScroll = () => {
      const now = performance.now();
      const nextScrollTop = scroller.scrollTop;
      const deltaTs = now - lastScrollTs;
      if (deltaTs >= 12) {
        const instantaneousVelocity = Math.abs((nextScrollTop - lastScrollTop) * 1000 / deltaTs);
        if (smoothedScrollVelocity === 0) {
          smoothedScrollVelocity = instantaneousVelocity;
        } else {
          smoothedScrollVelocity = (smoothedScrollVelocity * 0.72) + (instantaneousVelocity * 0.28);
        }
      }
      lastScrollTop = nextScrollTop;
      lastScrollTs = now;
      lastScrollAt = now;

      if (!deferHighRes) {
        if (smoothedScrollVelocity >= HIGH_VELOCITY_ENTER_PX_PER_S) {
          if (highVelocityStartedAt === 0) {
            highVelocityStartedAt = now;
          } else if (now - highVelocityStartedAt >= HIGH_VELOCITY_CONFIRM_MS) {
            deferHighRes = true;
          }
        } else {
          highVelocityStartedAt = 0;
        }
      }

      if (smoothedScrollVelocity >= ULTRA_FAST_SCROLL_ENTER_PX_PER_S) {
        ultraFastScroll = true;
      } else if (ultraFastScroll && smoothedScrollVelocity <= ULTRA_FAST_SCROLL_EXIT_PX_PER_S) {
        ultraFastScroll = false;
      }

      if (isScrolling) return;
      isScrolling = true;
      setAlbumGridScrolling(true);
      notifyScrollPressure(true);

      const settle = () => {
        if (performance.now() - lastScrollAt >= SCROLL_IDLE_MS) {
          isScrolling = false;
          smoothedScrollVelocity = 0;
          highVelocityStartedAt = 0;
          deferHighRes = false;
          ultraFastScroll = false;
          lastScrollTs = 0;
          setAlbumGridScrolling(false);
          notifyScrollPressure(false);
          flushArtworkDecodeQueue();
          scrollSettlerRaf = 0;
          return;
        }
        scrollSettlerRaf = requestAnimationFrame(settle);
      };

      scrollSettlerRaf = requestAnimationFrame(settle);
    };

    scroller.addEventListener('scroll', handleScroll, { passive: true });

    return () => {
      scroller.removeEventListener('scroll', handleScroll);
      if (scrollSettlerRaf) {
        cancelAnimationFrame(scrollSettlerRaf);
        scrollSettlerRaf = 0;
      }
      isScrolling = false;
      smoothedScrollVelocity = 0;
      highVelocityStartedAt = 0;
      deferHighRes = false;
      ultraFastScroll = false;
      lastScrollTs = 0;
      setAlbumGridScrolling(false);
      notifyScrollPressure(false);
      flushArtworkDecodeQueue();
    };
  });

  // Prefetching strategy:
  // We rely on virtua's bufferSize (pixels) to render rows just outside the viewport.
  // The <ArtworkImage> component handles loading:
  // 1. Shows LQIP immediately (embedded in album metadata)
  // 2. Fetches high-res thumbnail asynchronously on mount
  // This avoids the need for a full-library warmup which wastes bandwidth.

  async function loadAllAlbums() {
    if (loading) return;

    const hadAlbums = albums.length > 0;
    loading = true;
    if (!hadAlbums) {
      initialLoadComplete = false;
    }

    try {
      const stats = await getLibraryStats();
      const page = await listAlbumsPage(stats.albumCount + 10, undefined);
      albums = page.items;
      writeAlbumsViewCache(albums, stats);
    } catch (e) {
      console.error('Failed to load albums:', e);
    } finally {
      loading = false;
      initialLoadComplete = true;
    }
  }

  async function validateAlbumsCache(): Promise<void> {
    const cache = readAlbumsViewCache();
    if (!cache.hydrated) {
      return;
    }

    try {
      const stats: Pick<LibraryStats, 'albumCount' | 'lastScanCompletedMs'> = await getLibraryStats();
      if (isAlbumsViewCacheFresh(stats)) {
        return;
      }
      await loadAllAlbums();
    } catch (e) {
      console.warn('Failed to validate albums cache:', e);
    }
  }

  onMount(() => {
    setViewTitle('Albums');

    const cache = readAlbumsViewCache();
    if (cache.hydrated) {
      albums = cache.items;
      initialLoadComplete = true;
      void validateAlbumsCache();
    } else {
      void loadAllAlbums();
    }
    
    // Listen for library changes (after scan) to refresh artwork cache
    const handleLibraryChange = () => {
      resetAlbumArtworkCache();
      clearAlbumsViewCache();
      void loadAllAlbums();
    };
    window.addEventListener('sermon:library-changed', handleLibraryChange);
    
    return () => {
      window.removeEventListener('sermon:library-changed', handleLibraryChange);
    };
  });

  onDestroy(() => {
    setViewTitle('');
    clearAlphabetSelector();
  });

  // Update alphabet selector in TopBar whenever albums change
  $effect(() => {
    if (albums.length > 0) {
      setAlphabetSelector(alphabetItems, handleAlphabetSelect);
    }
  });


  function handleAlbumClick(album: AlbumListItem, event?: Event) {
    const clickCount = event instanceof MouseEvent ? event.detail : 1;
    if (clickCount > 1) return;
    selectAlbumSummary(album);
    toggleAlbumInline({
      albumArtistSort: album.albumArtistSort,
      albumTitleSort: album.albumTitleSort
    });
  }

  function handleAlbumDoubleClick(album: AlbumListItem) {
    selectAlbumSummary(album);
    openAlbumInlineFromItem(album);
  }

  function openArtworkPicker(album: AlbumListItem, e: Event) {
    e.stopPropagation();
    pickerAlbum = album;
    pickerOpen = true;
  }

  function closeArtworkPicker() {
    pickerOpen = false;
    pickerAlbum = null;
  }

  async function openTagEditorForAlbum(album: AlbumListItem) {
    try {
      // Fetch all tracks for the album to edit
      const page = await listAlbumTracksPage(album.albumArtistSort, album.albumTitleSort, 10000);
      await openTagEditorWindow(page.items.map((track) => track.id));
    } catch (e) {
      console.error('Failed to load album tracks for editing', e);
    }
  }

  function handleArtworkSurfaceUnsupported() {
    useArtworkSurface = false;
  }

  async function handlePlayAlbum(album: AlbumListItem) {
    const page = await listAlbumTracksPage(album.albumArtistSort, album.albumTitleSort, 1000);
    if (page.items.length > 0) {
      await playNowWithQueue(page.items.map(t => t.id), 0);
    }
  }

  async function handleQueueAlbumNext(album: AlbumListItem) {
    const page = await listAlbumTracksPage(album.albumArtistSort, album.albumTitleSort, 1000);
    if (page.items.length > 0) {
      await addToQueueNext(page.items.map(t => t.id));
    }
  }

  async function handleQueueAlbumLast(album: AlbumListItem) {
    const page = await listAlbumTracksPage(album.albumArtistSort, album.albumTitleSort, 1000);
    for (const track of page.items) {
      await addToQueue(track.id);
    }
  }

  async function handleArtworkSelected(_cacheKey: string) {
    if (!pickerAlbum) return;
    bumpAlbumArtworkVersion(pickerAlbum);
  }

</script>

<div 
  class="view-container" 
  class:scrolling={isScrolling}
  bind:clientWidth={containerWidth}
  use:fadeIn={{ duration: 300 }}
>
  {#if !initialLoadComplete && albums.length === 0}
    <div class="albums-grid">
      {#each Array(12) as _, i (i)}
        <SkeletonCard />
      {/each}
    </div>
  {:else if albums.length === 0}
    <div class="empty-state" use:fadeIn={{ duration: 300 }}>
      <Disc3 size={48} strokeWidth={1} />
      <p class="empty-title">No albums found</p>
      <p class="empty-hint">Add a library folder in Preferences to see your music</p>
    </div>
  {:else}
    <div class="list-wrapper" bind:this={listWrapperEl} use:fadeIn={{ duration: 220, y: 6 }}>
      <VList
        bind:this={vlistRef}
        data={displayRows}
        bufferSize={activeBufferSize}
        getKey={(item) =>
          item.type === 'row'
            ? `row-${item.rowIndex}`
            : `detail-${item.rowIndex}`
        }
      >
        {#snippet children(item)}
          {#if item.type === 'row'}
            <div class="grid-row" style="grid-template-columns: repeat({columns}, 1fr)">
              {#each item.albums as album (getAlbumKey(album))}
                <ContextMenu.Root>
                  <ContextMenu.Trigger>
                    {#snippet child({ props })}
                      <div
                        {...props}
                        class="card"
                        class:expanded={$expandedAlbum != null && !$expandedAlbum.isClosing && album.albumArtistSort === $expandedAlbum.albumArtistSort && album.albumTitleSort === $expandedAlbum.albumTitleSort}
                        role="button"
                        tabindex="0"
                        onkeydown={(e) => e.key === 'Enter' && handleAlbumClick(album)}
                        onclick={(e) => handleAlbumClick(album, e)}
                        ondblclick={() => handleAlbumDoubleClick(album)}
                      >
                        <div class="artwork">
                          {#if useArtworkSurface}
                            <div
                              class="artwork-slot"
                              use:registerArtworkSlot={{
                                id: `${getAlbumKey(album)}||${artworkCacheSeed}`,
                                cacheKey: album.artworkCacheKey ?? '',
                                artistSort: album.albumArtistSort,
                                titleSort: album.albumTitleSort
                              }}
                              data-artwork-id={`${getAlbumKey(album)}||${artworkCacheSeed}`}
                              data-cache-key={album.artworkCacheKey ?? ''}
                              data-artist-sort={album.albumArtistSort}
                              data-title-sort={album.albumTitleSort}
                              aria-label="{album.albumTitleDisplay} artwork"
                            ></div>
                          {:else}
                            <ArtworkImage
                              cacheKey={album.artworkCacheKey}
                              artistSort={album.albumArtistSort}
                              titleSort={album.albumTitleSort}
                              size={256}
                              deferHighRes={deferHighRes}
                              alt="{album.albumTitleDisplay} artwork"
                            />
                          {/if}
                        </div>
                        <div class="info">
                          <div class="title" title={album.albumTitleDisplay}>{album.albumTitleDisplay}</div>
                          <div class="artist" title={album.albumArtistDisplay}>{album.albumArtistDisplay}</div>
                          {#if album.year}<div class="year">{album.year}</div>{/if}
                        </div>
                      </div>
                    {/snippet}
                  </ContextMenu.Trigger>

                  <ContextMenu.Portal>
                    <ContextMenu.Content class="dropdown-content" data-testid="album-context-menu">
                      <ContextMenu.Item class="dropdown-item" onclick={() => handlePlayAlbum(album)}>Play Album</ContextMenu.Item>
                      <ContextMenu.Item class="dropdown-item" onclick={() => handleQueueAlbumNext(album)}>Queue Album Next</ContextMenu.Item>
                      <ContextMenu.Item class="dropdown-item" onclick={() => handleQueueAlbumLast(album)}>Queue Album Last</ContextMenu.Item>
                      <ContextMenu.Separator class="dropdown-separator" />
                      <ContextMenu.Item class="dropdown-item" onclick={() => openTagEditorForAlbum(album)}>Edit</ContextMenu.Item>
                    </ContextMenu.Content>
                  </ContextMenu.Portal>
                </ContextMenu.Root>
              {/each}
            </div>
          {:else}
            <div
              class="detail-row"
              class:opening={detailRowOpening && !sameRowSwitch && !($expandedAlbum?.isClosing ?? false)}
              class:closing={$expandedAlbum?.isClosing ?? false}
              style={`--inline-detail-close-duration: ${ALBUM_INLINE_CLOSE_DURATION_MS}ms`}
              onanimationend={handleDetailRowAnimationEnd}
            >
              <AlbumInlineDetail
                album={item.album}
                closing={$expandedAlbum?.isClosing ?? false}
                animationNonce={$expandedAlbum?.animationNonce ?? 0}
                selectedColumnIndex={expandedColumnIndex}
                totalColumns={columns}
                sameRowSwitch={sameRowSwitch}
                onClose={() => clearAlbumInline()}
                onedit={(trackIds) => {
                  void openTagEditorWindow(trackIds);
                }}
              />
            </div>
          {/if}
        {/snippet}
      </VList>

      {#if useArtworkSurface}
        <AlbumsArtworkSurface
          wrapperEl={listWrapperEl}
          enabled={useArtworkSurface}
          deferHighRes={deferHighRes}
          ultraFastMode={ultraFastScroll}
          artworkRadiusPx={artworkRadiusPx}
          on:unsupported={handleArtworkSurfaceUnsupported}
        />
      {/if}
    </div>
  {/if}
</div>

<ArtworkPickerModal
  open={pickerOpen}
  albumArtistSort={pickerAlbum?.albumArtistSort ?? ''}
  albumTitleSort={pickerAlbum?.albumTitleSort ?? ''}
  albumArtistDisplay={pickerAlbum?.albumArtistDisplay ?? ''}
  albumTitleDisplay={pickerAlbum?.albumTitleDisplay ?? ''}
  onclose={closeArtworkPicker}
  onselect={handleArtworkSelected}
/>

<style>
  .view-container {
    padding: 1rem;
    padding-top: 0;
    padding-right: 0;
    padding-bottom: 0;
    color: var(--text-primary);
    height: 100%;
    overflow-y: hidden;
    box-sizing: border-box;
    background: transparent;
    min-height: 100%;
    display: flex;
    flex-direction: column;
  }

  .albums-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 16px;
    padding-right: 1rem;
  }
  
  .list-wrapper {
    flex: 1;
    position: relative;
    padding-top: 0; /* Padding moved inside virtua scroll container */
  }
  
  .grid-row {
    display: grid;
    gap: 16px;
    margin-bottom: 16px;
    padding-right: 1rem;
    overflow: visible;
  }

  .grid-row > .card {
    min-width: 0; /* Allow grid items to shrink below content size */
    max-width: 100%;
  }

  .detail-row {
    margin-top: -6px;
    padding-right: 1rem;
    margin-bottom: 24px;
    display: grid;
    grid-template-rows: minmax(0, 1fr);
    transition:
      grid-template-rows var(--inline-detail-close-duration, 180ms) cubic-bezier(0.5, 0, 1, 1),
      margin-bottom var(--inline-detail-close-duration, 180ms) cubic-bezier(0.5, 0, 1, 1);
  }

  .detail-row.closing {
    grid-template-rows: minmax(0, 0fr);
    margin-bottom: 0;
    will-change: grid-template-rows, margin-bottom;
  }

  .detail-row.opening {
    animation: detail-row-open 180ms var(--ease-out) both;
  }

  @keyframes detail-row-open {
    from {
      grid-template-rows: minmax(0, 0fr);
      margin-bottom: 0;
    }
  }

  .detail-row > :global(.inline-detail-wrapper) {
    min-height: 0;
    overflow: hidden;
  }

  @media (prefers-reduced-motion: reduce) {
    .detail-row.opening {
      animation-duration: 1ms;
    }
  }

  .card {
    background: transparent;
    border-radius: var(--artwork-radius-albums, 10px);
    border: 2px solid transparent;
    box-shadow: none;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    cursor: pointer;
    position: relative;
    transition: background var(--motion-fast) var(--ease-out), box-shadow var(--motion-fast) var(--ease-out), filter var(--motion-medium) var(--ease-out);
    padding: 0;
    width: 100%;
    box-sizing: border-box;
  }

  .card.expanded {
    border-color: rgba(255, 255, 255, 0.2);
  }

  .card.expanded:hover {
    border-color: rgba(255, 255, 255, 0.25);
    box-shadow: var(--shadow-2);
  }

  .card:hover {
    background: var(--surface-hover);
    box-shadow: var(--shadow-2);
  }

  .view-container.scrolling .card {
    transition: background var(--motion-fast) var(--ease-out);
  }

  .view-container.scrolling .card:hover {
    box-shadow: none;
  }
  
  .card:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
  }


  .artwork {
    width: 100%;
    aspect-ratio: 1;
    overflow: hidden;
    border-radius: var(--artwork-radius-albums, 10px);
    position: relative;
    background: var(--surface-2);
  }

  .artwork-slot {
    width: 100%;
    height: 100%;
    border-radius: inherit;
    background: var(--surface-2);
  }

  .info {
    padding: 8px 2px 4px 2px;
    min-height: auto;
    display: flex;
    flex-direction: column;
    gap: 1px;
    align-items: center;
    text-align: center;
  }

  .title {
    font-weight: 600;
    margin-bottom: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 13px;
    line-height: 1.3;
    color: var(--text-primary);
    width: 100%;
  }

  .artist {
    font-size: 12px;
    color: var(--text-secondary);
    font-weight: 400;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.3;
    width: 100%;
  }

  .year {
    display: none;
  }


  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 48px;
    color: var(--text-tertiary);
    height: 400px;
  }

  .empty-state :global(svg) {
    opacity: 0.3;
  }

  .empty-title {
    font-size: 16px;
    font-weight: 500;
    margin: 0;
    color: var(--text-secondary);
  }

  .empty-hint {
    font-size: 13px;
    margin: 0;
    color: var(--text-tertiary);
  }

</style>
