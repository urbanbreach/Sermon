import { writable } from 'svelte/store';

export const viewTitle = writable<string>('');

export function setViewTitle(title: string): void {
  viewTitle.set(title);
}

export function clearViewTitle(): void {
  viewTitle.set('');
}
