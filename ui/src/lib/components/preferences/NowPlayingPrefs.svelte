<script lang="ts">
  const isMock = import.meta.env.SERMON_MOCK === '1';
</script>

<div class="category-content">
  <div class="setting">
    <label for="double-click">Double-click behavior</label>
    <select id="double-click" disabled>
      <option value="play-now">Play Now</option>
      <option value="add-queue">Add to Queue</option>
    </select>
    <span class="setting-hint">Takes effect in future update</span>
  </div>

  <div class="setting">
    <label for="queue-add">Queue Add Position</label>
    <select id="queue-add" disabled>
      <option value="end">End of Queue</option>
      <option value="next">Play Next</option>
    </select>
    <span class="setting-hint">Takes effect in future update</span>
  </div>

  <div class="setting">
    <label for="shuffle-mode">Shuffle Mode</label>
    <select id="shuffle-mode" disabled>
      <option value="tracks">Tracks</option>
      <option value="albums">Albums</option>
    </select>
    <span class="setting-hint">Takes effect in future update</span>
  </div>
</div>

<style>
  .category-content { display: flex; flex-direction: column; gap: 1rem; }
  .setting { display: flex; flex-direction: column; gap: 0.5rem; }
  .setting label { color: #ccc; }
  .setting-hint { font-size: 0.8rem; color: #888; }
  select { background: #222; color: #fff; border: 1px solid #333; padding: 0.5rem; border-radius: 4px; width: 250px; }
  select:disabled { opacity: 0.5; }
</style>
