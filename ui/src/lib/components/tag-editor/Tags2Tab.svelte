<script lang="ts">
  interface Props {
    disabled?: boolean;
  }

  let { disabled = false }: Props = $props();

  const leftFields = [
    'original artist',
    'original album',
    'original year',
    'quality',
    'tempo',
    'bpm',
    'mood',
    'occasion',
    'keywords',
    'language',
    'custom1',
    'custom2',
    'custom3',
    'custom4',
    'custom5',
  ] as const;

  const rightFields = [
    'custom6',
    'custom7',
    'custom8',
    'custom9',
    'custom10',
    'custom11',
    'custom12',
    'custom13',
    'custom14',
    'custom15',
    'custom16',
    'custom17',
    'custom18',
    'custom19',
    'custom20',
  ] as const;

  type FieldName = (typeof leftFields)[number] | (typeof rightFields)[number];

  const initialValues: Record<FieldName, string> = {
    'original artist': '',
    'original album': '',
    'original year': '',
    quality: '',
    tempo: '',
    bpm: '',
    mood: '',
    occasion: '',
    keywords: '',
    language: '',
    custom1: '',
    custom2: '',
    custom3: '',
    custom4: '',
    custom5: '',
    custom6: '',
    custom7: '',
    custom8: '',
    custom9: '',
    custom10: '',
    custom11: '',
    custom12: '',
    custom13: '',
    custom14: '',
    custom15: '',
    custom16: '',
    custom17: '',
    custom18: '',
    custom19: '',
    custom20: '',
  };

  let values = $state<Record<FieldName, string>>({ ...initialValues });
  let checked = $state<Record<FieldName, boolean>>(
    Object.fromEntries(Object.keys(initialValues).map((key) => [key, false])) as Record<
      FieldName,
      boolean
    >,
  );

  function setChecked(field: FieldName, value: boolean): void {
    checked = {
      ...checked,
      [field]: value,
    };
  }

  function setValue(field: FieldName, value: string): void {
    values = {
      ...values,
      [field]: value,
    };
    setChecked(field, true);
  }
</script>

<div class="tags2-tab">
  <div class="columns">
    <div class="column">
      {#each leftFields as field}
        <label class="row">
          <input
            type="checkbox"
            checked={checked[field]}
            onchange={(event) => setChecked(field, (event.target as HTMLInputElement).checked)}
            disabled={disabled}
          />
          <span>{field}:</span>
          <input
            type="text"
            value={values[field]}
            oninput={(event) => setValue(field, (event.target as HTMLInputElement).value)}
            disabled={disabled || !checked[field]}
          />
          <button type="button" class="ellipsis" disabled={disabled || !checked[field]}>...</button>
        </label>
      {/each}
    </div>

    <div class="column">
      {#each rightFields as field}
        <label class="row">
          <input
            type="checkbox"
            checked={checked[field]}
            onchange={(event) => setChecked(field, (event.target as HTMLInputElement).checked)}
            disabled={disabled}
          />
          <span>{field}:</span>
          <input
            type="text"
            value={values[field]}
            oninput={(event) => setValue(field, (event.target as HTMLInputElement).value)}
            disabled={disabled || !checked[field]}
          />
          <button type="button" class="ellipsis" disabled={disabled || !checked[field]}>...</button>
        </label>
      {/each}
    </div>
  </div>
</div>

<style>
  .tags2-tab {
    padding: 12px 0;
    height: 100%;
    overflow: auto;
  }

  .columns {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
  }

  .column {
    display: grid;
    gap: 6px;
  }

  .row {
    display: grid;
    grid-template-columns: 16px 110px minmax(0, 1fr) 28px;
    gap: 6px;
    align-items: center;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .row input[type='checkbox'] {
    width: 14px;
    height: 14px;
    accent-color: var(--theme-accent);
  }

  .row span {
    color: var(--text-secondary);
    text-transform: lowercase;
  }

  .row input[type='text'] {
    background: var(--surface-1);
    border: 1px solid var(--divider-color);
    color: var(--text-primary);
    font-size: 13px;
    padding: 5px 8px;
    border-radius: 0;
    min-width: 0;
  }

  .row input[type='text']:disabled {
    color: var(--text-disabled);
  }

  .ellipsis {
    height: 26px;
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-secondary);
    cursor: pointer;
    border-radius: 0;
    padding: 0;
  }

  .ellipsis:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
</style>
