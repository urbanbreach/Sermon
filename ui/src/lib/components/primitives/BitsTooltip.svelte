<script lang="ts" module>
  import { Tooltip } from 'bits-ui';
  export const Root = Tooltip.Root;
  export const Trigger = Tooltip.Trigger;
  export const Content = Tooltip.Content;
  export const Arrow = Tooltip.Arrow;
  export const Provider = Tooltip.Provider;
</script>

<style>
  :global(.tooltip-content) {
    background: var(--surface-2, rgba(255, 255, 255, 0.08));
    border: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
    border-radius: var(--radius-sm, 8px);
    padding: var(--space-2, 8px) var(--space-3, 12px);
    font-size: var(--text-meta, 12px);
    color: var(--text-primary, rgba(255, 255, 255, 0.92));
    box-shadow: var(--shadow-2, 0 4px 12px rgba(0, 0, 0, 0.5));
    z-index: 1000;
    animation: fadeIn var(--motion-fast, 0.15s) var(--ease-out, cubic-bezier(0.16, 1, 0.3, 1));
  }

  :global(.tooltip-arrow) {
    border-color: var(--divider-color, rgba(255, 255, 255, 0.07));
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }
</style>
