<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    label: string;
    description?: string;
    children?: Snippet;
  }

  let { label, description, children }: Props = $props();
</script>

<div class="setting-row">
  <div class="info">
    <div class="label">{label}</div>
    {#if description}
      <div class="description">{description}</div>
    {/if}
  </div>
  <div class="control">
    {@render children?.()}
  </div>
</div>

<style>
  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 0;
    gap: 16px;
    min-height: 48px;
  }

  .info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }

  .label {
    font-size: var(--text-body);
    color: var(--text-primary);
    font-weight: 500;
  }

  .description {
    font-size: var(--text-meta);
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .control {
    display: flex;
    align-items: center;
    justify-content: flex-end;
  }
</style>
