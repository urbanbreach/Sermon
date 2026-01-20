<script lang="ts">
  import { currentRouteName, navigate } from '../state/route';
  import { Disc3, Users, ListMusic, Activity, Settings } from '@lucide/svelte';

  type SimpleRouteName = 'albums' | 'artists' | 'tracks' | 'diagnostics' | 'preferences';

  const navItems = [
    { label: 'Albums', routeName: 'albums' as SimpleRouteName, icon: Disc3 },
    { label: 'Artists', routeName: 'artists' as SimpleRouteName, icon: Users },
    { label: 'Tracks', routeName: 'tracks' as SimpleRouteName, icon: ListMusic },
  ];

  function handleNavigate(routeName: SimpleRouteName) {
    navigate({ name: routeName });
  }
</script>

<nav class="left-nav" data-testid="glass-panel">
  <div class="nav-section">
    {#each navItems as item}
      <button 
        class:active={$currentRouteName === item.routeName}
        onclick={() => handleNavigate(item.routeName)}
      >
        <item.icon size={20} />
        <span>{item.label}</span>
      </button>
    {/each}
  </div>

  <div class="nav-section settings">
    <button 
      class:active={$currentRouteName === 'diagnostics'}
      onclick={() => handleNavigate('diagnostics')}
    >
      <Activity size={20} />
      <span>Diagnostics</span>
    </button>
    <button 
      class:active={$currentRouteName === 'preferences'}
      onclick={() => handleNavigate('preferences')}
    >
      <Settings size={20} />
      <span>Preferences</span>
    </button>
  </div>
</nav>

<style>
  .left-nav {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    width: var(--layout-sidebar-width);
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    border-right: 1px solid var(--glass-border);
    padding: 1rem;
    height: 100%;
    box-sizing: border-box;
  }

  .nav-section {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  button {
    background: transparent;
    border: none;
    color: #888;
    text-align: left;
    padding: 0.75rem 1rem;
    cursor: pointer;
    font-size: 1rem;
    border-radius: 8px;
    transition: color 0.2s, background 0.2s;
    display: flex;
    align-items: center;
    gap: 12px;
  }

  button:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.05);
  }

  button.active {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
    font-weight: 500;
  }
</style>
