<script lang="ts">
  import { canGoBack, currentRoute, goBack, navigate } from '../state/route';
  import TagEditor from '../components/TagEditor.svelte';

  const trackIds = $derived(
    $currentRoute.name === 'tag-editor-panel' ? $currentRoute.trackIds : [],
  );

  function closePanel(): void {
    if ($canGoBack) {
      goBack();
      return;
    }

    navigate({ name: 'tracks' });
  }
</script>

<div class="tag-editor-panel-view">
  {#if trackIds.length > 0}
    <TagEditor trackIds={trackIds} open={true} onclose={closePanel} embedded={true} />
  {:else}
    <div class="empty-state">
      <p>No tracks selected for editing.</p>
      <button type="button" onclick={closePanel}>Back to tracks</button>
    </div>
  {/if}
</div>

<style>
  .tag-editor-panel-view {
    width: 100%;
    height: 100%;
    min-height: 0;
    background: var(--surface-0);
    color: var(--text-primary);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .empty-state {
    margin: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    color: var(--text-secondary);
  }

  .empty-state button {
    background: var(--surface-2);
    border: 1px solid var(--divider-color);
    color: var(--text-primary);
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .empty-state button:hover {
    background: var(--surface-hover);
  }
</style>
