<script lang="ts">
  import ArtworkImage from '../ArtworkImage.svelte';
  import type { TrackTagSnapshot } from '../../types/library';

  interface Props {
    tracks: TrackTagSnapshot[];
    values: Record<string, string>;
    checked: Record<string, boolean>;
    disabled?: boolean;
    onChange: (field: string, value: string) => void;
    onCheck: (field: string, checked: boolean) => void;
  }

  let { tracks, values, checked, disabled = false, onChange, onCheck }: Props = $props();

  let isMulti = $derived(tracks.length > 1);
  let firstTrack = $derived(tracks[0]);

  function handleInput(field: string, event: Event): void {
    onChange(field, (event.target as HTMLInputElement).value);
    if (isMulti) {
      onCheck(field, true);
    }
  }

  function handleTextArea(field: string, event: Event): void {
    onChange(field, (event.target as HTMLTextAreaElement).value);
    if (isMulti) {
      onCheck(field, true);
    }
  }

  function handleCheckbox(field: string, event: Event): void {
    onCheck(field, (event.target as HTMLInputElement).checked);
  }

  function isFieldDisabled(field: string): boolean {
    return disabled || (isMulti && !checked[field]);
  }
</script>

<div class="metadata-tab">
  <div class="left-column">
    <div class="artwork-head">artwork:</div>
    <div class="artwork-frame">
      <ArtworkImage
        cacheKey={null}
        artistSort={firstTrack?.albumArtist?.toLowerCase() ?? firstTrack?.artist?.toLowerCase() ?? ''}
        titleSort={firstTrack?.album?.toLowerCase() ?? ''}
        size={180}
        alt="Track artwork"
      />
    </div>

    <div class="rating-row">
      <input type="checkbox" disabled />
      <span>track rating:</span>
    </div>
    <div class="stars" aria-hidden="true">★★★★★</div>

    <div class="rating-row">
      <input type="checkbox" disabled />
      <span>album rating:</span>
    </div>
    <div class="stars" aria-hidden="true">★★★★★</div>
  </div>

  <div class="right-column">
    <label class="row">
      {#if isMulti}
        <input type="checkbox" checked={checked.title} onchange={(event) => handleCheckbox('title', event)} disabled={disabled} />
      {/if}
      <span>track title:</span>
      <input type="text" value={values.title} oninput={(event) => handleInput('title', event)} disabled={isFieldDisabled('title')} />
    </label>

    <label class="row with-action">
      {#if isMulti}
        <input type="checkbox" checked={checked.artist} onchange={(event) => handleCheckbox('artist', event)} disabled={disabled} />
      {/if}
      <span>artist:</span>
      <input type="text" value={values.artist} oninput={(event) => handleInput('artist', event)} disabled={isFieldDisabled('artist')} />
      <button type="button" disabled={isFieldDisabled('artist')}>...</button>
    </label>

    <label class="row with-action">
      {#if isMulti}
        <input
          type="checkbox"
          checked={checked.albumArtist}
          onchange={(event) => handleCheckbox('albumArtist', event)}
          disabled={disabled}
        />
      {/if}
      <span>album artist:</span>
      <input
        type="text"
        value={values.albumArtist}
        oninput={(event) => handleInput('albumArtist', event)}
        disabled={isFieldDisabled('albumArtist')}
      />
      <button type="button" disabled={isFieldDisabled('albumArtist')}>...</button>
    </label>

    <label class="row with-action">
      {#if isMulti}
        <input type="checkbox" checked={checked.album} onchange={(event) => handleCheckbox('album', event)} disabled={disabled} />
      {/if}
      <span>album:</span>
      <input type="text" value={values.album} oninput={(event) => handleInput('album', event)} disabled={isFieldDisabled('album')} />
      <button type="button" disabled={isFieldDisabled('album')}>...</button>
    </label>

    <div class="row split">
      <label class="inline-row">
        {#if isMulti}
          <input type="checkbox" checked={checked.year} onchange={(event) => handleCheckbox('year', event)} disabled={disabled} />
        {/if}
        <span>year:</span>
        <input type="text" value={values.year} oninput={(event) => handleInput('year', event)} disabled={isFieldDisabled('year')} />
      </label>

      <label class="inline-row compact">
        {#if isMulti}
          <input type="checkbox" checked={checked.trackNo} onchange={(event) => handleCheckbox('trackNo', event)} disabled={disabled} />
        {/if}
        <span>track:</span>
        <input
          type="text"
          value={values.trackNo}
          oninput={(event) => handleInput('trackNo', event)}
          disabled={isFieldDisabled('trackNo')}
        />
        <span class="of">of</span>
      </label>

      <label class="inline-row compact">
        {#if isMulti}
          <input type="checkbox" checked={checked.discNo} onchange={(event) => handleCheckbox('discNo', event)} disabled={disabled} />
        {/if}
        <span>disc:</span>
        <input
          type="text"
          value={values.discNo}
          oninput={(event) => handleInput('discNo', event)}
          disabled={isFieldDisabled('discNo')}
        />
        <span class="of">of</span>
      </label>
    </div>

    <label class="row with-action">
      {#if isMulti}
        <input
          type="checkbox"
          checked={checked.publisher}
          onchange={(event) => handleCheckbox('publisher', event)}
          disabled={disabled}
        />
      {/if}
      <span>publisher:</span>
      <input
        type="text"
        value={values.publisher}
        oninput={(event) => handleInput('publisher', event)}
        disabled={isFieldDisabled('publisher')}
      />
      <button type="button" disabled={isFieldDisabled('publisher')}>...</button>
    </label>

    <label class="row with-action">
      {#if isMulti}
        <input
          type="checkbox"
          checked={checked.composer}
          onchange={(event) => handleCheckbox('composer', event)}
          disabled={disabled}
        />
      {/if}
      <span>composer:</span>
      <input
        type="text"
        value={values.composer}
        oninput={(event) => handleInput('composer', event)}
        disabled={isFieldDisabled('composer')}
      />
      <button type="button" disabled={isFieldDisabled('composer')}>...</button>
    </label>

    <label class="row with-action">
      {#if isMulti}
        <input
          type="checkbox"
          checked={checked.conductor}
          onchange={(event) => handleCheckbox('conductor', event)}
          disabled={disabled}
        />
      {/if}
      <span>conductor:</span>
      <input
        type="text"
        value={values.conductor}
        oninput={(event) => handleInput('conductor', event)}
        disabled={isFieldDisabled('conductor')}
      />
      <button type="button" disabled={isFieldDisabled('conductor')}>...</button>
    </label>

    <label class="row with-action">
      {#if isMulti}
        <input type="checkbox" checked={checked.genre} onchange={(event) => handleCheckbox('genre', event)} disabled={disabled} />
      {/if}
      <span>genre:</span>
      <input type="text" value={values.genre} oninput={(event) => handleInput('genre', event)} disabled={isFieldDisabled('genre')} />
      <button type="button" disabled={isFieldDisabled('genre')}>...</button>
    </label>

    <label class="row textarea-row">
      {#if isMulti}
        <input
          type="checkbox"
          checked={checked.comments}
          onchange={(event) => handleCheckbox('comments', event)}
          disabled={disabled}
        />
      {/if}
      <span>comments:</span>
      <textarea
        value={values.comments}
        oninput={(event) => handleTextArea('comments', event)}
        disabled={isFieldDisabled('comments')}
      ></textarea>
    </label>

    <label class="row with-action">
      {#if isMulti}
        <input
          type="checkbox"
          checked={checked.grouping}
          onchange={(event) => handleCheckbox('grouping', event)}
          disabled={disabled}
        />
      {/if}
      <span>grouping:</span>
      <input
        type="text"
        value={values.grouping}
        oninput={(event) => handleInput('grouping', event)}
        disabled={isFieldDisabled('grouping')}
      />
      <button type="button" disabled={isFieldDisabled('grouping')}>...</button>
    </label>
  </div>
</div>

<style>
  .metadata-tab {
    height: 100%;
    display: grid;
    grid-template-columns: 190px 1fr;
    gap: 16px;
    padding: 10px 0;
    overflow: auto;
  }

  .left-column {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .artwork-head {
    color: var(--text-secondary);
    font-size: 13px;
    text-transform: lowercase;
  }

  .artwork-frame {
    width: 168px;
    height: 168px;
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    overflow: hidden;
  }

  .artwork-frame :global(.artwork-container) {
    width: 100%;
    height: 100%;
    border-radius: 0;
  }

  .rating-row {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-secondary);
    font-size: 13px;
    margin-top: 2px;
  }

  .rating-row input {
    width: 14px;
    height: 14px;
    accent-color: var(--theme-accent);
  }

  .stars {
    color: var(--text-disabled);
    letter-spacing: 1px;
    margin-left: 20px;
    font-size: 17px;
    line-height: 1;
  }

  .right-column {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .row {
    display: grid;
    grid-template-columns: 16px 108px minmax(0, 1fr);
    gap: 6px;
    align-items: center;
  }

  .row span {
    color: var(--text-secondary);
    font-size: 13px;
    text-transform: lowercase;
  }

  .row input[type='checkbox'] {
    width: 14px;
    height: 14px;
    accent-color: var(--theme-accent);
  }

  .row input[type='text'],
  .row textarea {
    min-width: 0;
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-primary);
    font-size: 13px;
    padding: 5px 8px;
    border-radius: 0;
    font-family: inherit;
  }

  .row input[type='text']:disabled,
  .row textarea:disabled {
    color: var(--text-disabled);
  }

  .row textarea {
    min-height: 66px;
    resize: vertical;
  }

  .row.with-action {
    grid-template-columns: 16px 108px minmax(0, 1fr) 28px;
  }

  .row.with-action button {
    height: 28px;
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-secondary);
    cursor: pointer;
    border-radius: 0;
    padding: 0;
  }

  .row.with-action button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .row.split {
    grid-template-columns: minmax(0, 1fr) auto auto;
    align-items: stretch;
    gap: 8px;
  }

  .inline-row {
    display: grid;
    grid-template-columns: 16px 50px minmax(0, 1fr);
    gap: 6px;
    align-items: center;
  }

  .inline-row.compact {
    grid-template-columns: 16px 42px 70px auto;
  }

  .inline-row .of {
    color: var(--text-tertiary);
    font-size: 12px;
    text-transform: lowercase;
  }

  .textarea-row {
    align-items: start;
  }

  .textarea-row span {
    margin-top: 6px;
  }

  @media (max-width: 880px) {
    .metadata-tab {
      grid-template-columns: 1fr;
    }

    .left-column {
      flex-direction: row;
      flex-wrap: wrap;
      align-items: center;
      gap: 10px 14px;
    }
  }
</style>
