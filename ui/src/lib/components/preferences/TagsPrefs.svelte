<script lang="ts">
  import { onMount } from 'svelte';
  import { tagsSettings, loadCategorySettings, saveCategorySetting, parseBool, type TagsSettings } from '../../state/preferences';

  const isMock = import.meta.env.SERMON_MOCK === '1';

  onMount(async () => {
    if (!isMock) {
      await loadCategorySettings('tags');
    }
  });

  function handleCustomTagChange(index: number, value: string) {
    if (!isMock) {
      saveCategorySetting('tags', `tags.custom_tag_${index}`, value);
    }
  }
</script>

<div class="category-content">
  <div class="setting-group">
    <h3>Custom Tags</h3>
    <div class="custom-tags-grid">
      {#if $tagsSettings}
        {#each Array(16) as _, i}
          {@const index = i + 1}
          <div class="setting">
            <label for="custom-tag-{index}">Custom Tag {index}</label>
            <input 
              type="text" 
              id="custom-tag-{index}"
              value={$tagsSettings[`tags.custom_tag_${index}` as keyof TagsSettings] || ''}
              onchange={(e) => handleCustomTagChange(index, e.currentTarget.value)}
              disabled={isMock}
              data-testid="prefs-tags-custom-{index}"
            />
          </div>
        {/each}
      {/if}
    </div>
  </div>

  <div class="setting-group">
    <h3>Tag Write Behavior</h3>
    
    <div class="setting">
      <label>Multi-Value Delimiter</label>
      <div class="delimiter-display">;</div>
      <span class="setting-hint">Read-only</span>
    </div>

    {#if $tagsSettings}
      <div class="setting">
        <label>
          <input 
            type="checkbox" 
            checked={parseBool($tagsSettings['tags.create_backup'])} 
            onchange={(e) => saveCategorySetting('tags', 'tags.create_backup', e.currentTarget.checked ? 'on' : 'off')} 
            disabled={isMock}
            data-testid="prefs-tags-backup"
          />
          Create backup before writing tags
        </label>
      </div>

      <div class="setting">
        <label>
          <input 
            type="checkbox" 
            checked={parseBool($tagsSettings['tags.sync_db_after_write'])} 
            onchange={(e) => saveCategorySetting('tags', 'tags.sync_db_after_write', e.currentTarget.checked ? 'on' : 'off')} 
            disabled={isMock}
            data-testid="prefs-tags-sync"
          />
          Update DB after tag write
        </label>
      </div>
    {/if}
  </div>
</div>

<style>
  .category-content { display: flex; flex-direction: column; gap: 2rem; }
  .setting-group { display: flex; flex-direction: column; gap: 1rem; }
  h3 { font-size: 1rem; color: #fff; margin: 0 0 0.5rem 0; border-bottom: 1px solid rgba(255,255,255,0.1); padding-bottom: 0.5rem; }
  .setting { display: flex; flex-direction: column; gap: 0.25rem; }
  .setting label { display: flex; align-items: center; gap: 0.5rem; color: #ccc; }
  .setting-hint { font-size: 0.8rem; color: #888; margin-left: 0; }
  
  .custom-tags-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 1rem;
  }

  input[type="text"] {
    background: #222;
    color: #fff;
    border: 1px solid #333;
    padding: 0.5rem;
    border-radius: 4px;
    width: 100%;
  }
  
  input[type="text"]:disabled { opacity: 0.5; }
  input[type="checkbox"] { margin-right: 0.5rem; }

  .delimiter-display {
    background: #222;
    color: #fff;
    border: 1px solid #333;
    padding: 0.5rem;
    border-radius: 4px;
    width: 50px;
    text-align: center;
    opacity: 0.7;
  }
</style>
