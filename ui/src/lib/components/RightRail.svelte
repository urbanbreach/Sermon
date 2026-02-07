<script lang="ts">
  import { railMode, isRailOpen, setRailMode, toggleRail, albumTracks, albumTracksLoading } from '../state/rightRail';
  import { currentTrack, currentTrackFull, playNow, queue, playNowWithQueue } from '../state/playback';
  import ArtworkImage from './ArtworkImage.svelte';
  import { currentLyrics, currentLineIndex, lyricsStatus, fetchLyricsForTrack } from '../state/lyrics';
  import { railSplitRatio } from '../state/effects';
  import { navigate } from '../state/route';
  import { Disc3, MicVocal, Maximize2, Volume2, ListMusic } from '@lucide/svelte';
  import { derived } from 'svelte/store';
  import { fade, slide } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { fadeIn, pressScale } from '../utils/animations';
  import type { TrackRow } from '../types/library';
  import VerticalResizeHandle from './VerticalResizeHandle.svelte';
  
  let railContentHeight = $state(0);
  let lyricsScrollEl: HTMLDivElement | undefined = $state(undefined);

  // Auto-fetch lyrics when lyrics tab is opened and status is idle
  $effect(() => {
    const mode = $railMode;
    const track = $currentTrack;
    const status = $lyricsStatus;
    if (mode === 'lyrics' && track && status === 'idle') {
      fetchLyricsForTrack(track.id);
    }
  });

  // Auto-scroll to active lyric line
  $effect(() => {
    const idx = $currentLineIndex;
    if (!lyricsScrollEl || idx < 0) return;
    const activeLine = lyricsScrollEl.querySelector('.lyric-line.active');
    if (activeLine) {
      activeLine.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  });

  function goFullscreenLyrics() {
    navigate({ name: 'lyrics-fullscreen' });
  }

  // Helper: Format duration from ms to M:SS
  function formatDuration(ms: number | undefined): string {
    if (!ms) return '—';
    const totalSecs = Math.floor(ms / 1000);
    const mins = Math.floor(totalSecs / 60);
    const secs = totalSecs % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  // Helper: Build audio format info string
  function formatAudioInfo(track: TrackRow | null): string {
    if (!track) return '';
    const parts: string[] = [];
    
    // DSD detection - show DSD rate instead of PCM info
    if (track.dsdRateHz) {
      // Map DSD rate to common names
      const dsdRate = track.dsdRateHz;
      let dsdName: string;
      if (dsdRate <= 2900000) dsdName = 'DSD64';
      else if (dsdRate <= 5700000) dsdName = 'DSD128';
      else if (dsdRate <= 11400000) dsdName = 'DSD256';
      else dsdName = 'DSD512';
      
      parts.push(dsdName);
      
      // Show DSD rate in MHz
      const mhz = dsdRate / 1000000;
      parts.push(`${mhz.toFixed(4)} MHz`);
      
      // Channels
      if (track.dsdChannels) {
        parts.push(track.dsdChannels === 2 ? 'Stereo' : track.dsdChannels === 1 ? 'Mono' : `${track.dsdChannels}ch`);
      }
      
      // Duration
      if (track.durationMs) {
        parts.push(formatDuration(track.durationMs));
      }
      
      return parts.join(', ');
    }
    
    // Codec
    if (track.codec) {
      parts.push(track.codec.toUpperCase());
    }
    
    // Bit depth
    if (track.bitDepth) {
      parts.push(`${track.bitDepth} bit`);
    }
    
    // Sample rate
    if (track.sampleRate) {
      const kHz = track.sampleRate / 1000;
      parts.push(`${kHz} kHz`);
    }
    
    // Channels
    if (track.channels) {
      parts.push(track.channels === 2 ? 'Stereo' : track.channels === 1 ? 'Mono' : `${track.channels}ch`);
    }
    
    // Duration
    if (track.durationMs) {
      parts.push(formatDuration(track.durationMs));
    }
    
    return parts.join(', ');
  }

  function normalizeAlbumContext(
    albumArtist?: string | null,
    artist?: string | null,
    album?: string | null
  ) {
    const normalizedAlbumArtist = (albumArtist ?? '').trim().toLowerCase();
    const normalizedArtist = (artist ?? '').trim().toLowerCase();
    const normalizedAlbum = (album ?? '').trim().toLowerCase();

    return {
      artistSort: normalizedAlbumArtist || normalizedArtist || 'unknown artist',
      titleSort: normalizedAlbum || 'unknown album'
    };
  }

  // Helper: Check if album has multiple discs
  function hasMultipleDiscs(tracks: TrackRow[]): boolean {
    const discs = new Set(tracks.map(t => t.discNo ?? 1));
    return discs.size > 1 || (discs.size === 1 && !discs.has(1));
  }

  // Helper: Group tracks by disc number
  interface DiscGroup {
    discNo: number;
    tracks: TrackRow[];
  }
  
  function groupTracksByDisc(tracks: TrackRow[]): DiscGroup[] {
    const groups = new Map<number, TrackRow[]>();
    
    for (const track of tracks) {
      const discNo = track.discNo ?? 1;
      if (!groups.has(discNo)) {
        groups.set(discNo, []);
      }
      groups.get(discNo)!.push(track);
    }
    
    // Sort by disc number
    const sortedDiscs = Array.from(groups.keys()).sort((a, b) => a - b);
    return sortedDiscs.map(discNo => ({
      discNo,
      tracks: groups.get(discNo)!
    }));
  }

  // Heuristic: Check if queue matches album exactly
  const isAlbumQueue = derived([queue, albumTracks, currentTrackFull], ([$queue, $albumTracks, $currentTrackFull]) => {
    if (!$currentTrackFull || $queue.length === 0 || $albumTracks.length === 0) return false;
    if ($queue.length !== $albumTracks.length) return false;

    // Check if all queue items match the current album context
    // We check if IDs match exactly in order
    for (let i = 0; i < $queue.length; i++) {
      if ($queue[i].track_id !== $albumTracks[i].id) {
        return false;
      }
    }
    return true;
  });

  // Derived: Grouped tracks
  const discGroups = derived(albumTracks, ($tracks) => groupTracksByDisc($tracks));
  const showDiscDividers = derived(albumTracks, ($tracks) => hasMultipleDiscs($tracks));
  
  // Track count for header
  const trackCount = derived([albumTracks, queue, isAlbumQueue], ([$albumTracks, $queue, $isAlbumQueue]) => {
    return $isAlbumQueue ? $albumTracks.length : $queue.length;
  });

  function handleKeydown(e: KeyboardEvent, trackId: number | undefined) {
    if (e.key === 'Enter' && trackId) {
      playNow(trackId);
    }
  }
  
  function handleQueueKeydown(e: KeyboardEvent, trackId: number, index: number) {
    if (e.key === 'Enter') {
      // For queue items, we might want to play from that index?
      // But playNow(trackId) plays the track. 
      // If we want to jump to that position in queue, we need a different API or just playNow if it handles it.
      // playNow(trackId) usually replaces the queue or plays from library.
      // playNowWithQueue is for playing a list.
      // If we are in the queue view, clicking an item should probably just skip to that item in the queue?
      // But the current API doesn't seem to expose "skip to index".
      // `playNow` in `playback.ts` calls `api.queuePlayNow(trackId)`.
      // Let's stick to playNow for now as it's safe.
      playNow(trackId);
    }
  }
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
    <!-- Header with mode toggle -->
    <div class="rail-header">
      <div class="header-pills">
        <!-- Mode icons pill -->
        <div class="mode-pill">
          <button 
            class="pill-icon"
            class:active={$railMode === 'now-playing'}
            onclick={() => setRailMode('now-playing')}
            title="Queue"
            use:pressScale={{ scale: 0.95 }}
          >
            {#if $isAlbumQueue}
              <Disc3 size={16} />
            {:else}
              <ListMusic size={16} />
            {/if}
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
        
        <!-- Track count pill -->
        <div class="count-pill">
          {$trackCount} {$trackCount === 1 ? 'track' : 'tracks'}
        </div>
      </div>
    </div>
    
    <!-- Content based on mode -->
    <div class="rail-content" bind:clientHeight={railContentHeight} use:fadeIn={{ duration: 200, delay: 100 }}>
      {#if $railMode === 'now-playing'}
        <!-- Playing Tracks Section -->
        <div class="section playing-tracks-section" style="height: {Math.floor(railContentHeight * $railSplitRatio)}px;">
          {#if $currentTrackFull || $queue.length > 0}
            <!-- Album Info Card (Only show if we have current track info) -->
            {#if $currentTrackFull}
              {@const albumContext = normalizeAlbumContext(
                $currentTrackFull.albumArtist,
                $currentTrackFull.artist || $currentTrack?.artist,
                $currentTrackFull.album || $currentTrack?.album
              )}
              <div class="album-info-card">
                <div class="album-thumb-wrapper">
                  <ArtworkImage
                    artistSort={albumContext.artistSort}
                    titleSort={albumContext.titleSort}
                    size={128}
                    class="album-thumb"
                  />
                </div>
                <div class="album-info">
                  <div class="album-artist">{$currentTrackFull.albumArtist || $currentTrackFull.artist || '—'}</div>
                  <div class="album-title">{$currentTrackFull.album || 'Unknown Album'}</div>
                  <div class="album-meta">
                    {#if $currentTrackFull.year}
                      <span>{$currentTrackFull.year}</span>
                    {/if}
                    {#if $currentTrackFull.year && $currentTrackFull.genre}
                      <span class="meta-dot">•</span>
                    {/if}
                    {#if $currentTrackFull.genre}
                      <span>{$currentTrackFull.genre}</span>
                    {/if}
                  </div>
                </div>
              </div>
            {/if}

            <!-- Track List -->
            {#if $albumTracksLoading}
              <div class="loading-state">Loading tracks...</div>
            {:else if $isAlbumQueue}
              <div class="track-list">
                {#each $discGroups as group}
                  {#if $showDiscDividers}
                    <div class="disc-divider">Disc {group.discNo}</div>
                  {/if}
                  {#each group.tracks as track (track.id)}
                    {@const isPlaying = track.id === $currentTrack?.id}
                    {@const albumContext = normalizeAlbumContext(track.albumArtist, track.artist, track.album)}
                    <div 
                      class="track-row"
                      class:playing={isPlaying}
                      role="button"
                      tabindex="0"
                      ondblclick={() => track.id && playNow(track.id)}
                      onkeydown={(e) => handleKeydown(e, track.id)}
                    >
                      <span class="track-no">
                        {#if isPlaying}
                          <Volume2 size={14} class="playing-icon" />
                        {:else}
                          {track.trackNo ?? '—'}
                        {/if}
                      </span>
                      <span class="track-title">{track.title || '—'}</span>
                      <span class="track-artist">{track.artist || '—'}</span>
                    </div>
                  {/each}
                {/each}
              </div>
            {:else if $queue.length > 0}
              <div class="track-list">
                {#each $queue as item, i (item.track_id + '-' + i)}
                  {@const isPlaying = item.track_id === $currentTrack?.id}
                  {@const queueContext = normalizeAlbumContext(undefined, item.artist, item.album)}
                  <div 
                    class="track-row"
                    class:playing={isPlaying}
                    role="button"
                    tabindex="0"
                    ondblclick={() => playNow(item.track_id)}
                    onkeydown={(e) => handleQueueKeydown(e, item.track_id, i)}
                  >
                    <span class="track-no">
                      {#if isPlaying}
                        <Volume2 size={14} class="playing-icon" />
                      {:else}
                        {i + 1}
                      {/if}
                    </span>
                    <span class="track-title">{item.title || '—'}</span>
                    <span class="track-artist">{item.artist || '—'}</span>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="empty-state" use:fadeIn={{ duration: 300 }}>
                <Disc3 size={24} strokeWidth={1.5} />
                <span>Nothing playing</span>
              </div>
            {/if}
          {:else}
            <div class="empty-state" use:fadeIn={{ duration: 300 }}>
              <Disc3 size={24} strokeWidth={1.5} />
              <span>Nothing playing</span>
            </div>
          {/if}
        </div>

        <VerticalResizeHandle containerHeight={railContentHeight} />

        <!-- Track Information Section -->
        <div class="section track-info-section">
          {#if $currentTrackFull}
            {@const albumContext = normalizeAlbumContext(
              $currentTrackFull.albumArtist,
              $currentTrackFull.artist || $currentTrack?.artist,
              $currentTrackFull.album || $currentTrack?.album
            )}
            <div class="track-info-list">
              <div class="info-row">
                <span class="info-value title">{$currentTrackFull.title || '—'}</span>
              </div>
              <div class="info-row">
                <span class="info-value">{$currentTrackFull.artist || '—'}</span>
              </div>
              <div class="info-row">
                <span class="info-value">{$currentTrackFull.album || '—'}</span>
              </div>
              {#if $currentTrackFull.year}
                <div class="info-row">
                  <span class="info-value">{$currentTrackFull.year}</span>
                </div>
              {/if}
              {#if $currentTrackFull.genre}
                <div class="info-row">
                  <span class="info-value">{$currentTrackFull.genre}</span>
                </div>
              {/if}
              <div class="info-row format">
                <span class="info-value">{formatAudioInfo($currentTrackFull)}</span>
              </div>
            </div>

            <!-- Large Artwork -->
            <div class="large-artwork-container">
              <ArtworkImage 
                artistSort={albumContext.artistSort}
                titleSort={albumContext.titleSort}
                size={512} 
                class="large-artwork" 
              />
            </div>
          {:else}
            <div class="empty-state" use:fadeIn={{ duration: 300 }}>
              <Disc3 size={24} strokeWidth={1.5} />
              <span>No track information</span>
            </div>
          {/if}
        </div>
        
      {:else if $railMode === 'lyrics'}
        <div class="section lyrics-section">
          <div class="lyrics-header-row">
            <h3 class="section-header">Lyrics</h3>
            <button class="fullscreen-btn" onclick={goFullscreenLyrics} title="Fullscreen" use:pressScale={{ scale: 0.95 }}>
              <Maximize2 size={16} />
            </button>
          </div>

          {#if $lyricsStatus === 'loading'}
            <div class="empty-state" data-testid="lyrics-loading" use:fadeIn={{ duration: 300 }}>
              <MicVocal size={24} strokeWidth={1.5} />
              <span>Loading lyrics…</span>
            </div>
          {:else if $lyricsStatus === 'error'}
            <div class="empty-state" data-testid="lyrics-error" use:fadeIn={{ duration: 300 }}>
              <MicVocal size={24} strokeWidth={1.5} />
              <span>Failed to load lyrics</span>
            </div>
          {:else if $currentLyrics && $currentLyrics.length > 0}
            <div class="lyrics-rail-content" data-testid="lyrics-lines">
              <div class="lyrics-mask-top"></div>
              <div class="lyrics-scroll-container" bind:this={lyricsScrollEl}>
                {#each $currentLyrics as line, i}
                  <p
                    class="lyric-line"
                    class:active={i === $currentLineIndex}
                    class:before={i < $currentLineIndex}
                    class:after={i > $currentLineIndex}
                  >
                    {line || '\u00A0'}
                  </p>
                {/each}
              </div>
              <div class="lyrics-mask-bottom"></div>
            </div>
          {:else}
            <div class="empty-state" data-testid="lyrics-empty" use:fadeIn={{ duration: 300 }}>
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
    background: var(--surface-1);
    border-left: 1px solid var(--divider-color);
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
      background: var(--bar-bg, rgba(10, 10, 10, 0.95));
      box-shadow: var(--shadow-3);
    }
    
    .rail-scrim {
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background: rgba(0,0,0,0.5);
      z-index: 89;
    }
  }

  /* ============================================
     HEADER
     ============================================ */
  .rail-header {
    padding: 8px 12px;
    -webkit-app-region: no-drag;
  }

  .header-pills {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .mode-pill {
    display: flex;
    align-items: center;
    gap: 2px;
    background: var(--surface-1);
    padding: 4px;
    border-radius: 999px;
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
    color: var(--text-primary);
    background: var(--surface-2);
  }

  .count-pill {
    background: var(--surface-1);
    padding: 6px 12px;
    border-radius: 999px;
    font-size: 12px;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  /* ============================================
     CONTENT AREA
     ============================================ */
  .rail-content {
    flex: 1;
    overflow: hidden;
    padding: 0 12px 12px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-height: 0;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-height: 0;
  }

  .playing-tracks-section {
    flex: none;
    min-height: 80px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .playing-tracks-section .track-list {
    flex: 1;
    overflow-y: auto;
  }

  .track-info-section {
    flex: 1;
    min-height: 80px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .section-header {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  /* ============================================
     ALBUM INFO CARD
     ============================================ */
  .album-info-card {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    padding: 6px;
    border-radius: 6px;
    background: var(--surface-1);
    border: 1px solid var(--divider-color);
  }

  .album-thumb-wrapper {
    width: 48px;
    height: 48px;
    border-radius: 3px;
    overflow: hidden;
    background: #222;
    flex-shrink: 0;
  }

  .album-info-card :global(.album-thumb) {
    width: 100%;
    height: 100%;
  }


  .album-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .album-artist {
    font-size: 12px;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .album-title {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .album-meta {
    font-size: 11px;
    color: var(--text-tertiary);
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .meta-dot {
    opacity: 0.5;
  }

  /* ============================================
     TRACK LIST
     ============================================ */
  .track-list {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .disc-divider {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-tertiary);
    padding: 6px 4px 3px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    border-bottom: 1px solid var(--divider-color);
    margin-bottom: 3px;
  }

  .track-row {
    display: grid;
    grid-template-columns: 20px 1fr auto;
    gap: 6px;
    align-items: center;
    padding: 2px 2px;
    border-radius: 3px;
    cursor: default;
    transition: background var(--motion-fast) var(--ease-out);
    font-size: 12px;
    line-height: 1.3;
  }

  .track-row:hover {
    background: var(--surface-hover);
  }

  .track-row:focus {
    background: var(--surface-2);
    outline: none;
    box-shadow: inset 0 0 0 1px var(--divider-color);
  }

  .track-row.playing {
    color: var(--text-primary);
  }

  .track-row.playing .track-no {
    color: var(--text-primary);
  }

  .track-no {
    text-align: right;
    color: var(--text-tertiary);
    font-size: 11px;
    display: flex;
    align-items: center;
    justify-content: flex-end;
  }

  .track-no :global(.playing-icon) {
    color: var(--text-primary);
  }

  .track-title {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-primary);
    font-size: 12px;
  }

  .track-artist {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-tertiary);
    font-size: 11px;
    text-align: right;
  }

  /* ============================================
     TRACK INFORMATION
     ============================================ */
  .track-info-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex-shrink: 0;
  }

  .info-row {
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.3;
  }

  .info-row .info-value.title {
    font-weight: 600;
    color: var(--text-primary);
    font-size: 13px;
  }

  .info-row.format {
    margin-top: 2px;
    font-size: 10px;
    color: var(--text-tertiary);
  }

  /* ============================================
     LARGE ARTWORK
     ============================================ */
  .large-artwork-container {
    margin-top: 8px;
    flex: 1;
    min-height: 0;
    display: flex;
    justify-content: center;
    overflow: hidden;
  }

  .large-artwork-container :global(.artwork-container.large-artwork) {
    width: auto;
    height: 100%;
    aspect-ratio: 1 / 1;
    max-width: 100%;
    min-height: 0;
    max-height: 100%;
    border-radius: var(--artwork-radius-sidebar, 6px);
    background: transparent;
    flex: 0 0 auto;
  }

  .large-artwork-container :global(.artwork-container.large-artwork img) {
    object-fit: contain;
  }

  /* ============================================
     EMPTY STATE
     ============================================ */
  .empty-state {
    color: var(--text-tertiary);
    font-size: 13px;
    padding: 48px 24px;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    background: var(--surface-1);
    border-radius: var(--radius-md);
    border: 1px dashed rgba(255, 255, 255, 0.1);
  }

  .empty-state :global(svg) {
    color: var(--text-disabled);
    opacity: 1;
  }

  .loading-state {
    color: var(--text-tertiary);
    font-size: 12px;
    padding: 16px;
    text-align: center;
  }

  /* ============================================
     LYRICS SECTION - APPLE-LIKE FLUID MOTION
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
    position: relative;
    overflow: hidden;
    min-height: 0;
  }

  .lyrics-scroll-container {
    height: 100%;
    overflow-y: auto;
    padding: 48px 0;
    scrollbar-width: none;
    scroll-behavior: smooth;
  }

  .lyrics-scroll-container::-webkit-scrollbar {
    display: none;
  }

  /* Gradient masks for edge softening */
  .lyrics-mask-top,
  .lyrics-mask-bottom {
    position: absolute;
    left: 0;
    right: 0;
    height: 48px;
    pointer-events: none;
    z-index: 2;
  }

  .lyrics-mask-top {
    top: 0;
    background: linear-gradient(to bottom, var(--surface-1), transparent);
  }

  .lyrics-mask-bottom {
    bottom: 0;
    background: linear-gradient(to top, var(--surface-1), transparent);
  }

  .lyric-line {
    margin: 0;
    padding: 8px 0;
    font-size: 20px;
    font-weight: 600;
    line-height: 1.35;
    color: rgba(255, 255, 255, 0.25);
    transition: color 0.4s cubic-bezier(0.25, 0.1, 0.25, 1),
                transform 0.4s cubic-bezier(0.25, 0.1, 0.25, 1),
                opacity 0.4s cubic-bezier(0.25, 0.1, 0.25, 1);
    transform-origin: left center;
    opacity: 0.5;
  }

  .lyric-line.active {
    color: #fff;
    font-weight: 700;
    transform: scale(1.03);
    opacity: 1;
  }

  .lyric-line.before {
    color: rgba(255, 255, 255, 0.2);
    opacity: 0.4;
  }

  .lyric-line.after {
    color: rgba(255, 255, 255, 0.35);
    opacity: 0.6;
  }

  /* ============================================
     ANIMATIONS
     ============================================ */
  .mode-pill button:nth-child(1) { animation-delay: 0ms; }
  .mode-pill button:nth-child(2) { animation-delay: 30ms; }

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
    .lyric-line {
      transition: none;
      transform: none !important;
    }
    .lyrics-scroll-container {
      scroll-behavior: auto;
    }
  }
</style>
