<script lang="ts">
  import { currentRouteName, navigate } from '../state/route';
  import { Disc3, Users, ListMusic, Activity, Settings, ChevronDown, ChevronRight, Search, X } from '@lucide/svelte';
  import { pressScale } from '../utils/animations';

  type SimpleRouteName = 'albums' | 'artists' | 'tracks' | 'diagnostics' | 'preferences';

  // Collapsible section state
  let expandedSections = $state<Record<string, boolean>>({
    'Library': true,
    'System': true
  });

  const sections = [
    {
      header: 'Library',
      collapsible: true,
      items: [
        { label: 'Albums', routeName: 'albums' as SimpleRouteName, icon: Disc3 },
        { label: 'Artists', routeName: 'artists' as SimpleRouteName, icon: Users },
        { label: 'Tracks', routeName: 'tracks' as SimpleRouteName, icon: ListMusic },
      ]
    },
    {
      header: 'System',
      collapsible: true,
      items: [
        { label: 'Diagnostics', routeName: 'diagnostics' as SimpleRouteName, icon: Activity },
        { label: 'Preferences', routeName: 'preferences' as SimpleRouteName, icon: Settings },
      ]
    }
  ];

  function handleNavigate(routeName: SimpleRouteName) {
    navigate({ name: routeName });
  }

  function toggleSection(header: string) {
    expandedSections[header] = !expandedSections[header];
  }

  let searchQuery = $state('');
  let searchFocused = $state(false);

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && searchQuery.trim()) {
      navigate({ name: 'search-results', query: searchQuery.trim() });
    }
    if (e.key === 'Escape') {
      searchQuery = '';
      (e.target as HTMLInputElement).blur();
    }
  }

  function clearSearch() {
    searchQuery = '';
  }
</script>

<nav class="left-nav" data-testid="glass-panel">
  <!-- Cider-style Search Bar -->
  <div class="search-container">
    <div class="search-bar" class:focused={searchFocused}>
      <Search size={14} strokeWidth={1.5} />
      <input
        type="text"
        placeholder="Search"
        bind:value={searchQuery}
        onfocus={() => searchFocused = true}
        onblur={() => searchFocused = false}
        onkeydown={handleSearchKeydown}
      />
      {#if searchQuery}
        <button class="clear-btn" onclick={clearSearch}>
          <X size={14} />
        </button>
      {/if}
      <span class="shortcut-hint">⌘K</span>
    </div>
  </div>

  <!-- Navigation Sections -->
  {#each sections as section}
    <div class="nav-section">
      <button 
        class="section-header"
        onclick={() => toggleSection(section.header)}
        class:collapsed={!expandedSections[section.header]}
      >
        <span>{section.header}</span>
        {#if section.collapsible}
          <span class="chevron">
            {#if expandedSections[section.header]}
              <ChevronDown size={14} strokeWidth={2} />
            {:else}
              <ChevronRight size={14} strokeWidth={2} />
            {/if}
          </span>
        {/if}
      </button>
      
      {#if expandedSections[section.header]}
        <div class="section-items">
          {#each section.items as item}
            <button 
              class="nav-item"
              class:active={$currentRouteName === item.routeName}
              onclick={() => handleNavigate(item.routeName)}
              use:pressScale
            >
              <item.icon size={18} strokeWidth={1.5} />
              <span>{item.label}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/each}
  
</nav>

<style>
  .left-nav {
    display: flex;
    flex-direction: column;
    width: var(--layout-sidebar-width, 250px);
    /* Integrated look: transparent background, no independent glass effect */
    background: transparent;
    /* No backdrop-filter - unified with window background */
    padding: 12px 12px 12px 12px;
    height: 100%;
    box-sizing: border-box;
    gap: 12px;
    user-select: none;
    box-shadow: none;
  }

  /* Search container - fits within sidebar padding */
  .search-container {
    display: flex;
    align-items: center;
    padding: 0;
    -webkit-app-region: no-drag;
    flex-shrink: 0;
    box-sizing: border-box;
  }

  .search-bar {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 10px;
    background: var(--surface-1);
    border: 1px solid var(--glass-border);
    border-radius: 6px;
    color: var(--text-tertiary);
    cursor: text;
    transition: all var(--motion-fast) var(--ease-out);
    font-size: 12px;
    box-sizing: border-box;
  }

  .search-bar:hover {
    background: var(--surface-2);
    border-color: rgba(255, 255, 255, 0.12);
  }

  .search-bar.focused {
    border-color: var(--accent-medium);
    background: var(--surface-2);
    box-shadow: var(--focus-ring);
  }

  .search-bar input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-size: 13px;
    font-family: inherit;
    min-width: 0;
  }

  .search-bar input::placeholder {
    color: var(--text-tertiary);
  }

  .clear-btn {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    padding: 2px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    transition: all var(--motion-fast) var(--ease-out);
    flex-shrink: 0;
  }

  .clear-btn:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .shortcut-hint {
    font-size: 10px;
    color: var(--text-disabled);
    padding: 2px 4px;
    background: var(--surface-1);
    border-radius: 4px;
    font-family: system-ui;
    flex-shrink: 0;
  }

  /* Navigation Sections */
  .nav-section {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-tertiary);
    padding: 8px 12px;
    margin-bottom: 2px;
    background: transparent;
    border: none;
    cursor: pointer;
    transition: color var(--motion-fast) var(--ease-out);
    -webkit-app-region: no-drag;
  }

  .section-header:hover {
    color: var(--text-secondary);
  }

  .section-header .chevron {
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.6;
    transition: transform var(--motion-fast) var(--ease-out), opacity var(--motion-fast) var(--ease-out);
  }

  .section-header.collapsed .chevron {
    opacity: 0.4;
  }

  .section-items {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    background: transparent;
    border: none;
    color: var(--text-primary);
    text-align: left;
    padding: 0 12px;
    height: 36px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    border-radius: 8px;
    transition: all var(--motion-fast) var(--ease-out);
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    -webkit-app-region: no-drag;
  }

  .nav-item:hover:not(.active) {
    background: var(--surface-hover);
  }

  .nav-item.active {
    background: var(--accent-weak);
    color: var(--theme-accent);
    font-weight: 500;
    border: 1px solid var(--accent-medium);
    box-shadow: var(--shadow-1);
  }

  .nav-item :global(svg) {
    transition: stroke var(--motion-fast) var(--ease-out);
    flex-shrink: 0;
  }

</style>
