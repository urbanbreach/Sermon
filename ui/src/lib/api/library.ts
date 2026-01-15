import { invoke } from '@tauri-apps/api/core';
import type { TrackRow, LibraryFolder, SortBy, SortDirection } from '../types/library';

export async function addFolder(path: string): Promise<LibraryFolder> {
  return invoke('cmd_library_add_folder', { path });
}

export async function listFolders(): Promise<LibraryFolder[]> {
  return invoke('cmd_library_list_folders');
}

export async function listTracks(sortBy: SortBy, direction: SortDirection): Promise<TrackRow[]> {
  return invoke('cmd_library_list_tracks', { sortBy, direction });
}

export async function startScan(path: string): Promise<{ scan_id: number }> {
  return invoke('cmd_scan_start', { path });
}
