<script lang="ts">
  import { onMount } from 'svelte';
  
  interface Props {
    open: boolean;
    onclose: () => void;
    children?: import('svelte').Snippet;
    class?: string;
    [key: string]: any;
  }
  let { open = $bindable(), onclose, children, class: className, ...rest }: Props = $props();
  
  let menuRef: HTMLDivElement | undefined = $state();
  
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onclose();
    }
  }
  
  function handleClickOutside(e: MouseEvent) {
    if (menuRef && !menuRef.contains(e.target as Node)) {
      onclose();
    }
  }
  
  $effect(() => {
    if (open) {
      document.addEventListener('keydown', handleKeydown);
      document.addEventListener('click', handleClickOutside, true);
      menuRef?.focus();
    }
    return () => {
      document.removeEventListener('keydown', handleKeydown);
      document.removeEventListener('click', handleClickOutside, true);
    };
  });
</script>

{#if open}
  <div 
    class="menu {className || ''}"
    bind:this={menuRef}
    role="menu"
    tabindex="-1"
    {...rest}
  >
    {@render children?.()}
  </div>
{/if}

<style>
  .menu {
    position: absolute;
    min-width: 160px;
    padding: var(--space-1) 0;
    background: var(--surface-floating, #141414);
    border: 1px solid var(--divider);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-2);
    z-index: 1000;
    outline: none;
  }
</style>
