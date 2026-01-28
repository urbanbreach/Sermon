<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { FolderPlus, RefreshCw, HardDrive } from '@lucide/svelte';
  import { 
    folders, addFolder, loadFolders, removeFolder, toggleFolderEnabled,
    scanStatus, scanProgress, startScan
  } from '../../state/library';
  import { librarySettings, loadCategorySettings, saveCategorySetting, parseBool } from '../../state/preferences';
  import { getLibraryStats } from '../../api/library';
  import type { LibraryStats } from '../../types/library';
  import FolderListItem from './FolderListItem.svelte';

  const isMock = import.meta.env.SERMON_MOCK === '1';
  
  let selectedFolderId = $state<number | null>(null);
  let libraryStats = $state<LibraryStats | null>(null);
  let showRemoveConfirm = $state(false);
  let folderToRemove = $state<number | null>(null);

  onMount(async () => {
    if (!isMock) {
      await Promise.all([loadFolders(), loadCategorySettings('library')]);
      refreshStats();
    }
  });

  async function refreshStats() {
    try {
      libraryStats = await getLibraryStats();
    } catch (e) {
      console.error('Failed to load library stats:', e);
    }
  }

  async function handleAddFolder() {
    if (isMock) return;
    
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Music Folder'
      });
      
      if (selected && typeof selected === 'string') {
        await addFolder(selected);
        await refreshStats();
      }
    } catch (e) {
      console.error('Failed to add folder:', e);
    }
  }

  async function handleRemoveFolder(folderId: number) {
    folderToRemove = folderId;
    showRemoveConfirm = true;
  }

  async function confirmRemoveFolder() {
    if (folderToRemove === null) return;
    
    try {
      await removeFolder(folderToRemove);
      if (selectedFolderId === folderToRemove) {
        selectedFolderId = null;
      }
      await refreshStats();
    } catch (e) {
      console.error('Failed to remove folder:', e);
    } finally {
      showRemoveConfirm = false;
      folderToRemove = null;
    }
  }

  async function handleToggleEnabled(folderId: number, enabled: boolean) {
    try {
      await toggleFolderEnabled(folderId, enabled);
    } catch (e) {
      console.error('Failed to toggle folder:', e);
    }
  }

  async function handleScanLibrary() {
    try {
      await startScan();
    } catch (e) {
      console.error('Failed to start scan:', e);
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatDate(ms: number | undefined): string {
    if (!ms) return 'Never';
    return new Date(ms).toLocaleString();
  }
</script>

<div class="category-content">
  <div class="setting-group">
    <div class="section-header">
      <h3>Library Folders</h3>
      <button class="btn btn-primary" onclick={handleAddFolder} disabled={isMock}>
        <FolderPlus size={14} />
        Add Folder
      </button>
    </div>

    <div class="folder-list">
      {#each $folders as folder (folder.id)}
        <FolderListItem 
          {folder}
          selected={selectedFolderId === folder.id}
          disabled={isMock}
          onSelect={() => selectedFolderId = folder.id}
          onRemove={() => handleRemoveFolder(folder.id)}
          onToggleEnabled={(enabled) => handleToggleEnabled(folder.id, enabled)}
        />
      {:else}
        <div class="empty-state">
          <HardDrive size={32} />
          <p>No folders added</p>
          <span>Click "Add Folder" to start building your library</span>
        </div>
      {/each}
    </div>
  </div>

  <div class="setting-group">
    <h3>Scanning</h3>
    
    <div class="scan-controls">
      <button 
        class="btn btn-secondary" 
        onclick={handleScanLibrary} 
        disabled={isMock || $scanStatus === 'scanning' || $folders.length === 0}
      >
        <RefreshCw size={14} class={$scanStatus === 'scanning' ? 'spinning' : ''} />
        {$scanStatus === 'scanning' ? 'Scanning...' : 'Scan Library Now'}
      </button>
      
      {#if $scanStatus === 'scanning'}
        <div class="scan-progress">
          <div class="progress-bar">
            <div 
              class="progress-fill" 
              style="width: {$scanProgress.total > 0 ? ($scanProgress.scanned / $scanProgress.total * 100) : 0}%"
            ></div>
          </div>
          <span class="progress-text">{$scanProgress.scanned} / {$scanProgress.total}</span>
        </div>
      {/if}
    </div>

    {#if $librarySettings}
      <div class="setting">
        <label>
          <input 
            type="checkbox" 
            checked={parseBool($librarySettings['library.scan_on_startup'])} 
            onchange={(e) => saveCategorySetting('library', 'library.scan_on_startup', e.currentTarget.checked ? 'on' : 'off')} 
            disabled={isMock} 
          />
          Scan library on startup
        </label>
        <span class="setting-hint">Automatically check for new and removed files when the app starts</span>
      </div>
    {/if}
  </div>

  {#if libraryStats}
    <div class="setting-group">
      <h3>Library Statistics</h3>
      <div class="stats-grid">
        <div class="stat-item">
          <span class="stat-value">{libraryStats.trackCount.toLocaleString()}</span>
          <span class="stat-label">Tracks</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{libraryStats.albumCount.toLocaleString()}</span>
          <span class="stat-label">Albums</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{libraryStats.artistCount.toLocaleString()}</span>
          <span class="stat-label">Artists</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{formatBytes(libraryStats.dbSizeBytes)}</span>
          <span class="stat-label">Database Size</span>
        </div>
      </div>
      <div class="last-scan">
        Last scan: {formatDate(libraryStats.lastScanCompletedMs)}
      </div>
    </div>
  {/if}
</div>

{#if showRemoveConfirm}
  <div class="modal-overlay" onclick={() => showRemoveConfirm = false}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <h4>Remove Folder?</h4>
      <p>This will remove the folder from your library and delete all associated track data.</p>
      <p class="modal-warning">This action cannot be undone.</p>
      <div class="modal-actions">
        <button class="btn btn-secondary" onclick={() => showRemoveConfirm = false}>Cancel</button>
        <button class="btn btn-danger" onclick={confirmRemoveFolder}>Remove</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .category-content { 
    display: flex; 
    flex-direction: column; 
    gap: 1.5rem; 
  }
  
  .setting-group {
    border: 1px solid var(--glass-border);
    border-radius: 8px;
    padding: 1rem;
  }
  
  .setting-group h3 {
    margin: 0 0 1rem 0;
    font-size: 1rem;
    color: #aaa;
    border-bottom: 1px solid var(--glass-border);
    padding-bottom: 0.5rem;
  }
  
  .section-header { 
    display: flex; 
    justify-content: space-between; 
    align-items: center; 
    margin-bottom: 1rem;
    border-bottom: 1px solid var(--glass-border);
    padding-bottom: 0.5rem;
  }
  
  .section-header h3 {
    margin: 0;
    border-bottom: none;
    padding-bottom: 0;
  }
  
  .folder-list { 
    display: flex; 
    flex-direction: column; 
    gap: 0.5rem; 
    max-height: 300px;
    overflow-y: auto;
  }
  
  .empty-state { 
    padding: 2rem; 
    text-align: center; 
    color: #666;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }
  
  .empty-state p {
    margin: 0;
    font-size: 1rem;
    color: #888;
  }
  
  .empty-state span {
    font-size: 0.85rem;
  }
  
  .setting { 
    display: flex; 
    flex-direction: column; 
    gap: 0.25rem; 
  }
  
  .setting label { 
    display: flex; 
    align-items: center; 
    gap: 0.5rem; 
    color: #ccc;
    cursor: pointer;
  }
  
  .setting-hint {
    font-size: 0.8rem;
    color: #666;
    margin-left: 1.5rem;
  }
  
  .btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    font-size: 0.9rem;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
    border: 1px solid transparent;
  }
  
  .btn-primary {
    background: var(--accent-color, #4af);
    color: #000;
    border-color: var(--accent-color, #4af);
  }
  
  .btn-primary:hover {
    filter: brightness(1.1);
  }
  
  .btn-secondary {
    background: rgba(255, 255, 255, 0.1);
    color: #ccc;
    border-color: var(--glass-border);
  }
  
  .btn-secondary:hover {
    background: rgba(255, 255, 255, 0.15);
  }
  
  .btn-danger {
    background: rgba(255, 80, 80, 0.8);
    color: #fff;
    border-color: rgba(255, 80, 80, 0.8);
  }
  
  .btn-danger:hover {
    background: rgba(255, 80, 80, 1);
  }
  
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .scan-controls {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }
  
  .scan-progress {
    display: flex;
    align-items: center;
    gap: 1rem;
  }
  
  .progress-bar {
    flex: 1;
    height: 6px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
    overflow: hidden;
  }
  
  .progress-fill {
    height: 100%;
    background: var(--accent-color, #4af);
    transition: width 0.3s ease;
  }
  
  .progress-text {
    font-size: 0.8rem;
    color: #888;
    min-width: 80px;
    text-align: right;
  }
  
  :global(.spinning) {
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1rem;
    margin-bottom: 1rem;
  }
  
  .stat-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1rem;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 8px;
  }
  
  .stat-value {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--accent-color, #4af);
  }
  
  .stat-label {
    font-size: 0.8rem;
    color: #888;
    margin-top: 0.25rem;
  }
  
  .last-scan {
    font-size: 0.85rem;
    color: #666;
    text-align: center;
  }
  
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  
  .modal-content {
    background: #1a1a1a;
    border: 1px solid var(--glass-border);
    border-radius: 12px;
    padding: 1.5rem;
    max-width: 400px;
    width: 90%;
  }
  
  .modal-content h4 {
    margin: 0 0 1rem 0;
    color: #fff;
  }
  
  .modal-content p {
    margin: 0 0 0.5rem 0;
    color: #aaa;
    font-size: 0.9rem;
  }
  
  .modal-warning {
    color: #f66 !important;
    font-weight: 500;
  }
  
  .modal-actions {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
    margin-top: 1.5rem;
  }
</style>
