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
  PROVIDER_ITUNES: 'artwork.provider.itunes',
  PROVIDER_DEEZER: 'artwork.provider.deezer',
} as const;

// Stores
export const reduceEffects = writable<boolean>(false);
export const themeBlur = writable<boolean>(true);
export const themeGlow = writable<boolean>(true);
export const themeBorderHighlight = writable<boolean>(true);

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

function parseBool(value: string | null, defaultValue: boolean): boolean {
  if (value === null) return defaultValue;
  return value === 'on';
}

// Load all settings from DB
export async function loadEffectsSettings(): Promise<void> {
  const [reduce, blur, glow, border, itunes, deezer] = await Promise.all([
    getSetting(KEYS.REDUCE_EFFECTS),
    getSetting(KEYS.THEME_BLUR),
    getSetting(KEYS.THEME_GLOW),
    getSetting(KEYS.THEME_BORDER_HIGHLIGHT),
    getSetting(KEYS.PROVIDER_ITUNES),
    getSetting(KEYS.PROVIDER_DEEZER),
  ]);

  reduceEffects.set(parseBool(reduce, false));
  themeBlur.set(parseBool(blur, true));
  themeGlow.set(parseBool(glow, true));
  themeBorderHighlight.set(parseBool(border, true));
  providerItunes.set(parseBool(itunes, true));
  providerDeezer.set(parseBool(deezer, true));

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
}
