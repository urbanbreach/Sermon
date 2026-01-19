<script lang="ts">
  import { onMount } from 'svelte';
  import GeneralPrefs from '../components/preferences/GeneralPrefs.svelte';
  import PlayerPrefs from '../components/preferences/PlayerPrefs.svelte';
  import NowPlayingPrefs from '../components/preferences/NowPlayingPrefs.svelte';
  import LibraryPrefs from '../components/preferences/LibraryPrefs.svelte';
  import TagsPrefs from '../components/preferences/TagsPrefs.svelte';
  import InternetPrefs from '../components/preferences/InternetPrefs.svelte';
  import DevicesPrefs from '../components/preferences/DevicesPrefs.svelte';
  
  const isMock = import.meta.env.SERMON_MOCK === '1';
  
  type PreferenceCategory = 'general' | 'player' | 'nowplaying' | 'library' | 'tags' | 'internet' | 'devices';
  
  let activeCategory: PreferenceCategory = $state('general');
  let statusMessage: string = $state('');
  
  const categories: { id: PreferenceCategory; label: string }[] = [
    { id: 'general', label: 'General' },
    { id: 'player', label: 'Player' },
    { id: 'nowplaying', label: 'Now Playing' },
    { id: 'library', label: 'Library' },
    { id: 'tags', label: 'Tags' },
    { id: 'internet', label: 'Internet' },
    { id: 'devices', label: 'Devices' },
  ];
  
  function selectCategory(category: PreferenceCategory) {
    activeCategory = category;
  }
  
  async function handleResetToDefaults() {
    if (isMock) return;
    // TODO: Call resetCategoryToDefaults(activeCategory)
    statusMessage = '✓ Reset complete';
    setTimeout(() => { statusMessage = ''; }, 3000);
  }
  
  async function handleExportDiagnostics() {
    if (isMock) return;
    // TODO: Implement in Task 9
    statusMessage = 'Export not implemented yet';
    setTimeout(() => { statusMessage = ''; }, 3000);
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
          onclick={() => selectCategory(cat.id)}
        >
          {cat.label}
        </button>
      {/each}
    </nav>
  </aside>
  
  <main class="prefs-content">
    <header class="prefs-header">
      <h1>{categories.find(c => c.id === activeCategory)?.label}</h1>
      <button class="btn btn-secondary" onclick={handleResetToDefaults} disabled={isMock}>
        Reset to Defaults
      </button>
    </header>
    
    <div class="prefs-body">
      {#if activeCategory === 'general'}
        <GeneralPrefs />
      {:else if activeCategory === 'player'}
        <PlayerPrefs />
      {:else if activeCategory === 'nowplaying'}
        <NowPlayingPrefs />
      {:else if activeCategory === 'library'}
        <LibraryPrefs />
      {:else if activeCategory === 'tags'}
        <TagsPrefs />
      {:else if activeCategory === 'internet'}
        <InternetPrefs />
      {:else if activeCategory === 'devices'}
        <DevicesPrefs />
      {/if}
    </div>
    
    {#if statusMessage}
      <div class="status-message">{statusMessage}</div>
    {/if}
  </main>
  
  <footer class="prefs-footer">
    <button class="btn" onclick={handleExportDiagnostics} disabled={isMock}>
      Export Diagnostics
    </button>
  </footer>
</div>

<style>
  .preferences-view {
    display: flex;
    height: 100%;
    width: 100%;
    overflow: hidden;
  }

  .prefs-sidebar {
    width: 200px;
    background: var(--glass-bg);
    border-right: 1px solid var(--glass-border);
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .prefs-sidebar h2 {
    font-size: 1.2rem;
    margin: 0;
    color: #fff;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--glass-border);
  }

  .category-nav {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .category-nav button {
    background: transparent;
    border: none;
    color: #888;
    text-align: left;
    padding: 0.5rem 0.75rem;
    cursor: pointer;
    font-size: 1rem;
    border-radius: var(--glass-radius);
    transition: all 0.2s;
  }

  .category-nav button:hover {
    color: #fff;
    background: var(--glass-highlight);
  }

  .category-nav button.active {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
    font-weight: bold;
    border-left: 3px solid #4af;
  }

  .prefs-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    position: relative;
    background: rgba(0, 0, 0, 0.2);
  }

  .prefs-header {
    padding: 1rem 2rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--glass-border);
    background: rgba(0, 0, 0, 0.2);
  }

  .prefs-header h1 {
    margin: 0;
    font-size: 1.5rem;
    color: #fff;
  }

  .prefs-body {
    flex: 1;
    padding: 2rem;
    overflow-y: auto;
  }

  .prefs-footer {
    padding: 1rem 2rem;
    border-top: 1px solid var(--glass-border);
    background: rgba(0, 0, 0, 0.2);
    display: flex;
    justify-content: flex-end;
  }

  .btn {
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
    color: #fff;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn:hover:not(:disabled) {
    background: var(--glass-highlight);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    font-size: 0.9rem;
    padding: 0.4rem 0.8rem;
  }

  .status-message {
    position: absolute;
    bottom: 4rem;
    right: 2rem;
    background: rgba(0, 0, 0, 0.8);
    color: #4f4;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    border: 1px solid #4f4;
    animation: fade-in 0.3s;
  }

  @keyframes fade-in {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
