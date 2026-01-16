<script lang="ts">
  import { currentRoute, type Route } from '../state/route';

  const navItems: { label: string; route: Route }[] = [
    { label: 'Albums', route: 'albums' },
    { label: 'Artists', route: 'artists' },
    { label: 'Tracks', route: 'tracks' },
  ];

  function navigate(route: Route) {
    currentRoute.set(route);
  }
</script>

<nav class="left-nav">
  <div class="nav-section">
    {#each navItems as item}
      <button 
        class:active={$currentRoute === item.route}
        on:click={() => navigate(item.route)}
      >
        {item.label}
      </button>
    {/each}
  </div>

  <div class="nav-section settings">
    <button 
      class:active={$currentRoute === 'settings'}
      on:click={() => navigate('settings')}
    >
      Settings
    </button>
    <button 
      class:active={$currentRoute === 'diagnostics'}
      on:click={() => navigate('diagnostics')}
    >
      Diagnostics
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
