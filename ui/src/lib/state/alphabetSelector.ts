import { writable } from 'svelte/store';

export interface AlphabetSelectorState {
  items: { sortKey: string }[];
  onSelect: ((index: number) => void) | null;
}

const initialState: AlphabetSelectorState = {
  items: [],
  onSelect: null,
};

export const alphabetSelector = writable<AlphabetSelectorState>(initialState);

export function setAlphabetSelector(
  items: { sortKey: string }[],
  onSelect: (index: number) => void
) {
  alphabetSelector.set({ items, onSelect });
}

export function clearAlphabetSelector() {
  alphabetSelector.set(initialState);
}
