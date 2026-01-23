<script lang="ts">
  import { railMode, isRailOpen, setRailMode, toggleRail } from '../state/rightRail';
  import { queue, currentIndex, playNow } from '../state/playback';
  import { currentArtworkUrl } from '../state/artwork';
  import { currentLyrics, lyricsContext } from '../state/lyrics';
  import { navigate } from '../state/route';
  import { ListMusic, Disc3, Radio, MicVocal, Maximize2, Infinity } from '@lucide/svelte';
  import { derived } from 'svelte/store';
  import { fade, slide } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { fadeIn, pressScale } from '../utils/animations';
  import { VList } from 'virtua/svelte';
  
  function goFullscreenLyrics() {
    navigate({ name: 'lyrics-fullscreen' });
  }
  
  // Derive Now Playing and Up Next from queue
  const nowPlayingTrack = derived([queue, currentIndex], ([$queue, $currentIndex]) => {
    const index = $currentIndex ?? 0;
    return $queue[index] ?? null;
  });
  
  const upNextTracks = derived([queue, currentIndex], ([$queue, $currentIndex]) => {
    const index = $currentIndex ?? 0;
    return $queue.slice(index + 1);
  });
  
  // Item count for the header pill
  const itemCount = derived([queue], ([$queue]) => $queue.length);

  function handleKeydown(e: KeyboardEvent, trackId: number) {
    if (e.key === 'Enter') {
      playNow(trackId);
    }
  }
  
  // Mock autoplay suggestions
  const autoplaySuggestions = [
    { title: 'Similar Track 1', artist: 'Artist Name' },
    { title: 'Similar Track 2', artist: 'Another Artist' },
    { title: 'Similar Track 3', artist: 'Third Artist' },
  ];
</script>

{#if $isRailOpen}
  <!-- Scrim for overlay mode (<1280px) -->
  <div 
    class="rail-scrim" 
    onclick={toggleRail} 
    role="button" 
    tabindex="0" 
    onkeydown={(e) => e.key === 'Enter' && toggleRail()}
    transition:fade={{ duration: 200 }}
  ></div>
  
  <aside 
    class="right-rail-panel"
    transition:slide={{ duration: 250, easing: cubicOut, axis: 'x' }}
  >
    <!-- Cider-style top pill header -->
    <div class="rail-header">
      <div class="header-pills">
        <!-- Mode icons pill -->
        <div class="mode-pill">
          <button 
            class="pill-icon"
            class:active={$railMode === 'now-playing'}
            onclick={() => setRailMode('now-playing')}
            title="Now Playing"
            use:pressScale={{ scale: 0.95 }}
          >
            <Disc3 size={16} />
          </button>
          <button 
            class="pill-icon"
            class:active={$railMode === 'up-next'}
            onclick={() => setRailMode('up-next')}
            title="Up Next"
            use:pressScale={{ scale: 0.95 }}
          >
            <ListMusic size={16} />
          </button>
          <button 
            class="pill-icon"
            class:active={$railMode === 'autoplay'}
            onclick={() => setRailMode('autoplay')}
            title="Autoplay"
            use:pressScale={{ scale: 0.95 }}
          >
            <Radio size={16} />
          </button>
          <button 
            class="pill-icon"
            class:active={$railMode === 'lyrics'}
            onclick={() => setRailMode('lyrics')}
            title="Lyrics"
            use:pressScale={{ scale: 0.95 }}
          >
            <MicVocal size={16} />
          </button>
        </div>
        
        <!-- Item count pill -->
        <div class="count-pill">
          {$itemCount} {$itemCount === 1 ? 'item' : 'items'}
        </div>
      </div>
    </div>
    
    <!-- Content based on mode -->
    <div class="rail-content" use:fadeIn={{ duration: 200, delay: 100 }}>
      {#if $railMode === 'now-playing'}
        <div class="section">
          <h3 class="section-header">Now Playing</h3>
          {#if $nowPlayingTrack}
            <div class="now-playing-card">
              {#if $currentArtworkUrl}
                <img src={$currentArtworkUrl} alt="" class="np-artwork" />
              {:else}
                <div class="np-artwork-placeholder"></div>
              {/if}
              <div class="np-info">
                <div class="np-title">{$nowPlayingTrack.title || '—'}</div>
                <div class="np-artist">{$nowPlayingTrack.artist || '—'}</div>
              </div>
            </div>
          {:else}
            <div class="empty-state" use:fadeIn={{ duration: 300 }}>
              <Disc3 size={24} strokeWidth={1.5} />
              <span>Nothing playing</span>
            </div>
          {/if}
        </div>
        
        <div class="section">
          <div class="section-header-row">
            <h3 class="section-header">Playing Next</h3>
            {#if $nowPlayingTrack?.album}
              <span class="section-subtitle">From {$nowPlayingTrack.album}</span>
            {/if}
          </div>
          {#if $upNextTracks.length > 0}
            <div class="queue-list">
              <VList data={$upNextTracks} getKey={(item: { track_id: number }) => item.track_id} itemSize={52}>
                {#snippet children(item: { track_id: number; title?: string; artist?: string })}
                  <div 
                    class="queue-item"
                    role="button"
                    tabindex="0"
                    ondblclick={() => playNow(item.track_id)}
                    onkeydown={(e) => handleKeydown(e, item.track_id)}
                  >
                    <div class="q-thumb-placeholder"></div>
                    <div class="q-info">
                      <span class="q-title">{item.title || '—'}</span>
                      <span class="q-artist">{item.artist || '—'}</span>
                    </div>
                  </div>
                {/snippet}
              </VList>
            </div>
          {:else}
            <div class="empty-state" use:fadeIn={{ duration: 300 }}>
              <ListMusic size={24} strokeWidth={1.5} />
              <span>Queue is empty</span>
            </div>
          {/if}
        </div>
        
        <!-- Autoplay section in now-playing mode -->
        <div class="section">
          <div class="section-header-row">
            <div class="header-with-icon">
              <Infinity size={14} />
              <h3 class="section-header">Autoplay</h3>
            </div>
            <span class="section-subtitle">Similar music will keep playing</span>
          </div>
          <div class="queue-list">
            {#each autoplaySuggestions as suggestion}
              <div class="queue-item autoplay-item">
                <div class="q-thumb-placeholder"></div>
                <div class="q-info">
                  <span class="q-title">{suggestion.title}</span>
                  <span class="q-artist">{suggestion.artist}</span>
                </div>
              </div>
            {/each}
          </div>
        </div>
        
      {:else if $railMode === 'up-next'}
        <div class="section">
          <div class="section-header-row">
            <h3 class="section-header">Up Next</h3>
            <span class="item-count-label">{$upNextTracks.length} tracks</span>
          </div>
          {#if $upNextTracks.length > 0}
            <div class="queue-list">
              <VList data={$upNextTracks} getKey={(item: { track_id: number }) => item.track_id} itemSize={52}>
                {#snippet children(item: { track_id: number; title?: string; artist?: string })}
                  <div 
                    class="queue-item"
                    role="button"
                    tabindex="0"
                    ondblclick={() => playNow(item.track_id)}
                    onkeydown={(e) => handleKeydown(e, item.track_id)}
                  >
                    <div class="q-thumb-placeholder"></div>
                    <div class="q-info">
                      <span class="q-title">{item.title || '—'}</span>
                      <span class="q-artist">{item.artist || '—'}</span>
                    </div>
                  </div>
                {/snippet}
              </VList>
            </div>
          {:else}
            <div class="empty-state" use:fadeIn={{ duration: 300 }}>
              <ListMusic size={24} strokeWidth={1.5} />
              <span>Queue is empty</span>
            </div>
          {/if}
        </div>
        
      {:else if $railMode === 'autoplay'}
        <div class="section">
          <div class="section-header-row">
            <div class="header-with-icon">
              <Infinity size={14} />
              <h3 class="section-header">Autoplay</h3>
            </div>
          </div>
          <p class="autoplay-description">Similar music will play when your queue ends.</p>
          <div class="queue-list">
            {#each autoplaySuggestions as suggestion}
              <div class="queue-item autoplay-item">
                <div class="q-thumb-placeholder"></div>
                <div class="q-info">
                  <span class="q-title">{suggestion.title}</span>
                  <span class="q-artist">{suggestion.artist}</span>
                </div>
              </div>
            {/each}
          </div>
        </div>
        
      {:else if $railMode === 'lyrics'}
        <div class="section lyrics-section">
          <div class="lyrics-header-row">
            <h3 class="section-header">Lyrics</h3>
            <button class="fullscreen-btn" onclick={goFullscreenLyrics} title="Fullscreen" use:pressScale={{ scale: 0.95 }}>
              <Maximize2 size={16} />
            </button>
          </div>
          {#if $currentLyrics && $currentLyrics.length > 0}
            <div class="lyrics-rail-content">
              <VList data={$lyricsContext.lines}>
                {#snippet children(line: string, i: number)}
                  <p 
                    class="lyric-line"
                    class:active={i === $lyricsContext.activeIndex}
                    class:before={i < $lyricsContext.activeIndex}
                    class:after={i > $lyricsContext.activeIndex}
                  >
                    {line || '\u00A0'}
                  </p>
                {/snippet}
              </VList>
            </div>
          {:else}
            <div class="empty-state" use:fadeIn={{ duration: 300 }}>
              <MicVocal size={24} strokeWidth={1.5} />
              <span>Lyrics not available</span>
            </div>
          {/if}
        </div>
      {/if}
    </div>
    
  </aside>
{/if}

<style>
  /* Base styles */
  .right-rail-panel {
    width: var(--layout-rail-width, 320px);
    height: 100%;
    background: transparent;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    box-sizing: border-box;
    z-index: 90;
    position: relative;
  }

  /* Persistent mode (>=1280px) */
  @media (min-width: 1280px) {
    .right-rail-panel {
      position: relative;
    }
    .rail-scrim {
      display: none;
    }
  }

  /* Overlay mode (<1280px) */
  @media (max-width: 1279px) {
    .right-rail-panel {
      position: absolute;
      top: 0;
      right: 0;
      bottom: 0;
      background: rgba(10, 10, 10, 0.95);
      box-shadow: -2px 0 12px rgba(0,0,0,0.25);
    }
    
    .rail-scrim {
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background: rgba(0,0,0,0.5);
      z-index: 89;
      backdrop-filter: blur(2px);
    }
  }

  /* ============================================
     CIDER-STYLE TOP PILL HEADER
     ============================================ */
  .rail-header {
    padding: 12px 16px;
    -webkit-app-region: no-drag;
  }

  .header-pills {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .mode-pill {
    display: flex;
    align-items: center;
    gap: 2px;
    background: var(--surface-1);
    padding: 4px;
    border-radius: 999px;
    border: 1px solid var(--glass-border);
  }

  .pill-icon {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    padding: 6px 8px;
    border-radius: 999px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--motion-fast) var(--ease-out);
  }

  .pill-icon:hover {
    color: var(--text-secondary);
    background: var(--surface-hover);
  }

  .pill-icon.active {
    color: var(--theme-accent);
    background: var(--accent-weak);
  }

  .count-pill {
    background: var(--surface-1);
    padding: 6px 12px;
    border-radius: 999px;
    font-size: 12px;
    color: var(--text-secondary);
    white-space: nowrap;
    border: 1px solid var(--glass-border);
  }

  /* ============================================
     CONTENT AREA
     ============================================ */
  .rail-content {
    flex: 1;
    overflow-y: auto;
    padding: 0 16px 80px 16px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  /* ============================================
     SECTION HEADERS - CIDER NEUTRAL STYLE
     ============================================ */
  .section-header {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .section-header-row {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .section-subtitle {
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .header-with-icon {
    display: flex;
    align-items: center;
    gap: 6px;
    color: rgba(255, 255, 255, 0.9);
  }

  .header-with-icon .section-header {
    margin: 0;
  }

  .item-count-label {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.4);
  }

  /* ============================================
     EMPTY STATE - SUBTLE, NO DASHED BORDER
     ============================================ */
  .empty-state {
    color: var(--text-tertiary);
    font-size: 13px;
    padding: 32px 16px;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    background: var(--surface-1);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
  }

  .empty-state :global(svg) {
    opacity: 0.5;
    color: var(--text-disabled);
  }

  .autoplay-description {
    color: rgba(255, 255, 255, 0.45);
    font-size: 13px;
    margin: 0 0 8px 0;
  }

  /* ============================================
     NOW PLAYING CARD
     ============================================ */
  .now-playing-card {
    display: flex;
    gap: 12px;
    align-items: center;
    padding: 8px;
    border-radius: 8px;
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.05), rgba(255, 255, 255, 0.02));
    border: 1px solid var(--glass-border);
    box-shadow: var(--shadow-1);
  }

  .np-artwork {
    width: 48px;
    height: 48px;
    border-radius: 6px;
    object-fit: cover;
    background: #222;
  }

  .np-artwork-placeholder {
    width: 48px;
    height: 48px;
    border-radius: 6px;
    background: linear-gradient(135deg, #2a2a2a, #1a1a1a);
  }

  .np-info {
    flex: 1;
    min-width: 0;
  }

  .np-title {
    font-weight: 600;
    font-size: 14px;
    color: rgba(255, 255, 255, 0.95);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 2px;
  }

  .np-artist {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.5);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* ============================================
     QUEUE LIST - WITH THUMBNAILS
     ============================================ */
  .queue-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .queue-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 8px;
    border-radius: 6px;
    cursor: default;
    transition: background var(--motion-fast) var(--ease-out);
    height: 52px;
    box-sizing: border-box;
  }

  .queue-item:hover {
    background: var(--surface-hover);
  }

  .queue-item:focus {
    background: var(--surface-2);
    outline: none;
    box-shadow: inset 0 0 0 1px var(--accent-medium);
  }

  .q-thumb-placeholder {
    width: 40px;
    height: 40px;
    border-radius: 4px;
    background: linear-gradient(135deg, #2a2a2a, #1a1a1a);
    flex-shrink: 0;
  }

  .q-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    justify-content: center;
    gap: 2px;
  }

  .q-title {
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: rgba(255, 255, 255, 0.9);
  }

  .q-artist {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.45);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .autoplay-item {
    opacity: 0.7;
  }

  /* ============================================
     LYRICS SECTION
     ============================================ */
  .lyrics-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .lyrics-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  .lyrics-header-row .section-header {
    margin: 0;
  }

  .fullscreen-btn {
    background: rgba(255, 255, 255, 0.08);
    border: none;
    color: rgba(255, 255, 255, 0.6);
    padding: 6px;
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .fullscreen-btn:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.12);
  }

  .lyrics-rail-content {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
    scrollbar-width: thin;
  }

  .lyric-line {
    margin: 0;
    padding: 6px 0;
    font-size: 20px;
    font-weight: 500;
    line-height: 1.4;
    transition: all 0.3s ease;
    color: rgba(255, 255, 255, 0.3);
  }

  .lyric-line.active {
    color: #fff;
    font-weight: 700;
  }

  .lyric-line.before {
    color: rgba(255, 255, 255, 0.25);
  }

  .lyric-line.after {
    color: rgba(255, 255, 255, 0.35);
  }

  .mode-pill button:nth-child(1) { animation-delay: 0ms; }
  .mode-pill button:nth-child(2) { animation-delay: 30ms; }
  .mode-pill button:nth-child(3) { animation-delay: 60ms; }
  .mode-pill button:nth-child(4) { animation-delay: 90ms; }

  @keyframes fadeSlideIn {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .right-rail-panel .pill-icon {
    animation: fadeSlideIn 0.2s ease-out backwards;
  }

  @media (prefers-reduced-motion: reduce) {
    .right-rail-panel {
      transition: none;
    }
    .pill-icon {
      animation: none;
    }
  }
</style>
