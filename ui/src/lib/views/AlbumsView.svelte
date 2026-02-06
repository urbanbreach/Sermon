<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { getLibraryStats, listAlbumsPage, listAlbumTracksPage } from '../api/library';
  import type { AlbumListItem } from '../types/library';
  import { setViewTitle } from '../state/viewTitle';
  import { setAlphabetSelector, clearAlphabetSelector } from '../state/alphabetSelector';
  import {
    albumArtworkCacheVersion,
    getAlbumKey,
    bumpAlbumArtworkVersion,
    resetAlbumArtworkCache
  } from '../state/albumArtwork';
  import ArtworkPickerModal from '../components/ArtworkPickerModal.svelte';
  import TagEditor from '../components/TagEditor.svelte';
  import * as ContextMenu from '../components/primitives/ContextMenu.svelte';
import { playNowWithQueue, addToQueue, addToQueueNext } from '../state/playback';
  import AlbumInlineDetail from '../components/AlbumInlineDetail.svelte';
  import ArtworkCanvas from '../components/ArtworkCanvas.svelte';
  import { expandedAlbum, toggleAlbumInline, openAlbumInlineFromItem, clearAlbumInline } from '../state/albumInline';
  import SkeletonCard from '../components/SkeletonCard.svelte';
  import { Disc3 } from '@lucide/svelte';
  import { fadeIn } from '../utils/animations';
  import { setAlbumGridScrolling } from '../utils/artworkDecodeQueue';
  import { VList } from 'virtua/svelte';
  import { fade, slide } from 'svelte/transition';

  let albums: AlbumListItem[] = $state([]);
  let loading = $state(false);
  let initialLoadComplete = $state(false);
  let artworkCacheSeed = $derived($albumArtworkCacheVersion);

  // Artwork picker modal state
  let pickerOpen = $state(false);
  let pickerAlbum: AlbumListItem | null = $state(null);

  // Tag editor state
  let tagEditorOpen = $state(false);
  let editingTrackIds = $state<number[]>([]);
  let contextMenuAlbum: AlbumListItem | null = $state(null);

  // Virtualization state
  let containerWidth = $state(0);
  type RowItem =
    | { type: 'row'; albums: AlbumListItem[]; rowIndex: number }
    | { type: 'detail'; album: AlbumListItem; rowIndex: number };

  let vlistRef: VList<RowItem> | undefined = $state();
  let listWrapperEl = $state<HTMLDivElement | null>(null);
  let isScrolling = $state(false);
  let deferHighRes = $state(false);
  let smoothedScrollVelocity = 0;
  let highVelocityStartedAt = 0;
  let lastScrollTop = 0;
  let lastScrollTs = 0;
  let scrollSettlerRaf = 0;
  let lastScrollAt = 0;
  const SCROLL_IDLE_MS = 140;
  const HIGH_VELOCITY_ENTER_PX_PER_S = 14000;
  const HIGH_VELOCITY_CONFIRM_MS = 120;
  let activeBufferSize = $derived(deferHighRes ? 80 : 220);
  
  // Compute columns based on container width (min 160px + 20px gap)
  // containerWidth - 32 accounts for 1rem (16px) padding on each side
  let columns = $derived(Math.max(1, Math.floor((containerWidth - 32 + 20) / 180)));
  
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
            contextMenuAlbum = null;
            deferHighRes = true;
          }
        } else {
          highVelocityStartedAt = 0;
        }
      }

      if (isScrolling) return;
      isScrolling = true;
      setAlbumGridScrolling(true);

      const settle = () => {
        if (performance.now() - lastScrollAt >= SCROLL_IDLE_MS) {
          isScrolling = false;
          smoothedScrollVelocity = 0;
          highVelocityStartedAt = 0;
          deferHighRes = false;
          contextMenuAlbum = null;
          lastScrollTs = 0;
          setAlbumGridScrolling(false);
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
      contextMenuAlbum = null;
      lastScrollTs = 0;
      setAlbumGridScrolling(false);
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
    loading = true;
    initialLoadComplete = false;

    try {
      const { albumCount } = await getLibraryStats();
      const page = await listAlbumsPage(albumCount + 10, undefined);
      albums = page.items;
    } catch (e) {
      console.error('Failed to load albums:', e);
    } finally {
      loading = false;
      initialLoadComplete = true;
    }
  }

  onMount(() => {
    setViewTitle('Albums');
    void loadAllAlbums();
    
    // Listen for library changes (after scan) to refresh artwork cache
    const handleLibraryChange = () => {
      // Reset album list to reload from scratch
      albums = [];
      resetAlbumArtworkCache();
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
    toggleAlbumInline({
      albumArtistSort: album.albumArtistSort,
      albumTitleSort: album.albumTitleSort
    });
  }

  function handleAlbumDoubleClick(album: AlbumListItem) {
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
      editingTrackIds = page.items.map(t => t.id);
      tagEditorOpen = true;
    } catch (e) {
      console.error('Failed to load album tracks for editing', e);
    }
  }

  function closeTagEditor() {
    tagEditorOpen = false;
    editingTrackIds = [];
  }

  function handleAlbumContextMenu(album: AlbumListItem) {
    contextMenuAlbum = album;
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

  async function handleContextMenuPlayAlbum() {
    const album = contextMenuAlbum;
    if (!album) return;
    await handlePlayAlbum(album);
  }

  async function handleContextMenuQueueAlbumNext() {
    const album = contextMenuAlbum;
    if (!album) return;
    await handleQueueAlbumNext(album);
  }

  async function handleContextMenuQueueAlbumLast() {
    const album = contextMenuAlbum;
    if (!album) return;
    await handleQueueAlbumLast(album);
  }

  async function handleContextMenuEditAlbum() {
    const album = contextMenuAlbum;
    if (!album) return;
    await openTagEditorForAlbum(album);
  }

</script>

<div 
  class="view-container" 
  class:scrolling={isScrolling}
  bind:clientWidth={containerWidth}
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
    <div class="list-wrapper" bind:this={listWrapperEl}>
      <ContextMenu.Root>
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
                  {#if deferHighRes}
                    <div
                      class="card"
                      role="button"
                      tabindex="0"
                      onkeydown={(e) => e.key === 'Enter' && handleAlbumClick(album)}
                      onclick={(e) => handleAlbumClick(album, e)}
                      ondblclick={() => handleAlbumDoubleClick(album)}
                    >
                      <div class="artwork">
                        <ArtworkCanvas
                          cacheKey={album.artworkCacheKey}
                          artistSort={album.albumArtistSort}
                          titleSort={album.albumTitleSort}
                          size={256}
                          deferHighRes={deferHighRes}
                          alt="{album.albumTitleDisplay} artwork"
                        />
                      </div>
                      <div class="info">
                        <div class="title" title={album.albumTitleDisplay}>{album.albumTitleDisplay}</div>
                        <div class="artist" title={album.albumArtistDisplay}>{album.albumArtistDisplay}</div>
                        {#if album.year}<div class="year">{album.year}</div>{/if}
                      </div>
                    </div>
                  {:else}
                    <ContextMenu.Trigger>
                      {#snippet child({ props })}
                        <div
                          {...props}
                          class="card"
                          role="button"
                          tabindex="0"
                          onmousedown={(e) => e.button === 2 && handleAlbumContextMenu(album)}
                          onkeydown={(e) => e.key === 'Enter' && handleAlbumClick(album)}
                          onclick={(e) => handleAlbumClick(album, e)}
                          ondblclick={() => handleAlbumDoubleClick(album)}
                        >
                          <div class="artwork">
                            <ArtworkCanvas
                              cacheKey={album.artworkCacheKey}
                              artistSort={album.albumArtistSort}
                              titleSort={album.albumTitleSort}
                              size={256}
                              deferHighRes={deferHighRes}
                              alt="{album.albumTitleDisplay} artwork"
                            />
                          </div>
                          <div class="info">
                            <div class="title" title={album.albumTitleDisplay}>{album.albumTitleDisplay}</div>
                            <div class="artist" title={album.albumArtistDisplay}>{album.albumArtistDisplay}</div>
                            {#if album.year}<div class="year">{album.year}</div>{/if}
                          </div>
                        </div>
                      {/snippet}
                    </ContextMenu.Trigger>
                  {/if}
                {/each}
              </div>
            {:else}
              <div class="detail-row" transition:slide={{ duration: 220 }}>
                <div class="detail-fade" in:fade={{ duration: 220 }} out:fade={{ duration: 150 }}>
                  <AlbumInlineDetail
                    album={item.album}
                    onClose={() => clearAlbumInline()}
                    onedit={(trackIds) => {
                      editingTrackIds = trackIds;
                      tagEditorOpen = true;
                    }}
                  />
                </div>
              </div>
            {/if}
          {/snippet}
        </VList>

        <ContextMenu.Portal>
          <ContextMenu.Content class="dropdown-content" data-testid="album-context-menu">
            <ContextMenu.Item class="dropdown-item" disabled={!contextMenuAlbum} onclick={handleContextMenuPlayAlbum}>Play Album</ContextMenu.Item>
            <ContextMenu.Item class="dropdown-item" disabled={!contextMenuAlbum} onclick={handleContextMenuQueueAlbumNext}>Queue Album Next</ContextMenu.Item>
            <ContextMenu.Item class="dropdown-item" disabled={!contextMenuAlbum} onclick={handleContextMenuQueueAlbumLast}>Queue Album Last</ContextMenu.Item>
            <ContextMenu.Separator class="dropdown-separator" />
            <ContextMenu.Item class="dropdown-item" disabled={!contextMenuAlbum} onclick={handleContextMenuEditAlbum}>Edit</ContextMenu.Item>
          </ContextMenu.Content>
        </ContextMenu.Portal>
      </ContextMenu.Root>
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

<TagEditor trackIds={editingTrackIds} open={tagEditorOpen} onclose={closeTagEditor} />

<style>
  .view-container {
    padding: 1rem;
    padding-top: 12px;
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
    gap: 20px;
    padding-right: 1rem;
  }
  
  .list-wrapper {
    flex: 1;
    padding-top: 0; /* Padding moved inside virtua scroll container */
  }
  
  .grid-row {
    display: grid;
    gap: 20px;
    margin-bottom: 20px;
    padding-right: 1rem;
    overflow: visible;
  }

  .grid-row > .card {
    min-width: 0; /* Allow grid items to shrink below content size */
    max-width: 100%;
  }

  .detail-row {
    padding-right: 1rem;
    margin-bottom: 24px;
  }

  .detail-fade {
    will-change: opacity, transform;
  }

  .card {
    background: transparent;
    border-radius: var(--artwork-radius-albums, 10px);
    border: none;
    box-shadow: none;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    cursor: pointer;
    position: relative;
    transition: background var(--motion-fast) var(--ease-out), box-shadow var(--motion-fast) var(--ease-out);
    padding: 0;
    width: 100%;
    box-sizing: border-box;
  }

  .card:hover {
    background: var(--surface-hover);
    box-shadow: var(--shadow-2);
  }

  .view-container.scrolling .card {
    transition: background var(--motion-fast) var(--ease-out);
    pointer-events: none;
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

  .info {
    padding: 10px 0 0 0;
    min-height: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
    align-items: center;
    text-align: center;
  }

  .title {
    font-weight: 600;
    margin-bottom: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 14px;
    line-height: 1.3;
    color: var(--text-primary);
    width: 100%;
  }

  .artist {
    font-size: 13px;
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
