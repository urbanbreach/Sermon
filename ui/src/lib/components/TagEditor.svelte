<script lang="ts">
  import Modal from './Modal.svelte';
  import { updateTrackTags, getTrackTagsBatch, updateTrackTagsBatch } from '../api/library';
  import { loadTracks } from '../state/library';
  import type { TrackRow, TagPatch, NumberPatch, BatchUpdateRequest } from '../types/library';
  import MetadataTab from './tag-editor/MetadataTab.svelte';
  import ArtworkTab from './tag-editor/ArtworkTab.svelte';
  import InspectorTab from './tag-editor/InspectorTab.svelte';
  import LyricsTab from './tag-editor/LyricsTab.svelte';

  interface Props {
    trackIds: number[];
    open: boolean;
    onclose: () => void;
  }

  let { trackIds, open, onclose }: Props = $props();

  // MusicBee-style tabs
  const tabs = [
    { id: 'properties', label: 'Properties' },
    { id: 'tags', label: 'Tags' },
    { id: 'tags2', label: 'Tags (2)' },
    { id: 'sorting', label: 'Sorting' },
    { id: 'artwork', label: 'Artwork' },
    { id: 'lyrics', label: 'Lyrics' },
    { id: 'settings', label: 'Settings' },
  ];

  // State
  let tracks = $state<TrackRow[]>([]);
  let loading = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let activeTab = $state('properties');
  let showConfirmModal = $state(false);

  // Form State
  let fieldValues = $state<Record<string, string>>({});
  let fieldChecked = $state<Record<string, boolean>>({});
  let lyrics = $state('');

  // Derived
  let isMulti = $derived(trackIds.length > 1);
  let canApply = $derived(!loading && !saving && (isMulti ? Object.values(fieldChecked).some(v => v) : true));

  // Load tracks when opening
  $effect(() => {
    if (open && trackIds.length > 0) {
      loadData();
    } else if (!open) {
      // Reset state on close
      tracks = [];
      fieldValues = {};
      fieldChecked = {};
      lyrics = '';
      error = null;
      activeTab = 'properties';
      showConfirmModal = false;
    }
  });

  async function loadData() {
    loading = true;
    error = null;
    try {
      tracks = await getTrackTagsBatch(trackIds);
      initializeForm();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function initializeForm() {
    if (tracks.length === 0) return;

    const fields = ['title', 'artist', 'album', 'albumArtist', 'genre', 'trackNo', 'discNo', 'year'];
    const newValues: Record<string, string> = {};
    const newChecked: Record<string, boolean> = {};

    // Initialize fields
    for (const field of fields) {
      const firstVal = getField(tracks[0], field);
      const allSame = tracks.every(t => getField(t, field) === firstVal);
      
      if (allSame) {
        newValues[field] = firstVal;
      } else {
        newValues[field] = ''; // Mixed state
      }
      
      // In multi-mode, default to unchecked (don't update)
      // In single-mode, checked doesn't matter but we can set true
      newChecked[field] = !isMulti; 
    }

    fieldValues = newValues;
    fieldChecked = newChecked;
    lyrics = ''; // TODO: Load lyrics if available in TrackRow (not yet)
  }

  function getField(track: TrackRow, field: string): string {
    const val = (track as any)[field];
    return val === undefined || val === null ? '' : String(val);
  }

  function handleFieldChange(field: string, value: string) {
    fieldValues[field] = value;
  }

  function handleFieldCheck(field: string, checked: boolean) {
    fieldChecked[field] = checked;
  }

  // Build patches
  function buildTagPatch(field: string, value: string, checked: boolean): TagPatch {
    if (isMulti && !checked) return { op: 'leave' };
    if (value === '') return { op: 'clear' }; // Or should empty string be clear? Yes.
    return { op: 'set', value: value.trim() };
  }

  function buildNumberPatch(field: string, value: string, checked: boolean): NumberPatch {
    if (isMulti && !checked) return { op: 'leave' };
    if (value === '') return { op: 'clear' };
    const num = parseInt(value, 10);
    if (isNaN(num)) return { op: 'leave' }; // Should validate before
    return { op: 'set', value: num };
  }

  async function handleApply() {
    if (isMulti) {
      showConfirmModal = true;
    } else {
      await performSave();
    }
  }

  async function performSave() {
    saving = true;
    error = null;
    showConfirmModal = false;

    try {
      if (isMulti) {
        const request: BatchUpdateRequest = {
          trackIds,
          createBackup: true, // TODO: Make configurable
          title: buildTagPatch('title', fieldValues.title, fieldChecked.title),
          artist: buildTagPatch('artist', fieldValues.artist, fieldChecked.artist),
          album: buildTagPatch('album', fieldValues.album, fieldChecked.album),
          albumArtist: buildTagPatch('albumArtist', fieldValues.albumArtist, fieldChecked.albumArtist),
          genre: buildTagPatch('genre', fieldValues.genre, fieldChecked.genre),
          trackNo: buildNumberPatch('trackNo', fieldValues.trackNo, fieldChecked.trackNo),
          discNo: buildNumberPatch('discNo', fieldValues.discNo, fieldChecked.discNo),
          year: buildNumberPatch('year', fieldValues.year, fieldChecked.year),
        };
        await updateTrackTagsBatch(request);
      } else {
        // Single track update
        await updateTrackTags({
          trackId: tracks[0].id,
          createBackup: true,
          title: buildTagPatch('title', fieldValues.title, true),
          artist: buildTagPatch('artist', fieldValues.artist, true),
          album: buildTagPatch('album', fieldValues.album, true),
          albumArtist: buildTagPatch('albumArtist', fieldValues.albumArtist, true),
          genre: buildTagPatch('genre', fieldValues.genre, true),
          trackNo: buildNumberPatch('trackNo', fieldValues.trackNo, true),
          discNo: buildNumberPatch('discNo', fieldValues.discNo, true),
          year: buildNumberPatch('year', fieldValues.year, true),
        });
      }

      await loadTracks(); // Refresh library view
      onclose();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<Modal {open} title={isMulti ? `Edit ${tracks.length} Tracks` : 'Edit Tags'} onclose={onclose} preventClose={saving} draggable={true}>
  <div class="tag-editor">
    {#if loading}
      <div class="loading">Loading tags...</div>
    {:else if showConfirmModal}
      <div class="confirm-overlay">
        <h3>Confirm Bulk Update</h3>
        <p>You are about to update tags for <strong>{tracks.length} tracks</strong>.</p>
        <p class="warning">This operation cannot be undone easily (backups are created).</p>
        
        <div class="changes-list">
          <h4>Changes to apply:</h4>
          <ul>
            {#each Object.entries(fieldChecked) as [field, checked]}
              {#if checked}
                <li>
                  <strong>{field}:</strong> 
                  {#if fieldValues[field]}
                    "{fieldValues[field]}"
                  {:else}
                    <em>(Clear)</em>
                  {/if}
                </li>
              {/if}
            {/each}
          </ul>
        </div>

        <div class="confirm-actions">
          <button class="btn btn-secondary" onclick={() => showConfirmModal = false}>Cancel</button>
          <button class="btn btn-danger" onclick={performSave}>Confirm Update</button>
        </div>
      </div>
    {:else}
      <div class="tab-bar">
        {#each tabs as tab}
          <button
            class="tab"
            class:active={activeTab === tab.id}
            onclick={() => activeTab = tab.id}
          >
            {tab.label}
          </button>
        {/each}
      </div>

      <div class="tab-content">
        {#if activeTab === 'properties'}
          <InspectorTab {tracks} />
        {:else if activeTab === 'tags'}
          <MetadataTab 
            {tracks} 
            values={fieldValues} 
            checked={fieldChecked} 
            onChange={handleFieldChange}
            onCheck={handleFieldCheck}
            disabled={saving}
          />
        {:else if activeTab === 'tags2'}
          <div class="placeholder-tab">Tags (2) - Not implemented yet</div>
        {:else if activeTab === 'sorting'}
          <div class="placeholder-tab">Sorting - Not implemented yet</div>
        {:else if activeTab === 'artwork'}
          <ArtworkTab {tracks} disabled={saving} />
        {:else if activeTab === 'lyrics'}
          <LyricsTab lyrics={lyrics} onChange={(v) => lyrics = v} disabled={saving} />
        {:else if activeTab === 'settings'}
          <div class="placeholder-tab">Settings - Not implemented yet</div>
        {/if}
      </div>

      {#if error}
        <div class="error-banner">{error}</div>
      {/if}

      <div class="actions">
        <button class="btn btn-secondary" onclick={onclose} disabled={saving}>Cancel</button>
        <button class="btn btn-primary" onclick={handleApply} disabled={!canApply}>
          {saving ? 'Saving...' : 'Apply'}
        </button>
      </div>
    {/if}
  </div>
</Modal>

<style>
  .tag-editor {
    width: 600px;
    min-height: 400px;
    display: flex;
    flex-direction: column;
  }

  .loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #888;
  }

  .tab-bar {
    display: flex;
    gap: 0;
    margin-bottom: 1rem;
    border-bottom: 1px solid var(--divider-color, rgba(255, 255, 255, 0.12));
  }

  .tab {
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    padding: 0.6rem 1rem;
    font-size: 0.85rem;
    color: #888;
    cursor: pointer;
    transition: all 0.15s ease;
    margin-bottom: -1px;
  }

  .tab:hover {
    color: #ccc;
    background: rgba(255, 255, 255, 0.03);
  }

  .tab.active {
    color: #fff;
    border-bottom-color: #4af;
    background: rgba(68, 170, 255, 0.08);
  }

  .placeholder-tab {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #666;
    font-style: italic;
    min-height: 200px;
  }

  .tab-content {
    flex: 1;
    min-height: 300px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
  }

  .btn {
    padding: 0.5rem 1.25rem;
    border-radius: 6px;
    font-size: 0.95rem;
    cursor: pointer;
    transition: all 0.2s;
    border: 1px solid transparent;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: rgba(255, 255, 255, 0.08);
    border-color: var(--divider-color, rgba(255, 255, 255, 0.07));
    color: #ccc;
  }

  .btn-secondary:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }

  .btn-primary {
    background: rgba(68, 170, 255, 0.2);
    border-color: rgba(68, 170, 255, 0.4);
    color: #4af;
  }

  .btn-primary:hover:not(:disabled) {
    background: rgba(68, 170, 255, 0.3);
    color: #6cf;
  }

  .btn-danger {
    background: rgba(255, 68, 68, 0.2);
    border-color: rgba(255, 68, 68, 0.4);
    color: #f88;
  }

  .btn-danger:hover:not(:disabled) {
    background: rgba(255, 68, 68, 0.3);
    color: #faa;
  }

  .error-banner {
    margin-top: 1rem;
    padding: 0.75rem;
    background: rgba(255, 68, 68, 0.15);
    border: 1px solid rgba(255, 68, 68, 0.3);
    border-radius: 6px;
    color: #f88;
  }

  /* Confirm Overlay */
  .confirm-overlay {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem;
  }

  .confirm-overlay h3 {
    margin: 0;
    font-size: 1.2rem;
    color: #fff;
  }

  .warning {
    color: #fa8;
    font-style: italic;
  }

  .changes-list {
    flex: 1;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 6px;
    padding: 1rem;
    overflow-y: auto;
  }

  .changes-list h4 {
    margin: 0 0 0.5rem 0;
    font-size: 0.9rem;
    color: #ccc;
  }

  .changes-list ul {
    margin: 0;
    padding-left: 1.5rem;
    color: #ccc;
  }

  .changes-list li {
    margin-bottom: 0.25rem;
  }

  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    margin-top: 1rem;
  }
</style>
