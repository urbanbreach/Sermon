<script lang="ts">
  import { currentRoute } from '../state/route';

  function openNowPlaying() {
    currentRoute.set('now-playing');
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div class="bottom-bar" on:click={openNowPlaying} role="button" tabindex="0">
  <div class="now-playing-info">
    <div class="placeholder-art"></div>
    <div class="track-details">
      <div class="title">Nothing Playing</div>
      <div class="artist">Select a track</div>
    </div>
  </div>

  <div class="controls">
    <span>⏮</span>
    <span class="play">▶</span>
    <span>⏭</span>
  </div>

  <div class="volume">
    Vol
  </div>
</div>

<style>
  .bottom-bar {
    height: 80px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    border-top: 1px solid var(--glass-border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1rem;
    cursor: pointer;
    transition: background 0.2s;
  }

  .bottom-bar:hover {
    background: var(--glass-highlight);
  }

  .now-playing-info {
    display: flex;
    align-items: center;
    gap: 1rem;
    width: 250px;
  }

  .placeholder-art {
    width: 50px;
    height: 50px;
    background: #333;
    border-radius: 4px;
  }

  .track-details {
    display: flex;
    flex-direction: column;
  }

  .title { color: #fff; font-size: 0.9rem; }
  .artist { color: #888; font-size: 0.8rem; }

  .controls {
    display: flex;
    gap: 1.5rem;
    color: #fff;
    font-size: 1.2rem;
  }
  
  .play {
    font-size: 1.5rem;
  }

  .volume {
    width: 250px;
    text-align: right;
    color: #888;
  }
</style>
