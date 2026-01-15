import { writable, derived, get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import type { TrackRow, LibraryFolder, ScanProgress, ScanComplete, SortBy, SortDirection } from '../types/library';
import * as api from '../api/library';
import { Fixtures } from '../data/fixtures';

const isMock = import.meta.env.SERMON_MOCK === '1';

// Core state
export const folders = writable<LibraryFolder[]>([]);
export const selectedFolderId = writable<number | null>(null);
export const tracks = writable<TrackRow[]>([]);
export const sortBy = writable<SortBy>('title');
export const sortDirection = writable<SortDirection>('asc');

// Scan state
export const scanStatus = writable<'idle' | 'scanning'>('idle');
export const scanProgress = writable<ScanProgress>({ scanned: 0, total: 0 });
export const scanError = writable<string | null>(null);

// Derived
export const selectedFolder = derived(
  [folders, selectedFolderId],
  ([$folders, $selectedId]) => $folders.find(f => f.id === $selectedId) || null
);

// Actions
export async function loadFolders(): Promise<void> {
  if (isMock) return;
  
  try {
    const result = await api.listFolders();
    folders.set(result);
    
    // Auto-select first folder if none selected
    const current = get(selectedFolderId);
    if (current === null && result.length > 0) {
      selectedFolderId.set(result[0].id);
    }
  } catch (e) {
    console.error('Failed to load folders:', e);
  }
}

export async function loadTracks(): Promise<void> {
  if (isMock) {
    // Use fixtures
    const fixtureTracks = Fixtures.getTracks();
    tracks.set(fixtureTracks.map((t, i) => ({
      id: i,
      library_folder_id: 1,
      path: `/mock/${t.title}.flac`,
      title: t.title,
      artist: Fixtures.getArtist(t.artistId)?.name,
      album: Fixtures.getAlbum(t.albumId)?.title,
      duration_ms: t.durationMs,
      is_missing: false,
    } as TrackRow)));
    return;
  }
  
  try {
    const sb = get(sortBy);
    const dir = get(sortDirection);
    const result = await api.listTracks(sb, dir);
    tracks.set(result);
  } catch (e) {
    console.error('Failed to load tracks:', e);
  }
}

export async function addFolder(path: string): Promise<void> {
  if (isMock) return;
  
  try {
    scanError.set(null);
    const folder = await api.addFolder(path);
    folders.update(f => [...f, folder]);
    selectedFolderId.set(folder.id);
    
    // Start scan
    await startScan(folder.path);
  } catch (e) {
    scanError.set(String(e));
    console.error('Failed to add folder:', e);
  }
}

export async function startScan(path?: string): Promise<void> {
  if (isMock) return;
  
  const folder = get(selectedFolder);
  const scanPath = path || folder?.path;
  
  if (!scanPath) {
    scanError.set('No folder selected');
    return;
  }
  
  try {
    scanError.set(null);
    scanStatus.set('scanning');
    scanProgress.set({ scanned: 0, total: 0 });
    await api.startScan(scanPath);
  } catch (e) {
    scanStatus.set('idle');
    scanError.set(String(e));
    console.error('Failed to start scan:', e);
  }
}

export function toggleSortDirection(): void {
  sortDirection.update(d => d === 'asc' ? 'desc' : 'asc');
  loadTracks();
}

export function setSortBy(field: SortBy): void {
  sortBy.set(field);
  loadTracks();
}

// Event listeners (call once on app init)
export function initEventListeners(): void {
  if (isMock) return;
  
  listen<ScanProgress>('evt_scan_progress', (event) => {
    scanProgress.set(event.payload);
  });
  
  listen<ScanComplete>('evt_scan_complete', (event) => {
    scanStatus.set('idle');
    scanProgress.set({ scanned: event.payload.scanned, total: event.payload.total });
    
    if (event.payload.errors > 0) {
      scanError.set(`Scan completed with ${event.payload.errors} errors`);
    }
    
    // Reload tracks after scan
    loadTracks();
  });
}

// Initialize
export function initLibrary(): void {
  initEventListeners();
  loadFolders();
  loadTracks();
}
