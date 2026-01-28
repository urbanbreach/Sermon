<script lang="ts">
  import { canGoBack, canGoForward, goBack, goForward, navigate, currentRouteName } from '../state/route';
  import { toggleRail, isRailOpen } from '../state/rightRail';
  import { viewTitle } from '../state/viewTitle';
  import { sidebarVisible } from '../state/effects';
  import { alphabetSelector } from '../state/alphabetSelector';
  import { pressScale } from '../utils/animations';
  import { ChevronLeft, ChevronRight, PanelRight, Disc3, Users, ListMusic, Activity, Settings, Search } from '@lucide/svelte';
  import AlphabetSelector from './AlphabetSelector.svelte';

  type SimpleRouteName = 'albums' | 'artists' | 'tracks' | 'diagnostics' | 'preferences';

  const navItems = [
    { label: 'Albums', routeName: 'albums' as SimpleRouteName, icon: Disc3 },
    { label: 'Artists', routeName: 'artists' as SimpleRouteName, icon: Users },
    { label: 'Tracks', routeName: 'tracks' as SimpleRouteName, icon: ListMusic },
  ];

  const systemItems = [
    { label: 'Diagnostics', routeName: 'diagnostics' as SimpleRouteName, icon: Activity },
    { label: 'Settings', routeName: 'preferences' as SimpleRouteName, icon: Settings },
  ];

  function handleNavigate(routeName: SimpleRouteName) {
    navigate({ name: routeName });
  }
</script>

<div class="top-bar">
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

    {#if !$sidebarVisible}
      <div class="top-nav-divider"></div>
      {#each navItems as item}
        <button 
          class="top-nav-item"
          class:active={$currentRouteName === item.routeName}
          onclick={() => handleNavigate(item.routeName)}
          use:pressScale
        >
          <item.icon size={16} strokeWidth={1.5} />
          <span>{item.label}</span>
        </button>
      {/each}
    {/if}
  </div>

  {#if $viewTitle && $sidebarVisible}
    <h1 class="view-title">{$viewTitle}</h1>
  {/if}

  {#if $alphabetSelector.items.length > 0 && $alphabetSelector.onSelect}
    <div class="alphabet-wrapper">
      <AlphabetSelector items={$alphabetSelector.items} onSelect={$alphabetSelector.onSelect} />
    </div>
  {/if}

  <div class="right-controls">
    {#if !$sidebarVisible}
      {#each systemItems as item}
        <button 
          class="top-nav-item system-item"
          class:active={$currentRouteName === item.routeName}
          onclick={() => handleNavigate(item.routeName)}
          title={item.label}
          use:pressScale
        >
          <item.icon size={16} strokeWidth={1.5} />
        </button>
      {/each}
      <div class="top-nav-divider"></div>
    {/if}
    <button 
      class="nav-btn rail-toggle" 
      class:active={$isRailOpen}
      onclick={toggleRail} 
      title="Toggle Queue" 
      use:pressScale
    >
      <PanelRight size={20} strokeWidth={1.5} />
    </button>
  </div>
</div>

<style>
  .top-bar {
    height: var(--layout-header-height);
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border-bottom: 1px solid var(--glass-border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    -webkit-app-region: drag;
    user-select: none;
    position: relative;
    z-index: 100;
  }



  .nav-buttons {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    -webkit-app-region: no-drag;
  }

  .view-title {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
    white-space: nowrap;
    pointer-events: none;
  }

  .alphabet-wrapper {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    -webkit-app-region: no-drag;
    pointer-events: auto;
  }

  .nav-btn {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    background: var(--surface-1);
    border: 1px solid var(--glass-border);
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--motion-fast) var(--ease-out);
    padding: 0;
    -webkit-app-region: no-drag;
    box-shadow: var(--shadow-1);
  }

  .nav-btn:hover:not(:disabled) {
    background: var(--surface-hover);
    border-color: rgba(255, 255, 255, 0.12);
    box-shadow: var(--shadow-2);
    color: var(--text-primary);
  }

  .nav-btn:active:not(:disabled) {
    background: var(--surface-active);
    box-shadow: var(--shadow-inset);
    border-color: rgba(255, 255, 255, 0.08);
    transform: scale(0.97);
  }

  .nav-btn.active {
    background: var(--accent-weak);
    color: var(--theme-accent);
    border-color: var(--accent-medium);
    box-shadow: var(--shadow-glow);
  }

  .nav-btn:disabled {
    color: var(--text-disabled);
    cursor: default;
    background: rgba(255, 255, 255, 0.02);
    border-color: transparent;
    opacity: 0.6;
    box-shadow: none;
  }

  .right-controls {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    -webkit-app-region: no-drag;
  }

  .top-nav-divider {
    width: 1px;
    height: 20px;
    background: var(--glass-border);
    margin: 0 8px;
  }

  .top-nav-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--motion-fast) var(--ease-out);
    -webkit-app-region: no-drag;
  }

  .top-nav-item:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .top-nav-item.active {
    color: var(--theme-accent);
    background: var(--accent-weak);
  }

  .top-nav-item.system-item {
    padding: 6px 8px;
  }

  .top-nav-item.system-item span {
    display: none;
  }
</style>
