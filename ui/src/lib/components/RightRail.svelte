<script lang="ts">
  import { railMode, isRailOpen, setRailMode, toggleRail } from '../state/rightRail';
  import { queue, currentIndex, playNow } from '../state/playback';
  import { currentArtworkUrl } from '../state/artwork';
  import { currentLyrics, currentLineIndex, lyricsContext } from '../state/lyrics';
  import { navigate } from '../state/route';
  import { ListMusic, Disc3, Radio, MicVocal, X, Maximize2 } from '@lucide/svelte';
  import { derived } from 'svelte/store';
  
  function goFullscreenLyrics() {
    navigate({ name: 'lyrics-fullscreen' });
  }
  
  // Derive Now Playing and Up Next from queue
  // If currentIndex is null, we assume the first track is "current" (or nothing is playing yet but queue exists)
  // Actually, usually currentIndex is null means nothing playing.
  // But the prompt says: "If currentIndex is null: treat queue[0] as Now Playing and queue.slice(1) as Up Next"
  
  const nowPlayingTrack = derived([queue, currentIndex], ([$queue, $currentIndex]) => {
    const index = $currentIndex ?? 0;
    return $queue[index] ?? null;
  });
  
  const upNextTracks = derived([queue, currentIndex], ([$queue, $currentIndex]) => {
    const index = $currentIndex ?? 0;
    // Up next is everything AFTER the current index
    return $queue.slice(index + 1);
  });
  
  function formatDuration(ms?: number): string {
    if (!ms && ms !== 0) return '—';
    const minutes = Math.floor(ms / 60000);
    const seconds = Math.floor((ms % 60000) / 1000);
    return minutes + ":" + (seconds < 10 ? '0' : '') + seconds;
  }

  function handleKeydown(e: KeyboardEvent, trackId: number) {
    if (e.key === 'Enter') {
      playNow(trackId);
    }
  }
</script>

{#if $isRailOpen}
  <!-- Scrim for overlay mode (1100-1279px) -->
  <!-- We can use a media query in CSS to only show this scrim when in overlay mode -->
  <!-- or rely on the fact that >=1280 the rail pushes content so scrim might not be needed? -->
  <!-- Actually, the prompt says "overlay with scrim 1100-1279px". -->
  <!-- At >=1280px, layout is persistent (side-by-side). -->
  <!-- Since this component is mounted as a sibling, we need CSS to handle the positioning. -->
  <div class="rail-scrim" onclick={toggleRail} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && toggleRail()}></div>
  
  <aside class="right-rail-panel">
    <!-- Control strip -->
    <div class="rail-controls">
      <div class="mode-tabs">
        <button 
          class:active={$railMode === 'now-playing'}
          onclick={() => setRailMode('now-playing')}
          title="Now Playing"
          class="tab-btn"
        >
          <Disc3 size={18} />
        </button>
        <button 
          class:active={$railMode === 'up-next'}
          onclick={() => setRailMode('up-next')}
          title="Up Next"
          class="tab-btn"
        >
          <ListMusic size={18} />
        </button>
        <button 
          class:active={$railMode === 'autoplay'}
          onclick={() => setRailMode('autoplay')}
          title="Autoplay"
          class="tab-btn"
        >
          <Radio size={18} />
        </button>
        <button 
          class:active={$railMode === 'lyrics'}
          onclick={() => setRailMode('lyrics')}
          title="Lyrics"
          class="tab-btn"
        >
          <MicVocal size={18} />
        </button>
      </div>
      <button class="close-btn" onclick={toggleRail}>
        <X size={18} />
      </button>
    </div>
    
    <!-- Content based on mode -->
    <div class="rail-content">
      {#if $railMode === 'now-playing'}
        <div class="section">
          <h3>Now Playing</h3>
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
            <div class="empty-state">Nothing Playing</div>
          {/if}
        </div>
        
        <div class="section">
          <h3>Playing Next</h3>
          {#if $upNextTracks.length > 0}
            <div class="queue-list">
              {#each $upNextTracks as item, i}
                <div 
                  class="queue-item"
                  role="button"
                  tabindex="0"
                  ondblclick={() => playNow(item.track_id)}
                  onkeydown={(e) => handleKeydown(e, item.track_id)}
                >
                  <span class="q-index">{i + 1}</span>
                  <div class="q-info">
                    <span class="q-title">{item.title || '—'}</span>
                    <span class="q-artist">{item.artist || '—'}</span>
                  </div>
                  <span class="q-time">{formatDuration(item.duration_ms)}</span>
                </div>
              {/each}
            </div>
          {:else}
            <div class="empty-state">Up Next is empty</div>
          {/if}
        </div>
      {:else if $railMode === 'up-next'}
        <div class="section">
          <h3>Up Next</h3>
          {#if $upNextTracks.length > 0}
            <div class="queue-list">
              {#each $upNextTracks as item, i}
                <div 
                  class="queue-item"
                  role="button"
                  tabindex="0"
                  ondblclick={() => playNow(item.track_id)}
                  onkeydown={(e) => handleKeydown(e, item.track_id)}
                >
                  <span class="q-index">{i + 1}</span>
                  <div class="q-info">
                    <span class="q-title">{item.title || '—'}</span>
                    <span class="q-artist">{item.artist || '—'}</span>
                  </div>
                  <span class="q-time">{formatDuration(item.duration_ms)}</span>
                </div>
              {/each}
            </div>
          {:else}
            <div class="empty-state">Up Next is empty</div>
          {/if}
        </div>
      {:else if $railMode === 'autoplay'}
        <div class="section">
          <h3>Autoplay</h3>
          <div class="empty-state">Autoplay is Off</div>
          <p class="hint">Similar music will play when your queue ends.</p>
        </div>
      {:else if $railMode === 'lyrics'}
        <div class="section lyrics-section">
          <div class="lyrics-header-row">
            <h3>Lyrics</h3>
            <button class="fullscreen-btn" onclick={goFullscreenLyrics} title="Fullscreen">
              <Maximize2 size={16} />
            </button>
          </div>
          {#if $currentLyrics && $currentLyrics.length > 0}
            <div class="lyrics-rail-content">
              {#each $lyricsContext.lines as line, i}
                <p 
                  class="lyric-line"
                  class:active={i === $lyricsContext.activeIndex}
                  class:before={i < $lyricsContext.activeIndex}
                  class:after={i > $lyricsContext.activeIndex}
                >
                  {line || '\u00A0'}
                </p>
              {/each}
            </div>
          {:else}
            <div class="empty-state">Lyrics not available for this track.</div>
          {/if}
        </div>
      {/if}
    </div>
  </aside>
{/if}

<style>
  /* Base styles */
  .right-rail-panel {
    width: var(--layout-rail-width, 300px);
    height: 100%;
    background: var(--glass-bg, rgba(20, 20, 20, 0.95));
    backdrop-filter: blur(var(--glass-blur, 20px));
    -webkit-backdrop-filter: blur(var(--glass-blur, 20px));
    border-left: 1px solid var(--glass-border, rgba(255, 255, 255, 0.1));
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    box-sizing: border-box;
    z-index: 90; /* High enough to be above content, but below modals */
  }

  /* Persistent mode (>=1280px) */
  @media (min-width: 1280px) {
    .right-rail-panel {
      position: relative; /* Sibling in flex container */
    }
    .rail-scrim {
      display: none;
    }
    .close-btn {
      display: none; /* Usually persistent rails don't have a close button, or maybe it toggles visibility? */
      /* The prompt includes a close button in the controls HTML. Let's keep it visible if the user wants to close it manually. */
      /* Actually, prompt says: ">=1280px: isRailOpen=true (persistent, no overlay)" */
      /* If strictly persistent, maybe no close button? But toggleRail exists. */
      /* I'll leave the close button visible as it's good UX to be able to hide sidebars. */
    }
  }

  /* Overlay mode (<1280px) */
  @media (max-width: 1279px) {
    .right-rail-panel {
      position: absolute;
      top: 0;
      right: 0;
      bottom: 0;
      box-shadow: -5px 0 20px rgba(0,0,0,0.5);
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

  .rail-controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px;
    border-bottom: 1px solid var(--glass-border, rgba(255, 255, 255, 0.1));
    -webkit-app-region: no-drag;
  }

  .mode-tabs {
    display: flex;
    gap: 4px;
    background: rgba(255, 255, 255, 0.05);
    padding: 4px;
    border-radius: 6px;
  }

  .tab-btn {
    background: transparent;
    border: none;
    color: #888;
    padding: 6px;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .tab-btn:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
  }

  .tab-btn.active {
    background: var(--accent-color, #4af);
    color: #000; /* Contrast on accent */
  }

  .close-btn {
    background: transparent;
    border: none;
    color: #888;
    padding: 6px;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .close-btn:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
  }

  .rail-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .section h3 {
    margin: 0 0 12px 0;
    font-size: 14px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.8);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .empty-state {
    color: #666;
    font-size: 14px;
    padding: 20px 0;
    text-align: center;
    background: rgba(255, 255, 255, 0.02);
    border-radius: 8px;
    border: 1px dashed rgba(255, 255, 255, 0.1);
  }

  .hint {
    color: #555;
    font-size: 12px;
    margin-top: 8px;
    text-align: center;
  }

  /* Now Playing Card */
  .now-playing-card {
    background: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    padding: 12px;
    display: flex;
    gap: 12px;
    align-items: center;
    margin-bottom: 8px;
  }

  .np-artwork {
    width: 48px;
    height: 48px;
    border-radius: 4px;
    object-fit: cover;
    background: #222;
  }

  .np-artwork-placeholder {
    width: 48px;
    height: 48px;
    border-radius: 4px;
    background: linear-gradient(45deg, #222, #333);
  }

  .np-info {
    flex: 1;
    min-width: 0;
  }

  .np-title {
    font-weight: 600;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 2px;
  }

  .np-artist {
    font-size: 12px;
    color: #888;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Queue List */
  .queue-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .queue-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px; /* 36px total height approx: 16+8+8=32 + borders */
    border-radius: 6px;
    cursor: default;
    transition: background 0.1s;
    height: 36px; /* Explicit height per requirement */
    box-sizing: border-box;
  }

  .queue-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .queue-item:focus {
    background: rgba(255, 255, 255, 0.08);
    outline: none;
  }

  .q-index {
    color: #555;
    font-size: 12px;
    width: 20px;
    text-align: right;
  }

  .q-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    justify-content: center;
  }

  .q-title {
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: #eee;
  }

  .q-artist {
    font-size: 11px;
    color: #777;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .q-time {
    font-size: 12px;
    color: #666;
    margin-left: 8px;
  }

  /* Lyrics Rail Styles */
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

  .lyrics-header-row h3 {
    margin: 0;
  }

  .fullscreen-btn {
    background: rgba(255, 255, 255, 0.1);
    border: none;
    color: #888;
    padding: 6px;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .fullscreen-btn:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.15);
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
    font-size: 22px;
    font-weight: 500;
    line-height: 1.4;
    transition: all 0.3s ease;
    color: rgba(255, 255, 255, 0.3);
  }

  .lyric-line.active {
    color: #fff;
    font-weight: 700;
    font-size: 22px;
  }

  .lyric-line.before {
    color: rgba(255, 255, 255, 0.25);
  }

  .lyric-line.after {
    color: rgba(255, 255, 255, 0.35);
  }
</style>
