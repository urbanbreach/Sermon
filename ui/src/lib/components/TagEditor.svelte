<script lang="ts">
  import Modal from './Modal.svelte';
  import { updateTrackTags, getTrackTagsBatch, updateTrackTagsBatch } from '../api/library';
  import { getLyricsForTrack } from '../api/lyrics';
  import { showTagEditorInMainPanel } from '../state/tagEditorWindow';
  import { loadTracks } from '../state/library';
  import type { TrackTagSnapshot, TagPatch, NumberPatch, BatchUpdateRequest } from '../types/library';

  import MetadataTab from './tag-editor/MetadataTab.svelte';
  import Tags2Tab from './tag-editor/Tags2Tab.svelte';
  import ArtworkTab from './tag-editor/ArtworkTab.svelte';
  import InspectorTab from './tag-editor/InspectorTab.svelte';
  import LyricsTab from './tag-editor/LyricsTab.svelte';
  import AutoTagTab from './tag-editor/AutoTagTab.svelte';
  import SortingTab from './tag-editor/SortingTab.svelte';
  import SettingsTab from './tag-editor/SettingsTab.svelte';

  type FieldKey =
    | 'title'
    | 'artist'
    | 'album'
    | 'albumArtist'
    | 'genre'
    | 'publisher'
    | 'composer'
    | 'conductor'
    | 'comments'
    | 'grouping'
    | 'trackNo'
    | 'discNo'
    | 'year';

  type FieldValueMap = Record<FieldKey, string>;
  type FieldCheckedMap = Record<FieldKey, boolean>;

  interface AutoTagCandidate {
    title: string;
    artist: string;
    album?: string;
    year?: string;
    trackNo?: number;
    discNo?: number;
  }

  interface Props {
    trackIds: number[];
    open: boolean;
    onclose: () => void;
    windowed?: boolean;
    embedded?: boolean;
  }

  const FIELD_KEYS: FieldKey[] = [
    'title',
    'artist',
    'album',
    'albumArtist',
    'genre',
    'publisher',
    'composer',
    'conductor',
    'comments',
    'grouping',
    'trackNo',
    'discNo',
    'year',
  ];

  const FIELD_LABELS: Record<FieldKey, string> = {
    title: 'Track Title',
    artist: 'Artist',
    album: 'Album',
    albumArtist: 'Album Artist',
    genre: 'Genre',
    publisher: 'Publisher',
    composer: 'Composer',
    conductor: 'Conductor',
    comments: 'Comments',
    grouping: 'Grouping',
    trackNo: 'Track #',
    discNo: 'Disc #',
    year: 'Year',
  };

  const EMPTY_VALUES: FieldValueMap = {
    title: '',
    artist: '',
    album: '',
    albumArtist: '',
    genre: '',
    publisher: '',
    composer: '',
    conductor: '',
    comments: '',
    grouping: '',
    trackNo: '',
    discNo: '',
    year: '',
  };

  const EMPTY_CHECKED: FieldCheckedMap = {
    title: false,
    artist: false,
    album: false,
    albumArtist: false,
    genre: false,
    publisher: false,
    composer: false,
    conductor: false,
    comments: false,
    grouping: false,
    trackNo: false,
    discNo: false,
    year: false,
  };

  const tabs = [
    { id: 'properties', label: 'Properties' },
    { id: 'tags', label: 'Tags' },
    { id: 'tags2', label: 'Tags (2)' },
    { id: 'sorting', label: 'Sorting' },
    { id: 'artwork', label: 'Artwork' },
    { id: 'lyrics', label: 'Lyrics' },
    { id: 'settings', label: 'Settings' },
  ] as const;

  let { trackIds, open, onclose, windowed = false, embedded = false }: Props = $props();

  let tracks = $state<TrackTagSnapshot[]>([]);
  let loading = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let notice = $state<string | null>(null);
  let activeTab = $state<(typeof tabs)[number]['id']>('properties');
  let showConfirmModal = $state(false);
  let showAutoTagAssistant = $state(false);

  let createBackup = $state(true);
  let closeAfterApply = $state(true);

  $effect(() => {
    closeAfterApply = !embedded;
  });

  let fieldValues = $state<FieldValueMap>({ ...EMPTY_VALUES });
  let fieldChecked = $state<FieldCheckedMap>({ ...EMPTY_CHECKED });
  let initialFieldValues = $state<FieldValueMap>({ ...EMPTY_VALUES });
  let initialFieldChecked = $state<FieldCheckedMap>({ ...EMPTY_CHECKED });

  let lyrics = $state('');
  let syncedLyrics = $state('');
  let lyricist = $state('');
  let markNoLyrics = $state(false);
  let initialLyrics = $state('');
  let initialSyncedLyrics = $state('');
  let initialLyricist = $state('');
  let initialMarkNoLyrics = $state(false);

  let isMulti = $derived(trackIds.length > 1);
  let isFramelessMode = $derived(windowed || embedded);
  let isEditorActive = $derived(
    isFramelessMode ? trackIds.length > 0 : open && trackIds.length > 0,
  );
  let canApply = $derived(
    !loading && !saving && (isMulti ? FIELD_KEYS.some((field) => fieldChecked[field]) : true),
  );

  $effect(() => {
    if (isEditorActive) {
      void loadData();
    } else {
      resetEditorState();
    }
  });

  function resetEditorState(): void {
    tracks = [];
    fieldValues = { ...EMPTY_VALUES };
    fieldChecked = { ...EMPTY_CHECKED };
    initialFieldValues = { ...EMPTY_VALUES };
    initialFieldChecked = { ...EMPTY_CHECKED };
    lyrics = '';
    syncedLyrics = '';
    lyricist = '';
    markNoLyrics = false;
    initialLyrics = '';
    initialSyncedLyrics = '';
    initialLyricist = '';
    initialMarkNoLyrics = false;
    error = null;
    notice = null;
    activeTab = 'properties';
    showConfirmModal = false;
    showAutoTagAssistant = false;
  }

  async function loadData(): Promise<void> {
    loading = true;
    error = null;
    notice = null;

    try {
      tracks = await getTrackTagsBatch(trackIds);
      initializeForm();
      await loadLyricsData();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function getField(track: TrackTagSnapshot, field: FieldKey): string {
    switch (field) {
      case 'title':
        return track.title ?? '';
      case 'artist':
        return track.artist ?? '';
      case 'album':
        return track.album ?? '';
      case 'albumArtist':
        return track.albumArtist ?? '';
      case 'genre':
        return track.genre ?? '';
      case 'trackNo':
        return track.trackNo == null ? '' : String(track.trackNo);
      case 'discNo':
        return track.discNo == null ? '' : String(track.discNo);
      case 'year':
        return track.year == null ? '' : String(track.year);
      default:
        return '';
    }
  }

  function initializeForm(): void {
    if (tracks.length === 0) {
      return;
    }

    const nextValues: FieldValueMap = { ...EMPTY_VALUES };
    const nextChecked: FieldCheckedMap = { ...EMPTY_CHECKED };

    for (const field of FIELD_KEYS) {
      const firstValue = getField(tracks[0], field);
      const allSame = tracks.every((track) => getField(track, field) === firstValue);
      nextValues[field] = allSame ? firstValue : '';
      nextChecked[field] = !isMulti;
    }

    fieldValues = nextValues;
    fieldChecked = nextChecked;
    initialFieldValues = { ...nextValues };
    initialFieldChecked = { ...nextChecked };
  }

  async function loadLyricsData(): Promise<void> {
    if (tracks.length !== 1) {
      lyrics = '';
      syncedLyrics = '';
      lyricist = '';
      markNoLyrics = false;
      initialLyrics = '';
      initialSyncedLyrics = '';
      initialLyricist = '';
      initialMarkNoLyrics = false;
      return;
    }

    try {
      const response = await getLyricsForTrack(tracks[0].trackId);
      lyrics = response.plainLyrics ?? '';
      syncedLyrics = response.syncedLyrics ?? '';
      lyricist = response.lyricist ?? '';
      markNoLyrics = response.source === 'manual-none';
      initialLyrics = lyrics;
      initialSyncedLyrics = syncedLyrics;
      initialLyricist = lyricist;
      initialMarkNoLyrics = markNoLyrics;
    } catch {
      lyrics = '';
      syncedLyrics = '';
      lyricist = '';
      markNoLyrics = false;
      initialLyrics = '';
      initialSyncedLyrics = '';
      initialLyricist = '';
      initialMarkNoLyrics = false;
    }
  }

  function resetEdits(): void {
    fieldValues = { ...initialFieldValues };
    fieldChecked = { ...initialFieldChecked };
    lyrics = initialLyrics;
    syncedLyrics = initialSyncedLyrics;
    lyricist = initialLyricist;
    markNoLyrics = initialMarkNoLyrics;
    error = null;
    notice = null;
  }

  function handleFieldChange(field: FieldKey, value: string): void {
    fieldValues = {
      ...fieldValues,
      [field]: value,
    };
    notice = null;
  }

  function handleFieldCheck(field: FieldKey, checked: boolean): void {
    fieldChecked = {
      ...fieldChecked,
      [field]: checked,
    };
    notice = null;
  }

  function handleMetadataChange(field: string, value: string): void {
    if (!FIELD_KEYS.includes(field as FieldKey)) {
      return;
    }

    handleFieldChange(field as FieldKey, value);
  }

  function handleMetadataCheck(field: string, checked: boolean): void {
    if (!FIELD_KEYS.includes(field as FieldKey)) {
      return;
    }

    handleFieldCheck(field as FieldKey, checked);
  }

  function markFieldForApply(field: FieldKey): void {
    if (!isMulti) {
      return;
    }

    fieldChecked = {
      ...fieldChecked,
      [field]: true,
    };
  }

  function handleAutoTagApply(event: CustomEvent<AutoTagCandidate>): void {
    const candidate = event.detail;
    const nextValues: FieldValueMap = { ...fieldValues };

    if (candidate.title?.trim()) {
      nextValues.title = candidate.title.trim();
      markFieldForApply('title');
    }

    if (candidate.artist?.trim()) {
      nextValues.artist = candidate.artist.trim();
      markFieldForApply('artist');
    }

    if (candidate.album?.trim()) {
      nextValues.album = candidate.album.trim();
      markFieldForApply('album');
    }

    if (candidate.year?.trim()) {
      nextValues.year = candidate.year.trim();
      markFieldForApply('year');
    }

    if (candidate.trackNo != null) {
      nextValues.trackNo = String(candidate.trackNo);
      markFieldForApply('trackNo');
    }

    if (candidate.discNo != null) {
      nextValues.discNo = String(candidate.discNo);
      markFieldForApply('discNo');
    }

    fieldValues = nextValues;
    showAutoTagAssistant = false;
    activeTab = 'tags';
    notice = 'Auto-tag suggestion applied. Review and save.';
  }

  function handleSortingApply(
    partialValues: Partial<Pick<FieldValueMap, 'album' | 'albumArtist' | 'artist' | 'composer'>>,
  ): void {
    fieldValues = {
      ...fieldValues,
      ...partialValues,
    };

    if (partialValues.album != null) {
      markFieldForApply('album');
    }
    if (partialValues.albumArtist != null) {
      markFieldForApply('albumArtist');
    }
    if (partialValues.artist != null) {
      markFieldForApply('artist');
    }
    if (partialValues.composer != null) {
      markFieldForApply('composer');
    }

    notice = 'Sorting values applied to editable fields.';
  }

  async function handleShowInMainPanel(): Promise<void> {
    await showTagEditorInMainPanel(trackIds);

    if (windowed) {
      onclose();
    }
  }

  function buildTagPatch(value: string, checked: boolean): TagPatch {
    if (isMulti && !checked) {
      return { op: 'leave' };
    }

    const trimmed = value.trim();
    if (trimmed === '') {
      return { op: 'clear' };
    }

    return { op: 'set', value: trimmed };
  }

  function buildNumberPatch(value: string, checked: boolean): NumberPatch {
    if (isMulti && !checked) {
      return { op: 'leave' };
    }

    const trimmed = value.trim();
    if (trimmed === '') {
      return { op: 'clear' };
    }

    const parsed = Number.parseInt(trimmed, 10);
    if (Number.isNaN(parsed)) {
      return { op: 'leave' };
    }

    return { op: 'set', value: parsed };
  }

  async function handleApply(): Promise<void> {
    if (isMulti) {
      showConfirmModal = true;
      return;
    }

    await performSave();
  }

  async function performSave(): Promise<void> {
    saving = true;
    error = null;
    notice = null;
    showConfirmModal = false;

    try {
      if (isMulti) {
        const request: BatchUpdateRequest = {
          trackIds,
          createBackup,
          title: buildTagPatch(fieldValues.title, fieldChecked.title),
          artist: buildTagPatch(fieldValues.artist, fieldChecked.artist),
          album: buildTagPatch(fieldValues.album, fieldChecked.album),
          albumArtist: buildTagPatch(fieldValues.albumArtist, fieldChecked.albumArtist),
          genre: buildTagPatch(fieldValues.genre, fieldChecked.genre),
          publisher: buildTagPatch(fieldValues.publisher, fieldChecked.publisher),
          composer: buildTagPatch(fieldValues.composer, fieldChecked.composer),
          conductor: buildTagPatch(fieldValues.conductor, fieldChecked.conductor),
          comments: buildTagPatch(fieldValues.comments, fieldChecked.comments),
          grouping: buildTagPatch(fieldValues.grouping, fieldChecked.grouping),
          lyricist: { op: 'leave' },
          plainLyrics: { op: 'leave' },
          syncedLyrics: { op: 'leave' },
          trackNo: buildNumberPatch(fieldValues.trackNo, fieldChecked.trackNo),
          discNo: buildNumberPatch(fieldValues.discNo, fieldChecked.discNo),
          year: buildNumberPatch(fieldValues.year, fieldChecked.year),
        };

        await updateTrackTagsBatch(request);
      } else {
        if (tracks.length === 0) {
          throw new Error('No track loaded to update.');
        }

        const plainLyricsPatch: TagPatch = markNoLyrics
          ? { op: 'clear' }
          : buildTagPatch(lyrics, true);
        const syncedLyricsPatch: TagPatch = markNoLyrics
          ? { op: 'clear' }
          : buildTagPatch(syncedLyrics, true);
        const lyricistPatch: TagPatch = markNoLyrics
          ? { op: 'clear' }
          : buildTagPatch(lyricist, true);

        await updateTrackTags({
          trackId: tracks[0].trackId,
          createBackup,
          title: buildTagPatch(fieldValues.title, true),
          artist: buildTagPatch(fieldValues.artist, true),
          album: buildTagPatch(fieldValues.album, true),
          albumArtist: buildTagPatch(fieldValues.albumArtist, true),
          genre: buildTagPatch(fieldValues.genre, true),
          publisher: buildTagPatch(fieldValues.publisher, true),
          composer: buildTagPatch(fieldValues.composer, true),
          conductor: buildTagPatch(fieldValues.conductor, true),
          comments: buildTagPatch(fieldValues.comments, true),
          grouping: buildTagPatch(fieldValues.grouping, true),
          lyricist: lyricistPatch,
          plainLyrics: plainLyricsPatch,
          syncedLyrics: syncedLyricsPatch,
          trackNo: buildNumberPatch(fieldValues.trackNo, true),
          discNo: buildNumberPatch(fieldValues.discNo, true),
          year: buildNumberPatch(fieldValues.year, true),
        });
      }

      await loadTracks();

      if (closeAfterApply || !isFramelessMode) {
        onclose();
      } else {
        notice = 'Changes saved successfully.';
        await loadData();
      }
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

{#snippet editorContent()}
  <div class="tag-editor" class:windowed={isFramelessMode}>
    {#if loading}
      <div class="loading">Loading tags...</div>
    {:else if showConfirmModal}
      <div class="confirm-overlay">
        <h3>Confirm Bulk Save</h3>
        <p>You are about to save metadata updates for <strong>{tracks.length} tracks</strong>.</p>

        <div class="changes-list">
          <h4>Fields to update:</h4>
          <ul>
            {#each FIELD_KEYS as field}
              {#if fieldChecked[field]}
                <li>
                  <strong>{FIELD_LABELS[field]}:</strong>
                  {#if fieldValues[field].trim()}
                    "{fieldValues[field].trim()}"
                  {:else}
                    <em>(Clear)</em>
                  {/if}
                </li>
              {/if}
            {/each}
          </ul>
        </div>

        <div class="confirm-actions">
          <button class="btn btn-secondary" onclick={() => (showConfirmModal = false)}>Cancel</button>
          <button class="btn btn-primary" onclick={performSave}>Save Changes</button>
        </div>
      </div>
    {:else}
      <div class="command-row">
        <div class="command-left">
          <button class="cmd-btn" onclick={resetEdits} disabled={saving}>Undo</button>
          <button class="cmd-btn" onclick={() => (activeTab = 'properties')} disabled={saving}>Tag Inspector</button>
          <button class="cmd-btn" onclick={() => (activeTab = 'tags')} disabled={saving}>Rename</button>
          <button class="cmd-btn" onclick={() => (showAutoTagAssistant = true)} disabled={saving}>
            Auto-Tag
          </button>
        </div>

        {#if windowed}
          <button class="show-main" onclick={handleShowInMainPanel} disabled={saving}>Show in Main Panel</button>
        {/if}
      </div>

      <div class="tab-bar">
        {#each tabs as tab}
          <button class="tab" class:active={activeTab === tab.id} onclick={() => (activeTab = tab.id)}>
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
            onChange={handleMetadataChange}
            onCheck={handleMetadataCheck}
            disabled={saving}
          />
        {:else if activeTab === 'tags2'}
          <Tags2Tab disabled={saving} />
        {:else if activeTab === 'sorting'}
          <SortingTab
            values={{
              album: fieldValues.album,
              albumArtist: fieldValues.albumArtist,
              artist: fieldValues.artist,
              composer: fieldValues.composer,
            }}
            disabled={saving}
            onApply={handleSortingApply}
          />
        {:else if activeTab === 'artwork'}
          <ArtworkTab {tracks} disabled={saving} />
        {:else if activeTab === 'lyrics'}
          <LyricsTab
            {lyrics}
            {syncedLyrics}
            {lyricist}
            {markNoLyrics}
            disabled={saving || isMulti}
            onChange={(value) => (lyrics = value)}
            onChangeSynced={(value) => (syncedLyrics = value)}
            onChangeLyricist={(value) => (lyricist = value)}
            onMarkNoLyricsChange={(value) => (markNoLyrics = value)}
            onSearchInternet={() => (showAutoTagAssistant = true)}
          />
        {:else if activeTab === 'settings'}
          <SettingsTab
            {isMulti}
            trackCount={tracks.length}
            {createBackup}
            {closeAfterApply}
            disabled={saving}
            onCreateBackupChange={(value) => (createBackup = value)}
            onCloseAfterApplyChange={(value) => (closeAfterApply = value)}
            onResetForm={resetEdits}
          />
        {/if}
      </div>

      {#if error}
        <div class="error-banner">{error}</div>
      {/if}

      {#if notice}
        <div class="notice-banner">{notice}</div>
      {/if}

      <div class="actions-row">
        <div class="nav-group">
          <button class="nav-btn" type="button" disabled aria-label="Previous track">←</button>
          <button class="nav-btn" type="button" disabled aria-label="Next track">→</button>
        </div>

        <div class="action-group">
          <button class="btn btn-primary" onclick={handleApply} disabled={!canApply}>
            {saving ? 'Saving...' : 'Save'}
          </button>
          <button class="btn btn-secondary" onclick={onclose} disabled={saving}>Close</button>
        </div>
      </div>
    {/if}

    {#if showAutoTagAssistant}
      <div class="auto-tag-overlay">
        <div class="auto-tag-panel">
          <div class="auto-tag-head">
            <h3>Auto-Tag</h3>
            <button type="button" class="cmd-btn" onclick={() => (showAutoTagAssistant = false)}>
              Close
            </button>
          </div>
          <AutoTagTab on:apply={handleAutoTagApply} />
        </div>
      </div>
    {/if}
  </div>
{/snippet}

{#if isFramelessMode}
  <div class="tag-editor-shell" class:embedded-shell={embedded}>
    {@render editorContent()}
  </div>
{:else}
  <Modal
    {open}
    title={isMulti ? `Edit ${tracks.length} Tracks` : 'Edit Tags'}
    onclose={onclose}
    preventClose={saving}
    draggable={true}
  >
    {@render editorContent()}
  </Modal>
{/if}

<style>
  .tag-editor {
    width: 920px;
    min-height: 560px;
    height: 100%;
    display: flex;
    flex-direction: column;
    position: relative;
    background: var(--surface-0);
  }

  .tag-editor.windowed {
    width: 100%;
    min-height: 0;
    padding: 10px 12px;
  }

  .tag-editor-shell {
    width: 100%;
    height: 100%;
    min-height: 0;
    display: flex;
    flex: 1;
    background: var(--surface-0);
  }

  .tag-editor-shell.embedded-shell {
    border-top: 1px solid var(--divider-color);
  }

  .loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    font-size: 14px;
  }

  .command-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 30px;
    padding: 0 2px 6px;
  }

  .command-left {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .cmd-btn,
  .show-main {
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 13px;
    padding: 4px 6px;
    cursor: pointer;
    border-radius: 4px;
  }

  .cmd-btn:hover,
  .show-main:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .cmd-btn:disabled,
  .show-main:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tab-bar {
    display: flex;
    align-items: flex-end;
    border-bottom: 1px solid var(--divider-color);
    margin-bottom: 0;
    overflow-x: auto;
  }

  .tab {
    border: 1px solid var(--divider-color);
    border-bottom: none;
    border-radius: 4px 4px 0 0;
    background: transparent;
    color: var(--text-tertiary);
    padding: 6px 12px;
    font-size: 13px;
    cursor: pointer;
    margin-right: 2px;
    margin-bottom: -1px;
  }

  .tab:hover {
    color: var(--text-primary);
  }

  .tab.active {
    color: var(--text-primary);
    border-bottom: 1px solid var(--surface-0);
  }

  .tab-content {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .actions-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-top: 1px solid var(--divider-color);
    padding: 10px 0 0;
    margin-top: 8px;
  }

  .nav-group,
  .action-group {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .nav-btn {
    width: 34px;
    height: 30px;
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-secondary);
    cursor: not-allowed;
    border-radius: 0;
  }

  .btn {
    min-width: 92px;
    border: 1px solid var(--divider-color);
    border-radius: 0;
    padding: 6px 14px;
    font-size: 13px;
    cursor: pointer;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: var(--surface-1);
    color: var(--text-primary);
  }

  .btn-primary {
    background: var(--accent-weak);
    border-color: var(--accent-medium);
    color: var(--theme-accent);
  }

  .error-banner,
  .notice-banner {
    margin-top: 8px;
    border: 1px solid;
    padding: 8px 10px;
    font-size: 13px;
  }

  .error-banner {
    border-color: rgba(239, 68, 68, 0.35);
    background: rgba(239, 68, 68, 0.12);
    color: #fca5a5;
  }

  .notice-banner {
    border-color: rgba(16, 185, 129, 0.35);
    background: rgba(16, 185, 129, 0.12);
    color: #86efac;
  }

  .confirm-overlay {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px 0;
  }

  .confirm-overlay h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: 18px;
  }

  .confirm-overlay p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 14px;
  }

  .changes-list {
    flex: 1;
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    padding: 10px;
    overflow: auto;
  }

  .changes-list h4 {
    margin: 0 0 6px;
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 500;
  }

  .changes-list ul {
    margin: 0;
    padding-left: 16px;
    color: var(--text-primary);
    font-size: 13px;
    display: grid;
    gap: 4px;
  }

  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .auto-tag-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 20;
    padding: 18px;
  }

  .auto-tag-panel {
    width: min(980px, 100%);
    max-height: 90%;
    border: 1px solid var(--divider-color);
    background: var(--surface-0);
    padding: 10px;
    overflow: auto;
  }

  .auto-tag-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .auto-tag-head h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: 15px;
  }
</style>
