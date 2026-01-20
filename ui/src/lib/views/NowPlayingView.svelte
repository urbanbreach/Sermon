<script lang="ts">
  import { goBack } from '../state/route';
  import { 
    currentTrack, playbackState, queue, currentIndex, progress, 
    positionMs, durationMs, seek, audioDebug, isPlaying, playNow
  } from '../state/playback';
  import { Fixtures } from '../data/fixtures';
  import { getArtworkBestForTrack, getArtworkBytes } from '../api/artwork';
  import { computeThemeFromImageSrc, applyThemeToDocument, resetTheme } from '../theme/dynamicTheme';
  
  let isDebugging = false;
  let artworkUrl: string | null = $state(null);
  let lastTrackId: number | null = $state(null);

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

  async function loadArtwork(track: typeof $currentTrack) {
    if (!track) {
      artworkUrl = null;
      lastTrackId = null;
      return;
    }

    // Skip if same track
    if (track.id === lastTrackId && artworkUrl) return;
    lastTrackId = track.id;

    // Mock/Snapshot mode: use fixtures
    if (import.meta.env.SERMON_MOCK === '1') {
      const fixtureAlbums = Fixtures.getAlbums();
      const fixtureArtists = Fixtures.getArtists();

      // Derive album key from current track
      const albumTitleSort = (track.album?.trim().toLowerCase()) || 'unknown album';
      const albumArtistSort = (track.artist?.trim().toLowerCase()) || 'unknown artist';

      for (const fixtureAlbum of fixtureAlbums) {
        const artist = fixtureArtists.find(a => a.id === fixtureAlbum.artistId);
        const artistSort = (artist?.name?.trim().toLowerCase()) || 'unknown artist';
        const titleSort = (fixtureAlbum.title?.trim().toLowerCase()) || 'unknown album';

        if (artistSort === albumArtistSort && titleSort === albumTitleSort) {
          const url = Fixtures.getArtworkPath(fixtureAlbum.artworkFile);
          if (url) {
            artworkUrl = url;
          }
          return;
        }
      }
      artworkUrl = null;
      return;
    }

    // Runtime mode: use IPC
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

  // React to track changes
  $effect(() => {
    loadArtwork($currentTrack);
  });

  // React to artwork changes - compute and apply theme
  $effect(() => {
    if (artworkUrl) {
      computeThemeFromImageSrc(artworkUrl)
        .then(theme => applyThemeToDocument(theme))
        .catch(err => {
          console.error('Failed to compute theme:', err);
          resetTheme();
        });
    } else {
      resetTheme();
    }
  });
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
        {#if artworkUrl}
          <img src={artworkUrl} alt="Album artwork" class="art-image" />
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

    <div class="queue-area">
      <h3>Queue</h3>
      <div class="queue-list">
        {#each $queue as item, i}
          <div 
            class="queue-item" 
            class:active={i === $currentIndex}
            role="button"
            tabindex="0"
            ondblclick={() => playNow(item.track_id)}
            onkeydown={(e) => e.key === 'Enter' && playNow(item.track_id)}
          >
            <span class="q-index">{i + 1}</span>
            <div class="q-info">
              <span class="q-title">{item.title}</span>
              <span class="q-artist">{item.artist}</span>
            </div>
            <span class="q-time">{formatDuration(item.duration_ms || 0)}</span>
          </div>
        {:else}
          <div class="empty-queue">Queue is empty</div>
        {/each}
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
    background: #0a0a0a;
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
    background: none;
    border: 1px solid var(--glass-border);
    padding: 0.5rem 1rem;
    color: #888;
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.2s;
  }
  .back-btn:hover, .debug-btn:hover {
    color: #fff;
    border-color: #fff;
    background: rgba(255,255,255,0.1);
  }
  .debug-btn.active {
    background: #4af;
    color: #000;
    border-color: #4af;
  }

  .main-layout {
    display: flex;
    gap: 4rem;
    height: 100%;
    overflow: hidden;
  }

  .track-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2rem;
  }

  .queue-area {
    width: 350px;
    display: flex;
    flex-direction: column;
    background: var(--glass-bg);
    border-radius: 12px;
    padding: 1rem;
    border: 1px solid var(--glass-border);
  }

  .queue-area h3 {
    margin: 0 0 1rem 0;
    font-size: 1.2rem;
    color: #ccc;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--glass-border);
  }

  .queue-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .queue-item {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem;
    border-radius: 4px;
    cursor: default;
    transition: background 0.1s;
  }
  .queue-item:hover {
    background: rgba(255,255,255,0.05);
  }
  .queue-item.active {
    background: rgba(74, 175, 255, 0.1);
    border-left: 3px solid #4af;
  }
  .queue-item.active .q-title {
    color: #4af;
  }

  .q-index { color: #555; font-size: 0.8rem; width: 20px; }
  .q-info { flex: 1; overflow: hidden; }
  .q-title { display: block; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .q-artist { display: block; font-size: 0.8rem; color: #888; }
  .q-time { color: #666; font-size: 0.8rem; }
  .empty-queue { color: #555; text-align: center; padding: 2rem; }

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
  .info-large h1 { font-size: 2.5rem; margin: 0 0 0.5rem 0; letter-spacing: -1px; }
  .info-large h2 { font-size: 1.5rem; color: #aaa; margin: 0 0 0.5rem 0; font-weight: normal; }
  .info-large h3 { font-size: 1.1rem; color: #666; margin: 0; font-weight: normal; }

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
    background: #333;
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
  }

  .debug-overlay {
    position: absolute;
    top: 80px;
    left: 2rem;
    width: 300px;
    background: rgba(0,0,0,0.9);
    border: 1px solid #333;
    padding: 1rem;
    border-radius: 8px;
    font-family: monospace;
    font-size: 0.85rem;
    pointer-events: none; /* Let clicks pass through? No, might want to copy text */
    pointer-events: auto;
  }
  .debug-overlay h4 { color: #4af; margin: 0 0 1rem 0; border-bottom: 1px solid #333; padding-bottom: 0.5rem; }
  .debug-row { display: flex; justify-content: space-between; margin-bottom: 0.5rem; }
  .debug-row label { color: #888; }
  .debug-section { margin-top: 1rem; border-top: 1px dashed #333; padding-top: 0.5rem; }
  .debug-section h5 { margin: 0 0 0.5rem 0; color: #aaa; }
</style>
