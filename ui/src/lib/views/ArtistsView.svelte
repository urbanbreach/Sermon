<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { VList } from 'virtua/svelte';
  import { listArtistsPage } from '../api/library';
  import type { ArtistListItem, ArtistCursor } from '../types/library';
  import { navigate } from '../state/route';
  import { setViewTitle } from '../state/viewTitle';
  import { setAlphabetSelector, clearAlphabetSelector } from '../state/alphabetSelector';

  let artists: ArtistListItem[] = $state([]);
  let loading = $state(false);
  let nextCursor: ArtistCursor | undefined = $state(undefined);
  let hasMore = $state(true);
  let initialLoadComplete = $state(false);
  let scrollContainer: HTMLElement | undefined = $state();
  let vlistRef: VList<ArtistListItem> | undefined = $state();

  let alphabetItems = $derived(artists.map(a => ({ sortKey: a.artistSort })));

  function handleAlphabetSelect(index: number) {
    if (vlistRef) {
      vlistRef.scrollToIndex(index, { align: 'start', smooth: true });
    }
  }

  onMount(async () => {
    setViewTitle('Artists');
    await loadMore();
    initialLoadComplete = true;
  });

  onDestroy(() => {
    setViewTitle('');
    clearAlphabetSelector();
  });

  // Update alphabet selector in TopBar whenever artists change
  $effect(() => {
    if (artists.length > 0) {
      setAlphabetSelector(alphabetItems, handleAlphabetSelect);
    }
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
  {#if !initialLoadComplete && artists.length === 0}
    <div class="loading-state">Loading...</div>
  {:else if artists.length === 0}
    <div class="empty-state">No artists found</div>
{:else}
    <div class="list-wrapper">
      <VList bind:this={vlistRef} data={artists} itemSize={52}>
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
    padding: 1rem;
    padding-top: 12px;
    padding-right: 0;
    padding-bottom: calc(var(--layout-player-height, 80px) + 2rem);
    color: #fff;
    height: 100%;
    overflow-y: auto;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    background: transparent;
    min-height: 100%;
  }

  .list-wrapper {
    flex-grow: 1;
    min-height: 200px;
    display: flex;
    flex-direction: column;
  }

  /* Compact row layout - 52px height for higher density */
  .item {
    height: 52px;
    padding: 0 var(--space-3, 12px);
    padding-right: 1rem;
    margin-bottom: 0;
    border-radius: var(--radius-sm, 8px);
    border: none;
    background: transparent;
    display: flex;
    align-items: center;
    gap: 12px;
    cursor: pointer;
    transition: background var(--motion-fast) var(--ease-out);
    box-sizing: border-box;
  }

  .item:hover {
    background: var(--surface-hover);
  }

  .item:focus-visible {
    background: var(--surface-hover);
    box-shadow: inset 0 0 0 1px var(--accent-medium);
    outline: none;
  }

  .avatar {
    width: 36px;
    height: 36px;
    background: linear-gradient(135deg, #3a3a3a, #252525);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    font-size: 14px;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .name {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .stats {
    font-size: 12px;
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
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
