<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import TagEditorTitleBar from '../components/TagEditorTitleBar.svelte';
  import TagEditor from '../components/TagEditor.svelte';

  const editorWindow = getCurrentWindow();

  let trackIds = $state<number[]>([]);
  let isReady = $state(false);
  let unlistenOpen: UnlistenFn | null = null;

  function parseTrackIds(rawIds: string | null): number[] {
    if (!rawIds) {
      return [];
    }

    const normalized: number[] = [];
    for (const part of rawIds.split(',')) {
      const parsed = Number.parseInt(part.trim(), 10);
      if (Number.isInteger(parsed) && parsed > 0 && !normalized.includes(parsed)) {
        normalized.push(parsed);
      }
    }
    return normalized;
  }

  async function applyTrackIds(nextTrackIds: number[]): Promise<void> {
    trackIds = nextTrackIds;
    isReady = true;

    const title =
      nextTrackIds.length > 1
        ? `Edit ${nextTrackIds.length} Tracks`
        : 'Edit Tags';

    await editorWindow.setTitle(`Sermon - ${title}`);
  }

  function getTrackIdsFromLocation(): number[] {
    const params = new URLSearchParams(window.location.search);
    return parseTrackIds(params.get('trackIds'));
  }

  async function closeEditorWindow(): Promise<void> {
    await editorWindow.close();
  }

  onMount(async () => {
    await applyTrackIds(getTrackIdsFromLocation());

    unlistenOpen = await editorWindow.listen<number[]>('tag-editor://open', async (event) => {
      const payloadTrackIds = Array.isArray(event.payload)
        ? event.payload.filter((id) => Number.isInteger(id) && id > 0)
        : [];
      await applyTrackIds(payloadTrackIds);
    });
  });

  onDestroy(() => {
    if (unlistenOpen) {
      void unlistenOpen();
      unlistenOpen = null;
    }
  });
</script>

<div class="tag-editor-window-view">
  <TagEditorTitleBar trackCount={trackIds.length} />

  {#if isReady && trackIds.length > 0}
    <TagEditor trackIds={trackIds} open={true} onclose={closeEditorWindow} windowed={true} />
  {:else}
    <div class="empty-state">
      <p>No tracks selected for editing.</p>
      <button class="close-btn" onclick={closeEditorWindow}>Close</button>
    </div>
  {/if}
</div>

<style>
  .tag-editor-window-view {
    width: 100%;
    height: 100%;
    background: var(--surface-0, #0a0a0a);
    color: var(--text-primary);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--text-secondary);
  }

  .close-btn {
    background: var(--surface-2);
    border: 1px solid var(--divider-color);
    color: var(--text-primary);
    padding: 8px 14px;
    border-radius: 8px;
    cursor: pointer;
  }

  .close-btn:hover {
    background: var(--surface-hover);
  }
</style>
