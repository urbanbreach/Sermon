<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import Modal from './Modal.svelte';
  import { updateTrackTags } from '../api/library';
  import { loadTracks } from '../state/library';
  import type { TrackRow, TagPatch, NumberPatch, TagWriteStatusEvent } from '../types/library';
  import { X } from '@lucide/svelte';

  interface Props {
    track: TrackRow | null;
    open: boolean;
    onclose: () => void;
  }

  let { track, open, onclose }: Props = $props();

  // Form state - initialized from track
  let title = $state('');
  let artist = $state('');
  let album = $state('');
  let albumArtist = $state('');
  let genre = $state('');
  let trackNo = $state('');
  let discNo = $state('');
  let year = $state('');

  // Track which fields have been explicitly cleared
  let clearedFields = $state<Set<string>>(new Set());

  // Original values for comparison
  let originalValues = $state<Record<string, string>>({});

  // UI state
  let createBackup = $state(true);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let retryStatus = $state<{ attempt: number; maxAttempts: number } | null>(null);

  // Event listener cleanup
  let unlistenStatus: (() => void) | null = null;

  // Initialize form when track changes
  $effect(() => {
    if (track && open) {
      title = track.title ?? '';
      artist = track.artist ?? '';
      album = track.album ?? '';
      albumArtist = track.albumArtist ?? '';
      genre = track.genre ?? '';
      trackNo = track.trackNo?.toString() ?? '';
      discNo = track.discNo?.toString() ?? '';
      year = track.year?.toString() ?? '';
      
      originalValues = {
        title: track.title ?? '',
        artist: track.artist ?? '',
        album: track.album ?? '',
        albumArtist: track.albumArtist ?? '',
        genre: track.genre ?? '',
        trackNo: track.trackNo?.toString() ?? '',
        discNo: track.discNo?.toString() ?? '',
        year: track.year?.toString() ?? '',
      };
      
      clearedFields = new Set();
      error = null;
      retryStatus = null;
    }
  });

  onMount(async () => {
    // Listen for tag write status events
    unlistenStatus = await listen<TagWriteStatusEvent>('evt_tag_write_status', (event) => {
      if (track && event.payload.trackId === track.id) {
        if (event.payload.phase === 'retry') {
          retryStatus = {
            attempt: event.payload.attempt,
            maxAttempts: event.payload.maxAttempts,
          };
        } else {
          retryStatus = null;
        }
      }
    });
  });

  onDestroy(() => {
    if (unlistenStatus) {
      unlistenStatus();
    }
  });

  // Check if a field has been modified
  function isModified(field: string, value: string): boolean {
    return value !== originalValues[field] || clearedFields.has(field);
  }

  // Check if form is dirty (any field modified)
  let isDirty = $derived(
    isModified('title', title) ||
    isModified('artist', artist) ||
    isModified('album', album) ||
    isModified('albumArtist', albumArtist) ||
    isModified('genre', genre) ||
    isModified('trackNo', trackNo) ||
    isModified('discNo', discNo) ||
    isModified('year', year)
  );

  // Validation: check for invalid empty strings (not cleared)
  function hasInvalidEmpty(field: string, value: string): boolean {
    // If field was originally non-empty and is now empty (whitespace only), and not explicitly cleared
    const trimmed = value.trim();
    const original = originalValues[field]?.trim() ?? '';
    return original !== '' && trimmed === '' && !clearedFields.has(field);
  }

  // Compute validation errors
  function getValidationErrors(): string[] {
    const errors: string[] = [];
    if (hasInvalidEmpty('title', title)) errors.push('Title: Use Clear to remove, or enter a value');
    if (hasInvalidEmpty('artist', artist)) errors.push('Artist: Use Clear to remove, or enter a value');
    if (hasInvalidEmpty('album', album)) errors.push('Album: Use Clear to remove, or enter a value');
    if (hasInvalidEmpty('albumArtist', albumArtist)) errors.push('Album Artist: Use Clear to remove, or enter a value');
    if (hasInvalidEmpty('genre', genre)) errors.push('Genre: Use Clear to remove, or enter a value');
    
    // Numeric validation
    if (trackNo.trim() !== '' && !clearedFields.has('trackNo') && (isNaN(Number(trackNo)) || Number(trackNo) < 0)) {
      errors.push('Track # must be a positive number');
    }
    if (discNo.trim() !== '' && !clearedFields.has('discNo') && (isNaN(Number(discNo)) || Number(discNo) < 0)) {
      errors.push('Disc # must be a positive number');
    }
    if (year.trim() !== '' && !clearedFields.has('year') && (isNaN(Number(year)) || Number(year) < 0)) {
      errors.push('Year must be a positive number');
    }
    
    return errors;
  }

  let validationErrors = $derived(getValidationErrors());

  let canApply = $derived(isDirty && validationErrors.length === 0 && !saving);

  // Build patch for a string field
  function buildTagPatch(field: string, value: string): TagPatch {
    if (clearedFields.has(field)) {
      return { op: 'clear' };
    }
    if (value.trim() !== originalValues[field]) {
      return { op: 'set', value: value.trim() };
    }
    return { op: 'leave' };
  }

  // Build patch for a number field
  function buildNumberPatch(field: string, value: string): NumberPatch {
    if (clearedFields.has(field)) {
      return { op: 'clear' };
    }
    const trimmed = value.trim();
    if (trimmed !== originalValues[field] && trimmed !== '') {
      return { op: 'set', value: parseInt(trimmed, 10) };
    }
    return { op: 'leave' };
  }

  // Clear a field
  function clearField(field: string) {
    clearedFields = new Set([...clearedFields, field]);
    switch (field) {
      case 'title': title = ''; break;
      case 'artist': artist = ''; break;
      case 'album': album = ''; break;
      case 'albumArtist': albumArtist = ''; break;
      case 'genre': genre = ''; break;
      case 'trackNo': trackNo = ''; break;
      case 'discNo': discNo = ''; break;
      case 'year': year = ''; break;
    }
  }

  // Handle input change - remove from cleared set if user types
  function handleInput(field: string) {
    if (clearedFields.has(field)) {
      const newSet = new Set(clearedFields);
      newSet.delete(field);
      clearedFields = newSet;
    }
  }

  async function handleApply() {
    if (!track || !canApply) return;

    saving = true;
    error = null;
    retryStatus = null;

    try {
      await updateTrackTags({
        trackId: track.id,
        createBackup,
        title: buildTagPatch('title', title),
        artist: buildTagPatch('artist', artist),
        album: buildTagPatch('album', album),
        albumArtist: buildTagPatch('albumArtist', albumArtist),
        genre: buildTagPatch('genre', genre),
        trackNo: buildNumberPatch('trackNo', trackNo),
        discNo: buildNumberPatch('discNo', discNo),
        year: buildNumberPatch('year', year),
      });

      // Refresh tracks list
      await loadTracks();
      
      // Close modal on success
      onclose();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
      retryStatus = null;
    }
  }

  function handleCancel() {
    if (!saving) {
      onclose();
    }
  }
</script>

<Modal {open} title="Edit Tags" onclose={handleCancel} preventClose={saving}>
  {#if track}
    <div class="tag-editor">
      {#if error}
        <div class="error-banner">
          <span class="error-icon">!</span>
          <span class="error-text">{error}</span>
        </div>
      {/if}

      {#if retryStatus}
        <div class="retry-banner">
          Retrying save... (attempt {retryStatus.attempt}/{retryStatus.maxAttempts})
        </div>
      {/if}

      <div class="form-grid">
        <!-- Title -->
        <div class="field">
          <label for="tag-title">Title</label>
          <div class="input-group">
            <input
              id="tag-title"
              type="text"
              bind:value={title}
              oninput={() => handleInput('title')}
              class:cleared={clearedFields.has('title')}
              disabled={saving}
            />
            <button
              type="button"
              class="clear-btn"
              onclick={() => clearField('title')}
              disabled={saving || clearedFields.has('title')}
              title="Clear field"
            ><X size={14} /></button>
          </div>
        </div>

        <!-- Artist -->
        <div class="field">
          <label for="tag-artist">Artist</label>
          <div class="input-group">
            <input
              id="tag-artist"
              type="text"
              bind:value={artist}
              oninput={() => handleInput('artist')}
              class:cleared={clearedFields.has('artist')}
              disabled={saving}
            />
            <button
              type="button"
              class="clear-btn"
              onclick={() => clearField('artist')}
              disabled={saving || clearedFields.has('artist')}
              title="Clear field"
            ><X size={14} /></button>
          </div>
        </div>

        <!-- Album -->
        <div class="field">
          <label for="tag-album">Album</label>
          <div class="input-group">
            <input
              id="tag-album"
              type="text"
              bind:value={album}
              oninput={() => handleInput('album')}
              class:cleared={clearedFields.has('album')}
              disabled={saving}
            />
            <button
              type="button"
              class="clear-btn"
              onclick={() => clearField('album')}
              disabled={saving || clearedFields.has('album')}
              title="Clear field"
            ><X size={14} /></button>
          </div>
        </div>

        <!-- Album Artist -->
        <div class="field">
          <label for="tag-album-artist">Album Artist</label>
          <div class="input-group">
            <input
              id="tag-album-artist"
              type="text"
              bind:value={albumArtist}
              oninput={() => handleInput('albumArtist')}
              class:cleared={clearedFields.has('albumArtist')}
              disabled={saving}
            />
            <button
              type="button"
              class="clear-btn"
              onclick={() => clearField('albumArtist')}
              disabled={saving || clearedFields.has('albumArtist')}
              title="Clear field"
            ><X size={14} /></button>
          </div>
        </div>

        <!-- Genre -->
        <div class="field">
          <label for="tag-genre">Genre</label>
          <div class="input-group">
            <input
              id="tag-genre"
              type="text"
              bind:value={genre}
              oninput={() => handleInput('genre')}
              class:cleared={clearedFields.has('genre')}
              disabled={saving}
            />
            <button
              type="button"
              class="clear-btn"
              onclick={() => clearField('genre')}
              disabled={saving || clearedFields.has('genre')}
              title="Clear field"
            ><X size={14} /></button>
          </div>
        </div>

        <!-- Track # -->
        <div class="field half">
          <label for="tag-track-no">Track #</label>
          <div class="input-group">
            <input
              id="tag-track-no"
              type="text"
              inputmode="numeric"
              bind:value={trackNo}
              oninput={() => handleInput('trackNo')}
              class:cleared={clearedFields.has('trackNo')}
              disabled={saving}
            />
            <button
              type="button"
              class="clear-btn"
              onclick={() => clearField('trackNo')}
              disabled={saving || clearedFields.has('trackNo')}
              title="Clear field"
            ><X size={14} /></button>
          </div>
        </div>

        <!-- Disc # -->
        <div class="field half">
          <label for="tag-disc-no">Disc #</label>
          <div class="input-group">
            <input
              id="tag-disc-no"
              type="text"
              inputmode="numeric"
              bind:value={discNo}
              oninput={() => handleInput('discNo')}
              class:cleared={clearedFields.has('discNo')}
              disabled={saving}
            />
            <button
              type="button"
              class="clear-btn"
              onclick={() => clearField('discNo')}
              disabled={saving || clearedFields.has('discNo')}
              title="Clear field"
            ><X size={14} /></button>
          </div>
        </div>

        <!-- Year -->
        <div class="field half">
          <label for="tag-year">Year</label>
          <div class="input-group">
            <input
              id="tag-year"
              type="text"
              inputmode="numeric"
              bind:value={year}
              oninput={() => handleInput('year')}
              class:cleared={clearedFields.has('year')}
              disabled={saving}
            />
            <button
              type="button"
              class="clear-btn"
              onclick={() => clearField('year')}
              disabled={saving || clearedFields.has('year')}
              title="Clear field"
            ><X size={14} /></button>
          </div>
        </div>
      </div>

      {#if validationErrors.length > 0}
        <div class="validation-errors">
          {#each validationErrors as err}
            <div class="validation-error">{err}</div>
          {/each}
        </div>
      {/if}

      <div class="options">
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={createBackup} disabled={saving} />
          Create backup before writing
        </label>
      </div>

      <div class="actions">
        <button class="btn btn-secondary" onclick={handleCancel} disabled={saving}>
          Cancel
        </button>
        <button class="btn btn-primary" onclick={handleApply} disabled={!canApply}>
          {#if saving}
            Saving...
          {:else}
            Apply
          {/if}
        </button>
      </div>
    </div>
  {/if}
</Modal>


<style>
  .tag-editor {
    min-width: 450px;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: rgba(255, 68, 68, 0.15);
    border: 1px solid rgba(255, 68, 68, 0.3);
    border-radius: 6px;
    padding: 0.75rem 1rem;
    margin-bottom: 1rem;
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

  .retry-banner {
    background: rgba(68, 170, 255, 0.15);
    border: 1px solid rgba(68, 170, 255, 0.3);
    border-radius: 6px;
    padding: 0.75rem 1rem;
    margin-bottom: 1rem;
    color: #4af;
    text-align: center;
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .field {
    grid-column: span 2;
  }

  .field.half {
    grid-column: span 1;
  }

  label {
    display: block;
    font-size: 0.85rem;
    color: #888;
    margin-bottom: 0.25rem;
  }

  .input-group {
    display: flex;
    gap: 0.25rem;
  }

  input[type="text"] {
    flex: 1;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--glass-border);
    border-radius: 6px;
    color: #fff;
    padding: 0.5rem 0.75rem;
    font-size: 0.95rem;
    outline: none;
    transition: all 0.2s;
  }

  input[type="text"]:focus {
    background: rgba(255, 255, 255, 0.12);
    border-color: rgba(68, 170, 255, 0.5);
  }

  input[type="text"]:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  input[type="text"].cleared {
    background: rgba(255, 170, 68, 0.1);
    border-color: rgba(255, 170, 68, 0.3);
    font-style: italic;
    color: #fa8;
  }

  input[type="text"].cleared::placeholder {
    color: #fa8;
  }

  .clear-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--glass-border);
    border-radius: 6px;
    color: #888;
    width: 32px;
    cursor: pointer;
    font-size: 1.2rem;
    transition: all 0.2s;
  }

  .clear-btn:hover:not(:disabled) {
    background: rgba(255, 68, 68, 0.2);
    border-color: rgba(255, 68, 68, 0.4);
    color: #f88;
  }

  .clear-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .validation-errors {
    margin-top: 1rem;
    padding: 0.75rem;
    background: rgba(255, 170, 68, 0.1);
    border: 1px solid rgba(255, 170, 68, 0.3);
    border-radius: 6px;
  }

  .validation-error {
    font-size: 0.85rem;
    color: #fa8;
    padding: 0.25rem 0;
  }

  .options {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--glass-border);
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    color: #ccc;
    font-size: 0.9rem;
  }

  input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: #4af;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--glass-border);
  }

  .btn {
    padding: 0.5rem 1.25rem;
    border-radius: 6px;
    font-size: 0.95rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--glass-border);
    color: #ccc;
  }

  .btn-secondary:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }

  .btn-primary {
    background: rgba(68, 170, 255, 0.2);
    border: 1px solid rgba(68, 170, 255, 0.4);
    color: #4af;
  }

  .btn-primary:hover:not(:disabled) {
    background: rgba(68, 170, 255, 0.3);
    color: #6cf;
  }
</style>
