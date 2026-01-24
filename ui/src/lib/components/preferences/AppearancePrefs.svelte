<script lang="ts">
  import { onMount } from 'svelte';
  import { 
    reduceEffects, themeBlur, themeGlow, themeBorderHighlight,
    blurPx, glowStrength, borderStrength, bgIntensity, bgNoiseOpacity, bgCrossfadeMs,
    bgStaticColor, bgDynamicLibrary, bgDynamicNowPlaying, bgDynamicAlbumDetail,
    accentColor,
    setReduceEffects, setThemeBlur, setThemeGlow, setThemeBorderHighlight,
    setBgStaticColor, setBgDynamicLibrary, setBgDynamicNowPlaying, setBgDynamicAlbumDetail,
    setAccentColor,
    applyEffects, loadEffectsSettings
  } from '../../state/effects';
  import { loadCategorySettings, saveCategorySetting } from '../../state/preferences';
  
  const isMock = import.meta.env.SERMON_MOCK === '1';
  
  onMount(async () => {
    if (!isMock) {
      // loadCategorySettings('appearance') removed - appearance settings are loaded via loadEffectsSettings()
      // which uses individual cmd_settings_get calls that work correctly
      await loadEffectsSettings();
    }
  });
  
  async function handleSliderChange(key: string, value: number) {
    if (isMock) return;
    await saveCategorySetting('appearance', key, String(value));
    await loadEffectsSettings();
  }

  async function handleColorChange(e: Event) {
    if (isMock) return;
    const target = e.target as HTMLInputElement;
    await setBgStaticColor(target.value);
  }

  async function handleAccentColorChange(e: Event) {
    if (isMock) return;
    const target = e.target as HTMLInputElement;
    await setAccentColor(target.value);
  }
</script>

<div class="category-content">
  <div class="setting">
    <label data-testid="appearance-toggle-reduce">
      <input 
        type="checkbox" 
        checked={$reduceEffects}
        onchange={(e) => setReduceEffects(e.currentTarget.checked)}
        disabled={isMock}
      />
      Reduce Effects
    </label>
    <span class="setting-hint">Disables blur, glow, and animation effects for better performance</span>
  </div>

  {#if !$reduceEffects}
    <div class="setting-group">
      <h3>Glass Effects</h3>
      
      <div class="setting">
        <label>
          <input 
            type="checkbox" 
            checked={$themeBlur}
            onchange={(e) => setThemeBlur(e.currentTarget.checked)}
            disabled={isMock}
          />
          Glass Blur Effect
        </label>
      </div>

      {#if $themeBlur}
        <div class="setting slider-setting">
          <label for="blur-px">Blur Amount: {$blurPx}px</label>
          <input 
            type="range" 
            id="blur-px"
            min="0" 
            max="32" 
            step="1"
            value={$blurPx}
            oninput={(e) => handleSliderChange('ui.theme.blur_px', parseFloat(e.currentTarget.value))}
            disabled={isMock}
          />
        </div>
      {/if}

      <div class="setting">
        <label>
          <input 
            type="checkbox" 
            checked={$themeGlow}
            onchange={(e) => setThemeGlow(e.currentTarget.checked)}
            disabled={isMock}
          />
          Theme Glow Effect
        </label>
      </div>

      {#if $themeGlow}
        <div class="setting slider-setting">
          <label for="glow-strength">Glow Strength: {($glowStrength * 100).toFixed(0)}%</label>
          <input 
            type="range" 
            id="glow-strength"
            min="0" 
            max="1" 
            step="0.05"
            value={$glowStrength}
            oninput={(e) => handleSliderChange('ui.theme.glow_strength', parseFloat(e.currentTarget.value))}
            disabled={isMock}
          />
        </div>
      {/if}

      <div class="setting">
        <label>
          <input 
            type="checkbox" 
            checked={$themeBorderHighlight}
            onchange={(e) => setThemeBorderHighlight(e.currentTarget.checked)}
            disabled={isMock}
          />
          Border Highlight Effect
        </label>
      </div>

      {#if $themeBorderHighlight}
        <div class="setting slider-setting">
          <label for="border-strength">Border Strength: {($borderStrength * 100).toFixed(0)}%</label>
          <input 
            type="range" 
            id="border-strength"
            min="0" 
            max="1" 
            step="0.05"
            value={$borderStrength}
            oninput={(e) => handleSliderChange('ui.theme.border_strength', parseFloat(e.currentTarget.value))}
            disabled={isMock}
          />
        </div>
      {/if}
    </div>

    <div class="setting-group">
      <h3>Colors</h3>

      <div class="setting color-setting">
        <label for="accent-color">Highlight Color</label>
        <div class="color-picker-row">
          <input 
            type="color" 
            id="accent-color"
            value={$accentColor}
            onchange={handleAccentColorChange}
            disabled={isMock}
          />
          <span class="color-value">{$accentColor}</span>
        </div>
        <span class="setting-hint">Used for buttons, selections, and UI accents</span>
      </div>
    </div>

    <div class="setting-group">
      <h3>Background</h3>

      <div class="setting">
        <label>
          <input 
            type="checkbox" 
            checked={$bgDynamicLibrary}
            onchange={(e) => setBgDynamicLibrary(e.currentTarget.checked)}
            disabled={isMock}
          />
          Dynamic Background in Library
        </label>
        <span class="setting-hint">Use artwork-based colors in library views</span>
      </div>

      <div class="setting">
        <label>
          <input 
            type="checkbox" 
            checked={$bgDynamicNowPlaying}
            onchange={(e) => setBgDynamicNowPlaying(e.currentTarget.checked)}
            disabled={isMock}
          />
          Dynamic Background in Now Playing
        </label>
        <span class="setting-hint">Use artwork-based colors in now playing view</span>
      </div>

      <div class="setting">
        <label>
          <input 
            type="checkbox" 
            checked={$bgDynamicAlbumDetail}
            onchange={(e) => setBgDynamicAlbumDetail(e.currentTarget.checked)}
            disabled={isMock}
          />
          Dynamic Background in Album Page
        </label>
        <span class="setting-hint">Use artwork-based colors in album detail view</span>
      </div>

      <div class="setting color-setting">
        <label for="bg-static-color">Static Background Color</label>
        <div class="color-picker-row">
          <input 
            type="color" 
            id="bg-static-color"
            value={$bgStaticColor}
            onchange={handleColorChange}
            disabled={isMock}
          />
          <span class="color-value">{$bgStaticColor}</span>
        </div>
        <span class="setting-hint">Used when dynamic background is disabled</span>
      </div>
      
      <div class="setting slider-setting" data-testid="appearance-slider-bg-intensity">
        <label for="bg-intensity">Intensity: {($bgIntensity * 100).toFixed(0)}%</label>
        <input 
          type="range" 
          id="bg-intensity"
          min="0" 
          max="0.6" 
          step="0.01"
          value={$bgIntensity}
          oninput={(e) => handleSliderChange('ui.background.intensity', parseFloat(e.currentTarget.value))}
          disabled={isMock}
        />
      </div>

      <div class="setting slider-setting">
        <label for="bg-noise">Noise Opacity: {($bgNoiseOpacity * 100).toFixed(0)}%</label>
        <input 
          type="range" 
          id="bg-noise"
          min="0" 
          max="0.3" 
          step="0.01"
          value={$bgNoiseOpacity}
          oninput={(e) => handleSliderChange('ui.background.noise_opacity', parseFloat(e.currentTarget.value))}
          disabled={isMock}
        />
      </div>

      <div class="setting slider-setting">
        <label for="bg-crossfade">Transition Speed: {$bgCrossfadeMs}ms</label>
        <input 
          type="range" 
          id="bg-crossfade"
          min="400" 
          max="2400" 
          step="100"
          value={$bgCrossfadeMs}
          oninput={(e) => handleSliderChange('ui.background.crossfade_ms', parseFloat(e.currentTarget.value))}
          disabled={isMock}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .category-content { display: flex; flex-direction: column; gap: 1rem; }
  .setting { display: flex; flex-direction: column; gap: 0.5rem; }
  .setting label { color: #ccc; }
  .setting-hint { font-size: 0.8rem; color: #888; }
  
  .setting-group {
    border: 1px solid var(--glass-border);
    border-radius: 8px;
    padding: 1rem;
    margin-top: 0.5rem;
  }
  
  .setting-group h3 {
    margin: 0 0 1rem 0;
    font-size: 1rem;
    color: #aaa;
    border-bottom: 1px solid var(--glass-border);
    padding-bottom: 0.5rem;
  }
  
  .slider-setting {
    margin-left: 1.5rem;
  }
  
  input[type="range"] {
    width: 200px;
    accent-color: #4af;
  }

  input[type="color"] {
    width: 48px;
    height: 32px;
    border: 1px solid var(--glass-border);
    border-radius: 4px;
    background: transparent;
    cursor: pointer;
    padding: 2px;
  }

  input[type="color"]::-webkit-color-swatch-wrapper {
    padding: 0;
  }

  input[type="color"]::-webkit-color-swatch {
    border: none;
    border-radius: 2px;
  }

  .color-setting {
    margin-left: 0;
  }

  .color-picker-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .color-value {
    font-family: monospace;
    font-size: 0.9rem;
    color: #888;
  }
  
  input[type="checkbox"] { margin-right: 0.5rem; }
</style>
