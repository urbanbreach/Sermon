<script lang="ts">
  import { pressScale } from '../utils/animations';

  interface Props {
    items: { sortKey: string }[];
    onSelect: (index: number) => void;
  }

  let { items, onSelect }: Props = $props();

  const letters = '#ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('');

  function getFirstLetterIndex(letter: string): number {
    if (letter === '#') {
      const idx = items.findIndex(item => {
        const first = item.sortKey.charAt(0).toUpperCase();
        return !/[A-Z]/.test(first);
      });
      return idx >= 0 ? idx : -1;
    }
    
    const idx = items.findIndex(item => {
      const first = item.sortKey.charAt(0).toUpperCase();
      return first === letter || first.localeCompare(letter) >= 0;
    });
    return idx >= 0 ? idx : -1;
  }

  function hasItemsForLetter(letter: string): boolean {
    if (letter === '#') {
      return items.some(item => {
        const first = item.sortKey.charAt(0).toUpperCase();
        return !/[A-Z]/.test(first);
      });
    }
    return items.some(item => item.sortKey.charAt(0).toUpperCase() === letter);
  }

  function handleLetterClick(letter: string) {
    const index = getFirstLetterIndex(letter);
    if (index >= 0) {
      onSelect(index);
    }
  }
</script>

<div class="alphabet-selector">
  {#each letters as letter}
    {@const hasItems = hasItemsForLetter(letter)}
    <button
      class="letter"
      class:disabled={!hasItems}
      disabled={!hasItems}
      onclick={() => handleLetterClick(letter)}
      use:pressScale
    >
      {letter}
    </button>
  {/each}
</div>

<style>
  .alphabet-selector {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 2px;
    padding: 8px 16px;
    background: transparent;
    flex-shrink: 0;
    user-select: none;
    flex-wrap: wrap;
  }

  .letter {
    width: 22px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all var(--motion-fast) var(--ease-out);
    -webkit-app-region: no-drag;
  }

  .letter:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .letter:active:not(:disabled) {
    color: var(--theme-accent);
    background: var(--accent-weak);
    transform: scale(0.95);
  }

  .letter.disabled {
    color: var(--text-disabled);
    cursor: default;
    opacity: 0.4;
  }
</style>
