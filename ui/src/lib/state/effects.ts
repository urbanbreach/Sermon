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
  WAVEFORM_SEEKBAR: 'ui.bottombar.waveform_seekbar',
  WAVEFORM_COLOR: 'ui.bottombar.waveform_color',
  WAVEFORM_STYLE: 'ui.bottombar.waveform_style',
  PROVIDER_ITUNES: 'artwork.provider.itunes',
  PROVIDER_DEEZER: 'artwork.provider.deezer',
  SIDEBAR_VISIBLE: 'ui.sidebar.visible',
  ARTWORK_ROUNDED_SIDEBAR: 'ui.artwork.rounded_sidebar',
  ARTWORK_ROUNDED_ALBUMS: 'ui.artwork.rounded_albums',
  ARTWORK_ROUNDED_ALBUM_DETAIL: 'ui.artwork.rounded_album_detail',
  
  // Liquid Glass keys
  GLASS_MAIN_BLUR: 'ui.theme.glass.main_blur',
  GLASS_EDGE_BLUR: 'ui.theme.glass.edge_blur',
  GLASS_EDGE_WIDTH: 'ui.theme.glass.edge_width',
  GLASS_MAIN_BG: 'ui.theme.glass.main_bg',
  GLASS_EDGE_BG: 'ui.theme.glass.edge_bg',
  GLASS_SHEEN_BLUR: 'ui.theme.glass.sheen_blur',
  GLASS_SHEEN_BG: 'ui.theme.glass.sheen_bg',
  GLASS_SHEEN_WIDTH: 'ui.theme.glass.sheen_width',
  GLASS_EDGE_GRADIENT_WIDTH: 'ui.theme.glass.edge_gradient_width',
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
export const bottomBarWaveformSeekbar = writable<boolean>(false);
export const waveformColor = writable<string>('#4aafff');
export const bottomBarWaveformStyle = writable<'pills' | 'raw'>('pills');

// Artwork provider toggles
export const providerItunes = writable<boolean>(true);
export const providerDeezer = writable<boolean>(true);

// Layout and Artwork UI stores
export const sidebarVisible = writable<boolean>(true);
export const artworkRoundedSidebar = writable<boolean>(true);
export const artworkRoundedAlbums = writable<boolean>(true);
export const artworkRoundedAlbumDetail = writable<boolean>(true);

// Layout dimensions
export const sidebarWidth = writable<number>(220);
export const railWidth = writable<number>(300);
export const railSplitRatio = writable<number>(0.5);

// Liquid Glass Stores
export const glassMainBlur = writable<number>(22);
export const glassEdgeBlur = writable<number>(14);
export const glassEdgeWidth = writable<number>(22);
export const glassMainBg = writable<string>('rgba(20, 20, 24, 0.38)');
export const glassEdgeBg = writable<string>('rgba(255, 255, 255, 0.12)');
export const glassSheenBlur = writable<number>(18);
export const glassSheenBg = writable<string>('rgba(255, 255, 255, 0.22)');
export const glassSheenWidth = writable<number>(30);
export const glassEdgeGradientWidth = writable<number>(28);

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
    accentColorVal, waveformSeekbarVal, waveformColorVal, waveformStyleVal,
    sidebarVisibleVal, roundedSidebarVal, roundedAlbumsVal, roundedAlbumDetailVal,
    glassMainBlurVal, glassEdgeBlurVal, glassEdgeWidthVal, glassMainBgVal, glassEdgeBgVal,
    glassSheenBlurVal, glassSheenBgVal, glassSheenWidthVal, glassEdgeGradientWidthVal
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
    getSetting(KEYS.WAVEFORM_SEEKBAR),
    getSetting(KEYS.WAVEFORM_COLOR),
    getSetting(KEYS.WAVEFORM_STYLE),
    getSetting(KEYS.SIDEBAR_VISIBLE),
    getSetting(KEYS.ARTWORK_ROUNDED_SIDEBAR),
    getSetting(KEYS.ARTWORK_ROUNDED_ALBUMS),
    getSetting(KEYS.ARTWORK_ROUNDED_ALBUM_DETAIL),
    getSetting(KEYS.GLASS_MAIN_BLUR),
    getSetting(KEYS.GLASS_EDGE_BLUR),
    getSetting(KEYS.GLASS_EDGE_WIDTH),
    getSetting(KEYS.GLASS_MAIN_BG),
    getSetting(KEYS.GLASS_EDGE_BG),
    getSetting(KEYS.GLASS_SHEEN_BLUR),
    getSetting(KEYS.GLASS_SHEEN_BG),
    getSetting(KEYS.GLASS_SHEEN_WIDTH),
    getSetting(KEYS.GLASS_EDGE_GRADIENT_WIDTH),
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
  bottomBarWaveformSeekbar.set(parseBool(waveformSeekbarVal, false));
  waveformColor.set(waveformColorVal || get(accentColor));
  bottomBarWaveformStyle.set((waveformStyleVal as 'pills' | 'raw') || 'pills');

  // Layout and Artwork stores
  sidebarVisible.set(parseBool(sidebarVisibleVal, true));
  artworkRoundedSidebar.set(parseBool(roundedSidebarVal, true));
  artworkRoundedAlbums.set(parseBool(roundedAlbumsVal, true));
  artworkRoundedAlbumDetail.set(parseBool(roundedAlbumDetailVal, true));

  // Liquid Glass stores
  glassMainBlur.set(parseNum(glassMainBlurVal, 22));
  glassEdgeBlur.set(parseNum(glassEdgeBlurVal, 14));
  glassEdgeWidth.set(parseNum(glassEdgeWidthVal, 22));
  glassMainBg.set(glassMainBgVal || 'rgba(20, 20, 24, 0.38)');
  glassEdgeBg.set(glassEdgeBgVal || 'rgba(255, 255, 255, 0.12)');
  glassSheenBlur.set(parseNum(glassSheenBlurVal, 18));
  glassSheenBg.set(glassSheenBgVal || 'rgba(255, 255, 255, 0.22)');
  glassSheenWidth.set(parseNum(glassSheenWidthVal, 30));
  glassEdgeGradientWidth.set(parseNum(glassEdgeGradientWidthVal, 28));

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

export async function setBottomBarWaveformSeekbar(value: boolean): Promise<void> {
  bottomBarWaveformSeekbar.set(value);
  await setSetting(KEYS.WAVEFORM_SEEKBAR, value ? 'on' : 'off');
}

export async function setWaveformColor(value: string): Promise<void> {
  waveformColor.set(value);
  await setSetting(KEYS.WAVEFORM_COLOR, value);
}

export async function setBottomBarWaveformStyle(value: 'pills' | 'raw'): Promise<void> {
  bottomBarWaveformStyle.set(value);
  await setSetting(KEYS.WAVEFORM_STYLE, value);
}

export async function setSidebarVisible(value: boolean): Promise<void> {
  sidebarVisible.set(value);
  await setSetting(KEYS.SIDEBAR_VISIBLE, value ? 'on' : 'off');
}

export async function setArtworkRoundedSidebar(value: boolean): Promise<void> {
  artworkRoundedSidebar.set(value);
  await setSetting(KEYS.ARTWORK_ROUNDED_SIDEBAR, value ? 'on' : 'off');
  applyAppearanceToCSS();
}

export async function setArtworkRoundedAlbums(value: boolean): Promise<void> {
  artworkRoundedAlbums.set(value);
  await setSetting(KEYS.ARTWORK_ROUNDED_ALBUMS, value ? 'on' : 'off');
  applyAppearanceToCSS();
}

export async function setArtworkRoundedAlbumDetail(value: boolean): Promise<void> {
  artworkRoundedAlbumDetail.set(value);
  await setSetting(KEYS.ARTWORK_ROUNDED_ALBUM_DETAIL, value ? 'on' : 'off');
  applyAppearanceToCSS();
}

// Layout dimension setters
export function setSidebarWidth(value: number): void {
  sidebarWidth.set(value);
}

export function setRailWidth(value: number): void {
  railWidth.set(value);
}

export function setRailSplitRatio(value: number): void {
  railSplitRatio.set(value);
}

// Liquid Glass Setters
export async function setGlassMainBlur(value: number): Promise<void> {
  glassMainBlur.set(value);
  await setSetting(KEYS.GLASS_MAIN_BLUR, String(value));
  applyAppearanceToCSS();
}

export async function setGlassEdgeBlur(value: number): Promise<void> {
  glassEdgeBlur.set(value);
  await setSetting(KEYS.GLASS_EDGE_BLUR, String(value));
  applyAppearanceToCSS();
}

export async function setGlassEdgeWidth(value: number): Promise<void> {
  glassEdgeWidth.set(value);
  await setSetting(KEYS.GLASS_EDGE_WIDTH, String(value));
  applyAppearanceToCSS();
}

export async function setGlassMainBg(value: string): Promise<void> {
  glassMainBg.set(value);
  await setSetting(KEYS.GLASS_MAIN_BG, value);
  applyAppearanceToCSS();
}

export async function setGlassEdgeBg(value: string): Promise<void> {
  glassEdgeBg.set(value);
  await setSetting(KEYS.GLASS_EDGE_BG, value);
  applyAppearanceToCSS();
}

export async function setGlassSheenBlur(value: number): Promise<void> {
  glassSheenBlur.set(value);
  await setSetting(KEYS.GLASS_SHEEN_BLUR, String(value));
  applyAppearanceToCSS();
}

export async function setGlassSheenBg(value: string): Promise<void> {
  glassSheenBg.set(value);
  await setSetting(KEYS.GLASS_SHEEN_BG, value);
  applyAppearanceToCSS();
}

export async function setGlassSheenWidth(value: number): Promise<void> {
  glassSheenWidth.set(value);
  await setSetting(KEYS.GLASS_SHEEN_WIDTH, String(value));
  applyAppearanceToCSS();
}

export async function setGlassEdgeGradientWidth(value: number): Promise<void> {
  glassEdgeGradientWidth.set(value);
  await setSetting(KEYS.GLASS_EDGE_GRADIENT_WIDTH, String(value));
  applyAppearanceToCSS();
}

export function applyEffects(): void {
  applyAppearanceToCSS();
}

export function applyAppearanceToCSS(): void {
  const root = document.documentElement;
  
  const accent = get(accentColor);
  root.style.setProperty('--theme-accent', accent);
  const rgb = hexToRgb(accent);
  if (rgb) {
    root.style.setProperty('--theme-accent-r', String(rgb.r));
    root.style.setProperty('--theme-accent-g', String(rgb.g));
    root.style.setProperty('--theme-accent-b', String(rgb.b));
  }
  
  root.style.setProperty('--artwork-radius-sidebar', get(artworkRoundedSidebar) ? '6px' : '0');
  root.style.setProperty('--artwork-radius-albums', get(artworkRoundedAlbums) ? '10px' : '0');
  root.style.setProperty('--artwork-radius-album-detail', get(artworkRoundedAlbumDetail) ? '12px' : '0');
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
  bottomBarWaveformSeekbar.set(settings['ui.bottombar.waveform_seekbar'] === 'on');
  waveformColor.set(settings['ui.bottombar.waveform_color'] || get(accentColor));
  bottomBarWaveformStyle.set((settings['ui.bottombar.waveform_style'] as 'pills' | 'raw') || 'pills');
  sidebarVisible.set(settings['ui.sidebar.visible'] !== 'off');
  artworkRoundedSidebar.set(settings['ui.artwork.rounded_sidebar'] !== 'off');
  artworkRoundedAlbums.set(settings['ui.artwork.rounded_albums'] !== 'off');
  artworkRoundedAlbumDetail.set(settings['ui.artwork.rounded_album_detail'] !== 'off');
  
  // Liquid Glass
  glassMainBlur.set(parseNum(settings['ui.theme.glass.main_blur'], 22));
  glassEdgeBlur.set(parseNum(settings['ui.theme.glass.edge_blur'], 14));
  glassEdgeWidth.set(parseNum(settings['ui.theme.glass.edge_width'], 22));
  glassMainBg.set(settings['ui.theme.glass.main_bg'] || 'rgba(20, 20, 24, 0.38)');
  glassEdgeBg.set(settings['ui.theme.glass.edge_bg'] || 'rgba(255, 255, 255, 0.12)');
  glassSheenBlur.set(parseNum(settings['ui.theme.glass.sheen_blur'], 18));
  glassSheenBg.set(settings['ui.theme.glass.sheen_bg'] || 'rgba(255, 255, 255, 0.22)');
  glassSheenWidth.set(parseNum(settings['ui.theme.glass.sheen_width'], 30));
  glassEdgeGradientWidth.set(parseNum(settings['ui.theme.glass.edge_gradient_width'], 28));

  applyEffects();
}


