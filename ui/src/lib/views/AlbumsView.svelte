<script lang="ts">
  import { onMount } from 'svelte';
  import { VList } from 'virtua/svelte';
  import { listAlbumsPage } from '../api/library';
  import type { AlbumListItem, AlbumCursor } from '../types/library';
  import { navigate } from '../state/route';

  let albums: AlbumListItem[] = $state([]);
  let loading = $state(false);
  let nextCursor: AlbumCursor | undefined = $state(undefined);
  let hasMore = $state(true);
  let initialLoadComplete = $state(false);
  let scrollContainer: HTMLElement | undefined = $state();
  
  let containerWidth = $state(0);
  const minItemWidth = 180; // approximate width of card + gap
  let columns = $derived(Math.max(1, Math.floor((containerWidth || 800) / minItemWidth)));
  
  let rows = $derived.by(() => {
    const res: AlbumListItem[][] = [];
    if (!albums.length) return res;
    
    for (let i = 0; i < albums.length; i += columns) {
      res.push(albums.slice(i, i + columns));
    }
    return res;
  });

  // Load initial page
  onMount(async () => {
    await loadMore();
    initialLoadComplete = true;
  });

  async function loadMore() {
    if (loading || !hasMore) return;
    loading = true;

    try {
      const page = await listAlbumsPage(50, nextCursor);
      albums = [...albums, ...page.items];
      nextCursor = page.nextCursor;
      hasMore = !!nextCursor;
    } catch (e) {
      console.error('Failed to load albums:', e);
    } finally {
      loading = false;
    }
  }

  function handleScroll(e: Event) {
    const target = e.target as HTMLElement;
    const remaining = target.scrollHeight - target.scrollTop - target.clientHeight;
    if (remaining < 500) {
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
</script>

<div 
  class="view-container" 
  bind:this={scrollContainer}
  bind:clientWidth={containerWidth}
  onscroll={handleScroll}
>
  <h1>Albums</h1>
  
  {#if !initialLoadComplete && albums.length === 0}
    <div class="loading-state">Loading...</div>
  {:else if albums.length === 0}
    <div class="empty-state">No albums found</div>
  {:else}
    <div class="grid-wrapper">
      <VList data={rows}>
        {#snippet children(row: AlbumListItem[])}
          <div 
            class="row"
            style:grid-template-columns="repeat({columns}, 1fr)"
          >
            {#each row as item}
              <div 
                class="card-wrapper"
                role="button"
                tabindex="0"
                onkeydown={(e) => e.key === 'Enter' && handleAlbumClick(item)}
                onclick={() => handleAlbumClick(item)}
              >
                <div class="card">
                  <div class="artwork-placeholder"></div>
                  <div class="info">
                    <div class="title" title={item.albumTitleDisplay}>{item.albumTitleDisplay}</div>
                    <div class="artist" title={item.albumArtistDisplay}>{item.albumArtistDisplay}</div>
                    {#if item.year}<div class="year">{item.year}</div>{/if}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/snippet}
      </VList>
      
      {#if loading}
        <div class="loading-more">Loading more...</div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .view-container {
    padding: 2rem;
    color: #fff;
    height: 100%;
    overflow-y: auto;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
  }

  h1 {
    margin-bottom: 1.5rem;
    flex-shrink: 0;
  }

  .grid-wrapper {
    flex-grow: 1;
    min-height: 200px;
  }
  
  .row {
    display: grid;
    gap: 1.5rem;
    margin-bottom: 1.5rem;
    padding-right: 1.5rem; /* Match gap to avoid horizontal scroll if any */
  }

  .card-wrapper {
    /* Width is handled by grid 1fr */
    aspect-ratio: 0.7; /* Approximation of card aspect ratio */
  }

  .card {
    background: var(--glass-highlight);
    border-radius: var(--glass-radius);
    border: 1px solid var(--glass-border);
    overflow: hidden;
    height: 100%;
    display: flex;
    flex-direction: column;
    transition: transform 0.2s, background-color 0.2s;
    cursor: pointer;
  }

  .card:hover {
    transform: translateY(-4px);
    background: var(--glass-border);
  }

  .artwork-placeholder {
    width: 100%;
    aspect-ratio: 1;
    background: #333;
    display: flex;
    align-items: center;
    justify-content: center;
    border-bottom: 1px solid rgba(255,255,255,0.05);
  }

  .info {
    padding: 0.8rem;
    overflow: hidden;
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .title {
    font-weight: 600;
    margin-bottom: 0.2rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 0.95rem;
  }

  .artist {
    font-size: 0.85rem;
    color: #aaa;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 0.2rem;
  }

  .year {
    font-size: 0.8rem;
    color: #666;
    margin-top: auto;
  }

  .loading-state, .empty-state {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 200px;
    font-size: 1.2rem;
    color: #888;
  }

  .loading-more {
    text-align: center;
    padding: 1rem;
    color: #888;
    width: 100%;
  }
</style>
