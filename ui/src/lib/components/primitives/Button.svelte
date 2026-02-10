<script lang="ts">
  type Variant = 'primary' | 'secondary' | 'ghost';
  interface Props {
    variant?: Variant;
    disabled?: boolean;
    onclick?: () => void;
    children?: import('svelte').Snippet;
  }
  let { variant = 'secondary', disabled = false, onclick, children }: Props = $props();
</script>

<button 
  class="btn btn-{variant}"
  {disabled}
  {onclick}
>
  {@render children?.()}
</button>

<style>
  .btn {
    font-family: inherit;
    font-size: var(--text-body);
    font-weight: 500;
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-sm);
    border: none;
    cursor: pointer;
    transition: background-color var(--motion-fast) var(--ease-out),
                color var(--motion-fast) var(--ease-out);
  }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-primary {
    background: var(--theme-accent);
    color: #000;
  }
  .btn-primary:hover:not(:disabled) { filter: brightness(1.1); }
  .btn-secondary {
    background: var(--surface-2);
    color: var(--text-primary);
  }
  .btn-secondary:hover:not(:disabled) { background: var(--surface-hover); }
  .btn-ghost {
    background: transparent;
    color: var(--text-secondary);
  }
  .btn-ghost:hover:not(:disabled) { background: var(--surface-1); color: var(--text-primary); }
</style>
