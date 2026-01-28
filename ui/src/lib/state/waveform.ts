import { writable } from 'svelte/store';
import { getWaveformPeaks } from '../api/waveform';

export interface WaveformState {
  status: 'idle' | 'loading' | 'ready' | 'error';
  trackId: number | null;
  durationMs: number;
  binMs: number;
  peaksU8: Uint8Array | null;
  error?: string;
}

const initialState: WaveformState = {
  status: 'idle',
  trackId: null,
  durationMs: 0,
  binMs: 0,
  peaksU8: null
};

export const waveformPeaks = writable<WaveformState>(initialState);

export function clearWaveformPeaks() {
  waveformPeaks.set(initialState);
}

export async function loadWaveformPeaks(trackId: number) {
  waveformPeaks.set({
    ...initialState,
    status: 'loading',
    trackId
  });

  try {
    if (import.meta.env.SERMON_MOCK === '1') {
      await new Promise(resolve => setTimeout(resolve, 50));
      
      const mockPeaks = new Uint8Array(100).map((_, i) => 
        Math.floor(128 + 64 * Math.sin(i * 0.3))
      );

      waveformPeaks.set({
        status: 'ready',
        trackId,
        durationMs: 300000,
        binMs: 3000,
        peaksU8: mockPeaks
      });
      return;
    }

    const response = await getWaveformPeaks(trackId);
    
    const peaksU8 = Uint8Array.from(atob(response.peaksBase64), c => c.charCodeAt(0));

    waveformPeaks.set({
      status: 'ready',
      trackId,
      durationMs: response.durationMs,
      binMs: response.binMs,
      peaksU8
    });

  } catch (e) {
    console.error('Failed to load waveform peaks:', e);
    waveformPeaks.set({
      ...initialState,
      status: 'error',
      trackId,
      error: e instanceof Error ? e.message : String(e)
    });
  }
}
