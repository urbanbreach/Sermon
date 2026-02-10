<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { Tags } from '@lucide/svelte';
  import WindowControls from './WindowControls.svelte';

  interface Props {
    trackCount: number;
  }

  let { trackCount }: Props = $props();

  const appWindow = getCurrentWindow();

  let title = $derived(trackCount > 1 ? `Edit ${trackCount} Tracks` : 'Edit Tags');

  async function handleDoubleClick(): Promise<void> {
    await appWindow.toggleMaximize();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="tag-editor-title-bar" ondblclick={handleDoubleClick}>
  <div class="left">
    <div class="icon-wrap">
      <Tags size={14} strokeWidth={1.7} />
    </div>
    <span class="title">{title}</span>
  </div>

  <div class="right">
    <WindowControls />
  </div>
</div>

<style>
  .tag-editor-title-bar {
    height: 44px;
    background: var(--surface-header, rgba(18, 18, 18, 0.95));
    border-bottom: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 0 0 16px;
    -webkit-app-region: drag;
    user-select: none;
    flex-shrink: 0;
    position: relative;
    z-index: 15;
  }

  .left {
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .icon-wrap {
    color: var(--text-tertiary, rgba(255, 255, 255, 0.45));
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .title {
    color: var(--text-primary, rgba(255, 255, 255, 0.92));
    font-size: 13px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .right {
    display: flex;
    align-items: stretch;
    justify-content: flex-end;
    height: 100%;
  }
</style>
