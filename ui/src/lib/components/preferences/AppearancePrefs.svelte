<script lang="ts">
  import { onMount } from 'svelte';
  import { 
    accentColor, sidebarVisible, bottomBarWaveformStyle,
    artworkRoundedSidebar, artworkRoundedAlbums, artworkRoundedAlbumDetail,
    bottomBarWaveformSeekbar,
    setAccentColor, setSidebarVisible, setBottomBarWaveformStyle,
    setArtworkRoundedSidebar, setArtworkRoundedAlbums, setArtworkRoundedAlbumDetail,
    setBottomBarWaveformSeekbar,
    loadEffectsSettings
  } from '../../state/effects';
  import SettingGroup from '../primitives/SettingGroup.svelte';
  import SettingRow from '../primitives/SettingRow.svelte';
  import { Check } from '@lucide/svelte';
  
  const isMock = import.meta.env.SERMON_MOCK === '1';

  const PRESET_COLORS = [
    '#3b82f6', // Blue
    '#ef4444', // Red
    '#10b981', // Green
    '#f59e0b', // Amber
    '#8b5cf6', // Violet
    '#ec4899', // Pink
    '#06b6d4', // Cyan
    '#f97316', // Orange
    '#ffffff', // White
  ];

  let customHex = $state('');

  onMount(async () => {
    if (!isMock) {
      await loadEffectsSettings();
    }
    customHex = $accentColor;
  });

  // Update local hex when store changes
  $effect(() => {
    customHex = $accentColor;
  });
  
  async function handleAccentColorChange(color: string) {
    if (isMock) return;
    await setAccentColor(color);
  }

  function handleHexInput(e: Event) {
    const target = e.target as HTMLInputElement;
    let value = target.value;
    
    // Basic hex validation
    if (value.startsWith('#')) {
      if (/^#[0-9A-Fa-f]{6}$/.test(value)) {
        handleAccentColorChange(value);
      }
    } else if (/^[0-9A-Fa-f]{6}$/.test(value)) {
       handleAccentColorChange('#' + value);
    }
    customHex = value;
  }
</script>

<div class="category-content">
  <SettingGroup title="Accent Color">
    <div class="accent-picker-container">
      <div class="color-grid">
        {#each PRESET_COLORS as color}
          <button 
            class="color-swatch" 
            style:background-color={color}
            class:active={$accentColor.toLowerCase() === color.toLowerCase()}
            onclick={() => handleAccentColorChange(color)}
            aria-label="Select color {color}"
          >
            {#if $accentColor.toLowerCase() === color.toLowerCase()}
              <Check size={14} color={color === '#ffffff' ? '#000' : '#fff'} />
            {/if}
          </button>
        {/each}
      </div>
      
      <div class="hex-input-wrapper">
        <span class="hex-prefix">#</span>
        <input 
          type="text" 
          class="hex-input" 
          value={customHex.replace('#', '')} 
          oninput={handleHexInput}
          maxlength="6"
        />
        <div class="current-swatch" style:background-color={$accentColor}></div>
      </div>
    </div>
  </SettingGroup>

  <SettingGroup title="Waveform">
    <SettingRow label="Waveform Seekbar" description="Show waveform visualization in the bottom player bar">
      <input 
        type="checkbox" 
        checked={$bottomBarWaveformSeekbar}
        onchange={(e) => setBottomBarWaveformSeekbar(e.currentTarget.checked)}
        disabled={isMock}
      />
    </SettingRow>

    {#if $bottomBarWaveformSeekbar}
      <SettingRow label="Waveform Style" description="Choose the visual style of the waveform">
        <div class="segmented-control">
          <button 
            class:active={$bottomBarWaveformStyle === 'pills'} 
            onclick={() => setBottomBarWaveformStyle('pills')}
            disabled={isMock}
          >
            Pills
          </button>
          <button 
            class:active={$bottomBarWaveformStyle === 'raw'} 
            onclick={() => setBottomBarWaveformStyle('raw')}
            disabled={isMock}
          >
            Raw
          </button>
        </div>
      </SettingRow>
    {/if}
  </SettingGroup>

  <SettingGroup title="Cover Art">
    <SettingRow label="Rounded corners in sidebar" description="Apply rounded corners to artwork in the sidebar">
      <input type="checkbox" checked={$artworkRoundedSidebar} onchange={(e) => setArtworkRoundedSidebar(e.currentTarget.checked)} disabled={isMock} />
    </SettingRow>
    <SettingRow label="Rounded corners on album grid" description="Apply rounded corners to album artwork in grid views">
      <input type="checkbox" checked={$artworkRoundedAlbums} onchange={(e) => setArtworkRoundedAlbums(e.currentTarget.checked)} disabled={isMock} />
    </SettingRow>
    <SettingRow label="Rounded corners on album detail" description="Apply rounded corners to artwork on album detail pages">
      <input type="checkbox" checked={$artworkRoundedAlbumDetail} onchange={(e) => setArtworkRoundedAlbumDetail(e.currentTarget.checked)} disabled={isMock} />
    </SettingRow>
  </SettingGroup>

  <SettingGroup title="Layout">
    <SettingRow label="Show Sidebar" description="Toggle the visibility of the left sidebar navigation">
      <input 
        type="checkbox" 
        checked={$sidebarVisible}
        onchange={(e) => setSidebarVisible(e.currentTarget.checked)}
        disabled={isMock}
      />
    </SettingRow>
  </SettingGroup>
</div>

<style>
  .category-content { 
    display: flex; 
    flex-direction: column; 
    gap: var(--space-6); 
  }

  /* Accent Picker Styles */
  .accent-picker-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-2) 0;
  }

  .color-grid {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
  }

  .color-swatch {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.1s ease, border-color 0.1s ease;
    padding: 0;
  }

  .color-swatch:hover {
    transform: scale(1.1);
  }

  .color-swatch.active {
    border-color: var(--text-primary, #fff);
  }

  .hex-input-wrapper {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    background: var(--surface-1, rgba(255, 255, 255, 0.04));
    border: 1px solid var(--divider, rgba(255, 255, 255, 0.07));
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm, 4px);
    width: fit-content;
  }

  .hex-prefix {
    color: var(--text-secondary, rgba(255, 255, 255, 0.70));
    font-family: var(--font-mono, monospace);
  }

  .hex-input {
    background: transparent;
    border: none;
    color: var(--text-primary, #fff);
    font-family: var(--font-mono, monospace);
    font-size: var(--text-body, 14px);
    width: 60px;
    outline: none;
    text-transform: uppercase;
  }

  .current-swatch {
    width: 20px;
    height: 20px;
    border-radius: 4px;
    border: 1px solid var(--divider, rgba(255, 255, 255, 0.1));
  }

  /* Segmented Control */
  .segmented-control {
    display: flex;
    background: var(--surface-1, rgba(255, 255, 255, 0.04));
    padding: 2px;
    border-radius: var(--radius-sm, 4px);
    border: 1px solid var(--divider, rgba(255, 255, 255, 0.07));
  }

  .segmented-control button {
    background: transparent;
    border: none;
    color: var(--text-secondary, rgba(255, 255, 255, 0.70));
    padding: var(--space-1) var(--space-3);
    font-size: var(--text-meta, 12px);
    cursor: pointer;
    border-radius: 2px;
    transition: all 0.15s ease;
  }

  .segmented-control button.active {
    background: var(--surface-2, rgba(255, 255, 255, 0.08));
    color: var(--text-primary, #fff);
    font-weight: 500;
  }

  .segmented-control button:hover:not(.active) {
    color: var(--text-primary, #fff);
  }

  /* Checkbox styling override if needed, though usually handled globally or by browser */
  input[type="checkbox"] {
    accent-color: var(--theme-accent, #3b82f6);
    width: 16px;
    height: 16px;
    cursor: pointer;
  }
</style>
