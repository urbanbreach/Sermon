<script lang="ts">
  import { currentRouteName, navigate } from '../state/route';
  import { Disc3, Users, ListMusic, Activity, Settings, ChevronDown, ChevronRight, Search, X } from '@lucide/svelte';
  import { pressScale } from '../utils/animations';
  import { currentTrack } from '../state/playback';
  import ArtworkImage from './ArtworkImage.svelte';

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

<nav class="left-nav">
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
              {#if $currentRouteName === item.routeName}
                <div class="active-indicator"></div>
              {/if}
              <item.icon size={18} strokeWidth={1.5} />
              <span>{item.label}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/each}
  
  <div class="nav-spacer"></div>

  {#if $currentTrack}
    <button 
      class="now-playing-widget"
      onclick={() => navigate({ name: 'now-playing' })}
      use:pressScale={{ scale: 0.98 }}
    >
      {#if ($currentTrack as any)?.artworkCacheKey}
        <ArtworkImage 
          cacheKey={($currentTrack as any)?.artworkCacheKey} 
          size={128} 
          class="np-widget-art" 
        />
      {:else}
        <div class="np-widget-art-placeholder">
          <Disc3 size={20} strokeWidth={1.5} />
        </div>
      {/if}
      <div class="np-widget-info">
        <span class="np-widget-title">{$currentTrack.title || '—'}</span>
        <span class="np-widget-artist">{$currentTrack.artist || '—'}</span>
      </div>
    </button>
  {/if}
</nav>

<style>
  .left-nav {
    display: flex;
    flex-direction: column;
    width: var(--layout-sidebar-width, 240px);
    /* Integrated look: matte background */
    background: var(--surface-1);
    border-right: 1px solid var(--divider-color);
    padding: 12px 12px 12px 12px;
    height: 100%;
    box-sizing: border-box;
    gap: 16px;
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
    margin-bottom: 4px;
  }

  .search-bar {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    padding: 0 10px;
    background: var(--surface-1);
    border: 1px solid var(--divider-color);
    border-radius: 8px;
    color: var(--text-tertiary);
    cursor: text;
    transition: all var(--motion-fast) var(--ease-out);
    font-size: 13px;
    box-sizing: border-box;
  }

  .search-bar:hover {
    background: var(--surface-2);
    border-color: rgba(255, 255, 255, 0.12);
  }

  .search-bar.focused {
    border-color: var(--focus-ring);
    background: var(--surface-2);
    box-shadow: 0 0 0 2px var(--focus-ring);
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
    padding: 2px 5px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 4px;
    font-family: system-ui;
    flex-shrink: 0;
    font-weight: 500;
  }

  /* Navigation Sections */
  .nav-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 600;
    color: var(--text-tertiary);
    padding: 0 12px;
    margin-bottom: 4px;
    background: transparent;
    border: none;
    cursor: pointer;
    transition: color var(--motion-fast) var(--ease-out);
    -webkit-app-region: no-drag;
    height: 24px;
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
    position: relative;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    text-align: left;
    padding: 0 12px 0 16px; /* Extra left padding for indicator space if needed, but we use absolute */
    height: 36px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    border-radius: 6px;
    transition: all var(--motion-fast) var(--ease-out);
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    -webkit-app-region: no-drag;
    overflow: hidden;
  }

  .nav-item:hover:not(.active) {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: var(--surface-2);
    color: var(--text-primary);
    font-weight: 600;
  }

  .active-indicator {
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 20px;
    background-color: var(--text-primary);
    border-radius: 0 4px 4px 0;
  }

  .nav-item :global(svg) {
    transition: stroke var(--motion-fast) var(--ease-out);
    flex-shrink: 0;
  }

  .nav-spacer {
    flex: 1;
  }

  .now-playing-widget {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px;
    background: var(--surface-1);
    border: 1px solid var(--divider-color);
    border-radius: 10px;
    cursor: pointer;
    transition: all var(--motion-fast) var(--ease-out);
    -webkit-app-region: no-drag;
    flex-shrink: 0;
    text-align: left;
    width: 100%;
    box-sizing: border-box;
    margin-top: auto;
  }

  .now-playing-widget:hover {
    background: var(--surface-2);
    border-color: rgba(255, 255, 255, 0.12);
    transform: translateY(-1px);
    box-shadow: var(--shadow-2);
  }

  :global(.np-widget-art) {
    width: 40px;
    height: 40px;
    border-radius: var(--artwork-radius-sidebar, 6px);
    object-fit: cover;
    flex-shrink: 0;
    box-shadow: var(--shadow-1);
  }

  .np-widget-art-placeholder {
    width: 40px;
    height: 40px;
    border-radius: var(--artwork-radius-sidebar, 6px);
    background: var(--surface-2);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary);
    flex-shrink: 0;
    border: 1px solid var(--divider-color);
  }

  .np-widget-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .np-widget-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .np-widget-artist {
    font-size: 11px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

</style>
