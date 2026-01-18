<script lang="ts">
  import { onMount } from 'svelte';
  import Modal from './Modal.svelte';
  import { getRawTags } from '../api/library';
  import type { RawTagsResult } from '../types/library';

  interface Props {
    trackId: number | null;
    open: boolean;
    onclose: () => void;
  }

  let { trackId, open, onclose }: Props = $props();

  let loading = $state(true);
  let error = $state<string | null>(null);
  let rawTags = $state<RawTagsResult | null>(null);

  // Load raw tags when opened
  $effect(() => {
    if (open && trackId !== null) {
      loadRawTags();
    }
  });

  async function loadRawTags() {
    if (trackId === null) return;
    
    loading = true;
    error = null;
    rawTags = null;

    try {
      rawTags = await getRawTags(trackId);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
</script>

<Modal {open} title="Raw Tags (Read-Only)" {onclose}>
  <div class="raw-tags-viewer">
    {#if loading}
      <div class="loading">Loading raw tags...</div>
    {:else if error}
      <div class="error">
        <span class="error-icon">!</span>
        <span>{error}</span>
      </div>
    {:else if rawTags}
      {#if rawTags.tags.length === 0}
        <div class="empty">No tags found in this file.</div>
      {:else}
        {#each rawTags.tags as tagGroup}
          <div class="tag-group">
            <h3 class="tag-type">{tagGroup.tagType}</h3>
            {#if tagGroup.items.length === 0}
              <div class="empty-group">No items in this tag type.</div>
            {:else}
              <table class="tags-table">
                <thead>
                  <tr>
                    <th>Key</th>
                    <th>Value</th>
                  </tr>
                </thead>
                <tbody>
                  {#each tagGroup.items as item}
                    <tr>
                      <td class="key">{item.key}</td>
                      <td class="value">{item.value}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          </div>
        {/each}
      {/if}
    {/if}

    <div class="actions">
      <button class="btn" onclick={onclose}>Close</button>
    </div>
  </div>
</Modal>

<style>
  .raw-tags-viewer {
    min-width: 500px;
    max-height: 60vh;
    overflow-y: auto;
  }

  .loading {
    text-align: center;
    color: #888;
    padding: 2rem;
  }

  .error {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: rgba(255, 68, 68, 0.15);
    border: 1px solid rgba(255, 68, 68, 0.3);
    border-radius: 6px;
    padding: 0.75rem 1rem;
    color: #f88;
  }

  .error-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    background: rgba(255, 68, 68, 0.3);
    border-radius: 50%;
    font-weight: bold;
    font-size: 0.8rem;
  }

  .empty {
    text-align: center;
    color: #666;
    padding: 2rem;
    font-style: italic;
  }

  .tag-group {
    margin-bottom: 1.5rem;
  }

  .tag-type {
    font-size: 0.9rem;
    color: #4af;
    margin: 0 0 0.5rem 0;
    padding-bottom: 0.25rem;
    border-bottom: 1px solid var(--glass-border);
  }

  .empty-group {
    font-size: 0.85rem;
    color: #666;
    font-style: italic;
    padding: 0.5rem 0;
  }

  .tags-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
  }

  .tags-table th {
    text-align: left;
    color: #888;
    font-weight: normal;
    padding: 0.4rem 0.5rem;
    border-bottom: 1px solid var(--glass-border);
  }

  .tags-table td {
    padding: 0.4rem 0.5rem;
    border-bottom: 1px solid var(--glass-highlight);
    vertical-align: top;
  }

  .tags-table .key {
    color: #aaa;
    font-family: monospace;
    white-space: nowrap;
    width: 40%;
  }

  .tags-table .value {
    color: #fff;
    word-break: break-word;
  }

  .tags-table tr:hover {
    background: var(--glass-highlight);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--glass-border);
  }

  .btn {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--glass-border);
    color: #ccc;
    padding: 0.5rem 1.25rem;
    border-radius: 6px;
    font-size: 0.95rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }
</style>
