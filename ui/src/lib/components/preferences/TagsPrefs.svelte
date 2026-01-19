<script lang="ts">
  const isMock = import.meta.env.SERMON_MOCK === '1';
</script>

<div class="category-content">
  <div class="setting">
    <label>
      <input type="checkbox" disabled />
      Backup file before writing tags
      <span class="setting-hint">Takes effect in future update</span>
    </label>
  </div>

  <div class="setting">
    <label for="write-behavior">Write Behavior</label>
    <select id="write-behavior" disabled>
      <option value="id3v2.3">ID3v2.3 (Compatible)</option>
      <option value="id3v2.4">ID3v2.4 (Standard)</option>
    </select>
    <span class="setting-hint">Takes effect in future update</span>
  </div>
</div>

<style>
  .category-content { display: flex; flex-direction: column; gap: 1rem; }
  .setting { display: flex; flex-direction: column; gap: 0.5rem; }
  .setting label { display: flex; align-items: center; gap: 0.5rem; color: #888; }
  .setting-hint { font-size: 0.8rem; color: #888; margin-left: 1.5rem; }
  select { background: #222; color: #fff; border: 1px solid #333; padding: 0.5rem; border-radius: 4px; width: 250px; }
  select:disabled { opacity: 0.5; }
</style>
