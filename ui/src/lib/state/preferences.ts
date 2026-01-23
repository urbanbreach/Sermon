/**
 * Preferences State Management
 * 
 * Manages category-based preference settings via backend API.
 * Uses the new cmd_settings_get_category/set_category/reset_category commands.
 */

import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Types for each category's settings
export interface PlayerSettings {
  'player.buffer_size_ms': string;
  'player.preload_next': string;
}

export interface LibrarySettings {
  'library.scan_on_startup': string;
}

export interface InternetSettings {
}

export interface AppearanceSettings {
  'ui.reduce_effects': string;
  'ui.theme.blur': string;
  'ui.theme.glow': string;
  'ui.theme.border_highlight': string;
  'ui.theme.blur_px': string;
  'ui.theme.glow_strength': string;
  'ui.theme.border_strength': string;
  'ui.background.intensity': string;
  'ui.background.noise_opacity': string;
  'ui.background.crossfade_ms': string;
}

export type PreferenceCategory = 'player' | 'library' | 'internet' | 'appearance';

export type CategorySettings = 
  | PlayerSettings 
  | LibrarySettings 
  | InternetSettings 
  | AppearanceSettings;

// Stores for each category
export const playerSettings = writable<PlayerSettings | null>(null);
export const librarySettings = writable<LibrarySettings | null>(null);
export const internetSettings = writable<InternetSettings | null>(null);
export const appearanceSettings = writable<AppearanceSettings | null>(null);

// Loading state
export const preferencesLoading = writable<boolean>(false);
export const preferencesError = writable<string | null>(null);

// Helper: Parse boolean from 'on'/'off' string
export function parseBool(value: string | undefined | null): boolean {
  return value === 'on';
}


// Load settings for a category
export async function loadCategorySettings(category: PreferenceCategory): Promise<void> {
  preferencesLoading.set(true);
  preferencesError.set(null);
  
  try {
    const settings = await invoke<Record<string, string>>('cmd_settings_get_category', { category });
    
    switch (category) {
      case 'player':
        playerSettings.set(settings as unknown as PlayerSettings);
        break;
      case 'library':
        librarySettings.set(settings as unknown as LibrarySettings);
        break;
      case 'internet':
        internetSettings.set(settings as unknown as InternetSettings);
        break;
      case 'appearance':
        appearanceSettings.set(settings as unknown as AppearanceSettings);
        break;
    }
  } catch (e) {
    console.error(`Failed to load ${category} settings:`, e);
    preferencesError.set(`Failed to load ${category} settings`);
  } finally {
    preferencesLoading.set(false);
  }
}

// Save a single setting within a category
export async function saveCategorySetting(
  category: PreferenceCategory,
  key: string,
  value: string
): Promise<void> {
  try {
    await invoke('cmd_settings_set_category', {
      category,
      settings: { [key]: value }
    });
    
    // Update local store
    await loadCategorySettings(category);
  } catch (e) {
    console.error(`Failed to save setting ${key}:`, e);
    preferencesError.set(`Failed to save setting`);
  }
}

// Reset a category to defaults
export async function resetCategoryToDefaults(category: PreferenceCategory): Promise<void> {
  try {
    await invoke('cmd_settings_reset_category', { category });
    // Reload to get fresh defaults
    await loadCategorySettings(category);
  } catch (e) {
    console.error(`Failed to reset ${category} settings:`, e);
    preferencesError.set(`Failed to reset ${category} settings`);
  }
}

// Load all categories at once (for initial load)
export async function loadAllPreferences(): Promise<void> {
  const categories: PreferenceCategory[] = [
    'player', 'library', 'internet', 'appearance'
  ];
  
  await Promise.all(categories.map(cat => loadCategorySettings(cat)));
}
