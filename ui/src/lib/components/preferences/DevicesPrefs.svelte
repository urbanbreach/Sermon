<script lang="ts">
  import { onMount } from 'svelte';
  import { devicesSettings, loadCategorySettings, saveCategorySetting, parseBool } from '../../state/preferences';
  
  const isMock = import.meta.env.SERMON_MOCK === '1';
  
  let restartRequired = $state(false);
  
  onMount(async () => {
    if (!isMock) {
      await loadCategorySettings('devices');
    }
  });
  
  function handleDopEnabledChange(checked: boolean) {
    if (!isMock) {
      saveCategorySetting('devices', 'devices.dsd_dop_enabled', checked ? 'on' : 'off');
    }
    // DoP changes require restart to take effect
    restartRequired = true;
  }
  
  function handleDopStrictChange(checked: boolean) {
    if (!isMock) {
      saveCategorySetting('devices', 'devices.dsd_dop_strict', checked ? 'on' : 'off');
    }
  }
</script>

<div class="category-content">
  {#if restartRequired}
    <div class="restart-banner">
      Restart required for DSD playback changes to take effect. Please close and reopen the application.
    </div>
  {/if}

  <div class="section">
    <h3>DSD Playback</h3>
    <p class="section-description">
      DSD (Direct Stream Digital) is a high-resolution audio format used by SACDs. 
      DoP (DSD over PCM) encapsulates DSD data within a PCM stream for USB DACs that support native DSD playback.
    </p>
    
    {#if $devicesSettings}
      <div class="setting">
        <label>
          <input 
            type="checkbox" 
            checked={parseBool($devicesSettings['devices.dsd_dop_enabled'])} 
            onchange={(e) => handleDopEnabledChange(e.currentTarget.checked)} 
            disabled={isMock} 
          />
          Enable DoP (DSD over PCM)
        </label>
        <span class="setting-hint">
          Enables DSD playback via DoP encoding. Your DAC must support DoP for this to work.
        </span>
      </div>

      <div class="setting">
        <label>
          <input 
            type="checkbox" 
            checked={parseBool($devicesSettings['devices.dsd_dop_strict'])} 
            onchange={(e) => handleDopStrictChange(e.currentTarget.checked)} 
            disabled={isMock || !parseBool($devicesSettings['devices.dsd_dop_enabled'])} 
          />
          Strict DoP Mode
        </label>
        <span class="setting-hint">
          When enabled, DSD files will only play if the DAC confirms native DSD support. 
          When disabled, playback will attempt DoP even if support is uncertain.
        </span>
      </div>

      <div class="info-box">
        <strong>Supported DSD Formats</strong>
        <ul>
          <li><strong>DSF</strong> - Sony DSD format with metadata support</li>
          <li><strong>DFF</strong> - DSDIFF format (Philips/Sony standard)</li>
        </ul>
        <strong>Supported Rates</strong>
        <ul>
          <li>DSD64 (2.8224 MHz) → DoP at 176.4 kHz</li>
          <li>DSD128 (5.6448 MHz) → DoP at 352.8 kHz</li>
          <li>DSD256 (11.2896 MHz) → DoP at 705.6 kHz</li>
        </ul>
      </div>
    {:else}
      <div class="loading">Loading device settings...</div>
    {/if}
  </div>
</div>

<style>
  .category-content { display: flex; flex-direction: column; gap: 1rem; }
  .section { display: flex; flex-direction: column; gap: 1rem; }
  .section h3 { margin: 0; color: #fff; font-size: 1.1rem; }
  .section-description { color: #888; font-size: 0.9rem; margin: 0; line-height: 1.5; }
  .setting { display: flex; flex-direction: column; gap: 0.5rem; }
  .setting label { color: #ccc; display: flex; align-items: center; cursor: pointer; }
  .setting-hint { font-size: 0.8rem; color: #888; margin-left: 1.5rem; }
  input[type="checkbox"] { margin-right: 0.5rem; }
  input[type="checkbox"]:disabled { opacity: 0.5; cursor: not-allowed; }
  .restart-banner { 
    padding: 0.75rem 1rem; 
    background: rgba(255, 170, 0, 0.15); 
    border: 1px solid #fa0; 
    border-radius: 4px; 
    color: #fa0; 
    font-size: 0.9rem; 
  }
  .info-box {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    padding: 1rem;
    margin-top: 0.5rem;
  }
  .info-box strong { color: #ccc; font-size: 0.85rem; display: block; margin-bottom: 0.5rem; }
  .info-box ul { margin: 0.25rem 0 0.75rem 1.25rem; padding: 0; color: #888; font-size: 0.8rem; }
  .info-box li { margin-bottom: 0.25rem; }
  .loading { color: #888; font-style: italic; }
</style>
