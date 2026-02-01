<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    children?: Snippet;
  }

  let { title, children }: Props = $props();
</script>

<div class="setting-group">
  <div class="header">
    <h3>{title}</h3>
  </div>
  <div class="content">
    {@render children?.()}
  </div>
</div>

<style>
  .setting-group {
    display: flex;
    flex-direction: column;
    margin-bottom: 24px;
  }

  .header {
    padding-bottom: 8px;
    border-bottom: 1px solid var(--divider-color);
    margin-bottom: 8px;
  }

  h3 {
    font-size: var(--text-section);
    color: var(--text-primary);
    font-weight: 600;
    margin: 0;
  }

  .content {
    display: flex;
    flex-direction: column;
  }
</style>
