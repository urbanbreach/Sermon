import { writable } from 'svelte/store';

export type Route = 'albums' | 'artists' | 'tracks' | 'settings' | 'now-playing';

export const currentRoute = writable<Route>('albums');
