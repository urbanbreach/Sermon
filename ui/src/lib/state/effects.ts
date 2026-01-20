/**
 * UI Effects Settings Store
 * 
 * Manages settings for reduce motion/transparency and theme effects.
 */

import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Settings keys
const KEYS = {
  REDUCE_EFFECTS: 'ui.reduce_effects',
  THEME_BLUR: 'ui.theme.blur',
  THEME_GLOW: 'ui.theme.glow',
  THEME_BORDER_HIGHLIGHT: 'ui.theme.border_highlight',
  THEME_BLUR_PX: 'ui.theme.blur_px',
  THEME_GLOW_STRENGTH: 'ui.theme.glow_strength',
  THEME_BORDER_STRENGTH: 'ui.theme.border_strength',
  BACKGROUND_INTENSITY: 'ui.background.intensity',
  BACKGROUND_NOISE_OPACITY: 'ui.background.noise_opacity',
  BACKGROUND_CROSSFADE_MS: 'ui.background.crossfade_ms',
  PROVIDER_ITUNES: 'artwork.provider.itunes',
  PROVIDER_DEEZER: 'artwork.provider.deezer',
} as const;

// Stores - Boolean toggles
export const reduceEffects = writable<boolean>(false);
export const themeBlur = writable<boolean>(true);
export const themeGlow = writable<boolean>(true);
export const themeBorderHighlight = writable<boolean>(true);

// Stores - Numeric sliders
export const blurPx = writable<number>(16);
export const glowStrength = writable<number>(0.35);
export const borderStrength = writable<number>(0.2);
export const bgIntensity = writable<number>(0.35);
export const bgNoiseOpacity = writable<number>(0.18);
export const bgCrossfadeMs = writable<number>(1200);

// Artwork provider toggles
export const providerItunes = writable<boolean>(true);
export const providerDeezer = writable<boolean>(true);

// API helpers
async function getSetting(key: string): Promise<string | null> {
  try {
    const response = await invoke<{ value: string | null }>('cmd_settings_get', { request: { key } });
    return response.value;
  } catch (e) {
    console.error(`Failed to get setting ${key}:`, e);
    return null;
  }
}

async function setSetting(key: string, value: string): Promise<void> {
  try {
    await invoke('cmd_settings_set', { request: { key, value } });
  } catch (e) {
    console.error(`Failed to set setting ${key}:`, e);
  }
}

function parseNum(value: string | null, defaultValue: number): number {
  if (value === null) return defaultValue;
  const parsed = parseFloat(value);
  return isNaN(parsed) ? defaultValue : parsed;
}

function parseBool(value: string | null, defaultValue: boolean): boolean {
  if (value === null) return defaultValue;
  return value === 'on';
}

// Load all settings from DB
export async function loadEffectsSettings(): Promise<void> {
  const [
    reduce, blur, glow, border, itunes, deezer,
    blurPxVal, glowStrengthVal, borderStrengthVal,
    bgIntensityVal, bgNoiseOpacityVal, bgCrossfadeMsVal
  ] = await Promise.all([
    getSetting(KEYS.REDUCE_EFFECTS),
    getSetting(KEYS.THEME_BLUR),
    getSetting(KEYS.THEME_GLOW),
    getSetting(KEYS.THEME_BORDER_HIGHLIGHT),
    getSetting(KEYS.PROVIDER_ITUNES),
    getSetting(KEYS.PROVIDER_DEEZER),
    getSetting(KEYS.THEME_BLUR_PX),
    getSetting(KEYS.THEME_GLOW_STRENGTH),
    getSetting(KEYS.THEME_BORDER_STRENGTH),
    getSetting(KEYS.BACKGROUND_INTENSITY),
    getSetting(KEYS.BACKGROUND_NOISE_OPACITY),
    getSetting(KEYS.BACKGROUND_CROSSFADE_MS),
  ]);

  reduceEffects.set(parseBool(reduce, false));
  themeBlur.set(parseBool(blur, true));
  themeGlow.set(parseBool(glow, true));
  themeBorderHighlight.set(parseBool(border, true));
  providerItunes.set(parseBool(itunes, true));
  providerDeezer.set(parseBool(deezer, true));

  // Numeric stores
  blurPx.set(parseNum(blurPxVal, 16));
  glowStrength.set(parseNum(glowStrengthVal, 0.35));
  borderStrength.set(parseNum(borderStrengthVal, 0.2));
  bgIntensity.set(parseNum(bgIntensityVal, 0.35));
  bgNoiseOpacity.set(parseNum(bgNoiseOpacityVal, 0.18));
  bgCrossfadeMs.set(parseNum(bgCrossfadeMsVal, 1200));

  // Apply effects immediately
  applyEffects();
}

// Save and apply a setting
export async function setReduceEffects(value: boolean): Promise<void> {
  reduceEffects.set(value);
  await setSetting(KEYS.REDUCE_EFFECTS, value ? 'on' : 'off');
  applyEffects();
}

export async function setThemeBlur(value: boolean): Promise<void> {
  themeBlur.set(value);
  await setSetting(KEYS.THEME_BLUR, value ? 'on' : 'off');
  applyEffects();
}

export async function setThemeGlow(value: boolean): Promise<void> {
  themeGlow.set(value);
  await setSetting(KEYS.THEME_GLOW, value ? 'on' : 'off');
  applyEffects();
}

export async function setThemeBorderHighlight(value: boolean): Promise<void> {
  themeBorderHighlight.set(value);
  await setSetting(KEYS.THEME_BORDER_HIGHLIGHT, value ? 'on' : 'off');
  applyEffects();
}

export async function setProviderItunes(value: boolean): Promise<void> {
  providerItunes.set(value);
  await setSetting(KEYS.PROVIDER_ITUNES, value ? 'on' : 'off');
}

export async function setProviderDeezer(value: boolean): Promise<void> {
  providerDeezer.set(value);
  await setSetting(KEYS.PROVIDER_DEEZER, value ? 'on' : 'off');
}

// Apply effects to document based on current settings
export function applyEffects(): void {
  const root = document.documentElement;
  const reduce = get(reduceEffects);
  const blur = get(themeBlur);
  const glow = get(themeGlow);
  const border = get(themeBorderHighlight);

  // When reduce_effects is on, disable all effects
  if (reduce) {
    root.style.setProperty('--glass-blur', '0px');
    root.classList.add('reduce-effects');
    root.classList.remove('effects-blur', 'effects-glow', 'effects-border');
  } else {
    root.classList.remove('reduce-effects');
    
    // Apply individual toggles
    if (blur) {
      root.style.setProperty('--glass-blur', '16px');
      root.classList.add('effects-blur');
    } else {
      root.style.setProperty('--glass-blur', '0px');
      root.classList.remove('effects-blur');
    }

    if (glow) {
      root.classList.add('effects-glow');
    } else {
      root.classList.remove('effects-glow');
    }

    if (border) {
      root.classList.add('effects-border');
    } else {
      root.classList.remove('effects-border');
    }
  }

  // Apply numeric CSS variables
  applyAppearanceToCSS();
}

// Apply appearance settings to CSS custom properties
export function applyAppearanceToCSS(): void {
  const root = document.documentElement;
  
  root.style.setProperty('--blur-px', `${get(blurPx)}px`);
  root.style.setProperty('--glow-strength', String(get(glowStrength)));
  root.style.setProperty('--border-strength', String(get(borderStrength)));
  root.style.setProperty('--bg-intensity', String(get(bgIntensity)));
  root.style.setProperty('--bg-noise-opacity', String(get(bgNoiseOpacity)));
  root.style.setProperty('--bg-crossfade-ms', String(get(bgCrossfadeMs)));
}

// Sync appearance settings from preferences store to effects stores
export function syncAppearanceToEffects(settings: Record<string, string>): void {
  reduceEffects.set(settings['ui.reduce_effects'] === 'on');
  themeBlur.set(settings['ui.theme.blur'] === 'on');
  themeGlow.set(settings['ui.theme.glow'] === 'on');
  themeBorderHighlight.set(settings['ui.theme.border_highlight'] === 'on');
  blurPx.set(parseNum(settings['ui.theme.blur_px'], 16));
  glowStrength.set(parseNum(settings['ui.theme.glow_strength'], 0.35));
  borderStrength.set(parseNum(settings['ui.theme.border_strength'], 0.2));
  bgIntensity.set(parseNum(settings['ui.background.intensity'], 0.35));
  bgNoiseOpacity.set(parseNum(settings['ui.background.noise_opacity'], 0.18));
  bgCrossfadeMs.set(parseNum(settings['ui.background.crossfade_ms'], 1200));
  
  applyEffects();
}

// Reset effects to defaults
export function resetEffectsDefaults(): void {
  reduceEffects.set(false);
  themeBlur.set(true);
  themeGlow.set(true);
  themeBorderHighlight.set(true);
  blurPx.set(16);
  glowStrength.set(0.35);
  borderStrength.set(0.2);
  bgIntensity.set(0.35);
  bgNoiseOpacity.set(0.18);
  bgCrossfadeMs.set(1200);
  
  applyEffects();
}
