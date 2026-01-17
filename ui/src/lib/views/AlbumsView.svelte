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
  
  let containerWidth = $state(0);
  const cardWidth = 180;
  const gap = 24;
  let columns = $derived(Math.max(1, Math.floor((containerWidth || 600) / (cardWidth + gap))));
  
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

  // Debounced scroll handler to prevent freezing
  let scrollTimeout: ReturnType<typeof setTimeout> | null = null;
  function handleScroll(e: Event) {
    if (scrollTimeout) clearTimeout(scrollTimeout);
    scrollTimeout = setTimeout(() => {
      const target = e.target as HTMLElement;
      const remaining = target.scrollHeight - target.scrollTop - target.clientHeight;
      if (remaining < 800 && hasMore && !loading) {
        loadMore();
      }
    }, 150);
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
          <div class="row">
            {#each row as item}
              <div 
                class="card"
                role="button"
                tabindex="0"
                onkeydown={(e) => e.key === 'Enter' && handleAlbumClick(item)}
                onclick={() => handleAlbumClick(item)}
              >
                <div class="artwork-placeholder">
                  <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                    <circle cx="12" cy="12" r="10"/>
                    <circle cx="12" cy="12" r="3"/>
                  </svg>
                </div>
                <div class="info">
                  <div class="title" title={item.albumTitleDisplay}>{item.albumTitleDisplay}</div>
                  <div class="artist" title={item.albumArtistDisplay}>{item.albumArtistDisplay}</div>
                  {#if item.year}<div class="year">{item.year}</div>{/if}
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
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
  
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 1.5rem;
    padding-bottom: 1.5rem;
  }

  .card {
    width: 180px;
    background: var(--glass-highlight);
    border-radius: var(--glass-radius);
    border: 1px solid var(--glass-border);
    overflow: hidden;
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
    height: 180px;
    background: linear-gradient(135deg, #2a2a2a 0%, #1a1a1a 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #555;
    flex-shrink: 0;
  }

  .info {
    padding: 0.75rem;
    min-height: 70px;
    display: flex;
    flex-direction: column;
  }

  .title {
    font-weight: 600;
    margin-bottom: 0.25rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 0.9rem;
    line-height: 1.2;
  }

  .artist {
    font-size: 0.8rem;
    color: #aaa;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.2;
  }

  .year {
    font-size: 0.75rem;
    color: #666;
    margin-top: auto;
    padding-top: 0.25rem;
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
  }
</style>
