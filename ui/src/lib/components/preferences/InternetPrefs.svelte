<script lang="ts">
  import { onMount } from 'svelte';
  import { providerItunes, providerDeezer, setProviderItunes, setProviderDeezer, loadEffectsSettings } from '../../state/effects';
  import { internetSettings, loadCategorySettings, saveCategorySetting, parseBool } from '../../state/preferences';

  const isMock = import.meta.env.SERMON_MOCK === '1';

  onMount(async () => {
    if (!isMock) {
      // effects.ts manages its own loading via loadEffectsSettings
      await Promise.all([loadEffectsSettings(), loadCategorySettings('internet')]);
    }
  });
</script>

<div class="category-content">
  <div class="setting-group">
    <h3>Metadata Providers</h3>
    
    <div class="setting">
      <label>
        <input type="checkbox" checked={$providerItunes} onchange={(e) => setProviderItunes(e.currentTarget.checked)} disabled={isMock} />
        iTunes Store
      </label>
      <span class="setting-hint">Used for album artwork and metadata lookup</span>
    </div>

    <div class="setting">
      <label>
        <input type="checkbox" checked={$providerDeezer} onchange={(e) => setProviderDeezer(e.currentTarget.checked)} disabled={isMock} />
        Deezer
      </label>
      <span class="setting-hint">Used for high-res album artwork</span>
    </div>
  </div>
</div>

<style>
  .category-content { display: flex; flex-direction: column; gap: 2rem; }
  .setting-group { display: flex; flex-direction: column; gap: 1rem; }
  h3 { font-size: 1rem; color: #fff; margin: 0 0 0.5rem 0; border-bottom: 1px solid rgba(255,255,255,0.1); padding-bottom: 0.5rem; }
  .setting { display: flex; flex-direction: column; gap: 0.25rem; }
  .setting label { display: flex; align-items: center; gap: 0.5rem; color: #ccc; }
  .setting-hint { font-size: 0.8rem; color: #888; margin-left: 1.5rem; }
  .coming-soon { font-size: 0.75rem; color: #666; font-style: italic; }
  input:disabled { opacity: 0.5; }
</style>
