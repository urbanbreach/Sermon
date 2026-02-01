<script lang="ts">
  import { goBack } from '../state/route';
  import { 
    currentTrack, playbackState, progress, 
    positionMs, durationMs, seek, audioDebug
  } from '../state/playback';
  import { currentArtworkUrl } from '../state/artwork';
  import { ArrowLeft, Disc, Activity, Cpu, Speaker } from 'lucide-svelte';
  
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

  // Helper to format sample rate (e.g. 44100 -> 44.1 kHz)
  function formatSampleRate(hz?: number): string {
    if (!hz) return 'Unknown';
    return (hz / 1000).toFixed(1) + ' kHz';
  }

  // Helper for channels
  function formatChannels(ch?: number): string {
    if (!ch) return 'Stereo';
    if (ch === 1) return 'Mono';
    if (ch === 2) return 'Stereo';
    return `${ch} Ch`;
  }
</script>

<div class="now-playing-view">
  <div class="top-nav">
    <button class="back-btn" onclick={handleGoBack}>
      <ArrowLeft size={20} />
      <span>Back</span>
    </button>
  </div>
  
  <div class="content-grid">
    <div class="artwork-section">
      <div class="art-container">
        {#if $currentArtworkUrl}
          <img src={$currentArtworkUrl} alt="Album artwork" class="art-image" />
        {:else}
          <div class="art-placeholder">
            <Disc size={64} strokeWidth={1} />
          </div>
        {/if}
      </div>
    </div>

    <div class="info-section">
      <div class="track-header">
        <h1>{$currentTrack?.title || 'Nothing Playing'}</h1>
        <h2>{$currentTrack?.artist || 'Unknown Artist'}</h2>
        <h3>{$currentTrack?.album || 'Unknown Album'}</h3>
      </div>

      {#if $audioDebug}
        <div class="tech-grid">
          <div class="tech-item">
            <span class="label">Format</span>
            <span class="value">{$audioDebug.decode_format.codec?.toUpperCase() || 'PCM'}</span>
          </div>
          <div class="tech-item">
            <span class="label">Bit Depth</span>
            <span class="value">{$audioDebug.decode_format.bit_depth}-bit</span>
          </div>
          <div class="tech-item">
            <span class="label">Sample Rate</span>
            <span class="value">{formatSampleRate($audioDebug.decode_format.sample_rate)}</span>
          </div>
          <div class="tech-item">
            <span class="label">Channels</span>
            <span class="value">{formatChannels($audioDebug.decode_format.channels)}</span>
          </div>
        </div>

        <div class="signal-path">
          <div class="path-header">
            <Activity size={14} />
            <span>SIGNAL PATH</span>
          </div>
          <div class="path-flow">
            <div class="node source">
              <span class="node-icon"><Disc size={14} /></span>
              <div class="node-info">
                <span class="node-title">Source</span>
                <span class="node-detail">
                  {$audioDebug.decode_format.codec?.toUpperCase()} 
                  {formatSampleRate($audioDebug.decode_format.sample_rate)} / {$audioDebug.decode_format.bit_depth}bit
                </span>
              </div>
            </div>
            
            <div class="connector"></div>

            <div class="node engine">
              <span class="node-icon"><Cpu size={14} /></span>
              <div class="node-info">
                <span class="node-title">Engine</span>
                <span class="node-detail">
                  {$audioDebug.output_mode === 'exclusive' ? 'WASAPI Exclusive' : 'WASAPI Shared'}
                  {#if $audioDebug.bit_perfect === 'yes'}
                    <span class="badge-perfect">BIT-PERFECT</span>
                  {/if}
                </span>
              </div>
            </div>

            <div class="connector"></div>

            <div class="node output">
              <span class="node-icon"><Speaker size={14} /></span>
              <div class="node-info">
                <span class="node-title">Output</span>
                <span class="node-detail">
                  {$audioDebug.device_name}
                  <br/>
                  {formatSampleRate($audioDebug.output_format.sample_rate)} / {$audioDebug.output_format.bit_depth}bit
                </span>
              </div>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <div class="bottom-controls">
    <div class="scrubber-container">
      <div class="time-info">
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

<style>
  .now-playing-view {
    position: absolute;
    inset: 0;
    background: transparent;
    z-index: 100;
    display: flex;
    flex-direction: column;
    padding: 2rem;
    color: var(--text-primary);
    box-sizing: border-box;
    overflow: hidden;
  }

  .top-nav {
    margin-bottom: 2rem;
  }

  .back-btn {
    background: rgba(255, 255, 255, 0.05);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    padding: 0.5rem 1rem;
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

  .content-grid {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4rem;
    align-items: center;
    max-width: 1400px;
    margin: 0 auto;
    width: 100%;
    padding-bottom: 4rem; /* Space for bottom controls */
  }

  .artwork-section {
    display: flex;
    justify-content: flex-end;
  }

  .art-container {
    width: 100%;
    max-width: 500px;
    aspect-ratio: 1;
    background: #111;
    border-radius: 4px; /* Brutalist: sharp or small radius */
    box-shadow: 0 30px 80px rgba(0,0,0,0.5);
    position: relative;
    overflow: hidden;
  }

  .art-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .art-placeholder {
    width: 100%;
    height: 100%;
    background: #1a1a1a;
    display: flex;
    align-items: center;
    justify-content: center;
    color: rgba(255, 255, 255, 0.1);
  }

  .info-section {
    display: flex;
    flex-direction: column;
    justify-content: center;
    max-width: 600px;
  }

  .track-header {
    margin-bottom: 3rem;
  }

  .track-header h1 {
    font-size: clamp(2rem, 4vw, 3.5rem);
    font-weight: 800;
    line-height: 1.1;
    margin: 0 0 0.5rem 0;
    letter-spacing: -0.03em;
  }

  .track-header h2 {
    font-size: clamp(1.2rem, 2vw, 1.75rem);
    font-weight: 500;
    color: var(--text-secondary);
    margin: 0 0 0.25rem 0;
    letter-spacing: -0.01em;
  }

  .track-header h3 {
    font-size: 1.1rem;
    color: var(--text-tertiary);
    margin: 0;
    font-weight: 400;
  }

  .tech-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 2rem;
    margin-bottom: 3rem;
    padding-bottom: 2rem;
    border-bottom: 1px solid rgba(255,255,255,0.1);
  }

  .tech-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .tech-item .label {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-tertiary);
    font-weight: 600;
  }

  .tech-item .value {
    font-family: 'JetBrains Mono', monospace;
    font-size: 1.25rem;
    color: var(--text-primary);
  }

  .signal-path {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .path-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-tertiary);
    font-weight: 600;
  }

  .path-flow {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    position: relative;
  }

  .node {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem;
    background: rgba(255,255,255,0.03);
    border: 1px solid rgba(255,255,255,0.05);
    border-radius: 4px;
  }

  .node-icon {
    color: var(--text-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .node-info {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .node-title {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .node-detail {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.75rem;
    color: var(--text-tertiary);
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .connector {
    width: 1px;
    height: 0.75rem;
    background: rgba(255,255,255,0.1);
    margin-left: 1.25rem; /* Align with icon center roughly */
  }

  .badge-perfect {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
    font-size: 0.6rem;
    padding: 0.1rem 0.3rem;
    border-radius: 2px;
    font-weight: 600;
  }

  .bottom-controls {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 2rem 4rem;
    background: linear-gradient(to top, rgba(0,0,0,0.8), transparent);
  }

  .scrubber-container {
    width: 100%;
    max-width: 800px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .time-info {
    display: flex;
    justify-content: space-between;
    font-size: 0.8rem;
    color: var(--text-tertiary);
    font-family: 'JetBrains Mono', monospace;
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
    border-radius: 0; /* Square thumb for brutalist feel */
    transition: transform 0.1s;
    margin-top: -4px;
  }
  
  .scrubber::-webkit-slider-runnable-track {
    height: 4px;
    border-radius: 2px;
  }
  
  .scrubber::-webkit-slider-thumb:hover {
    transform: scale(1.5);
  }

  @media (max-width: 900px) {
    .content-grid {
      grid-template-columns: 1fr;
      gap: 2rem;
      text-align: center;
    }

    .artwork-section {
      justify-content: center;
    }

    .info-section {
      align-items: center;
    }

    .track-header h1 {
      font-size: 2rem;
    }

    .tech-grid {
      width: 100%;
      text-align: left;
    }

    .signal-path {
      width: 100%;
      text-align: left;
    }
  }
</style>
