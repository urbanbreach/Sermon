import { get, writable } from 'svelte/store';
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

let activeWaveformRequestId = 0;

export function clearWaveformPeaks() {
  activeWaveformRequestId += 1;
  waveformPeaks.set(initialState);
}

export async function loadWaveformPeaks(trackId: number) {
  const current = get(waveformPeaks);
  if (current.trackId === trackId && (current.status === 'ready' || current.status === 'loading')) {
    return;
  }

  const requestId = ++activeWaveformRequestId;

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

    if (requestId !== activeWaveformRequestId) {
      return;
    }
    
    const peaksRaw = atob(response.peaksBase64);
    const peaksLen = peaksRaw.length;
    const peaksU8 = new Uint8Array(peaksLen);
    for (let i = 0; i < peaksLen; i++) {
      peaksU8[i] = peaksRaw.charCodeAt(i);
    }

    waveformPeaks.set({
      status: 'ready',
      trackId,
      durationMs: response.durationMs,
      binMs: response.binMs,
      peaksU8
    });

  } catch (e) {
    if (requestId !== activeWaveformRequestId) {
      return;
    }

    console.error('Failed to load waveform peaks:', e);
    waveformPeaks.set({
      ...initialState,
      status: 'error',
      trackId,
      error: e instanceof Error ? e.message : String(e)
    });
  }
}
