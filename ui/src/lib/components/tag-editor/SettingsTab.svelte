<script lang="ts">
  interface Props {
    isMulti: boolean;
    trackCount: number;
    createBackup: boolean;
    closeAfterApply: boolean;
    disabled?: boolean;
    onCreateBackupChange: (value: boolean) => void;
    onCloseAfterApplyChange: (value: boolean) => void;
    onResetForm: () => void;
  }

  let {
    isMulti,
    trackCount,
    createBackup,
    closeAfterApply,
    disabled = false,
    onCreateBackupChange,
    onCloseAfterApplyChange,
    onResetForm,
  }: Props = $props();
</script>

<div class="settings-tab">
  <section class="panel">
    <h3>Write Settings</h3>
    <p>Control safety and post-save behavior for metadata writes.</p>

    <label class="toggle-row">
      <input
        type="checkbox"
        checked={createBackup}
        onchange={(e) => onCreateBackupChange((e.target as HTMLInputElement).checked)}
        disabled={disabled}
      />
      <div>
        <div class="label">Create backup before write</div>
        <div class="hint">Creates a backup copy before applying updates to audio files.</div>
      </div>
    </label>

    <label class="toggle-row">
      <input
        type="checkbox"
        checked={closeAfterApply}
        onchange={(e) => onCloseAfterApplyChange((e.target as HTMLInputElement).checked)}
        disabled={disabled}
      />
      <div>
        <div class="label">Close editor after apply</div>
        <div class="hint">If disabled, this window stays open and refreshes values after save.</div>
      </div>
    </label>
  </section>

  <section class="panel">
    <h3>Session</h3>
    <div class="summary-grid">
      <div class="key">Mode</div>
      <div class="value">{isMulti ? 'Multi-track batch edit' : 'Single-track edit'}</div>

      <div class="key">Tracks selected</div>
      <div class="value">{trackCount}</div>
    </div>

    <div class="actions">
      <button class="btn" onclick={onResetForm} disabled={disabled}>Reset unsaved field edits</button>
    </div>
  </section>
</div>

<style>
  .settings-tab {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 0.5rem 0;
  }

  .panel {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
    border-radius: 9px;
    padding: 0.9rem;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .panel h3 {
    margin: 0;
    color: #fff;
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .panel > p {
    margin: -0.35rem 0 0;
    color: #9a9a9a;
    font-size: 0.83rem;
  }

  .toggle-row {
    display: flex;
    align-items: flex-start;
    gap: 0.7rem;
  }

  .toggle-row input {
    margin-top: 2px;
  }

  .label {
    color: #dedede;
    font-size: 0.9rem;
  }

  .hint {
    margin-top: 0.15rem;
    color: #8a8a8a;
    font-size: 0.8rem;
  }

  .summary-grid {
    display: grid;
    grid-template-columns: 150px 1fr;
    gap: 0.5rem 0.8rem;
  }

  .key {
    color: #8f8f8f;
    font-size: 0.83rem;
  }

  .value {
    color: #ececec;
    font-size: 0.88rem;
  }

  .actions {
    display: flex;
    justify-content: flex-start;
  }

  .btn {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
    color: #efefef;
    border-radius: 7px;
    padding: 0.5rem 0.9rem;
    cursor: pointer;
  }

  .btn:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
</style>
