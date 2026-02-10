<script lang="ts">
  interface Props {
    text: string;
    children?: import('svelte').Snippet;
  }
  let { text, children }: Props = $props();
</script>

<div class="tooltip-wrapper">
  {@render children?.()}
  <span class="tooltip">{text}</span>
</div>

<style>
  .tooltip-wrapper {
    position: relative;
    display: inline-flex;
  }
  .tooltip {
    position: absolute;
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%);
    padding: var(--space-1) var(--space-2);
    background: var(--surface-floating);
    color: var(--text-primary);
    font-size: var(--text-meta);
    border-radius: var(--radius-sm);
    white-space: nowrap;
    opacity: 0;
    pointer-events: none;
    transition: opacity var(--motion-fast) var(--ease-out);
    z-index: 1001;
  }
  .tooltip-wrapper:hover .tooltip { opacity: 1; }
</style>
