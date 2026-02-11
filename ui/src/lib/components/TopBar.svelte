<script lang="ts">
  import { canGoBack, canGoForward, goBack, goForward, navigate, currentRouteName } from '../state/route';
  import { toggleRail, isRailOpen } from '../state/rightRail';
  import { viewTitle } from '../state/viewTitle';
  import { sidebarVisible, reduceEffects } from '../state/effects';
  import { alphabetSelector } from '../state/alphabetSelector';
  import { pressScale } from '../utils/animations';
  import { ChevronLeft, ChevronRight, PanelRight, Disc3, Users, ListMusic, Settings, Music2 } from '@lucide/svelte';
  import AlphabetSelector from './AlphabetSelector.svelte';
  import { SegmentedControl } from './primitives';
  import WindowControls from './WindowControls.svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  const appWindow = getCurrentWindow();

  async function handleDoubleClick() {
    await appWindow.toggleMaximize();
  }

  function handleTopBarKeydown(event: KeyboardEvent) {
    if (event.target !== event.currentTarget) {
      return;
    }

    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      void handleDoubleClick();
    }
  }

  type SimpleRouteName = 'albums' | 'artists' | 'tracks' | 'preferences' | 'now-playing';

  const navItems = [
    { label: 'Albums', routeName: 'albums' as SimpleRouteName, icon: Disc3 },
    { label: 'Artists', routeName: 'artists' as SimpleRouteName, icon: Users },
    { label: 'Tracks', routeName: 'tracks' as SimpleRouteName, icon: ListMusic },
    { label: 'Now Playing', routeName: 'now-playing' as SimpleRouteName, icon: Music2 },
  ];

  const systemItems = [
    { label: 'Settings', routeName: 'preferences' as SimpleRouteName, icon: Settings },
  ];

  function handleNavigate(routeName: string) {
    navigate({ name: routeName as SimpleRouteName });
  }

  // Transform navItems for SegmentedControl
  const segmentedItems = navItems.map(item => ({
    id: item.routeName,
    label: item.label,
    icon: item.icon
  }));
</script>

<div class="top-bar" class:np-transparent={$currentRouteName === 'now-playing' && !$reduceEffects} class:np-matte={$currentRouteName === 'now-playing' && $reduceEffects} role="button" tabindex="0" aria-label="Application top bar" ondblclick={handleDoubleClick} onkeydown={handleTopBarKeydown}>
  <!-- Left Region: Navigation & Tabs -->
  <div class="region-left">
    <div class="nav-buttons">
      <button 
        class="nav-btn" 
        onclick={goBack} 
        disabled={!$canGoBack} 
        title="Go Back" 
        use:pressScale
      >
        <ChevronLeft size={20} strokeWidth={2} />
      </button>
      <button 
        class="nav-btn" 
        onclick={goForward} 
        disabled={!$canGoForward} 
        title="Go Forward" 
        use:pressScale
      >
        <ChevronRight size={20} strokeWidth={2} />
      </button>
    </div>

    {#if !$sidebarVisible}
      <div class="segmented-nav">
        <SegmentedControl 
          items={segmentedItems} 
          value={$currentRouteName} 
          onchange={handleNavigate} 
        />
      </div>
    {/if}
  </div>

  <!-- Center Region: title/alphabet -->
  <div class="region-center">
    {#if $alphabetSelector.items.length > 0 && $alphabetSelector.onSelect}
      <div class="alphabet-wrapper" data-testid="topbar-alphabet">
        <AlphabetSelector items={$alphabetSelector.items} onSelect={$alphabetSelector.onSelect} />
      </div>
    {:else if $viewTitle && $sidebarVisible}
      <h1 class="view-title" data-testid="topbar-title">{$viewTitle}</h1>
    {/if}
  </div>

  <!-- Right Region: Status Cluster -->
  <div class="region-right">
    {#if !$sidebarVisible}
      <div class="system-controls">
        {#each systemItems as item (item.routeName)}
          <button 
            class="nav-btn system-btn"
            class:active={$currentRouteName === item.routeName}
            onclick={() => handleNavigate(item.routeName)}
            title={item.label}
            use:pressScale
          >
            <item.icon size={18} strokeWidth={1.5} />
          </button>
        {/each}
      </div>
      <div class="divider"></div>
    {/if}

    {#if $currentRouteName !== 'now-playing'}
      <button 
        class="nav-btn rail-toggle" 
        class:active={$isRailOpen}
        onclick={toggleRail} 
        title="Toggle Queue" 
        use:pressScale
      >
        <PanelRight size={20} strokeWidth={1.5} />
      </button>
    {/if}

    <WindowControls />
  </div>
</div>

<style>
  .top-bar {
    height: 44px; /* Fixed height as per spec */
    background: var(--surface-header, rgba(18, 18, 18, 0.95));
    border-bottom: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding: 0 0 0 16px; /* No right padding for window controls */
    -webkit-app-region: drag;
    user-select: none;
    position: relative;
    z-index: 100;
    gap: 16px;
  }

  /* Regions */
  .region-left {
    display: flex;
    align-items: center;
    gap: 16px;
    justify-content: flex-start;
    min-width: 0; /* Allow shrinking */
  }

  .region-center {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 0;
  }

  .region-right {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 12px;
    height: 100%;
  }

  /* Navigation Buttons */
  .nav-buttons {
    display: flex;
    align-items: center;
    gap: 8px;
    -webkit-app-region: no-drag;
  }

  .segmented-nav {
    -webkit-app-region: no-drag;
  }

  /* Title & Alphabet */
  .view-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .alphabet-wrapper {
    -webkit-app-region: no-drag;
    pointer-events: auto;
  }

  /* Buttons */
  .nav-btn {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    background: transparent;
    border: 1px solid transparent;
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--motion-fast) var(--ease-out);
    padding: 0;
    -webkit-app-region: no-drag;
  }

  .nav-btn:hover:not(:disabled) {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  .nav-btn:active:not(:disabled) {
    background: var(--surface-active);
    transform: scale(0.97);
  }

  .nav-btn.active {
    background: var(--accent-weak);
    color: var(--theme-accent);
  }

  .nav-btn:disabled {
    color: var(--text-disabled);
    cursor: default;
    opacity: 0.5;
  }

  /* System Controls */
  .system-controls {
    display: flex;
    align-items: center;
    gap: 4px;
    -webkit-app-region: no-drag;
  }

  .divider {
    width: 1px;
    height: 16px;
    background: var(--divider-color, rgba(255, 255, 255, 0.07));
  }

  /* Now Playing transparent blur mode */
  .top-bar.np-transparent {
    background: rgba(0, 0, 0, 0.2);
    backdrop-filter: blur(20px) saturate(1.2);
    -webkit-backdrop-filter: blur(20px) saturate(1.2);
    border-bottom-color: rgba(255, 255, 255, 0.05);
  }

  /* Now Playing matte fallback (reduce effects) */
  .top-bar.np-matte {
    background: rgba(18, 18, 18, 0.85);
    border-bottom-color: rgba(255, 255, 255, 0.05);
  }

</style>
