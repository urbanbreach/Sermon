import { invoke } from '@tauri-apps/api/core';
import type { LyricsResponse } from '../types/lyrics';

export async function getLyricsForTrack(trackId: number): Promise<LyricsResponse> {
  return invoke('cmd_lyrics_get_for_track', { request: { trackId } });
}
