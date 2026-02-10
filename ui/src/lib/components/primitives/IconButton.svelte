<script lang="ts">
  interface Props {
    label: string; // Required for accessibility
    disabled?: boolean;
    active?: boolean;
    onclick?: () => void;
    children?: import('svelte').Snippet;
  }
  let { label, disabled = false, active = false, onclick, children }: Props = $props();
</script>

<button 
  class="icon-btn"
  class:active
  {disabled}
  {onclick}
  aria-label={label}
  title={label}
>
  {@render children?.()}
</button>

<style>
  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: background-color var(--motion-fast) var(--ease-out),
                color var(--motion-fast) var(--ease-out);
  }
  .icon-btn:hover:not(:disabled) { background: var(--surface-1); color: var(--text-primary); }
  .icon-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .icon-btn.active { background: var(--surface-2); color: var(--text-primary); }
</style>
