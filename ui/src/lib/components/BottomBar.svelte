<script lang="ts">
  import { navigate } from '../state/route';
  import { 
    currentTrack, playbackState, togglePlayPause, next, previous, 
    volume, setVolume, playbackError, switchToDefault, progress, audioDebug 
  } from '../state/playback';
  import { getArtworkBestForTrack, getArtworkBytes } from '../api/artwork';

  let artworkUrl: string | null = $state(null);
  let lastTrackId: number | null = $state(null);
  let isUnity = $derived($audioDebug?.policy === 'strict' && $audioDebug?.output_mode === 'exclusive');

  async function loadArtwork(track: typeof $currentTrack) {
    if (!track) {
      artworkUrl = null;
      lastTrackId = null;
      return;
    }

    // Skip if same track
    if (track.id === lastTrackId && artworkUrl) return;
    lastTrackId = track.id;

    try {
      const best = await getArtworkBestForTrack(track.id);
      if (best.source !== 'none' && best.cacheKey && best.mime) {
        const bytes = await getArtworkBytes(best.cacheKey, best.mime);
        artworkUrl = `data:${bytes.mime};base64,${bytes.bytesBase64}`;
      } else {
        artworkUrl = null;
      }
    } catch (e) {
      console.error('Failed to load track artwork:', e);
      artworkUrl = null;
    }
  }

  $effect(() => {
    loadArtwork($currentTrack);
  });

  function openNowPlaying() {
    navigate({ name: 'now-playing' });
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') {
      openNowPlaying();
    }
  }
</script>

{#if $playbackError}
  <div class="error-banner">
     <span>Error: {$playbackError.message}</span>
     {#if $playbackError.action === 'switch_to_default'}
        <button onclick={switchToDefault}>Switch to Default Device</button>
     {/if}
  </div>
{/if}

<div class="bottom-bar" onclick={openNowPlaying} role="button" tabindex="0" onkeypress={handleKey}>
  <div class="progress-bar-container">
    <div class="progress-bar-fill" style="width: {$progress * 100}%"></div>
  </div>

  <div class="now-playing-info">
    {#if artworkUrl}
      <img src={artworkUrl} alt="" class="artwork" />
    {:else}
      <div class="placeholder-art"></div>
    {/if}
    <div class="track-details">
      <div class="title">{$currentTrack?.title || 'Nothing Playing'}</div>
      <div class="artist">{$currentTrack?.artist || 'Select a track'}</div>
      {#if $audioDebug && $currentTrack}
        <div class="bit-perfect-status" class:is-perfect={$audioDebug.bit_perfect === 'yes'}>
          <div class="dot"></div>
          <span class="status-text" title={$audioDebug.bit_perfect === 'yes' ? 'Bit-Perfect' : $audioDebug.bit_perfect_reason}>
            {$audioDebug.bit_perfect === 'yes' ? 'Bit-Perfect' : ($audioDebug.bit_perfect_reason || 'Not Bit-Perfect')}
          </span>
        </div>
      {/if}
    </div>
  </div>

  <div class="controls" onclick={(e) => e.stopPropagation()}>
    <button class="control-btn" onclick={previous}>⏮</button>
    <button class="control-btn play" onclick={togglePlayPause}>
      {#if $playbackState === 'playing'} ⏸ {:else} ▶ {/if}
    </button>
    <button class="control-btn" onclick={next}>⏭</button>
  </div>

  <div class="volume" onclick={(e) => e.stopPropagation()}>
    {#if isUnity}
      <span class="vol-label-unity">Unity</span>
    {:else}
      <span class="vol-icon">🔊</span>
    {/if}
    <input 
      type="range" 
      min="0" 
      max="1" 
      step="0.01" 
      value={isUnity ? 1.0 : $volume} 
      disabled={isUnity}
      oninput={(e) => setVolume(e.currentTarget.valueAsNumber)} 
    />
  </div>
</div>

<style>
  .error-banner {
    background: #d32f2f;
    color: white;
    padding: 0.5rem;
    text-align: center;
    font-size: 0.9rem;
    display: flex;
    justify-content: center;
    gap: 1rem;
    align-items: center;
  }
  .error-banner button {
    background: white;
    color: #d32f2f;
    border: none;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
  }

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
    position: relative;
  }

  .progress-bar-container {
    position: absolute;
    top: -2px;
    left: 0;
    width: 100%;
    height: 2px;
    background: transparent;
  }
  .progress-bar-fill {
    height: 100%;
    background: #4af;
    transition: width 0.2s linear;
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

  .artwork {
    width: 50px;
    height: 50px;
    border-radius: 4px;
    flex-shrink: 0;
    object-fit: cover;
  }

  .placeholder-art {
    width: 50px;
    height: 50px;
    background: #333;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .track-details {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .title { 
    color: #fff; 
    font-size: 0.9rem; 
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .artist { 
    color: #888; 
    font-size: 0.8rem; 
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .controls {
    display: flex;
    gap: 1.5rem;
    align-items: center;
  }
  
  .control-btn {
    background: none;
    border: none;
    color: #fff;
    font-size: 1.2rem;
    cursor: pointer;
    opacity: 0.8;
    transition: opacity 0.2s, transform 0.1s;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .control-btn:hover {
    opacity: 1;
    transform: scale(1.1);
  }
  .control-btn:active {
    transform: scale(0.95);
  }

  .play {
    font-size: 1.8rem;
    width: 40px;
    height: 40px;
  }

  .volume {
    width: 200px;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: #888;
  }
  
  .vol-icon {
    font-size: 1.2rem;
  }

  input[type=range] {
    width: 100%;
    height: 4px;
    background: rgba(255,255,255,0.2);
    border-radius: 2px;
    appearance: none;
  }
  input[type=range]::-webkit-slider-thumb {
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #fff;
    cursor: pointer;
  }
  
  input[type=range]:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  input[type=range]:disabled::-webkit-slider-thumb {
    background: #888;
    cursor: not-allowed;
  }

  .vol-label-unity {
    font-size: 0.8rem;
    color: #4af;
    text-transform: uppercase;
    font-weight: bold;
    letter-spacing: 0.5px;
  }

  .bit-perfect-status {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin-top: 0.2rem;
    font-size: 0.75rem;
  }
  
  .bit-perfect-status .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #fa4;
    flex-shrink: 0;
  }
  
  .bit-perfect-status.is-perfect .dot {
    background: #4f4;
    box-shadow: 0 0 5px rgba(68, 255, 68, 0.4);
  }
  
  .status-text {
    color: #aaa;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 150px;
  }
</style>
