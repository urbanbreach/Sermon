<script lang="ts">
  interface Props {
    disabled?: boolean;
    onclick?: () => void;
    children?: import('svelte').Snippet;
  }
  let { disabled = false, onclick, children }: Props = $props();
</script>

<button 
  class="menu-item"
  role="menuitem"
  {disabled}
  {onclick}
>
  {@render children?.()}
</button>

<style>
  .menu-item {
    display: block;
    width: 100%;
    padding: var(--space-2) var(--space-3);
    text-align: left;
    font-family: inherit;
    font-size: var(--text-body);
    color: var(--text-primary);
    background: transparent;
    border: none;
    cursor: pointer;
    transition: background-color var(--motion-fast) var(--ease-out);
  }
  .menu-item:hover:not(:disabled) { background: var(--surface-hover); }
  .menu-item:focus-visible { background: var(--surface-hover); outline: none; }
  .menu-item:disabled { color: var(--text-disabled); cursor: not-allowed; }
</style>
