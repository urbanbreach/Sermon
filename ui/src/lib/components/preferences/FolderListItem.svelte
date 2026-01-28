<script lang="ts">
  import { X, FolderOpen } from '@lucide/svelte';
  import type { LibraryFolder } from '../../types/library';
  import { getFolderTrackCount } from '../../api/library';

  interface Props {
    folder: LibraryFolder;
    selected: boolean;
    disabled?: boolean;
    onSelect: () => void;
    onRemove: () => void;
    onToggleEnabled: (enabled: boolean) => void;
  }

  let { folder, selected, disabled = false, onSelect, onRemove, onToggleEnabled }: Props = $props();
  
  let trackCount = $state<number | null>(null);
  
  $effect(() => {
    getFolderTrackCount(folder.id).then((count: number) => {
      trackCount = count;
    }).catch(() => {
      trackCount = null;
    });
  });
  
  function handleCheckboxChange(e: Event) {
    e.stopPropagation();
    const target = e.target as HTMLInputElement;
    onToggleEnabled(target.checked);
  }
  
  function handleRemoveClick(e: Event) {
    e.stopPropagation();
    onRemove();
  }
</script>

<div 
  class="folder-item" 
  class:selected
  class:disabled={!folder.enabled}
  onclick={onSelect}
  role="button"
  tabindex="0"
  onkeydown={(e) => e.key === 'Enter' && onSelect()}
>
  <div class="folder-left">
    <input 
      type="checkbox" 
      checked={folder.enabled}
      onchange={handleCheckboxChange}
      onclick={(e) => e.stopPropagation()}
      {disabled}
    />
    <FolderOpen size={16} class="folder-icon" />
    <span class="folder-path" title={folder.path}>{folder.path}</span>
  </div>
  
  <div class="folder-right">
    {#if trackCount !== null}
      <span class="track-count">{trackCount} tracks</span>
    {/if}
    <button 
      class="remove-btn" 
      onclick={handleRemoveClick}
      title="Remove folder"
      {disabled}
    >
      <X size={14} />
    </button>
  </div>
</div>

<style>
  .folder-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid transparent;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  
  .folder-item:hover {
    background: rgba(255, 255, 255, 0.06);
    border-color: var(--glass-border);
  }
  
  .folder-item.selected {
    background: rgba(255, 255, 255, 0.08);
    border-color: var(--accent-color, #4af);
  }
  
  .folder-item.disabled {
    opacity: 0.5;
  }
  
  .folder-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex: 1;
    min-width: 0;
  }
  
  .folder-left input[type="checkbox"] {
    margin: 0;
    cursor: pointer;
  }
  
  .folder-left :global(.folder-icon) {
    color: #888;
    flex-shrink: 0;
  }
  
  .folder-path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #ccc;
    font-size: 0.9rem;
  }
  
  .folder-right {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-shrink: 0;
  }
  
  .track-count {
    font-size: 0.8rem;
    color: #666;
  }
  
  .remove-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: #666;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  
  .remove-btn:hover {
    background: rgba(255, 100, 100, 0.2);
    color: #f66;
  }
  
  .remove-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
</style>
