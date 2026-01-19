<script lang="ts">
  const isMock = import.meta.env.SERMON_MOCK === '1';
</script>

<div class="category-content">
  <div class="setting">
    <label>
      <input type="checkbox" disabled />
      Enable DSD over PCM (DoP)
      <span class="coming-soon">[Coming Soon]</span>
    </label>
    <span class="setting-hint">Requires a compatible DAC</span>
  </div>
</div>

<style>
  .category-content { display: flex; flex-direction: column; gap: 1rem; }
  .setting { display: flex; flex-direction: column; gap: 0.5rem; }
  .setting label { display: flex; align-items: center; gap: 0.5rem; color: #888; }
  .setting-hint { font-size: 0.8rem; color: #666; margin-left: 1.5rem; }
  .coming-soon { font-size: 0.75rem; color: #666; font-style: italic; }
  input:disabled { opacity: 0.5; }
</style>
