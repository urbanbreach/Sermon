import { invoke } from '@tauri-apps/api/core';

export interface WaveformPeaksResponse {
  durationMs: number;
  binMs: number;
  peaksBase64: string;
  formatVersion: number;
}

export async function getWaveformPeaks(trackId: number): Promise<WaveformPeaksResponse> {
  return invoke('cmd_waveform_get_peaks', { trackId });
}
