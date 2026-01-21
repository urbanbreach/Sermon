<script lang="ts">
  import { currentRouteName, navigate } from '../state/route';
  import { Disc3, Users, ListMusic, Activity, Settings, ChevronDown, ChevronRight, Search, Music } from 'lucide-svelte';
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
</script>

<nav class="left-nav" data-testid="glass-panel">
  <!-- Cider-style Search Bar -->
  <div class="search-container">
    <div class="search-bar">
      <span class="search-placeholder">Search</span>
      <Search size={16} strokeWidth={1.5} />
    </div>
    <button class="music-btn" title="Browse Music">
      <Music size={18} strokeWidth={1.5} />
    </button>
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
              use:pressScale={{ scale: 0.98 }}
            >
              <item.icon size={18} strokeWidth={1.5} />
              <span>{item.label}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/each}
  
  <!-- Spacer to push profile to bottom -->
  <div class="nav-spacer"></div>
  
  <!-- User Profile Footer (Cider-style) -->
  <div class="profile-footer">
    <div class="profile-avatar">
      <span>S</span>
    </div>
    <div class="profile-info">
      <span class="profile-name">Sermon User</span>
    </div>
  </div>
</nav>

<style>
  .left-nav {
    display: flex;
    flex-direction: column;
    width: var(--layout-sidebar-width, 250px);
    background: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(40px);
    -webkit-backdrop-filter: blur(40px);
    /* Remove border-right - using divider element instead */
    padding: 12px 10px;
    height: 100%;
    box-sizing: border-box;
    gap: 16px;
    user-select: none;
    /* Remove card-like shadow, keep subtle inner highlight */
    box-shadow: none;
  }

  /* Cider-style Search Bar */
  .search-container {
    display: flex;
    gap: 8px;
    padding: 0 4px;
    -webkit-app-region: no-drag;
  }

  .search-bar {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    height: 34px;
    padding: 0 12px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: var(--text-tertiary);
    cursor: text;
    transition: all var(--motion-fast) var(--ease-standard);
  }

  .search-bar:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .search-placeholder {
    font-size: 13px;
    font-weight: 400;
    color: #AAAAAA;
  }

  .music-btn {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--glass-bg-light);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 50%;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--motion-fast) var(--ease-standard);
  }

  .music-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
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
    font-size: 11px;
    font-weight: 600;
    color: #CCCCCC;
    padding: 6px 12px;
    margin-bottom: 2px;
    background: transparent;
    border: none;
    cursor: pointer;
    transition: color var(--motion-fast) var(--ease-standard);
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
    transition: transform var(--motion-fast) var(--ease-standard);
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
    color: var(--text-secondary);
    text-align: left;
    padding: 8px 12px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    border-radius: 6px;
    transition: all var(--motion-fast) var(--ease-standard);
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    -webkit-app-region: no-drag;
  }

  .nav-item:hover {
    background: rgba(255, 255, 255, 0.06);
  }

  .nav-item.active {
    background: #9C2737;
    color: #FFFFFF;
    font-weight: 600;
    border: none;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.1);
  }

  .nav-item :global(svg) {
    transition: stroke var(--motion-fast) var(--ease-standard);
    flex-shrink: 0;
  }

  /* Spacer */
  .nav-spacer {
    flex: 1;
  }

  /* Cider-style Profile Footer */
  .profile-footer {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    margin: 0 4px;
    background: transparent;
    cursor: pointer;
    transition: background var(--motion-fast) var(--ease-standard);
    -webkit-app-region: no-drag;
  }

  .profile-footer:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .profile-avatar {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--theme-accent), rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.7));
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    font-weight: 700;
    color: #000;
  }

  .profile-info {
    flex: 1;
    min-width: 0;
  }

  .profile-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
