<script lang="ts">
  import { navigate } from '../state/route';
  import { searchSuggest } from '../api/library';
  import type { SearchHit } from '../types/library';

  let query = $state('');
  let results = $state<SearchHit[]>([]);
  let activeIndex = $state<number | null>(null);
  let showDropdown = $state(false);
  let inputElement: HTMLInputElement;
  
  let debounceTimer: ReturnType<typeof setTimeout>;

  function handleInput() {
    clearTimeout(debounceTimer);
    if (query.length < 1) {
      results = [];
      showDropdown = false;
      return;
    }

    debounceTimer = setTimeout(async () => {
      try {
        const res = await searchSuggest(query, 12);
        results = res.results;
        showDropdown = results.length > 0;
        activeIndex = null;
      } catch (e) {
        console.error('Search failed:', e);
      }
    }, 200);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (!showDropdown && results.length > 0) {
        showDropdown = true;
        activeIndex = 0;
        return;
      }
      if (activeIndex === null) {
        activeIndex = 0;
      } else {
        activeIndex = Math.min(activeIndex + 1, results.length - 1);
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (!showDropdown && results.length > 0) {
        showDropdown = true;
        activeIndex = results.length - 1;
        return;
      }
      if (activeIndex === null) {
        activeIndex = results.length - 1;
      } else {
        activeIndex = Math.max(activeIndex - 1, 0);
      }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (showDropdown && activeIndex !== null && results[activeIndex]) {
        selectResult(results[activeIndex]);
      } else {
        navigate({ name: 'search-results', query });
        showDropdown = false;
      }
    } else if (e.key === 'Escape') {
      if (showDropdown) {
        showDropdown = false;
        activeIndex = null;
      } else {
        query = '';
      }
    }
  }

  function selectResult(hit: SearchHit) {
    if (hit.type === 'track') {
      navigate({ name: 'now-playing' });
    } else if (hit.type === 'album') {
      navigate({ 
        name: 'album-detail', 
        albumArtistSort: hit.albumArtistSort, 
        albumTitleSort: hit.albumTitleSort 
      });
    } else if (hit.type === 'artist') {
      navigate({ 
        name: 'artist-detail', 
        artistSort: hit.artistSort 
      });
    }
    showDropdown = false;
    query = '';
  }

  function handleBlur() {
    // Delay to allow click to register
    setTimeout(() => {
      showDropdown = false;
    }, 200);
  }

  function handleFocus() {
    if (results.length > 0 && query.length >= 1) {
      showDropdown = true;
    }
  }
</script>

<div class="top-bar">
  <div class="search-container">
    <input 
      bind:this={inputElement}
      type="text" 
      class="search-input"
      placeholder="Search..." 
      bind:value={query} 
      oninput={handleInput}
      onkeydown={handleKeydown}
      onfocus={handleFocus}
      onblur={handleBlur}
    />
    
    {#if showDropdown}
      <div class="search-dropdown">
        {#each results as hit, index (index)}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div 
            class="search-item"
            class:active={index === activeIndex}
            onmouseenter={() => activeIndex = index}
            onclick={() => selectResult(hit)}
          >
            {#if hit.type === 'track'}
              <span class="icon">🎵</span>
              <span class="text">{hit.title} - {hit.artist}</span>
            {:else if hit.type === 'album'}
              <span class="icon">💿</span>
              <span class="text">{hit.albumTitleDisplay} - {hit.albumArtistDisplay}</span>
            {:else if hit.type === 'artist'}
              <span class="icon">👤</span>
              <span class="text">{hit.artistDisplay}</span>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <div class="window-controls">
    <!-- Window controls placeholder -->
    <span>_</span>
    <span>□</span>
    <span>×</span>
  </div>
</div>

<style>
  .top-bar {
    height: 40px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    border-bottom: 1px solid var(--glass-border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1rem;
    -webkit-app-region: drag;
    user-select: none;
    z-index: 100; /* Ensure dropdown is above content */
    position: relative;
  }

  .search-container {
    position: relative;
    -webkit-app-region: no-drag;
    width: 300px;
  }

  .search-input {
    width: 100%;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #fff;
    font-size: 0.9rem;
    padding: 6px 12px;
    border-radius: 6px;
    outline: none;
    transition: all 0.2s ease;
  }

  .search-input:focus {
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .search-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 4px;
    background: rgba(20, 20, 20, 0.95);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    max-height: 400px;
    overflow-y: auto;
  }

  .search-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    cursor: pointer;
    font-size: 0.9rem;
    color: #ccc;
    transition: background 0.1s;
  }

  .search-item.active,
  .search-item:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }

  .icon {
    opacity: 0.7;
  }

  .text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .window-controls {
    display: flex;
    gap: 1rem;
    color: #888;
    -webkit-app-region: no-drag;
  }
  
  span {
    cursor: pointer;
  }
  
  span:hover {
    color: #fff;
  }
</style>
