<script lang="ts">
  import { onMount } from 'svelte';
  import PlayerPrefs from '../components/preferences/PlayerPrefs.svelte';
  import LibraryPrefs from '../components/preferences/LibraryPrefs.svelte';
  import InternetPrefs from '../components/preferences/InternetPrefs.svelte';
  import AppearancePrefs from '../components/preferences/AppearancePrefs.svelte';
  import DevicesPrefs from '../components/preferences/DevicesPrefs.svelte';
  import { resetCategoryToDefaults } from '../state/preferences';
  import { loadEffectsSettings } from '../state/effects';
  import { Settings, Volume2, Library, Globe, Palette, RotateCcw, Check, X, Disc } from '@lucide/svelte';
  
  const isMock = import.meta.env.SERMON_MOCK === '1';
  
  type PreferenceCategory = 'player' | 'library' | 'internet' | 'appearance' | 'devices';
  
  let activeCategory: PreferenceCategory = $state('player');
  let statusMessage: string = $state('');
  let statusType: 'success' | 'error' = $state('success');
  
  const categories: { id: PreferenceCategory; label: string; icon: typeof Settings }[] = [
    { id: 'player', label: 'Player', icon: Volume2 },
    { id: 'devices', label: 'Devices', icon: Disc },
    { id: 'library', label: 'Library', icon: Library },
    { id: 'internet', label: 'Internet', icon: Globe },
    { id: 'appearance', label: 'Appearance', icon: Palette },
  ];

  
  function selectCategory(category: PreferenceCategory) {
    activeCategory = category;
  }
  
  async function handleResetToDefaults() {
    if (isMock) return;
    try {
      await resetCategoryToDefaults(activeCategory);
      statusType = 'success';
      statusMessage = 'Reset complete';
      setTimeout(() => { statusMessage = ''; }, 3000);
    } catch (e) {
      statusType = 'error';
      statusMessage = 'Reset failed';
      setTimeout(() => { statusMessage = ''; }, 3000);
    }
  }
</script>

<div class="preferences-view">
  <aside class="prefs-sidebar">
    <header class="sidebar-header">
      <h2>Preferences</h2>
    </header>
    <nav class="category-nav">
      {#each categories as cat}
        <button
          class:active={activeCategory === cat.id}
          data-category={cat.id}
          data-testid="prefs-nav-{cat.id}"
          onclick={() => selectCategory(cat.id)}
        >
          <cat.icon size={18} />
          <span>{cat.label}</span>
        </button>
      {/each}
    </nav>
  </aside>
  
  <main class="prefs-content">
    <header class="prefs-header">
      <h1>{categories.find(c => c.id === activeCategory)?.label}</h1>
      <button class="btn btn-secondary" onclick={handleResetToDefaults} disabled={isMock} data-testid="prefs-reset-defaults">
        <RotateCcw size={14} />
        Reset to Defaults
      </button>
    </header>
    
    <div class="prefs-body">
      {#if activeCategory === 'player'}
        <PlayerPrefs />
      {:else if activeCategory === 'devices'}
        <DevicesPrefs />
      {:else if activeCategory === 'library'}
        <LibraryPrefs />
      {:else if activeCategory === 'internet'}
        <InternetPrefs />
      {:else if activeCategory === 'appearance'}
        <AppearancePrefs />
      {/if}
    </div>

    
    {#if statusMessage}
      <div class="status-message" class:error={statusType === 'error'}>
        {#if statusType === 'success'}
          <Check size={14} />
        {:else}
          <X size={14} />
        {/if}
        {statusMessage}
      </div>
    {/if}
  </main>
</div>

<style>
  .preferences-view {
    display: flex;
    height: 100%;
    width: 100%;
    overflow: hidden;
    background: var(--surface-0, #0a0a0a);
    color: var(--text-primary, rgba(255, 255, 255, 0.92));
  }

  .prefs-sidebar {
    width: 240px;
    background: var(--surface-1, rgba(255, 255, 255, 0.04));
    border-right: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    flex-shrink: 0;
  }

  .sidebar-header {
    padding-bottom: var(--space-4);
    border-bottom: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
  }

  .prefs-sidebar h2 {
    font-size: var(--text-section, 18px);
    font-weight: 600;
    margin: 0;
    color: var(--text-primary, rgba(255, 255, 255, 0.92));
    letter-spacing: -0.01em;
  }

  .category-nav {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .category-nav button {
    background: transparent;
    border: none;
    color: var(--text-secondary, rgba(255, 255, 255, 0.70));
    text-align: left;
    padding: var(--space-2) var(--space-3);
    cursor: pointer;
    font-size: var(--text-body, 14px);
    border-radius: var(--radius-sm, 4px);
    transition: all 0.15s ease;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    font-weight: 500;
  }

  .category-nav button:hover {
    color: var(--text-primary, rgba(255, 255, 255, 0.92));
    background: var(--surface-hover, rgba(255, 255, 255, 0.10));
  }

  .category-nav button.active {
    color: var(--text-primary, rgba(255, 255, 255, 0.92));
    background: var(--surface-active, rgba(255, 255, 255, 0.06));
  }

  .prefs-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    position: relative;
    background: var(--surface-0, #0a0a0a);
    min-width: 0;
  }

  .prefs-header {
    padding: var(--space-6) var(--space-8);
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
    flex-shrink: 0;
  }

  .prefs-header h1 {
    margin: 0;
    font-size: var(--text-view-title, 22px);
    font-weight: 600;
    color: var(--text-primary, rgba(255, 255, 255, 0.92));
    letter-spacing: -0.02em;
  }

  .prefs-body {
    flex: 1;
    padding: var(--space-8);
    padding-bottom: calc(var(--layout-player-height) + var(--space-8));
    overflow-y: auto;
    max-width: 800px;
  }

  .btn {
    background: var(--surface-2, rgba(255, 255, 255, 0.08));
    border: 1px solid transparent;
    color: var(--text-primary, rgba(255, 255, 255, 0.92));
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-sm, 4px);
    cursor: pointer;
    transition: all 0.15s ease;
    font-size: var(--text-body, 14px);
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    font-weight: 500;
  }

  .btn:hover:not(:disabled) {
    background: var(--surface-hover, rgba(255, 255, 255, 0.10));
  }

  .btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-secondary {
    font-size: var(--text-meta, 12px);
    padding: var(--space-2) var(--space-3);
    background: transparent;
    border: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
    color: var(--text-secondary, rgba(255, 255, 255, 0.70));
  }
  
  .btn-secondary:hover:not(:disabled) {
    border-color: var(--text-secondary, rgba(255, 255, 255, 0.70));
    color: var(--text-primary, rgba(255, 255, 255, 0.92));
  }

  .status-message {
    position: absolute;
    bottom: 4.5rem;
    right: var(--space-6);
    background: rgba(40, 167, 69, 0.15);
    color: #4ade80;
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-sm, 4px);
    border: 1px solid rgba(40, 167, 69, 0.3);
    animation: slide-up 0.2s ease;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-body, 14px);
  }

  .status-message.error {
    background: rgba(220, 53, 69, 0.15);
    color: #f87171;
    border-color: rgba(220, 53, 69, 0.3);
  }

  @keyframes slide-up {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
