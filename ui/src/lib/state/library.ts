import { writable, derived, get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import type { TrackRow, LibraryFolder, ScanProgress, ScanComplete, QuickScanComplete, SortBy, SortDirection, FolderOptions } from '../types/library';
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
      libraryFolderId: 1,
      path: `/mock/${t.title}.flac`,
      title: t.title,
      artist: Fixtures.getArtist(t.artistId)?.name,
      album: Fixtures.getAlbum(t.albumId)?.title,
      durationMs: t.durationMs,
      isMissing: false,
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

export async function removeFolder(folderId: number): Promise<void> {
  if (isMock) return;
  
  try {
    await api.removeFolder(folderId);
    folders.update(f => f.filter(folder => folder.id !== folderId));
    
    const currentSelected = get(selectedFolderId);
    if (currentSelected === folderId) {
      const remaining = get(folders);
      selectedFolderId.set(remaining.length > 0 ? remaining[0].id : null);
    }
    
    await loadTracks();
  } catch (e) {
    console.error('Failed to remove folder:', e);
    throw e;
  }
}

export async function toggleFolderEnabled(folderId: number, enabled: boolean): Promise<void> {
  if (isMock) return;
  
  try {
    await api.updateFolderEnabled(folderId, enabled);
    folders.update(f => f.map(folder => 
      folder.id === folderId ? { ...folder, enabled } : folder
    ));
  } catch (e) {
    console.error('Failed to toggle folder enabled:', e);
    throw e;
  }
}

export async function updateFolderOptions(folderId: number, options: Partial<FolderOptions>): Promise<void> {
  if (isMock) return;
  
  try {
    await api.updateFolderOptions(folderId, options);
    await loadFolders();
  } catch (e) {
    console.error('Failed to update folder options:', e);
    throw e;
  }
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

  // Listen for quick scan completion (startup scan)
  listen<QuickScanComplete>('evt_quick_scan_complete', (event) => {
    const { filesAdded, filesMarkedMissing, filesRestored } = event.payload;
    console.log(`Quick scan complete: +${filesAdded} added, -${filesMarkedMissing} missing, ↺${filesRestored} restored`);
    
    // Reload library data if any changes were detected
    if (filesAdded > 0 || filesMarkedMissing > 0 || filesRestored > 0) {
      loadTracks();
      // Dispatch event for views to refresh
      window.dispatchEvent(new CustomEvent('sermon:library-changed', { 
        detail: event.payload 
      }));
    }
  });
}

// Initialize
export function initLibrary(): void {
  initEventListeners();
  loadFolders();
  loadTracks();
}
