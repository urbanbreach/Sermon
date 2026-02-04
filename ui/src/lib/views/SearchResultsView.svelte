<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { currentRoute, goBack, navigate } from '../state/route';
  import { playNow, addToQueue } from '../state/playback';
  import { searchTracksPage, searchAlbumsPage, searchArtistsPage } from '../api/library';
  import type { TrackRow, AlbumListItem, ArtistListItem, OffsetCursor, AlbumCursor, ArtistCursor } from '../types/library';
  import { openAlbumInlineFromItem } from '../state/albumInline';
  import ArtworkImage from '../components/ArtworkImage.svelte';
  import { ArrowLeft } from '@lucide/svelte';
  import { VList } from 'virtua/svelte';

  let query = $derived(
    $currentRoute.name === 'search-results' ? $currentRoute.query : ''
  );

  // --- Artists State ---
  let artists: ArtistListItem[] = $state([]);
  let artistsLoading = $state(false);
  let artistsCursor: ArtistCursor | undefined = $state(undefined);
  let artistsHasMore = $state(true);
  let artistsContainer: HTMLElement | undefined = $state();

  // --- Albums State ---
  let albums: AlbumListItem[] = $state([]);
  let albumsLoading = $state(false);
  let albumsCursor: AlbumCursor | undefined = $state(undefined);
  let albumsHasMore = $state(true);

  // --- Tracks State ---
  let tracks: TrackRow[] = $state([]);
  let tracksLoading = $state(false);
  let tracksCursor: OffsetCursor | undefined = $state(undefined);
  let tracksHasMore = $state(true);

  // --- Initialization ---
  let initializedQuery = '';

  $effect(() => {
    if (query && query !== initializedQuery) {
      initializedQuery = query;
      resetAndLoadAll();
    }
  });

  function resetAndLoadAll() {
    artists = [];
    artistsCursor = undefined;
    artistsHasMore = true;
    
    albums = [];
    albumsCursor = undefined;
    albumsHasMore = true;

    tracks = [];
    tracksCursor = undefined;
    tracksHasMore = true;

    loadMoreArtists();
    loadMoreAlbums();
    loadMoreTracks();
  }

  // --- Loaders ---

  async function loadMoreArtists() {
    if (artistsLoading || !artistsHasMore) return;
    artistsLoading = true;
    try {
      const page = await searchArtistsPage(query, 20, artistsCursor);
      artists = [...artists, ...page.items];
      artistsCursor = page.nextCursor;
      artistsHasMore = !!artistsCursor;
    } catch (e) {
      console.error('Failed to load artists:', e);
    } finally {
      artistsLoading = false;
    }
  }

  async function loadMoreAlbums() {
    if (albumsLoading || !albumsHasMore) return;
    albumsLoading = true;
    try {
      const page = await searchAlbumsPage(query, 20, albumsCursor);
      albums = [...albums, ...page.items];
      albumsCursor = page.nextCursor;
      albumsHasMore = !!albumsCursor;
    } catch (e) {
      console.error('Failed to load albums:', e);
    } finally {
      albumsLoading = false;
    }
  }

  async function loadMoreTracks() {
    if (tracksLoading || !tracksHasMore) return;
    tracksLoading = true;
    try {
      // Default sort by title for search results? Or relevance?
      // The API requires sortBy/direction, let's assume 'title' 'asc' for now or empty if backend handles relevance.
      // Based on API signature: query, sortBy, direction, limit, cursor
      // I'll use 'title', 'asc' as a safe default.
      const page = await searchTracksPage(query, 'title', 'asc', 50, tracksCursor);
      tracks = [...tracks, ...page.items];
      tracksCursor = page.nextCursor;
      tracksHasMore = !!tracksCursor;
    } catch (e) {
      console.error('Failed to load tracks:', e);
    } finally {
      tracksLoading = false;
    }
  }

  // --- Intersection Observers for Infinite Scroll ---
  
  // Use a simple action for intersection observer
  function infiniteScroll(node: HTMLElement, callback: () => void) {
    const observer = new IntersectionObserver((entries) => {
      if (entries[0].isIntersecting) {
        callback();
      }
    }, { threshold: 0.1, rootMargin: '100px' });

    observer.observe(node);

    return {
      destroy() {
        observer.disconnect();
      }
    };
  }

  // Horizontal scroll for artists
  function horizontalScroll(node: HTMLElement, callback: () => void) {
    const observer = new IntersectionObserver((entries) => {
      if (entries[0].isIntersecting) {
        callback();
      }
    }, { threshold: 0.1, root: artistsContainer, rootMargin: '0px 200px 0px 0px' }); // Load early

    observer.observe(node);
    return {
      destroy() {
        observer.disconnect();
      }
    };
  }

  // --- Interaction Handlers ---

  function handleArtistClick(artist: ArtistListItem) {
    navigate({
      name: 'artist-detail',
      artistSort: artist.artistSort
    });
  }

  function handleAlbumClick(album: AlbumListItem) {
    openAlbumInlineFromItem(album);
    navigate({ name: 'albums' });
  }

  function handleTrackClick(track: TrackRow) {
    playNow(track.id);
  }
  
  function handleTrackQueue(track: TrackRow, e: Event) {
    e.stopPropagation();
    addToQueue(track.id);
  }

  function formatTime(ms?: number) {
    if (!ms) return '—';
    const seconds = Math.floor(ms / 1000);
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  }
</script>

<div class="search-results-view">
  <header>
    <button class="back-btn" onclick={goBack} aria-label="Go back"><ArrowLeft size={18} /></button>
    <h1>Results for "{query}"</h1>
  </header>

  {#if !query}
    <div class="empty-state">Enter a search query</div>
  {:else}
    <!-- ARTISTS SECTION -->
    <section class="section artists-section">
      <h2>Artists ({artists.length}{artistsHasMore ? '+' : ''})</h2>
      <div class="artists-scroll" bind:this={artistsContainer}>
        {#each artists as artist (artist.artistSort)}
          <div 
            class="artist-card"
            role="button"
            tabindex="0"
            onclick={() => handleArtistClick(artist)}
            onkeydown={(e) => e.key === 'Enter' && handleArtistClick(artist)}
          >
            <div class="artist-avatar">
              <!-- Placeholder for artist image -->
              <span>{artist.artistDisplay[0]}</span>
            </div>
            <div class="artist-name" title={artist.artistDisplay}>{artist.artistDisplay}</div>
          </div>
        {/each}
        
        {#if artistsHasMore}
          <div class="loading-trigger-h" use:horizontalScroll={loadMoreArtists}>
            {#if artistsLoading}Loading...{/if}
          </div>
        {/if}
      </div>
    </section>

    <!-- ALBUMS SECTION -->
    <section class="section albums-section">
      <h2>Albums ({albums.length}{albumsHasMore ? '+' : ''})</h2>
      <div class="albums-grid">
        {#each albums as album (album.albumArtistSort + album.albumTitleSort)}
          <div 
            class="album-card"
            role="button"
            tabindex="0"
            onclick={() => handleAlbumClick(album)}
            onkeydown={(e) => e.key === 'Enter' && handleAlbumClick(album)}
          >
            <div class="album-art-container">
              <ArtworkImage
                cacheKey={album.artworkCacheKey}
                artistSort={album.albumArtistSort}
                titleSort={album.albumTitleSort}
                size={256}
                alt="{album.albumTitleDisplay} artwork"
              />
            </div>
            <div class="album-info">
              <div class="album-title" title={album.albumTitleDisplay}>{album.albumTitleDisplay}</div>
              <div class="album-artist" title={album.albumArtistDisplay}>{album.albumArtistDisplay}</div>
              {#if album.year}<div class="album-year">{album.year}</div>{/if}
            </div>
          </div>
        {/each}
      </div>
      
      {#if albumsHasMore}
        <div class="loading-trigger" use:infiniteScroll={loadMoreAlbums}>
          {#if albumsLoading}Loading more albums...{/if}
        </div>
      {/if}
    </section>

    <!-- TRACKS SECTION -->
    <section class="section tracks-section">
      <h2>Tracks ({tracks.length}{tracksHasMore ? '+' : ''})</h2>
      <div class="tracks-list">
        <VList data={tracks} getKey={(t) => t.id} itemSize={60} bufferSize={200}>
          {#snippet children(track)}
            <div 
              class="track-row"
              role="button"
              tabindex="0"
              onclick={() => handleTrackClick(track)}
              onkeydown={(e) => e.key === 'Enter' && handleTrackClick(track)}
            >
              <div class="track-main">
                <div class="track-title">{track.title || 'Unknown Title'}</div>
                <div class="track-details">
                  {track.artist || 'Unknown Artist'} • {track.album || 'Unknown Album'}
                </div>
              </div>
              <div class="track-meta">
                <span class="duration">{formatTime(track.durationMs)}</span>
                <button class="queue-btn" onclick={(e) => handleTrackQueue(track, e)} title="Add to Queue">+</button>
              </div>
            </div>
          {/snippet}
        </VList>
      </div>
      
      {#if tracksHasMore}
        <div class="loading-trigger" use:infiniteScroll={loadMoreTracks}>
          {#if tracksLoading}Loading more tracks...{/if}
        </div>
      {/if}
    </section>
  {/if}
</div>

<style>
  .search-results-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
    padding: 2rem;
    box-sizing: border-box;
    background: transparent;
    color: #fff;
  }

  header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .back-btn {
    background: var(--surface-1);
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    border: 1px solid var(--divider-color);
    color: #ccc;
    font-size: 1rem;
    cursor: pointer;
    padding: 0.5rem 1rem;
    border-radius: 20px;
    transition: all 0.2s;
    box-shadow: var(--shadow-2);
  }

  .back-btn:hover {
    background: var(--divider-color);
    color: #fff;
    transform: translateX(-2px);
  }

  h1 {
    font-size: 24px;
    font-weight: 600;
    margin: 0;
    text-shadow: 0 2px 4px rgba(0,0,0,0.5);
  }

  h2 {
    font-size: 14px;
    color: #aaa;
    margin-bottom: 1rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-bottom: 1px solid var(--divider-color);
    padding-bottom: 0.5rem;
  }

  .section {
    margin-bottom: 3rem;
  }

  /* Artists - Horizontal Scroll */
  .artists-scroll {
    display: flex;
    gap: 1.5rem;
    overflow-x: auto;
    padding-bottom: 1rem;
    scrollbar-width: thin;
  }

  .artist-card {
    flex: 0 0 140px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.8rem;
    cursor: pointer;
    transition: transform 0.2s;
  }

  .artist-card:hover {
    transform: translateY(-4px);
  }

  .artist-avatar {
    width: 120px;
    height: 120px;
    border-radius: 50%;
    background: var(--surface-1);
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    border: 1px solid var(--divider-color);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 2rem;
    font-weight: bold;
    color: #555;
    overflow: hidden;
    box-shadow: var(--shadow-2);
  }

  .artist-name {
    text-align: center;
    font-weight: 500;
    font-size: 14px;
    width: 100%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    text-shadow: 0 1px 2px rgba(0,0,0,0.5);
  }

  /* Albums - Grid */
  .albums-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 1.5rem;
  }

  .album-card {
    background: var(--surface-1);
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    border-radius: var(--radius-md);
    border: 1px solid var(--divider-color);
    box-shadow: var(--shadow-2);
    overflow: hidden;
    cursor: pointer;
    transition: transform 0.2s cubic-bezier(0.25, 0.46, 0.45, 0.94), border-color 0.2s;
  }

  .album-card:hover {
    transform: translateY(-4px);
    border-color: rgba(255,255,255,0.3);
  }

  .album-art-container {
    aspect-ratio: 1;
    background: #222;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    display: block;
    width: 100%;
  }

  .album-info {
    padding: 0.8rem;
  }

  .album-title {
    font-weight: 600;
    margin-bottom: 0.3rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 14px;
    text-shadow: 0 1px 2px rgba(0,0,0,0.5);
  }

  .album-artist {
    font-size: 0.85rem;
    color: #aaa;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .album-year {
    font-size: 0.8rem;
    color: #666;
    margin-top: 0.2rem;
  }

  /* Tracks - List */
  .tracks-list {
    display: flex;
    flex-direction: column;
    /* Remove gap and rely on item padding/margin inside VList if needed, 
       but here track-row has padding and VList handles flow. */
  }

  .track-row {
    display: flex;
    align-items: center;
    padding: 0.8rem;
    background: var(--surface-1);
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    border: 1px solid var(--divider-color);
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.2s, border-color 0.2s;
    margin-bottom: 0.5rem; /* Add margin here for spacing since VList doesn't support gap directly on container */
    box-sizing: border-box;
  }

  .track-row:hover {
    background: var(--surface-hover);
    border-color: rgba(255,255,255,0.3);
  }

  .track-main {
    flex: 1;
    min-width: 0;
  }

  .track-title {
    font-weight: 500;
    margin-bottom: 0.2rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 14px;
    text-shadow: 0 1px 2px rgba(0,0,0,0.5);
  }

  .track-details {
    font-size: 0.85rem;
    color: #888;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-meta {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-left: 1rem;
  }

  .duration {
    font-size: 0.85rem;
    color: #666;
    font-variant-numeric: tabular-nums;
  }

  .queue-btn {
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: #fff;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s, background 0.2s;
  }

  .track-row:hover .queue-btn {
    opacity: 1;
  }

  .queue-btn:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .loading-trigger {
    padding: 2rem;
    text-align: center;
    color: #666;
  }

  .loading-trigger-h {
    min-width: 50px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #666;
  }

  .empty-state {
    text-align: center;
    padding: 4rem;
    color: #666;
    font-size: 1.2rem;
  }
</style>
