import { writable } from 'svelte/store';

export type NowPlayingPanelMode = 'lyrics' | 'queue' | 'both';

/**
 * Active panel mode in the Now Playing view.
 * Driven by the in-panel toggle inside NowPlayingView.
 */
export const nowPlayingPanelMode = writable<NowPlayingPanelMode>('lyrics');
