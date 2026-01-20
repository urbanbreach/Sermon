<script lang="ts">
  import { goBack } from '../state/route';
  import { 
    currentTrack, playbackState, queue, currentIndex, progress, 
    positionMs, durationMs, seek, audioDebug, isPlaying, playNow
  } from '../state/playback';
  import { Fixtures } from '../data/fixtures';
  import { currentArtworkUrl } from '../state/artwork';
  
  let isDebugging = false;

  function handleGoBack() {
    goBack();
  }

  function formatDuration(ms: number): string {
    if (!ms && ms !== 0) return '--:--';
    const minutes = Math.floor(ms / 60000);
    const seconds = Math.floor((ms % 60000) / 1000);
    return minutes + ":" + (seconds < 10 ? '0' : '') + seconds;
  }

  function handleSeek(e: Event) {
    const target = e.target as HTMLInputElement;
    const percent = parseFloat(target.value);
    const ms = percent * ($durationMs || 0);
    seek(ms);
  }

  function toggleDebug() {
    isDebugging = !isDebugging;
  }
</script>

<div class="now-playing-view">
  <div class="top-nav">
    <button class="back-btn" onclick={handleGoBack}>&larr; Back</button>
    <button class="debug-btn" class:active={isDebugging} onclick={toggleDebug}>
      Wait what? (Debug)
    </button>
  </div>
  
  <div class="main-layout">
    <div class="track-area">
      <div class="art-large">
        {#if $currentArtworkUrl}
          <img src={$currentArtworkUrl} alt="Album artwork" class="art-image" />
        {:else}
          <div class="art-placeholder"></div>
        {/if}
      </div>
      
      <div class="info-large">
        <h1>{$currentTrack?.title || 'Nothing Playing'}</h1>
        <h2>{$currentTrack?.artist || 'Unknown Artist'}</h2>
        <h3>{$currentTrack?.album || 'Unknown Album'}</h3>
      </div>

      <div class="scrubber-area">
        <div class="time-labels">
          <span>{formatDuration($positionMs)}</span>
          <span>{formatDuration($durationMs)}</span>
        </div>
        <input 
          type="range" 
          class="scrubber"
          min="0" 
          max="1" 
          step="0.001" 
          value={$progress} 
          onchange={handleSeek}
        />
      </div>
    </div>
  </div>

  {#if isDebugging && $audioDebug}
    <div class="debug-overlay">
      <h4>Audio Engine Debug</h4>
      <div class="debug-grid">
        <div class="debug-row">
          <label>Output Mode:</label>
          <span>{$audioDebug.output_mode}</span>
        </div>
        <div class="debug-row">
          <label>Device:</label>
          <span>{$audioDebug.device_name}</span>
        </div>
        <div class="debug-section">
          <h5>Source Format</h5>
          <div>{$audioDebug.decode_format.codec || 'Unknown'} / {$audioDebug.decode_format.container || 'Unknown'}</div>
          <div>{$audioDebug.decode_format.sample_rate}Hz / {$audioDebug.decode_format.bit_depth}-bit / {$audioDebug.decode_format.channels}ch</div>
        </div>
        <div class="debug-section">
          <h5>Output Format</h5>
          <div>{$audioDebug.output_format.sample_rate}Hz / {$audioDebug.output_format.bit_depth}-bit / {$audioDebug.output_format.channels}ch</div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .now-playing-view {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0; /* Cover everything including bottom bar if needed? No, bottom bar is persistent */
    /* Adjust to fit within content-area */
    height: 100%;
    background: transparent;
    z-index: 100;
    padding: 2rem;
    display: flex;
    flex-direction: column;
    color: #fff;
    box-sizing: border-box;
    overflow: hidden;
  }

  .top-nav {
    display: flex;
    justify-content: space-between;
    margin-bottom: 2rem;
  }

  .back-btn, .debug-btn {
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    padding: 0.5rem 1rem;
    color: #ccc;
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.2s;
  }
  .back-btn:hover, .debug-btn:hover {
    color: #fff;
    border-color: #fff;
    background: var(--glass-border);
    transform: scale(1.02);
  }
  .debug-btn.active {
    background: #4af;
    color: #000;
    border-color: #4af;
  }

  .main-layout {
    display: flex;
    justify-content: center;
    height: 100%;
    overflow: hidden;
  }

  .track-area {
    width: 100%;
    max-width: 800px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2rem;
  }

/* Queue styles removed as queue has moved to RightRail */

  .art-large {
    width: 350px;
    height: 350px;
    background: #222;
    border-radius: 12px;
    box-shadow: 0 20px 50px rgba(0,0,0,0.5);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .art-placeholder {
    width: 100%;
    height: 100%;
    background: linear-gradient(45deg, #222, #333);
    border-radius: 12px;
  }

  .art-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 12px;
  }

  .info-large {
    text-align: center;
  }
  .info-large h1 { font-size: 2.5rem; margin: 0 0 0.5rem 0; letter-spacing: -1px; text-shadow: 0 4px 12px rgba(0,0,0,0.5); font-weight: 700; }
  .info-large h2 { font-size: 1.5rem; color: rgba(255,255,255,0.8); margin: 0 0 0.5rem 0; font-weight: normal; text-shadow: 0 2px 4px rgba(0,0,0,0.5); }
  .info-large h3 { font-size: 1.1rem; color: rgba(255,255,255,0.6); margin: 0; font-weight: normal; }

  .scrubber-area {
    width: 100%;
    max-width: 600px;
  }
  
  .time-labels {
    display: flex;
    justify-content: space-between;
    font-size: 0.8rem;
    color: #888;
    margin-bottom: 0.5rem;
  }

  .scrubber {
    width: 100%;
    height: 6px;
    background: rgba(255,255,255,0.2);
    border-radius: 3px;
    appearance: none;
    cursor: pointer;
  }
  .scrubber::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    background: #fff;
    border-radius: 50%;
    box-shadow: 0 0 10px rgba(0,0,0,0.5);
    transition: transform 0.1s;
  }
  .scrubber::-webkit-slider-thumb:hover {
    transform: scale(1.2);
  }

  .debug-overlay {
    position: absolute;
    top: 80px;
    left: 2rem;
    width: 300px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    padding: 1rem;
    border-radius: 8px;
    font-family: monospace;
    font-size: 0.85rem;
    pointer-events: auto;
    box-shadow: var(--glass-shadow);
  }
  .debug-overlay h4 { color: #4af; margin: 0 0 1rem 0; border-bottom: 1px solid rgba(255,255,255,0.1); padding-bottom: 0.5rem; }
  .debug-row { display: flex; justify-content: space-between; margin-bottom: 0.5rem; }
  .debug-row label { color: #aaa; }
  .debug-section { margin-top: 1rem; border-top: 1px dashed rgba(255,255,255,0.1); padding-top: 0.5rem; }
  .debug-section h5 { margin: 0 0 0.5rem 0; color: #aaa; }
</style>
