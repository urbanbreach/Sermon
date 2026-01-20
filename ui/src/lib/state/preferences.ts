/**
 * Preferences State Management
 * 
 * Manages category-based preference settings via backend API.
 * Uses the new cmd_settings_get_category/set_category/reset_category commands.
 */

import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Types for each category's settings
export interface GeneralSettings {
  'general.startup.with_windows': string;
  'general.startup.minimized': string;
}

export interface PlayerSettings {
  'player.buffer_size_ms': string;
  'player.preload_next': string;
}

export interface NowPlayingSettings {
  'nowplaying.double_click': string;
  'nowplaying.queue_add_position': string;
  'nowplaying.shuffle_mode': string;
}

export interface LibrarySettings {
  'library.scan_on_startup': string;
  'library.continuous_monitoring': string;
}

export interface TagsSettings {
  'tags.backup_before_write': string;
  'tags.write_behavior': string;
}

export interface InternetSettings {
  'internet.lastfm_enabled': string;
}

export interface DevicesSettings {
  'devices.dsd_dop_enabled': string;
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

export type PreferenceCategory = 'general' | 'player' | 'nowplaying' | 'library' | 'tags' | 'internet' | 'devices' | 'appearance';

export type CategorySettings = 
  | GeneralSettings 
  | PlayerSettings 
  | NowPlayingSettings 
  | LibrarySettings 
  | TagsSettings 
  | InternetSettings 
  | DevicesSettings
  | AppearanceSettings;

// Stores for each category
export const generalSettings = writable<GeneralSettings | null>(null);
export const playerSettings = writable<PlayerSettings | null>(null);
export const nowPlayingSettings = writable<NowPlayingSettings | null>(null);
export const librarySettings = writable<LibrarySettings | null>(null);
export const tagsSettings = writable<TagsSettings | null>(null);
export const internetSettings = writable<InternetSettings | null>(null);
export const devicesSettings = writable<DevicesSettings | null>(null);
export const appearanceSettings = writable<AppearanceSettings | null>(null);

// Loading state
export const preferencesLoading = writable<boolean>(false);
export const preferencesError = writable<string | null>(null);

// Helper: Parse boolean from 'on'/'off' string
export function parseBool(value: string | undefined | null): boolean {
  return value === 'on';
}

// Helper: Convert boolean to 'on'/'off' string
export function toBoolString(value: boolean): string {
  return value ? 'on' : 'off';
}

// Load settings for a category
export async function loadCategorySettings(category: PreferenceCategory): Promise<void> {
  preferencesLoading.set(true);
  preferencesError.set(null);
  
  try {
    const settings = await invoke<Record<string, string>>('cmd_settings_get_category', { category });
    
    switch (category) {
      case 'general':
        generalSettings.set(settings as unknown as GeneralSettings);
        break;
      case 'player':
        playerSettings.set(settings as unknown as PlayerSettings);
        break;
      case 'nowplaying':
        nowPlayingSettings.set(settings as unknown as NowPlayingSettings);
        break;
      case 'library':
        librarySettings.set(settings as unknown as LibrarySettings);
        break;
      case 'tags':
        tagsSettings.set(settings as unknown as TagsSettings);
        break;
      case 'internet':
        internetSettings.set(settings as unknown as InternetSettings);
        break;
      case 'devices':
        devicesSettings.set(settings as unknown as DevicesSettings);
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
    'general', 'player', 'nowplaying', 'library', 'tags', 'internet', 'devices', 'appearance'
  ];
  
  await Promise.all(categories.map(cat => loadCategorySettings(cat)));
}
