<script lang="ts">
  interface SortingValues {
    album: string;
    albumArtist: string;
    artist: string;
    composer: string;
  }

  interface Props {
    values: SortingValues;
    disabled?: boolean;
    onApply: (partialValues: Partial<SortingValues>) => void;
  }

  let { values, disabled = false, onApply }: Props = $props();

  let enableCustomSorting = $state(false);
  let sortAlbum = $state('');
  let sortAlbumArtist = $state('');
  let sortArtist = $state('');
  let sortComposer = $state('');

  $effect(() => {
    sortAlbum = values.album ?? '';
    sortAlbumArtist = values.albumArtist ?? '';
    sortArtist = values.artist ?? '';
    sortComposer = values.composer ?? '';
  });

  function applySortingValues(): void {
    if (!enableCustomSorting) {
      return;
    }

    onApply({
      album: sortAlbum,
      albumArtist: sortAlbumArtist,
      artist: sortArtist,
      composer: sortComposer,
    });
  }
</script>

<div class="sorting-tab">
  <label class="enable-row">
    <input type="checkbox" bind:checked={enableCustomSorting} disabled={disabled} />
    <span>enable custom sorting - use "sort as" values when sorting</span>
  </label>

  <div class="rows" class:disabled={!enableCustomSorting}>
    <div class="pair-row">
      <label class="field"><span>album:</span><input type="text" value={values.album ?? ''} disabled /></label>
      <label class="field"><span>sort as:</span><input type="text" bind:value={sortAlbum} disabled={disabled || !enableCustomSorting} /></label>
    </div>

    <div class="pair-row">
      <label class="field"><span>album artist:</span><input type="text" value={values.albumArtist ?? ''} disabled /></label>
      <label class="field"><span>sort as:</span><input type="text" bind:value={sortAlbumArtist} disabled={disabled || !enableCustomSorting} /></label>
    </div>

    <div class="pair-row">
      <label class="field"><span>artist:</span><input type="text" value={values.artist ?? ''} disabled /></label>
      <label class="field"><span>sort as:</span><input type="text" bind:value={sortArtist} disabled={disabled || !enableCustomSorting} /></label>
    </div>

    <div class="pair-row">
      <label class="field"><span>composer:</span><input type="text" value={values.composer ?? ''} disabled /></label>
      <label class="field"><span>sort as:</span><input type="text" bind:value={sortComposer} disabled={disabled || !enableCustomSorting} /></label>
    </div>
  </div>

  <div class="footer-block">
    <div class="hint">custom sort values for individual artists and composers:</div>
    <button type="button" class="secondary" disabled={disabled || !enableCustomSorting}>Edit Custom Sort Values...</button>
  </div>

  <div class="actions">
    <button type="button" class="primary" onclick={applySortingValues} disabled={disabled || !enableCustomSorting}>
      Apply Sorting Values
    </button>
  </div>
</div>

<style>
  .sorting-tab {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 12px 0;
    overflow: auto;
  }

  .enable-row {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-secondary);
    font-size: 14px;
  }

  .enable-row input {
    width: 14px;
    height: 14px;
    accent-color: var(--theme-accent);
  }

  .rows {
    display: grid;
    gap: 8px;
  }

  .rows.disabled {
    opacity: 0.65;
  }

  .pair-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .field {
    display: grid;
    grid-template-columns: 110px minmax(0, 1fr);
    gap: 6px;
    align-items: center;
  }

  .field span {
    color: var(--text-secondary);
    font-size: 13px;
    text-transform: lowercase;
  }

  .field input {
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-primary);
    font-size: 13px;
    padding: 6px 8px;
    border-radius: 0;
  }

  .field input:disabled {
    color: var(--text-disabled);
  }

  .footer-block {
    margin-top: 8px;
    border-top: 1px solid var(--divider-color);
    padding-top: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .hint {
    color: var(--text-secondary);
    font-size: 13px;
  }

  .secondary,
  .primary {
    width: fit-content;
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-primary);
    font-size: 13px;
    padding: 6px 10px;
    cursor: pointer;
    border-radius: 0;
  }

  .primary {
    border-color: var(--accent-medium);
    color: var(--theme-accent);
    background: var(--accent-weak);
  }

  .secondary:disabled,
  .primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
  }
</style>
