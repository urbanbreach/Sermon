<script lang="ts">
  import { createEventDispatcher, onDestroy, onMount } from 'svelte';
  import { bottomBarWaveformStyle } from '../state/effects';
  
  interface Props {
    peaks: Uint8Array | null;
    trackId: number | null;
    progress: number; // 0-1
    durationMs: number;
    style?: 'pills' | 'raw';
  }

  let { peaks, trackId, progress, durationMs, style }: Props = $props();
  
  // Derive effective style from prop or store
  let effectiveStyle = $derived(style ?? $bottomBarWaveformStyle);

  const dispatch = createEventDispatcher<{ seek: { ms: number } }>();

  // State
  let canvas: HTMLCanvasElement | undefined = $state();
  let container: HTMLDivElement | undefined = $state();
  let width = $state(0);
  let height = $state(0);
  let isDragging = $state(false);
  let dragProgress = $state(0);
  let hoverProgress: number | null = $state(null);
  let hoverX = $state(0);
  
  // Canvas dimension tracking (for efficiency - avoid resetting on every frame)
  let lastCanvasWidth = 0;
  let lastCanvasHeight = 0;
  let lastDpr = 0;

  // Derived
  let displayProgress = $derived(isDragging ? dragProgress : progress);
  let displayTime = $derived(formatTime((hoverProgress ?? displayProgress) * durationMs));
  
  const WAVEFORM_PLAYED_COLOR = 'rgba(255, 255, 255, 0.72)';
  const WAVEFORM_UNPLAYED_COLOR = 'rgba(255, 255, 255, 0.24)';
  const BAR_WIDTH = 2;
  const BAR_GAP = 1;
  const MIN_BAR_HEIGHT = 1;
  const GROW_DURATION_MS = 280;
  const MORPH_DURATION_MS = 240;
  const COLLAPSE_DURATION_MS = 180;
  const LOADING_LINE_DELAY_MS = 110;
  const BAR_DRAW_THRESHOLD = 0.004;

  let renderVersion = $state(0);

  let visualTrackId: number | null = null;
  let lastSeenTrackId: number | null = null;
  let lastSeenPeaksRef: Uint8Array | null = null;

  let displayPeaks: Float32Array | null = null;
  let transitionFromPeaks: Float32Array | null = null;
  let transitionToPeaks: Float32Array | null = null;
  let transitionTargetIsLine = false;

  let animationFrame = 0;
  let animationStartMs = 0;
  let animationDurationMs = 0;
  let loadingLineTimeout: ReturnType<typeof setTimeout> | undefined;

  function supportsReducedMotion(): boolean {
    return typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  }

  function hasVisiblePeaks(values: Float32Array | null): boolean {
    if (!values || values.length === 0) return false;
    for (let i = 0; i < values.length; i++) {
      if ((values[i] || 0) > BAR_DRAW_THRESHOLD) {
        return true;
      }
    }
    return false;
  }

  function normalizedPeaks(source: Uint8Array): Float32Array {
    const out = new Float32Array(source.length);
    for (let i = 0; i < source.length; i++) {
      out[i] = (source[i] || 0) / 255;
    }
    return out;
  }

  function samplePeakAt(source: Float32Array, position: number): number {
    if (source.length === 0) return 0;
    if (source.length === 1) return source[0] || 0;
    const clampedPos = Math.max(0, Math.min(source.length - 1, position));
    const low = Math.floor(clampedPos);
    const high = Math.min(low + 1, source.length - 1);
    const t = clampedPos - low;
    return (source[low] || 0) * (1 - t) + (source[high] || 0) * t;
  }

  function resamplePeaks(source: Float32Array | null, targetLength: number): Float32Array {
    if (targetLength <= 0) return new Float32Array(0);
    if (!source || source.length === 0) return new Float32Array(targetLength);
    if (source.length === targetLength) return source.slice();
    if (targetLength === 1) {
      const single = new Float32Array(1);
      single[0] = source[0] || 0;
      return single;
    }
    const out = new Float32Array(targetLength);
    const scale = (source.length - 1) / (targetLength - 1);
    for (let i = 0; i < targetLength; i++) {
      out[i] = samplePeakAt(source, i * scale);
    }
    return out;
  }

  function arraysRoughlyEqual(a: Float32Array, b: Float32Array): boolean {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
      if (Math.abs((a[i] || 0) - (b[i] || 0)) > 0.0015) {
        return false;
      }
    }
    return true;
  }

  function cancelAnimationLoop(): void {
    if (animationFrame) {
      cancelAnimationFrame(animationFrame);
      animationFrame = 0;
    }
  }

  function clearLoadingLineTimeout(): void {
    if (loadingLineTimeout) {
      clearTimeout(loadingLineTimeout);
      loadingLineTimeout = undefined;
    }
  }

  function easeOutCider(t: number): number {
    const clamped = Math.max(0, Math.min(1, t));
    return 1 - Math.pow(1 - clamped, 3.5);
  }

  function frameTransition(now: number): void {
    if (!transitionFromPeaks || !transitionToPeaks) {
      cancelAnimationLoop();
      return;
    }

    const elapsed = now - animationStartMs;
    const progress01 = animationDurationMs > 0 ? Math.min(1, elapsed / animationDurationMs) : 1;
    const eased = easeOutCider(progress01);
    const len = transitionFromPeaks.length;

    if (!displayPeaks || displayPeaks.length !== len) {
      displayPeaks = new Float32Array(len);
    }

    for (let i = 0; i < len; i++) {
      const from = transitionFromPeaks[i] || 0;
      const to = transitionToPeaks[i] || 0;
      displayPeaks[i] = from + (to - from) * eased;
    }

    renderVersion += 1;

    if (progress01 < 1) {
      animationFrame = requestAnimationFrame(frameTransition);
      return;
    }

    if (transitionTargetIsLine) {
      displayPeaks = null;
    } else if (transitionToPeaks) {
      displayPeaks = transitionToPeaks.slice();
    }

    transitionFromPeaks = null;
    transitionToPeaks = null;
    transitionTargetIsLine = false;
    animationFrame = 0;
    renderVersion += 1;
  }

  function startTransition(
    fromPeaks: Float32Array | null,
    toPeaks: Float32Array | null,
    durationMs: number,
    targetIsLine: boolean
  ): void {
    cancelAnimationLoop();

    const resolvedDuration = supportsReducedMotion() ? 0 : durationMs;
    if (resolvedDuration <= 0) {
      displayPeaks = targetIsLine ? null : toPeaks ? toPeaks.slice() : null;
      transitionFromPeaks = null;
      transitionToPeaks = null;
      transitionTargetIsLine = false;
      renderVersion += 1;
      return;
    }

    const targetLen = Math.max(toPeaks?.length ?? 0, fromPeaks?.length ?? 0);
    if (targetLen <= 0) {
      displayPeaks = null;
      renderVersion += 1;
      return;
    }

    const fromAligned = resamplePeaks(fromPeaks, targetLen);
    const toAligned = resamplePeaks(toPeaks, targetLen);

    if (arraysRoughlyEqual(fromAligned, toAligned)) {
      displayPeaks = targetIsLine ? null : toAligned;
      transitionFromPeaks = null;
      transitionToPeaks = null;
      transitionTargetIsLine = false;
      renderVersion += 1;
      return;
    }

    transitionFromPeaks = fromAligned;
    transitionToPeaks = toAligned;
    transitionTargetIsLine = targetIsLine;

    displayPeaks = fromAligned.slice();
    animationStartMs = performance.now();
    animationDurationMs = resolvedDuration;
    renderVersion += 1;

    animationFrame = requestAnimationFrame(frameTransition);
  }

  function transitionToWaveform(nextPeaks: Uint8Array, shouldMorph: boolean): void {
    clearLoadingLineTimeout();
    const target = normalizedPeaks(nextPeaks);
    const hasCurrentPeaks = hasVisiblePeaks(displayPeaks);

    if (hasCurrentPeaks) {
      startTransition(displayPeaks, target, shouldMorph ? MORPH_DURATION_MS : GROW_DURATION_MS, false);
      return;
    }

    startTransition(displayPeaks, target, GROW_DURATION_MS, false);
  }

  function transitionToLine(animate: boolean): void {
    clearLoadingLineTimeout();
    const hasCurrentPeaks = hasVisiblePeaks(displayPeaks);
    if (!animate || !hasCurrentPeaks) {
      cancelAnimationLoop();
      displayPeaks = null;
      transitionFromPeaks = null;
      transitionToPeaks = null;
      transitionTargetIsLine = false;
      renderVersion += 1;
      return;
    }

    startTransition(displayPeaks, null, COLLAPSE_DURATION_MS, true);
  }

  // React to track or waveform peak updates
  $effect(() => {
    const nextTrackId = trackId;
    const nextPeaks = peaks;
    const changed = nextTrackId !== lastSeenTrackId || nextPeaks !== lastSeenPeaksRef;
    if (!changed) return;

    lastSeenTrackId = nextTrackId;
    lastSeenPeaksRef = nextPeaks;

    isDragging = false;
    hoverProgress = null;

    const trackChanged = nextTrackId !== visualTrackId;

    if (nextPeaks && nextPeaks.length > 0) {
      transitionToWaveform(nextPeaks, trackChanged);
      visualTrackId = nextTrackId;
      return;
    }

    if (trackChanged && nextTrackId !== null && hasVisiblePeaks(displayPeaks)) {
      clearLoadingLineTimeout();
      const pendingTrackId = nextTrackId;
      loadingLineTimeout = setTimeout(() => {
        transitionToLine(true);
        visualTrackId = pendingTrackId;
      }, LOADING_LINE_DELAY_MS);
      return;
    }

    transitionToLine(trackChanged);
    visualTrackId = nextTrackId;
  });

  function formatTime(ms: number): string {
    if (!ms || ms < 0) return '0:00';
    const mins = Math.floor(ms / 60000);
    const secs = Math.floor((ms % 60000) / 1000);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function drawRawWaveform(
    ctx: CanvasRenderingContext2D,
    peaksData: Float32Array,
    displayProg: number,
    w: number,
    h: number
  ): void {
    const centerY = h / 2;
    const maxHalfHeight = h * 0.45;
    const peakLen = peaksData.length;

    if (peakLen === 0) {
      return;
    }

    // Helper to build envelope path (continuous mirrored polygon)
    function buildEnvelopePath(): void {
      ctx.beginPath();
      
      // Top edge: left to right
      for (let x = 0; x <= w; x++) {
        const peakIdx = Math.min(Math.floor((x / w) * peakLen), peakLen - 1);
        const amp = peaksData[peakIdx] || 0;
        const topY = centerY - amp * maxHalfHeight;
        if (x === 0) {
          ctx.moveTo(x, topY);
        } else {
          ctx.lineTo(x, topY);
        }
      }
      
      // Bottom edge: right to left (creates closed polygon)
      for (let x = w; x >= 0; x--) {
        const peakIdx = Math.min(Math.floor((x / w) * peakLen), peakLen - 1);
        const amp = peaksData[peakIdx] || 0;
        const bottomY = centerY + amp * maxHalfHeight;
        ctx.lineTo(x, bottomY);
      }
      
      ctx.closePath();
    }

    // Draw unplayed portion (full waveform in muted color)
    buildEnvelopePath();
    ctx.fillStyle = WAVEFORM_UNPLAYED_COLOR;
    ctx.fill();

    // Draw played portion with clipping
    const playedWidth = w * displayProg;
    if (playedWidth > 0) {
      ctx.save();
      ctx.beginPath();
      ctx.rect(0, 0, playedWidth, h);
      ctx.clip();
      
      buildEnvelopePath();
      ctx.fillStyle = WAVEFORM_PLAYED_COLOR;
      ctx.fill();
      
      ctx.restore();
    }
  }

  // Draw pills waveform (existing rounded bars)
  function drawPillsWaveform(
    ctx: CanvasRenderingContext2D,
    peaksData: Float32Array,
    displayProg: number,
    w: number,
    h: number
  ): void {
    const totalBars = Math.floor(w / (BAR_WIDTH + BAR_GAP));
    const step = peaksData.length / totalBars;

    if (totalBars <= 0 || peaksData.length === 0) {
      return;
    }

    for (let i = 0; i < totalBars; i++) {
      // Calculate peak value for this bar via max-pooling over the source
      // window to avoid low-resolution aliasing/blockiness.
      const windowStart = Math.floor(i * step);
      const windowEnd = Math.max(windowStart + 1, Math.floor((i + 1) * step));
      const end = Math.min(windowEnd, peaksData.length);

      let rawValue = 0;
      for (let idx = windowStart; idx < end; idx++) {
        rawValue = Math.max(rawValue, peaksData[idx] || 0);
      }

      const normalizedValue = rawValue;
      
      // Calculate dimensions
      const visualHeight = Math.max(MIN_BAR_HEIGHT, normalizedValue * h * 0.82);
      
      const x = i * (BAR_WIDTH + BAR_GAP);
      const y = (h - visualHeight) / 2;
      
      // Determine color state
      const barProgress = i / totalBars;
      const isPlayed = barProgress <= displayProg;
      
      if (isPlayed) {
        ctx.fillStyle = WAVEFORM_PLAYED_COLOR;
      } else {
        ctx.fillStyle = WAVEFORM_UNPLAYED_COLOR;
      }
      
      // Draw rounded bar
      ctx.beginPath();
      ctx.roundRect(x, y, BAR_WIDTH, visualHeight, 1);
      ctx.fill();
    }
  }

  function drawSlimLine(
    ctx: CanvasRenderingContext2D,
    displayProg: number,
    w: number,
    h: number
  ): void {
    const centerY = h / 2;
    const lineHeight = 1.5;
    const y = centerY - lineHeight / 2;

    ctx.fillStyle = WAVEFORM_UNPLAYED_COLOR;
    ctx.fillRect(0, y, w, lineHeight);

    const playedWidth = w * displayProg;
    if (playedWidth > 0) {
      ctx.fillStyle = WAVEFORM_PLAYED_COLOR;
      ctx.fillRect(0, y, playedWidth, lineHeight);
    }
  }

  // Draw hover line (shared by both modes)
  function drawHoverLine(
    ctx: CanvasRenderingContext2D,
    hoverProg: number | null,
    w: number,
    h: number
  ): void {
    if (hoverProg !== null) {
      const hoverXPos = hoverProg * w;
      ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
      ctx.fillRect(hoverXPos, 0, 1, h);
    }
  }

  // Resize Observer with debounce to prevent excessive redraws
  onMount(() => {
    if (!container) return;
    
    let resizeTimeout: ReturnType<typeof setTimeout> | undefined;
    const RESIZE_DEBOUNCE_MS = 50;
    
    const observer = new ResizeObserver((entries) => {
      // Debounce resize events to avoid excessive canvas redraws
      if (resizeTimeout) clearTimeout(resizeTimeout);
      resizeTimeout = setTimeout(() => {
        for (const entry of entries) {
          const rect = entry.contentRect;
          width = rect.width;
          height = rect.height;
        }
      }, RESIZE_DEBOUNCE_MS);
    });
    
    observer.observe(container);
    
    return () => {
      if (resizeTimeout) clearTimeout(resizeTimeout);
      observer.disconnect();
    };
  });

  onDestroy(() => {
    cancelAnimationLoop();
    clearLoadingLineTimeout();
  });

  // Drawing Logic
  $effect(() => {
    void renderVersion;
    if (!canvas || !width || !height) return;
    
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Handle high DPI - only resize canvas when dimensions change
    const dpr = window.devicePixelRatio || 1;
    const newCanvasWidth = width * dpr;
    const newCanvasHeight = height * dpr;
    
    if (newCanvasWidth !== lastCanvasWidth || newCanvasHeight !== lastCanvasHeight || dpr !== lastDpr) {
      canvas.width = newCanvasWidth;
      canvas.height = newCanvasHeight;
      ctx.scale(dpr, dpr);
      lastCanvasWidth = newCanvasWidth;
      lastCanvasHeight = newCanvasHeight;
      lastDpr = dpr;
    } else {
      // Reset transform and clear (ctx.scale accumulates)
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    }

    ctx.clearRect(0, 0, width, height);
    ctx.imageSmoothingEnabled = false;

    const peaksForDraw = displayPeaks;
    const shouldDrawWaveform = !!peaksForDraw && hasVisiblePeaks(peaksForDraw);

    if (!shouldDrawWaveform) {
      drawSlimLine(ctx, displayProgress, width, height);
    }

    if (peaksForDraw && shouldDrawWaveform) {
      // Draw waveform based on style
      if (effectiveStyle === 'raw') {
        drawRawWaveform(ctx, peaksForDraw, displayProgress, width, height);
      } else {
        drawPillsWaveform(ctx, peaksForDraw, displayProgress, width, height);
      }
    }

    // Draw Hover Line
    drawHoverLine(ctx, hoverProgress, width, height);
  });

  // Interaction Handlers
  function getProgress(e: MouseEvent | PointerEvent) {
    if (!container) return 0;
    const rect = container.getBoundingClientRect();
    if (rect.width <= 0) return 0;
    const x = e.clientX - rect.left;
    return Math.max(0, Math.min(1, x / rect.width));
  }

  function handlePointerDown(e: PointerEvent) {
    if (durationMs <= 0) return;
    container?.setPointerCapture(e.pointerId);
    isDragging = true;
    dragProgress = getProgress(e);
  }

  function handlePointerMove(e: PointerEvent) {
    const p = getProgress(e);
    
    if (isDragging) {
      dragProgress = p;
    }
    
    // Always update hover state if inside container (or dragging)
    hoverProgress = p;
    hoverX = p * width;
  }

  function handlePointerUp(e: PointerEvent) {
    if (isDragging) {
      const finalProgress = getProgress(e);
      dispatch('seek', { ms: finalProgress * durationMs });
      isDragging = false;
    }
    container?.releasePointerCapture(e.pointerId);
  }

  function handlePointerCancel(e: PointerEvent) {
    isDragging = false;
    container?.releasePointerCapture(e.pointerId);
  }

  function handlePointerLeave() {
    if (!isDragging) {
      hoverProgress = null;
    }
  }
</script>

<div 
  class="waveform-container" 
  bind:this={container}
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
  onpointercancel={handlePointerCancel}
  onpointerleave={handlePointerLeave}
  role="slider"
  aria-label="Seekbar"
  aria-valuemin="0"
  aria-valuemax={durationMs}
  aria-valuenow={displayProgress * durationMs}
  tabindex="0"
>
  <canvas bind:this={canvas}></canvas>
  
  {#if hoverProgress !== null}
    <div 
      class="tooltip"
      style="left: {hoverX}px"
    >
      {displayTime}
    </div>
  {/if}
</div>

<style>
  .waveform-container {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: visible;
    cursor: pointer;
    touch-action: none; /* Prevent scrolling while scrubbing */
    outline: none;
  }

  canvas {
    display: block;
    width: 100%;
    height: 100%;
    pointer-events: none; /* Let container handle events */
  }

  .tooltip {
    position: absolute;
    top: -32px; /* Position above */
    transform: translateX(-50%);
    background: #000;
    color: var(--text-primary);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    pointer-events: none;
    white-space: nowrap;
    border: 1px solid rgba(255, 255, 255, 0.2);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35);
    z-index: 120;
  }
  
  /* Focus styles for accessibility */
  .waveform-container:focus-visible {
    box-shadow: 0 0 0 2px rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.5);
    border-radius: 4px;
  }
</style>
