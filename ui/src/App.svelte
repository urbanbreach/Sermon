<script lang="ts">
import { onMount } from 'svelte';
  import { emit } from '@tauri-apps/api/event';
  import { currentRouteName } from './lib/state/route';
  import { initPlaybackListeners, restorePlaybackSession } from './lib/state/playback';
  import { initArtworkStore } from './lib/state/artwork';
  import { initRailResponsive, isRailOpen } from './lib/state/rightRail';
  import { loadEffectsSettings, sidebarVisible } from './lib/state/effects';

  // Suppress benign ResizeObserver loop errors (common with virtualized lists)
  if (typeof window !== 'undefined') {
    const resizeObserverErr = (e: ErrorEvent) => {
      if (e.message?.includes('ResizeObserver loop')) {
        e.stopPropagation();
        e.stopImmediatePropagation();
      }
    };
    window.addEventListener('error', resizeObserverErr);
  }
  
  import TopBar from './lib/components/TopBar.svelte';
  import BackgroundLayer from './lib/components/BackgroundLayer.svelte';
  import LeftNav from './lib/components/LeftNav.svelte';
  import BottomBar from './lib/components/BottomBar.svelte';
  
  import AlbumsView from './lib/views/AlbumsView.svelte';
  import AlbumDetailView from './lib/views/AlbumDetailView.svelte';
  import ArtistsView from './lib/views/ArtistsView.svelte';
  import ArtistDetailView from './lib/views/ArtistDetailView.svelte';
  import TracksView from './lib/views/TracksView.svelte';
  
  import DiagnosticsView from './lib/views/DiagnosticsView.svelte';
  import PreferencesView from './lib/views/PreferencesView.svelte';
  import NowPlayingView from './lib/views/NowPlayingView.svelte';
  import SearchResultsView from './lib/views/SearchResultsView.svelte';
  import LyricsView from './lib/views/LyricsView.svelte';

  import RightRail from './lib/components/RightRail.svelte';
  import ResizeHandle from './lib/components/ResizeHandle.svelte';

onMount(async () => {
    initPlaybackListeners();
    initArtworkStore();
    initRailResponsive();
    await loadEffectsSettings();

    // Snapshot mode: disable transitions
    if (import.meta.env.SERMON_SNAPSHOT === '1') {
      document.body.classList.add('snapshot-mode');
    }

    // First interactive
    try {
      await emit('sermon://first-interactive');
      console.log('first_interactive');
    } catch (e) {
      console.warn('Failed to emit first-interactive:', e);
      // Still log for browser dev
      console.log('first_interactive');
    }

    // Restore last playback session (always paused, never autoplay)
    try {
      await restorePlaybackSession();
    } catch (e) {
      console.warn('Failed to restore playback session:', e);
    }
  });

  $effect(() => {
    document.documentElement.style.setProperty('--layout-rail-open', $isRailOpen ? '1' : '0');
  });
</script>

<div class="app-shell">
  <BackgroundLayer />
  
  <!-- Fullscreen Lyrics (renders above everything when active) -->
  {#if $currentRouteName === 'lyrics-fullscreen'}
    <LyricsView />
  {:else}
    <TopBar />
    
    <div class="main-body">
      {#if $sidebarVisible}
        <LeftNav />
        <ResizeHandle side="left" minWidth={180} maxWidth={450} />
      {/if}
      
      <main class="content-area">
        <div class="content-row">
          <div class="view-viewport">
            {#if $currentRouteName === 'albums'}
              <AlbumsView />
            {:else if $currentRouteName === 'artists'}
              <ArtistsView />
            {:else if $currentRouteName === 'tracks'}
              <TracksView />
            {:else if $currentRouteName === 'diagnostics'}
              <DiagnosticsView />
            {:else if $currentRouteName === 'preferences'}
              <PreferencesView />
            {:else if $currentRouteName === 'album-detail'}
              <AlbumDetailView />
            {:else if $currentRouteName === 'artist-detail'}
              <ArtistDetailView />
            {:else if $currentRouteName === 'search-results'}
              <SearchResultsView />
            {/if}
            
            {#if $currentRouteName === 'now-playing'}
              <NowPlayingView />
            {/if}
</div>

          <ResizeHandle side="right" minWidth={280} maxWidth={500} />
          <RightRail />
        </div>
      </main>
    </div>
    
    <BottomBar />
  {/if}
</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background: transparent;
    color: #fff;
    overflow: hidden;
    position: relative; /* Ensure stacking context */
    z-index: 1;
    font-family: 'Inter Variable', Inter, sans-serif;
  }

  .main-body {
    display: flex;
    flex: 1;
    overflow: hidden;
    position: relative;
    z-index: 10;
    margin-bottom: var(--layout-player-height, 88px);
  }

  .content-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    position: relative;
    background: transparent;
    overflow: hidden;
  }

  .content-row {
    flex: 1;
    display: flex;
    overflow: hidden;
    position: relative;
  }

  .view-viewport {
    flex: 1;
    position: relative;
    overflow: clip;
    overflow-clip-margin: content-box 6px; /* Buffer for hover effects that may extend beyond content */
    display: flex;
    flex-direction: column;
  }

  /* Cider-style vertical dividers between columns */
  .divider-v {
    width: 1px;
    background: var(--divider-color, rgba(255, 255, 255, 0.07));
    flex-shrink: 0;
    align-self: stretch;
  }

</style>
