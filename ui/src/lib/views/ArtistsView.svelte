<script lang="ts">
  import { onMount } from 'svelte';
  import { VList } from 'virtua/svelte';
  import { listArtistsPage } from '../api/library';
  import type { ArtistListItem, ArtistCursor } from '../types/library';
  import { navigate } from '../state/route';

  let artists: ArtistListItem[] = $state([]);
  let loading = $state(false);
  let nextCursor: ArtistCursor | undefined = $state(undefined);
  let hasMore = $state(true);
  let initialLoadComplete = $state(false);
  let scrollContainer: HTMLElement | undefined = $state();

  onMount(async () => {
    await loadMore();
    initialLoadComplete = true;
  });

  async function loadMore() {
    if (loading || !hasMore) return;
    loading = true;

    try {
      const page = await listArtistsPage(50, nextCursor);
      artists = [...artists, ...page.items];
      nextCursor = page.nextCursor;
      hasMore = !!nextCursor;
    } catch (e) {
      console.error('Failed to load artists:', e);
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

  function handleArtistClick(artist: ArtistListItem) {
    navigate({
      name: 'artist-detail',
      artistSort: artist.artistSort
    });
  }

  function getInitials(name: string): string {
    return name.slice(0, 1).toUpperCase();
  }
</script>

<div 
  class="view-container" 
  bind:this={scrollContainer}
  onscroll={handleScroll}
>
  <h1>Artists</h1>

  {#if !initialLoadComplete && artists.length === 0}
    <div class="loading-state">Loading...</div>
  {:else if artists.length === 0}
    <div class="empty-state">No artists found</div>
  {:else}
    <div class="list-wrapper">
      <VList data={artists}>
        {#snippet children(artist: ArtistListItem)}
          <div 
            class="item"
            role="button"
            tabindex="0"
            onkeydown={(e) => e.key === 'Enter' && handleArtistClick(artist)}
            onclick={() => handleArtistClick(artist)}
          >
            <div class="avatar">{getInitials(artist.artistDisplay)}</div>
            <div class="info">
              <div class="name">{artist.artistDisplay}</div>
              <div class="stats">
                {artist.trackCount} track{artist.trackCount === 1 ? '' : 's'} • 
                {artist.albumCount} album{artist.albumCount === 1 ? '' : 's'}
              </div>
            </div>
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

  .list-wrapper {
    flex-grow: 1;
    min-height: 200px;
    display: flex;
    flex-direction: column;
  }

  .item {
    background: var(--glass-highlight);
    padding: 1rem;
    margin-bottom: 0.5rem;
    border-radius: var(--glass-radius);
    border: 1px solid transparent;
    display: flex;
    align-items: center;
    gap: 1rem;
    cursor: pointer;
    transition: background 0.2s, border-color 0.2s, transform 0.1s;
  }

  .item:hover {
    background: var(--glass-border);
    border-color: rgba(255,255,255,0.2);
    transform: translateX(4px);
  }

  .avatar {
    width: 48px;
    height: 48px;
    background: linear-gradient(135deg, #444, #222);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    font-size: 1.2rem;
    color: #fff;
    flex-shrink: 0;
    box-shadow: 0 2px 4px rgba(0,0,0,0.2);
    border: 1px solid rgba(255,255,255,0.1);
  }

  .info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .name {
    font-size: 1.1rem;
    font-weight: 500;
  }

  .stats {
    font-size: 0.85rem;
    color: #aaa;
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
