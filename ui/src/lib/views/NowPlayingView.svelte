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
    <button class="back-btn" onclick={handleGoBack}>
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>
      Back
    </button>
  </div>
  
  <div class="main-layout">
    <div class="track-area">
      <div class="art-large">
        {#if $currentArtworkUrl}
          <img src={$currentArtworkUrl} alt="Album artwork" class="art-image" />
        {:else}
          <div class="art-placeholder">
            <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" class="placeholder-icon"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
          </div>
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

  <div class="debug-container">
    <button class="debug-toggle" class:active={isDebugging} onclick={toggleDebug}>
      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.1a2 2 0 0 1-1-1.72v-.51a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
      Debug
    </button>

    {#if isDebugging && $audioDebug}
      <div class="debug-overlay">
        <h4>Audio Engine</h4>
        <div class="debug-grid">
          <div class="debug-row">
            <span class="label">Mode</span>
            <span>{$audioDebug.output_mode}</span>
          </div>
          <div class="debug-row">
            <span class="label">Device</span>
            <span class="truncate">{$audioDebug.device_name}</span>
          </div>
          <div class="debug-section">
            <h5>Source</h5>
            <div>{$audioDebug.decode_format.codec || 'Unknown'} / {$audioDebug.decode_format.container || 'Unknown'}</div>
            <div class="mono">{$audioDebug.decode_format.sample_rate}Hz / {$audioDebug.decode_format.bit_depth}-bit</div>
          </div>
          <div class="debug-section">
            <h5>Output</h5>
            <div class="mono">{$audioDebug.output_format.sample_rate}Hz / {$audioDebug.output_format.bit_depth}-bit</div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .now-playing-view {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    height: 100%;
    background: transparent;
    z-index: 100;
    padding: 2rem;
    display: flex;
    flex-direction: column;
    color: var(--text-primary);
    box-sizing: border-box;
    overflow: hidden;
  }

  .top-nav {
    position: absolute;
    top: 2rem;
    left: 2rem;
    z-index: 10;
  }

  .back-btn {
    background: rgba(255, 255, 255, 0.05);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    padding: 0.5rem 1rem 0.5rem 0.75rem;
    color: var(--text-secondary);
    cursor: pointer;
    border-radius: 99px;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    font-weight: 500;
  }
  .back-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .main-layout {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    width: 100%;
  }

  .track-area {
    width: 100%;
    max-width: 800px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2.5rem;
  }

  .art-large {
    width: clamp(350px, 40vh, 450px);
    height: clamp(350px, 40vh, 450px);
    background: #111;
    border-radius: var(--artwork-radius-album-detail, 12px);
    box-shadow: 0 24px 60px rgba(0,0,0,0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }
  
  .art-placeholder {
    width: 100%;
    height: 100%;
    background: linear-gradient(135deg, #1a1a1a, #2a2a2a);
    display: flex;
    align-items: center;
    justify-content: center;
    color: rgba(255, 255, 255, 0.1);
  }

  .art-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: var(--artwork-radius-album-detail, 12px);
  }

  .info-large {
    text-align: center;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-width: 600px;
  }
  
  .info-large h1 { 
    font-size: var(--text-view-title); 
    margin: 0; 
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }
  
  .info-large h2 { 
    font-size: var(--text-section); 
    color: var(--text-secondary); 
    margin: 0; 
    font-weight: 500;
  }
  
  .info-large h3 { 
    font-size: var(--text-body); 
    color: var(--text-tertiary); 
    margin: 0; 
    font-weight: 400;
  }

  .scrubber-area {
    width: 100%;
    max-width: 600px;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .time-labels {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
    font-weight: 500;
  }

  .scrubber {
    width: 100%;
    height: 4px;
    background: rgba(255,255,255,0.1);
    border-radius: 2px;
    appearance: none;
    cursor: pointer;
    outline: none;
  }
  
  .scrubber::-webkit-slider-thumb {
    appearance: none;
    width: 12px;
    height: 12px;
    background: #fff;
    border-radius: 50%;
    box-shadow: 0 2px 6px rgba(0,0,0,0.3);
    transition: transform 0.1s;
    margin-top: -4px; /* Center thumb on track */
  }
  
  .scrubber::-webkit-slider-runnable-track {
    height: 4px;
    border-radius: 2px;
  }
  
  .scrubber::-webkit-slider-thumb:hover {
    transform: scale(1.3);
  }

  /* Debug Section */
  .debug-container {
    position: absolute;
    bottom: 2rem;
    right: 2rem;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 1rem;
    z-index: 20;
  }

  .debug-toggle {
    background: transparent;
    border: none;
    color: var(--text-disabled);
    font-size: 0.75rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
    border-radius: 6px;
    transition: color 0.2s;
  }
  
  .debug-toggle:hover, .debug-toggle.active {
    color: var(--text-secondary);
    background: rgba(255, 255, 255, 0.05);
  }

  .debug-overlay {
    width: 260px;
    background: rgba(10, 10, 12, 0.9);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.08);
    padding: 1rem;
    border-radius: 8px;
    font-size: 0.75rem;
    box-shadow: 0 10px 30px rgba(0,0,0,0.5);
  }
  
  .debug-overlay h4 { 
    color: var(--text-secondary); 
    margin: 0 0 0.75rem 0; 
    font-size: 0.8rem;
    font-weight: 600;
    border-bottom: 1px solid rgba(255,255,255,0.05);
    padding-bottom: 0.5rem;
  }
  
  .debug-row { 
    display: flex; 
    justify-content: space-between; 
    margin-bottom: 0.4rem; 
    align-items: center;
  }
  
  .debug-row .label { 
    color: var(--text-tertiary); 
  }
  
  .debug-row span {
    color: var(--text-primary);
    text-align: right;
  }
  
  .truncate {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 140px;
  }
  
  .debug-section { 
    margin-top: 0.75rem; 
    border-top: 1px dashed rgba(255,255,255,0.1); 
    padding-top: 0.5rem; 
  }
  
  .debug-section h5 { 
    margin: 0 0 0.25rem 0; 
    color: var(--text-tertiary); 
    font-weight: 500;
    font-size: 0.7rem;
  }
  
  .debug-section div {
    color: var(--text-secondary);
    margin-bottom: 0.1rem;
  }
  
  .mono {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.7rem;
    opacity: 0.8;
  }
</style>
