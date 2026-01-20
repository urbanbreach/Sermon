<script lang="ts">
  import { onMount } from 'svelte';
  import { listAlbumsPage } from '../api/library';
  import type { AlbumListItem, AlbumCursor } from '../types/library';
  import { navigate } from '../state/route';
  import { Fixtures } from '../data/fixtures';
  import { getArtworkBestForAlbum, getArtworkBytes } from '../api/artwork';
  import ArtworkPickerModal from '../components/ArtworkPickerModal.svelte';

  let albums: AlbumListItem[] = $state([]);
  let loading = $state(false);
  let nextCursor: AlbumCursor | undefined = $state(undefined);
  let hasMore = $state(true);
  let initialLoadComplete = $state(false);
  let artworkUrls: Map<string, string> = $state(new Map());

  // Artwork picker modal state
  let pickerOpen = $state(false);
  let pickerAlbum: AlbumListItem | null = $state(null);

  function getAlbumKey(album: AlbumListItem): string {
    return `${album.albumArtistSort}||${album.albumTitleSort}`;
  }

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

  async function loadAlbumArtwork(album: AlbumListItem) {
    const key = getAlbumKey(album);
    if (artworkUrls.has(key)) return;

    // Mock/Snapshot mode: use fixtures
    if (import.meta.env.SERMON_MOCK === '1') {
      const fixtureAlbums = Fixtures.getAlbums();
      const fixtureArtists = Fixtures.getArtists();

      for (const fixtureAlbum of fixtureAlbums) {
        const artist = fixtureArtists.find(a => a.id === fixtureAlbum.artistId);
        const artistSort = (artist?.name?.trim().toLowerCase()) || 'unknown artist';
        const titleSort = (fixtureAlbum.title?.trim().toLowerCase()) || 'unknown album';

        if (artistSort === album.albumArtistSort && titleSort === album.albumTitleSort) {
          const artworkUrl = Fixtures.getArtworkPath(fixtureAlbum.artworkFile);
          if (artworkUrl) {
            artworkUrls = new Map(artworkUrls).set(key, artworkUrl);
          }
          return;
        }
      }
      return;
    }

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

  $effect(() => {
    // Load artwork for visible albums
    for (const album of albums) {
      loadAlbumArtwork(album);
    }
  });

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

<div class="view-container" onscroll={handleScroll}>
  <h1>Albums</h1>
  
  {#if !initialLoadComplete && albums.length === 0}
    <div class="loading-state">Loading...</div>
  {:else if albums.length === 0}
    <div class="empty-state">No albums found</div>
  {:else}
    <div class="albums-grid">
      {#each albums as album}
        {@const artworkUrl = artworkUrls.get(getAlbumKey(album))}
        <div 
          class="card"
          role="button"
          tabindex="0"
          onkeydown={(e) => e.key === 'Enter' && handleAlbumClick(album)}
          onclick={() => handleAlbumClick(album)}
        >
          {#if artworkUrl}
            <div class="artwork">
              <img src={artworkUrl} alt="" loading="lazy" />
            </div>
          {:else}
            <div class="artwork-placeholder">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="12" cy="12" r="10"/>
                <circle cx="12" cy="12" r="3"/>
              </svg>
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
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="2"/>
              <circle cx="8.5" cy="8.5" r="1.5"/>
              <path d="M21 15l-5-5L5 21"/>
            </svg>
          </button>
        </div>
      {/each}
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
    position: relative;
  }

  .card:hover {
    transform: translateY(-4px);
    background: var(--glass-border);
  }

  .card:hover .choose-artwork-btn {
    opacity: 1;
  }

  .choose-artwork-btn {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 32px;
    height: 32px;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: #aaa;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s, background 0.2s, color 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
  }

  .choose-artwork-btn:hover {
    background: rgba(74, 175, 255, 0.9);
    color: #000;
    border-color: #4af;
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

  .artwork {
    width: 100%;
    aspect-ratio: 1;
    overflow: hidden;
  }

  .artwork img {
    width: 100%;
    height: 100%;
    object-fit: cover;
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
