<script lang="ts">
  interface Props {
    lyrics: string;
    syncedLyrics: string;
    lyricist: string;
    markNoLyrics: boolean;
    disabled?: boolean;
    onChange: (value: string) => void;
    onChangeSynced: (value: string) => void;
    onChangeLyricist: (value: string) => void;
    onMarkNoLyricsChange: (value: boolean) => void;
    onSearchInternet?: () => void;
  }

  let {
    lyrics,
    syncedLyrics,
    lyricist,
    markNoLyrics,
    disabled = false,
    onChange,
    onChangeSynced,
    onChangeLyricist,
    onMarkNoLyricsChange,
    onSearchInternet,
  }: Props = $props();
</script>

<div class="lyrics-tab">
  <label class="row with-action">
    <span>lyricist:</span>
    <input
      type="text"
      value={lyricist}
      oninput={(event) => onChangeLyricist((event.target as HTMLInputElement).value)}
      disabled={disabled}
    />
    <button type="button" disabled={disabled}>...</button>
  </label>

  <label class="row">
    <span>lyrics:</span>
    <textarea
      value={lyrics}
      oninput={(event) => onChange((event.target as HTMLTextAreaElement).value)}
      disabled={disabled || markNoLyrics}
    ></textarea>
  </label>

  <label class="checkbox-row">
    <input
      type="checkbox"
      checked={markNoLyrics}
      onchange={(event) => onMarkNoLyricsChange((event.target as HTMLInputElement).checked)}
      disabled={disabled}
    />
    <span>mark as having no lyrics</span>
  </label>

  <label class="row synced-row">
    <span>synced lyrics:</span>
    <textarea
      value={syncedLyrics}
      oninput={(event) => onChangeSynced((event.target as HTMLTextAreaElement).value)}
      disabled={disabled || markNoLyrics}
      placeholder="[00:12.45] Line"
    ></textarea>
  </label>

  <div class="actions">
    <button type="button" onclick={() => onSearchInternet?.()} disabled={disabled}>Search Internet</button>
  </div>
</div>

<style>
  .lyrics-tab {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 0;
    overflow: auto;
  }

  .row {
    display: grid;
    grid-template-columns: 96px minmax(0, 1fr);
    gap: 8px;
    align-items: start;
  }

  .row span {
    color: var(--text-secondary);
    font-size: 13px;
    text-transform: lowercase;
    margin-top: 7px;
  }

  .row input,
  .row textarea {
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-primary);
    font-size: 13px;
    padding: 6px 8px;
    border-radius: 0;
    font-family: inherit;
  }

  .row input:disabled,
  .row textarea:disabled {
    color: var(--text-disabled);
  }

  .row textarea {
    min-height: 72px;
    resize: vertical;
  }

  .with-action {
    grid-template-columns: 96px minmax(0, 1fr) 28px;
    align-items: center;
  }

  .with-action span {
    margin-top: 0;
  }

  .with-action button {
    height: 30px;
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-secondary);
    border-radius: 0;
    cursor: pointer;
  }

  .with-action button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .checkbox-row input {
    width: 14px;
    height: 14px;
    accent-color: var(--theme-accent);
  }

  .synced-row {
    margin-top: 4px;
  }

  .actions {
    margin-top: auto;
    display: flex;
    justify-content: flex-end;
  }

  .actions button {
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-primary);
    font-size: 13px;
    padding: 6px 12px;
    border-radius: 0;
    cursor: pointer;
  }

  .actions button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
