<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listAlbumsPage } from '../api/library';
  import type { AlbumListItem, AlbumCursor } from '../types/library';
  import { navigate } from '../state/route';
  import { setViewTitle } from '../state/viewTitle';
  import { setAlphabetSelector, clearAlphabetSelector } from '../state/alphabetSelector';
  import { Fixtures } from '../data/fixtures';
  import { getArtworkBestForAlbum, getArtworkBytes } from '../api/artwork';
  import ArtworkPickerModal from '../components/ArtworkPickerModal.svelte';
  import SkeletonCard from '../components/SkeletonCard.svelte';
  import { MoreVertical, Play, Disc3 } from '@lucide/svelte';
  import { hoverScale, staggeredFadeIn, fadeIn } from '../utils/animations';
  import { VList } from 'virtua/svelte';

  let albums: AlbumListItem[] = $state([]);
  let loading = $state(false);
  let nextCursor: AlbumCursor | undefined = $state(undefined);
  let hasMore = $state(true);
  let initialLoadComplete = $state(false);
  let artworkUrls: Map<string, string> = $state(new Map());

  // Artwork picker modal state
  let pickerOpen = $state(false);
  let pickerAlbum: AlbumListItem | null = $state(null);

  // Virtualization state
  let containerWidth = $state(0);
  let vlistRef: VList<typeof rows[0]> | undefined = $state();
  
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

  let alphabetItems = $derived(albums.map(a => ({ sortKey: a.albumArtistSort })));

  function handleAlphabetSelect(index: number) {
    const rowIndex = Math.floor(index / columns);
    if (vlistRef) {
      vlistRef.scrollToIndex(rowIndex, { align: 'start', smooth: true });
    }
  }

  function getAlbumKey(album: AlbumListItem): string {
    return `${album.albumArtistSort}||${album.albumTitleSort}`;
  }

  onMount(async () => {
    setViewTitle('Albums');
    await loadMore();
    initialLoadComplete = true;
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

  async function loadMore() {
    if (loading || !hasMore) return;
    loading = true;

    try {
      const page = await listAlbumsPage(100, nextCursor);
      albums = [...albums, ...page.items];
      nextCursor = page.nextCursor;
      hasMore = !!nextCursor;
    } catch (e) {
      console.error('Failed to load albums:', e);
    } finally {
      loading = false;
    }
  }

  // VList range change handler for infinite scroll
  function handleRangeChange(startIndex: number, endIndex: number) {
    // Load more when approaching end of list
    const rowsRemaining = rows.length - endIndex;
    if (rowsRemaining < 5 && hasMore && !loading) {
      loadMore();
    }
  }

  function handleAlbumClick(album: AlbumListItem) {
    navigate({
      name: 'album-detail',
      albumArtistSort: album.albumArtistSort,
      albumTitleSort: album.albumTitleSort
    });
  }

  async function loadAlbumArtwork(album: AlbumListItem) {
    const key = getAlbumKey(album);
    if (artworkUrls.has(key)) return;

    // Runtime mode: use IPC
    try {
      const best = await getArtworkBestForAlbum(album.albumArtistSort, album.albumTitleSort);
      if (best.source !== 'none' && best.cacheKey && best.mime) {
        const bytes = await getArtworkBytes(best.cacheKey, best.mime);
        const dataUrl = `data:${bytes.mime};base64,${bytes.bytesBase64}`;
        artworkUrls = new Map(artworkUrls).set(key, dataUrl);
      }
    } catch (e) {
      console.error('Failed to load album artwork:', e);
    }
  }

  // Action for lazy loading artwork
  function lazyArtwork(node: HTMLElement, album: AlbumListItem) {
    loadAlbumArtwork(album);
    return {
      update(newAlbum: AlbumListItem) {
        loadAlbumArtwork(newAlbum);
      }
    };
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

  async function handleArtworkSelected(cacheKey: string) {
    if (!pickerAlbum) return;
    
    // Reload artwork for this album
    const key = getAlbumKey(pickerAlbum);
    artworkUrls = new Map(artworkUrls);
    artworkUrls.delete(key);
    
    // Force reload
    await loadAlbumArtwork(pickerAlbum);
  }
</script>

<div 
  class="view-container" 
  bind:clientWidth={containerWidth}
>
  {#if !initialLoadComplete && albums.length === 0}
    <div class="albums-grid">
      {#each Array(12) as _}
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
    <div class="list-wrapper">
      <VList bind:this={vlistRef} data={rows} getKey={(row) => getAlbumKey(row[0])}>
        {#snippet children(row)}
          <div class="grid-row" style="grid-template-columns: repeat({columns}, 1fr)">
            {#each row as album, i (getAlbumKey(album))}
              {@const artworkUrl = artworkUrls.get(getAlbumKey(album))}
              <div 
                class="card"
                role="button"
                tabindex="0"
                use:lazyArtwork={album}
                onkeydown={(e) => e.key === 'Enter' && handleAlbumClick(album)}
                onclick={() => handleAlbumClick(album)}
                use:hoverScale={{ scale: 1.02, duration: 200 }}
              >
                {#if artworkUrl}
                  <div class="artwork">
                    <img src={artworkUrl} alt="" loading="lazy" />
                    <div class="play-overlay">
                      <Play fill="white" size={24} />
                    </div>
                  </div>
                {:else}
                  <div class="artwork-placeholder">
                    <div class="play-overlay">
                      <Play fill="white" size={24} />
                    </div>
                  </div>
                {/if}
                <div class="info">
                  <div class="title" title={album.albumTitleDisplay}>{album.albumTitleDisplay}</div>
                  <div class="artist" title={album.albumArtistDisplay}>{album.albumArtistDisplay}</div>
                  {#if album.year}<div class="year">{album.year}</div>{/if}
                </div>
                <button 
                  class="choose-artwork-btn"
                  onclick={(e) => openArtworkPicker(album, e)}
                  title="Choose Artwork"
                >
                  <MoreVertical size={16} />
                </button>
              </div>
            {/each}
          </div>
        {/snippet}
      </VList>
    </div>
    
    {#if loading}
      <div class="loading-more">Loading more...</div>
    {/if}
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
    padding-top: 12px;
    padding-right: 0;
    padding-bottom: 0;
    color: #fff;
    height: 100%;
    overflow-y: auto;
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
  
  /* Allow hover scale shadow to overflow the scroll container top edge.
     Virtua generates: div[overflow:auto] > div > div > grid-rows
     We add padding inside the scroll container and use negative margin on 
     the first grid row to maintain scroll start position. */
  .list-wrapper :global(> div) {
    /* The virtua scroll container - add internal padding for hover scale expansion.
       contain: strict creates a paint boundary that clips at the content-box edge.
       Cards scale(1.02) which expands ~2.3px upward. 20px padding ensures no clipping. */
    padding-top: 20px !important;
  }
  
  .list-wrapper :global(> div > div),
  .list-wrapper :global(> div > div > div) {
    overflow: visible !important;
  }
  
  /* Add top margin to first visible row for hover scale expansion.
     The first div[position:absolute] is the first virtualized chunk.
     Using padding on inner containers won't work due to contain: size. */
  .list-wrapper :global(> div > div > div:first-child) {
    margin-top: 4px !important;
  }
  
  .grid-row {
    display: grid;
    gap: 20px;
    margin-bottom: 20px;
    padding-right: 1rem;
    overflow: visible;
  }

  .card {
    background: transparent;
    border-radius: var(--artwork-radius-albums, 10px);
    border: 1px solid transparent;
    box-shadow: none;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    cursor: pointer;
    position: relative;
    transition: box-shadow var(--motion-fast) var(--ease-out);
  }

  .card:hover {
    box-shadow: var(--shadow-2);
  }
  
  .card:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring), var(--shadow-2);
  }

  .play-overlay {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.75);
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    opacity: 0 !important;
    visibility: hidden;
    transition: all var(--motion-fast) var(--ease-out);
    pointer-events: none;
    z-index: 2;
    box-shadow: var(--shadow-2);
  }

  .card:hover .play-overlay {
    opacity: 1 !important;
    visibility: visible;
  }

  .card:hover .artwork img {
    opacity: 0.85;
  }

  .card:hover .choose-artwork-btn {
    opacity: 1;
  }

  .choose-artwork-btn {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.7);
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    opacity: 0;
    transition: all var(--motion-fast) var(--ease-out);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
  }

  .choose-artwork-btn:hover {
    background: var(--theme-accent);
    color: #000;
    box-shadow: var(--shadow-2);
  }

  .artwork-placeholder {
    width: 100%;
    aspect-ratio: 1;
    background: linear-gradient(135deg, rgba(255,255,255,0.05) 0%, rgba(255,255,255,0.02) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #555;
    border-radius: var(--artwork-radius-albums, 10px);
    position: relative;
  }

  .artwork {
    width: 100%;
    aspect-ratio: 1;
    overflow: hidden;
    border-radius: var(--artwork-radius-albums, 10px);
    position: relative;
  }

  .artwork img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: opacity 0.15s ease;
  }

  .info {
    padding: 8px 4px 4px 4px;
    min-height: auto;
    display: flex;
    flex-direction: column;
  }

  .title {
    font-weight: 600;
    margin-bottom: 0.25rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 13px;
    line-height: 1.3;
    color: var(--text-primary);
  }

  .artist {
    font-size: 12px;
    color: var(--text-tertiary);
    font-weight: 400;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.3;
  }

  .year {
    display: none;
  }

  .loading-state {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 200px;
    font-size: 1.2rem;
    color: #888;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 48px;
    color: rgba(255, 255, 255, 0.4);
    height: 400px;
  }

  .empty-state :global(svg) {
    opacity: 0.3;
  }

  .empty-title {
    font-size: 16px;
    font-weight: 500;
    margin: 0;
    color: rgba(255, 255, 255, 0.5);
  }

  .empty-hint {
    font-size: 13px;
    margin: 0;
    color: rgba(255, 255, 255, 0.35);
  }

  .loading-more {
    text-align: center;
    padding: 2rem;
    color: #888;
  }
</style>
