<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { onMount } from 'svelte';
  import { folders, selectedFolderId, scanStatus, scanProgress, scanError, addFolder, startScan } from '../state/library';
  import { devices, currentDevice, loadDevices, selectDevice } from '../state/playback';

  const isMock = import.meta.env.SERMON_MOCK === '1';

  onMount(() => {
    loadDevices();
  });

  async function handleAddFolder() {
    if (isMock) return;
    
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Music Folder'
    });
    
    if (selected && typeof selected === 'string') {
      await addFolder(selected);
    }
  }

  async function handleRescan() {
    if (isMock) return;
    await startScan();
  }
</script>

<div class="view-container">
  <h1>Settings</h1>

  <div class="section">
    <h2>Library Folders</h2>
    
    <div class="folder-list">
      {#each $folders as folder}
        <div 
          class="folder-item" 
          class:selected={$selectedFolderId === folder.id}
          on:click={() => selectedFolderId.set(folder.id)}
        >
          <span class="folder-path">{folder.path}</span>
          <span class="folder-status">{folder.enabled ? '✓' : '○'}</span>
        </div>
      {:else}
        <div class="empty-folders">No library folders configured</div>
      {/each}
    </div>

    <div class="folder-actions">
      <button class="btn" on:click={handleAddFolder} disabled={isMock}>
        Add Folder
      </button>
      <button 
        class="btn" 
        on:click={handleRescan} 
        disabled={isMock || $scanStatus === 'scanning' || $folders.length === 0}
      >
        {$scanStatus === 'scanning' ? 'Scanning...' : 'Rescan'}
      </button>
    </div>

    {#if $scanStatus === 'scanning'}
      <div class="scan-indicator">
        <div class="progress-bar">
          <div 
            class="progress-fill" 
            style="width: {$scanProgress.total > 0 ? ($scanProgress.scanned / $scanProgress.total) * 100 : 0}%"
          ></div>
        </div>
        <span class="progress-text">{$scanProgress.scanned} / {$scanProgress.total}</span>
      </div>
    {/if}

    {#if $scanError}
      <div class="error-message">{$scanError}</div>
    {/if}
  </div>
  
  <div class="section">
    <h2>General</h2>
    <div class="setting">
      <label>
        <input type="checkbox" checked />
        Start with Windows
      </label>
    </div>
  </div>

  <div class="section">
    <h2>Audio</h2>
    <div class="setting">
      <label>Output Device</label>
      <select 
        value={$currentDevice?.id || 'default'} 
        on:change={(e) => selectDevice(e.currentTarget.value)}
      >
        <option value="default">Default Output</option>
        {#each $devices as device}
           <!-- Skip default which is already handled above if needed, or list all -->
           {#if !device.is_default}
             <option value={device.id}>{device.name}</option>
           {/if}
        {/each}
      </select>
    </div>
  </div>

  <div class="section">
    <h2>Theme</h2>
    <div class="setting">
      <label>Mode</label>
      <select disabled>
        <option>Dark (Default)</option>
      </select>
    </div>
  </div>
</div>

<style>
  .view-container {
    padding: 2rem;
    color: #fff;
    height: 100%;
    overflow-y: auto;
  }
  .section {
    margin-bottom: 2rem;
  }
  h2 {
    font-size: 1.2rem;
    border-bottom: 1px solid #333;
    padding-bottom: 0.5rem;
    margin-bottom: 1rem;
    color: #ccc;
  }
  .setting {
    margin-bottom: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  select {
    background: #222;
    color: #fff;
    border: 1px solid #333;
    padding: 0.5rem;
    border-radius: 4px;
    width: 200px;
  }
  .folder-list {
    margin-bottom: 1rem;
    border: 1px solid var(--glass-border);
    border-radius: var(--glass-radius);
    overflow: hidden;
  }
  .folder-item {
    display: flex;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    background: var(--glass-bg);
    cursor: pointer;
    border-bottom: 1px solid var(--glass-border);
  }
  .folder-item:last-child {
    border-bottom: none;
  }
  .folder-item:hover {
    background: var(--glass-highlight);
  }
  .folder-item.selected {
    background: rgba(68, 170, 255, 0.2);
  }
  .folder-path {
    font-family: monospace;
    font-size: 0.9rem;
  }
  .folder-status {
    color: #4a4;
  }
  .empty-folders {
    padding: 1rem;
    text-align: center;
    color: #666;
  }
  .folder-actions {
    display: flex;
    gap: 0.5rem;
  }
  .btn {
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
    color: #fff;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
  }
  .btn:hover:not(:disabled) {
    background: var(--glass-highlight);
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .scan-indicator {
    margin-top: 1rem;
    display: flex;
    align-items: center;
    gap: 1rem;
  }
  .progress-bar {
    flex: 1;
    height: 8px;
    background: #333;
    border-radius: 4px;
    overflow: hidden;
  }
  .progress-fill {
    height: 100%;
    background: #4af;
    transition: width 0.2s;
  }
  .progress-text {
    font-size: 0.85rem;
    color: #888;
  }
  .error-message {
    margin-top: 0.5rem;
    padding: 0.5rem;
    background: rgba(255, 68, 68, 0.2);
    border: 1px solid #f44;
    border-radius: 4px;
    color: #f88;
    font-size: 0.85rem;
  }
</style>
