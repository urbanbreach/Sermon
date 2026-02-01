<script lang="ts">
  import { onMount } from 'svelte';
  import { 
    reduceEffects, themeBlur, themeGlow, themeBorderHighlight,
    blurPx, glowStrength, borderStrength, bgIntensity, bgNoiseOpacity, bgCrossfadeMs,
    bgStaticColor, bgDynamicLibrary, bgDynamicNowPlaying, bgDynamicAlbumDetail,
    accentColor, sidebarVisible, waveformColor, bottomBarWaveformStyle,
    artworkRoundedSidebar, artworkRoundedAlbums, artworkRoundedAlbumDetail,
    bottomBarWaveformSeekbar,
    setReduceEffects, setThemeBlur, setThemeGlow, setThemeBorderHighlight,
    setBgStaticColor, setBgDynamicLibrary, setBgDynamicNowPlaying, setBgDynamicAlbumDetail,
    setAccentColor, setSidebarVisible, setWaveformColor, setBottomBarWaveformStyle,
    setArtworkRoundedSidebar, setArtworkRoundedAlbums, setArtworkRoundedAlbumDetail,
    setBottomBarWaveformSeekbar,
    glassMainBlur, glassEdgeBlur, glassEdgeWidth, glassMainBg, glassEdgeBg,
    glassSheenBlur, glassSheenBg, glassSheenWidth, glassEdgeGradientWidth,
    setGlassMainBlur, setGlassEdgeBlur, setGlassEdgeWidth, setGlassMainBg, setGlassEdgeBg,
    setGlassSheenBlur, setGlassSheenBg, setGlassSheenWidth, setGlassEdgeGradientWidth,
    applyEffects, loadEffectsSettings
  } from '../../state/effects';
  import { loadCategorySettings, saveCategorySetting } from '../../state/preferences';
  import SettingGroup from '../primitives/SettingGroup.svelte';
  import SettingRow from '../primitives/SettingRow.svelte';
  
  const isMock = import.meta.env.SERMON_MOCK === '1';

  function rgbaToHex(rgba: string): string {
    const result = /rgba?\((\d+),\s*(\d+),\s*(\d+)/.exec(rgba);
    if (result) {
      const r = parseInt(result[1]).toString(16).padStart(2, '0');
      const g = parseInt(result[2]).toString(16).padStart(2, '0');
      const b = parseInt(result[3]).toString(16).padStart(2, '0');
      return `#${r}${g}${b}`;
    }
    return '#000000';
  }

  function hexToRgba(hex: string, alpha: number): string {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    if (result) {
      const r = parseInt(result[1], 16);
      const g = parseInt(result[2], 16);
      const b = parseInt(result[3], 16);
      return `rgba(${r}, ${g}, ${b}, ${alpha})`;
    }
    return `rgba(0, 0, 0, ${alpha})`;
  }
  
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
  <div class="setting-group">
    <h3>Layout</h3>
    
    <div class="setting">
      <label>
        <input 
          type="checkbox" 
          checked={$bottomBarWaveformSeekbar}
          onchange={(e) => setBottomBarWaveformSeekbar(e.currentTarget.checked)}
          disabled={isMock}
        />
        Waveform Seekbar
      </label>
      <span class="setting-hint">Use waveform visualization in bottom bar progress (MusicBee-style)</span>
    </div>

    {#if $bottomBarWaveformSeekbar}
      <div class="setting color-setting" style="margin-left: 1.5rem; margin-top: -0.5rem; margin-bottom: 0.5rem;">
        <label for="waveform-color">Waveform Color</label>
        <div class="color-picker-row">
          <input 
            type="color" 
            id="waveform-color"
            value={$waveformColor}
            onchange={(e) => setWaveformColor(e.currentTarget.value)}
            disabled={isMock}
          />
          <span class="color-value">{$waveformColor}</span>
        </div>
        <span class="setting-hint">Color of the played portion in waveform seekbar</span>
      </div>

      <div class="setting style-setting" style="margin-left: 1.5rem; margin-top: 0.5rem; margin-bottom: 0.5rem;">
        <span class="setting-label">Waveform Style</span>
        <div class="radio-group">
          <label class="radio-label">
            <input 
              type="radio" 
              name="waveform-style" 
              value="pills" 
              checked={$bottomBarWaveformStyle === 'pills'} 
              onchange={() => setBottomBarWaveformStyle('pills')}
              disabled={isMock}
            />
            Pills
          </label>
          <label class="radio-label">
            <input 
              type="radio" 
              name="waveform-style" 
              value="raw" 
              checked={$bottomBarWaveformStyle === 'raw'} 
              onchange={() => setBottomBarWaveformStyle('raw')}
              disabled={isMock}
            />
            Raw
          </label>
        </div>
        <span class="setting-hint">Pills uses rounded bars; Raw uses thin vertical lines</span>
      </div>
    {/if}

    <div class="setting">
      <label>
        <input 
          type="checkbox" 
          checked={$sidebarVisible}
          onchange={(e) => setSidebarVisible(e.currentTarget.checked)}
          disabled={isMock}
        />
        Show Sidebar
      </label>
      <span class="setting-hint">When hidden, navigation moves to the top bar</span>
    </div>
  </div>

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

    <SettingGroup title="Cover Art">
      <SettingRow label="Rounded corners in sidebar" description="Apply rounded corners to artwork in the sidebar and Now Playing">
        <input type="checkbox" checked={$artworkRoundedSidebar} onchange={(e) => setArtworkRoundedSidebar(e.currentTarget.checked)} disabled={isMock} />
      </SettingRow>
      <SettingRow label="Rounded corners on album grid" description="Apply rounded corners to album artwork in grid views">
        <input type="checkbox" checked={$artworkRoundedAlbums} onchange={(e) => setArtworkRoundedAlbums(e.currentTarget.checked)} disabled={isMock} />
      </SettingRow>
      <SettingRow label="Rounded corners on album detail" description="Apply rounded corners to artwork on album detail pages">
        <input type="checkbox" checked={$artworkRoundedAlbumDetail} onchange={(e) => setArtworkRoundedAlbumDetail(e.currentTarget.checked)} disabled={isMock} />
      </SettingRow>
    </SettingGroup>

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

    <!-- Liquid Glass Bar Settings -->
    <div class="setting-group">
      <h3>Liquid Glass Bar</h3>
      
      <div class="setting slider-setting">
        <label for="glass-main-blur">Main Blur: {$glassMainBlur}px</label>
        <input type="range" id="glass-main-blur" min="0" max="40" step="1" 
               value={$glassMainBlur}
               oninput={(e) => setGlassMainBlur(parseFloat(e.currentTarget.value))} 
               disabled={isMock} />
      </div>

      <div class="setting slider-setting">
        <label for="glass-edge-blur">Edge Blur: {$glassEdgeBlur}px</label>
        <input type="range" id="glass-edge-blur" min="0" max="40" step="1" 
               value={$glassEdgeBlur}
               oninput={(e) => setGlassEdgeBlur(parseFloat(e.currentTarget.value))}
               disabled={isMock} />
      </div>

      <div class="setting slider-setting">
        <label for="glass-edge-width">Edge Width: {$glassEdgeWidth}px</label>
        <input type="range" id="glass-edge-width" min="0" max="100" step="1" 
               value={$glassEdgeWidth}
               oninput={(e) => setGlassEdgeWidth(parseFloat(e.currentTarget.value))}
               disabled={isMock} />
      </div>

      <div class="setting slider-setting">
        <label for="glass-sheen-blur">Sheen Blur: {$glassSheenBlur}px</label>
        <input type="range" id="glass-sheen-blur" min="0" max="40" step="1" 
               value={$glassSheenBlur}
               oninput={(e) => setGlassSheenBlur(parseFloat(e.currentTarget.value))}
               disabled={isMock} />
      </div>

      <div class="setting slider-setting">
        <label for="glass-sheen-width">Sheen Width: {$glassSheenWidth}px</label>
        <input type="range" id="glass-sheen-width" min="0" max="100" step="1" 
               value={$glassSheenWidth}
               oninput={(e) => setGlassSheenWidth(parseFloat(e.currentTarget.value))}
               disabled={isMock} />
      </div>

      <div class="setting slider-setting">
        <label for="glass-edge-grad-width">Edge Gradient Width: {$glassEdgeGradientWidth}px</label>
        <input type="range" id="glass-edge-grad-width" min="0" max="100" step="1" 
               value={$glassEdgeGradientWidth}
               oninput={(e) => setGlassEdgeGradientWidth(parseFloat(e.currentTarget.value))}
               disabled={isMock} />
      </div>
      
      <div class="setting color-setting">
        <label for="glass-main-bg">Main Background</label>
        <div class="color-picker-row">
          <input type="color" id="glass-main-bg" value={rgbaToHex($glassMainBg)} 
                 onchange={(e) => setGlassMainBg(hexToRgba(e.currentTarget.value, 0.38))} 
                 disabled={isMock} />
          <span class="color-value">{$glassMainBg}</span>
        </div>
      </div>

      <div class="setting color-setting">
        <label for="glass-edge-bg">Edge Background</label>
        <div class="color-picker-row">
          <input type="color" id="glass-edge-bg" value={rgbaToHex($glassEdgeBg)} 
                 onchange={(e) => setGlassEdgeBg(hexToRgba(e.currentTarget.value, 0.12))} 
                 disabled={isMock} />
          <span class="color-value">{$glassEdgeBg}</span>
        </div>
      </div>

      <div class="setting color-setting">
        <label for="glass-sheen-bg">Sheen Background</label>
        <div class="color-picker-row">
          <input type="color" id="glass-sheen-bg" value={rgbaToHex($glassSheenBg)} 
                 onchange={(e) => setGlassSheenBg(hexToRgba(e.currentTarget.value, 0.22))} 
                 disabled={isMock} />
          <span class="color-value">{$glassSheenBg}</span>
        </div>
      </div>
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
  .setting label, .setting-label { color: #ccc; }
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
  
  .radio-group {
    display: flex;
    gap: 1.5rem;
    margin-top: 0.25rem;
  }

  .radio-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    font-size: 0.9rem;
    color: #ccc;
  }

  .radio-label input[type="radio"] {
    margin: 0;
    accent-color: #4af;
  }
  
  input[type="checkbox"] { margin-right: 0.5rem; }
</style>
