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
  BACKGROUND_STATIC_COLOR: 'ui.background.static_color',
  BACKGROUND_DYNAMIC_LIBRARY: 'ui.background.dynamic_library',
  BACKGROUND_DYNAMIC_NOW_PLAYING: 'ui.background.dynamic_now_playing',
  BACKGROUND_DYNAMIC_ALBUM_DETAIL: 'ui.background.dynamic_album_detail',
  ACCENT_COLOR: 'ui.theme.accent_color',
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

// Background mode stores
export const bgStaticColor = writable<string>('#1a1a2e');
export const bgDynamicLibrary = writable<boolean>(true);
export const bgDynamicNowPlaying = writable<boolean>(true);
export const bgDynamicAlbumDetail = writable<boolean>(true);

// Accent/highlight color
export const accentColor = writable<string>('#4aafff');

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
    bgIntensityVal, bgNoiseOpacityVal, bgCrossfadeMsVal,
    bgStaticColorVal, bgDynamicLibraryVal, bgDynamicNowPlayingVal, bgDynamicAlbumDetailVal,
    accentColorVal
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
    getSetting(KEYS.BACKGROUND_STATIC_COLOR),
    getSetting(KEYS.BACKGROUND_DYNAMIC_LIBRARY),
    getSetting(KEYS.BACKGROUND_DYNAMIC_NOW_PLAYING),
    getSetting(KEYS.BACKGROUND_DYNAMIC_ALBUM_DETAIL),
    getSetting(KEYS.ACCENT_COLOR),
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

  // Background mode stores
  bgStaticColor.set(bgStaticColorVal || '#1a1a2e');
  bgDynamicLibrary.set(parseBool(bgDynamicLibraryVal, true));
  bgDynamicNowPlaying.set(parseBool(bgDynamicNowPlayingVal, true));
  bgDynamicAlbumDetail.set(parseBool(bgDynamicAlbumDetailVal, true));

  // Accent color
  accentColor.set(accentColorVal || '#4aafff');

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

export async function setBgStaticColor(value: string): Promise<void> {
  bgStaticColor.set(value);
  await setSetting(KEYS.BACKGROUND_STATIC_COLOR, value);
  applyAppearanceToCSS();
}

export async function setBgDynamicLibrary(value: boolean): Promise<void> {
  bgDynamicLibrary.set(value);
  await setSetting(KEYS.BACKGROUND_DYNAMIC_LIBRARY, value ? 'on' : 'off');
}

export async function setBgDynamicNowPlaying(value: boolean): Promise<void> {
  bgDynamicNowPlaying.set(value);
  await setSetting(KEYS.BACKGROUND_DYNAMIC_NOW_PLAYING, value ? 'on' : 'off');
}

export async function setBgDynamicAlbumDetail(value: boolean): Promise<void> {
  bgDynamicAlbumDetail.set(value);
  await setSetting(KEYS.BACKGROUND_DYNAMIC_ALBUM_DETAIL, value ? 'on' : 'off');
}

export async function setAccentColor(value: string): Promise<void> {
  accentColor.set(value);
  await setSetting(KEYS.ACCENT_COLOR, value);
  applyAppearanceToCSS();
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
    root.classList.add('reduce-effects');
    root.classList.remove('effects-blur', 'effects-glow', 'effects-border');
  } else {
    root.classList.remove('reduce-effects');
    
    // Apply individual toggles
    if (blur) {
      root.classList.add('effects-blur');
    } else {
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
  root.style.setProperty('--bg-static-color', get(bgStaticColor));
  
  // Apply accent color and extract RGB components
  const accent = get(accentColor);
  root.style.setProperty('--theme-accent', accent);
  const rgb = hexToRgb(accent);
  if (rgb) {
    root.style.setProperty('--theme-accent-r', String(rgb.r));
    root.style.setProperty('--theme-accent-g', String(rgb.g));
    root.style.setProperty('--theme-accent-b', String(rgb.b));
  }
}

// Helper to convert hex to RGB
function hexToRgb(hex: string): { r: number; g: number; b: number } | null {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result ? {
    r: parseInt(result[1], 16),
    g: parseInt(result[2], 16),
    b: parseInt(result[3], 16)
  } : null;
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
  bgStaticColor.set(settings['ui.background.static_color'] || '#1a1a2e');
  bgDynamicLibrary.set(settings['ui.background.dynamic_library'] !== 'off');
  bgDynamicNowPlaying.set(settings['ui.background.dynamic_now_playing'] !== 'off');
  bgDynamicAlbumDetail.set(settings['ui.background.dynamic_album_detail'] !== 'off');
  accentColor.set(settings['ui.theme.accent_color'] || '#4aafff');
  
  applyEffects();
}


