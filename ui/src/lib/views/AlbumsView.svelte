<script lang="ts">
  import { onMount } from 'svelte';
  import { listAlbumsPage } from '../api/library';
  import type { AlbumListItem, AlbumCursor } from '../types/library';
  import { navigate } from '../state/route';

  let albums: AlbumListItem[] = $state([]);
  let loading = $state(false);
  let nextCursor: AlbumCursor | undefined = $state(undefined);
  let hasMore = $state(true);
  let initialLoadComplete = $state(false);

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

<div class="view-container" onscroll={handleScroll}>
  <h1>Albums</h1>
  
  {#if !initialLoadComplete && albums.length === 0}
    <div class="loading-state">Loading...</div>
  {:else if albums.length === 0}
    <div class="empty-state">No albums found</div>
  {:else}
    <div class="albums-grid">
      {#each albums as album}
        <div 
          class="card"
          role="button"
          tabindex="0"
          onkeydown={(e) => e.key === 'Enter' && handleAlbumClick(album)}
          onclick={() => handleAlbumClick(album)}
        >
          <div class="artwork-placeholder">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="12" cy="12" r="10"/>
              <circle cx="12" cy="12" r="3"/>
            </svg>
          </div>
          <div class="info">
            <div class="title" title={album.albumTitleDisplay}>{album.albumTitleDisplay}</div>
            <div class="artist" title={album.albumArtistDisplay}>{album.albumArtistDisplay}</div>
            {#if album.year}<div class="year">{album.year}</div>{/if}
          </div>
        </div>
      {/each}
    </div>
    
    {#if loading}
      <div class="loading-more">Loading more...</div>
    {/if}
  {/if}
</div>

<style>
  .view-container {
    padding: 2rem;
    color: #fff;
    height: 100%;
    overflow-y: auto;
    box-sizing: border-box;
  }

  h1 {
    margin-bottom: 1.5rem;
  }

  .albums-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 1.5rem;
  }

  .card {
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
    aspect-ratio: 1;
    background: linear-gradient(135deg, #2a2a2a 0%, #1a1a1a 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #555;
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
    padding: 2rem;
    color: #888;
  }
</style>
