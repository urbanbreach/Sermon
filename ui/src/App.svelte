<script lang="ts">
  import { onMount } from 'svelte';
  import { emit } from '@tauri-apps/api/event';
  import { currentRoute, currentRouteName } from './lib/state/route';
  import { initPlaybackListeners } from './lib/state/playback';
  
  import TopBar from './lib/components/TopBar.svelte';
  import LeftNav from './lib/components/LeftNav.svelte';
  import BottomBar from './lib/components/BottomBar.svelte';
  
  import AlbumsView from './lib/views/AlbumsView.svelte';
  import AlbumDetailView from './lib/views/AlbumDetailView.svelte';
  import ArtistsView from './lib/views/ArtistsView.svelte';
  import ArtistDetailView from './lib/views/ArtistDetailView.svelte';
  import TracksView from './lib/views/TracksView.svelte';
  import SettingsView from './lib/views/SettingsView.svelte';
  import DiagnosticsView from './lib/views/DiagnosticsView.svelte';
  import PreferencesView from './lib/views/PreferencesView.svelte';
  import NowPlayingView from './lib/views/NowPlayingView.svelte';
  import SearchResultsView from './lib/views/SearchResultsView.svelte';

  onMount(async () => {
    initPlaybackListeners();

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
  });
</script>

<div class="app-shell">
  <TopBar />
  
  <div class="main-body">
    <LeftNav />
    
    <main class="content-area">
      {#if $currentRouteName === 'albums'}
        <AlbumsView />
      {:else if $currentRouteName === 'artists'}
        <ArtistsView />
      {:else if $currentRouteName === 'tracks'}
        <TracksView />
      {:else if $currentRouteName === 'settings'}
        <SettingsView />
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
    </main>
  </div>
  
  <BottomBar />
</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background: #0a0a0a;
    color: #fff;
    overflow: hidden;
  }

  .main-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .content-area {
    flex: 1;
    position: relative; /* For NowPlaying overlay if needed */
    background: var(--glass-bg); /* Use glass bg for consistency, or keep opaque if intended */
    overflow: hidden;
  }
</style>
