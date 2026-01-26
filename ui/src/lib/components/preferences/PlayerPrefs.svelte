<script lang="ts">
  import { onMount } from 'svelte';
  import { devices, currentDevice, selectDevice, outputSettings, loadOutputSettings, saveOutputSettings, loadDevices, asioDrivers, loadAsioDrivers } from '../../state/playback';
  import { playerSettings, loadCategorySettings, saveCategorySetting, parseBool } from '../../state/preferences';
  import { openAsioControlPanel } from '../../api/playback';
  
  const isMock = import.meta.env.SERMON_MOCK === '1';
  
  let restartRequired = $state(false);
  let originalBufferSize: string | null = $state(null);
  let isOpeningPanel = $state(false);
  
  onMount(async () => {
    if (!isMock) {
      await Promise.all([loadDevices(), loadOutputSettings(), loadCategorySettings('player'), loadAsioDrivers()]);
    }
  });
  
  function handleBufferSizeChange(value: string) {
    if (originalBufferSize === null && $playerSettings) {
      originalBufferSize = $playerSettings['player.buffer_size_ms'];
    }
    if (value !== originalBufferSize) {
      restartRequired = true;
    }
    if (!isMock) {
      saveCategorySetting('player', 'player.buffer_size_ms', value);
    }
  }

  async function handleOpenControlPanel() {
    if (!$outputSettings?.asioDriver || isMock) return;
    
    isOpeningPanel = true;
    try {
      await openAsioControlPanel($outputSettings.asioDriver);
    } catch (e) {
      console.error('Failed to open ASIO control panel:', e);
    } finally {
      isOpeningPanel = false;
    }
  }
</script>

<div class="category-content">
  {#if restartRequired}
    <div class="restart-banner">
      ⚠️ Restart required for buffer size change to take effect. Please close and reopen the application.
    </div>
  {/if}

  <div class="setting">
    <label for="output-device">Output Device</label>
    <select id="output-device" value={$currentDevice?.id || 'default'} onchange={(e) => selectDevice(e.currentTarget.value)} disabled={isMock}>
      <option value="default">Default Output</option>
      {#each $devices as device}
        {#if !device.is_default}
          <option value={device.id}>{device.name}</option>
        {/if}
      {/each}
    </select>
  </div>

  {#if $outputSettings}
  <div class="setting">
    <label for="output-mode">Output Mode</label>
    <select id="output-mode" value={$outputSettings.mode} onchange={(e) => {
      const newMode = e.currentTarget.value as 'exclusive' | 'shared' | 'asio';
      saveOutputSettings({ ...$outputSettings!, mode: newMode });
    }} disabled={isMock}>
      <option value="shared">Shared (Windows Mixer)</option>
      <option value="exclusive">Exclusive (Bit-Perfect)</option>
      <option value="asio">ASIO (Professional)</option>
    </select>
  </div>

  {#if $outputSettings.mode === 'asio'}
    <div class="setting">
      <label for="asio-driver">ASIO Driver</label>
      <select id="asio-driver" value={$outputSettings.asioDriver || ''} onchange={(e) => saveOutputSettings({ ...$outputSettings!, asioDriver: e.currentTarget.value })} disabled={isMock}>
        <option value="">Select Driver...</option>
        {#each $asioDrivers as driver}
          <option value={driver.name}>{driver.name}</option>
        {/each}
      </select>
      {#if $asioDrivers.length === 0}
        <span class="setting-hint">No ASIO drivers found. Install an ASIO driver like ASIO4ALL.</span>
      {/if}
      {#if $outputSettings.asioDriver}
        <button 
          class="control-panel-btn"
          onclick={handleOpenControlPanel}
          disabled={isOpeningPanel || isMock}
        >
          {isOpeningPanel ? 'Opening...' : 'Open Control Panel'}
        </button>
      {/if}
    </div>
  {/if}

  {#if $outputSettings.mode !== 'asio'}
    <div class="setting">
      <label for="output-policy">Policy</label>
      <select id="output-policy" value={$outputSettings.policy} onchange={(e) => saveOutputSettings({ ...$outputSettings!, policy: e.currentTarget.value as 'strict' | 'compatibility' })} disabled={isMock}>
        <option value="strict">Strict (Exact Match)</option>
        <option value="compatibility">Compatibility (Allow Conversion)</option>
      </select>
    </div>

    <div class="setting">
      <label for="timing-mode">Timing Mode</label>
      <select id="timing-mode" value={$outputSettings.timing} onchange={(e) => saveOutputSettings({ ...$outputSettings!, timing: e.currentTarget.value as 'event' | 'polling' })} disabled={isMock}>
        <option value="polling">Polling (USB Compatible)</option>
        <option value="event">Event-Driven</option>
      </select>
    </div>
  {/if}

  <div class="setting">
      <label>
        <input type="checkbox" checked={$outputSettings.fade} onchange={(e) => saveOutputSettings({ ...$outputSettings!, fade: e.currentTarget.checked })} disabled={isMock} />
        Enable fade on format switch
      </label>
    </div>
  {/if}

  {#if $playerSettings}
    <div class="setting">
      <label for="buffer-size">Buffer Size (ms)</label>
      <select id="buffer-size" value={$playerSettings['player.buffer_size_ms']} onchange={(e) => handleBufferSizeChange(e.currentTarget.value)} disabled={isMock}>
        <option value="100">100ms (Low Latency)</option>
        <option value="250">250ms</option>
        <option value="500">500ms (Default)</option>
        <option value="1000">1000ms</option>
        <option value="2000">2000ms (High Buffer)</option>
      </select>
      <span class="setting-hint">Requires restart to take effect</span>
    </div>

    <div class="setting">
      <label>
        <input type="checkbox" checked={parseBool($playerSettings['player.preload_next'])} onchange={(e) => saveCategorySetting('player', 'player.preload_next', e.currentTarget.checked ? 'on' : 'off')} disabled={isMock} />
        Preload Next Track
      </label>
      <span class="setting-hint">Takes effect in future update</span>
    </div>
  {/if}
</div>

<style>
  .category-content { display: flex; flex-direction: column; gap: 1rem; }
  .setting { display: flex; flex-direction: column; gap: 0.5rem; }
  .setting label { color: #ccc; }
  .setting-hint { font-size: 0.8rem; color: #888; }
  select { background: #222; color: #fff; border: 1px solid #333; padding: 0.5rem; border-radius: 4px; width: 250px; }
  select:disabled { opacity: 0.5; }
  input[type="checkbox"] { margin-right: 0.5rem; }
  .restart-banner { padding: 0.75rem 1rem; background: rgba(255, 170, 0, 0.15); border: 1px solid #fa0; border-radius: 4px; color: #fa0; font-size: 0.9rem; }
  .control-panel-btn { background: #333; color: #fff; border: 1px solid #555; padding: 0.5rem 1rem; border-radius: 4px; cursor: pointer; margin-top: 0.5rem; width: fit-content; }
  .control-panel-btn:hover:not(:disabled) { background: #444; }
  .control-panel-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
