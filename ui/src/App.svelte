<script lang="ts">
  import { onMount } from 'svelte';
  import { emit } from '@tauri-apps/api/event';
  import { currentRoute } from './lib/state/route';
  
  import TopBar from './lib/components/TopBar.svelte';
  import LeftNav from './lib/components/LeftNav.svelte';
  import BottomBar from './lib/components/BottomBar.svelte';
  
  import AlbumsView from './lib/views/AlbumsView.svelte';
  import ArtistsView from './lib/views/ArtistsView.svelte';
  import TracksView from './lib/views/TracksView.svelte';
  import SettingsView from './lib/views/SettingsView.svelte';
  import NowPlayingView from './lib/views/NowPlayingView.svelte';

  onMount(async () => {
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
      {#if $currentRoute === 'albums'}
        <AlbumsView />
      {:else if $currentRoute === 'artists'}
        <ArtistsView />
      {:else if $currentRoute === 'tracks'}
        <TracksView />
      {:else if $currentRoute === 'settings'}
        <SettingsView />
      {/if}
      
      {#if $currentRoute === 'now-playing'}
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
