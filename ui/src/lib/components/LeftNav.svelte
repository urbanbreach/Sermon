<script lang="ts">
  import { currentRoute, currentRouteName, navigate } from '../state/route';

  type SimpleRouteName = 'albums' | 'artists' | 'tracks' | 'settings' | 'diagnostics' | 'preferences';

  const navItems: { label: string; routeName: SimpleRouteName }[] = [
    { label: 'Albums', routeName: 'albums' },
    { label: 'Artists', routeName: 'artists' },
    { label: 'Tracks', routeName: 'tracks' },
  ];

  function handleNavigate(routeName: SimpleRouteName) {
    navigate({ name: routeName });
  }
</script>

<nav class="left-nav">
  <div class="nav-section">
    {#each navItems as item}
      <button 
        class:active={$currentRouteName === item.routeName}
        onclick={() => handleNavigate(item.routeName)}
      >
        {item.label}
      </button>
    {/each}
  </div>

  <div class="nav-section settings">
    <button 
      class:active={$currentRouteName === 'settings'}
      onclick={() => handleNavigate('settings')}
    >
      Settings
    </button>
    <button 
      class:active={$currentRouteName === 'diagnostics'}
      onclick={() => handleNavigate('diagnostics')}
    >
      Diagnostics
    </button>
    <button 
      class:active={$currentRouteName === 'preferences'}
      onclick={() => handleNavigate('preferences')}
    >
      Preferences
    </button>
  </div>
</nav>

<style>
  .left-nav {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    width: 200px;
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
    padding: 0.5rem 1rem;
    cursor: pointer;
    font-size: 1rem;
    border-radius: 4px;
    transition: color 0.2s, background 0.2s;
  }

  button:hover {
    color: #fff;
    background: #222;
  }

  button.active {
    color: #fff;
    background: #333;
    font-weight: bold;
  }
</style>
