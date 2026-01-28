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
    <h2>Preferences</h2>
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
  }

  .prefs-sidebar {
    width: 220px;
    background: var(--glass-bg, rgba(20, 20, 20, 0.85));
    backdrop-filter: blur(var(--glass-blur, 18px));
    -webkit-backdrop-filter: blur(var(--glass-blur, 18px));
    border-right: 1px solid var(--glass-border, rgba(255, 255, 255, 0.08));
    padding: var(--space-4, 16px);
    display: flex;
    flex-direction: column;
    gap: var(--space-4, 16px);
  }

  .prefs-sidebar h2 {
    font-size: var(--text-section, 18px);
    font-weight: 600;
    margin: 0;
    color: #fff;
    padding-bottom: var(--space-3, 12px);
    border-bottom: 1px solid var(--glass-border, rgba(255, 255, 255, 0.08));
  }

  .category-nav {
    display: flex;
    flex-direction: column;
    gap: var(--space-1, 4px);
  }

  .category-nav button {
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.6);
    text-align: left;
    padding: var(--space-2, 8px) var(--space-3, 12px);
    cursor: pointer;
    font-size: var(--text-body, 14px);
    border-radius: var(--radius-sm, 8px);
    transition: all 0.15s ease;
    display: flex;
    align-items: center;
    gap: var(--space-3, 12px);
  }

  .category-nav button:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.06);
  }

  .category-nav button.active {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
    font-weight: 500;
  }

  .prefs-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    position: relative;
    background: rgba(0, 0, 0, 0.15);
    min-width: 0;
  }

  .prefs-header {
    padding: var(--space-4, 16px) var(--space-6, 24px);
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--glass-border, rgba(255, 255, 255, 0.08));
    background: rgba(0, 0, 0, 0.1);
    flex-shrink: 0;
  }

  .prefs-header h1 {
    margin: 0;
    font-size: var(--text-view-title, 22px);
    font-weight: 600;
    color: #fff;
  }

  .prefs-body {
    flex: 1;
    padding: var(--space-6, 24px);
    padding-bottom: calc(var(--layout-player-height, 80px) + var(--space-6, 24px));
    overflow-y: auto;
  }

  .btn {
    background: var(--glass-bg, rgba(255, 255, 255, 0.08));
    border: 1px solid var(--glass-border, rgba(255, 255, 255, 0.1));
    color: #fff;
    padding: var(--space-2, 8px) var(--space-4, 16px);
    border-radius: var(--radius-sm, 8px);
    cursor: pointer;
    transition: all 0.15s ease;
    font-size: var(--text-body, 14px);
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
  }

  .btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-secondary {
    font-size: var(--text-meta, 12px);
    padding: var(--space-1, 4px) var(--space-3, 12px);
    background: transparent;
  }

  .status-message {
    position: absolute;
    bottom: 4.5rem;
    right: var(--space-6, 24px);
    background: rgba(40, 167, 69, 0.15);
    color: #4ade80;
    padding: var(--space-2, 8px) var(--space-4, 16px);
    border-radius: var(--radius-sm, 8px);
    border: 1px solid rgba(40, 167, 69, 0.3);
    animation: slide-up 0.2s ease;
    display: flex;
    align-items: center;
    gap: var(--space-2, 8px);
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
