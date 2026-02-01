<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { bottomBarWaveformStyle } from '../state/effects';
  
  interface Props {
    peaks: Uint8Array | null;
    progress: number; // 0-1
    durationMs: number;
    style?: 'pills' | 'raw';
  }

  let { peaks, progress, durationMs, style }: Props = $props();
  
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
  
  const WAVEFORM_PLAYED_COLOR = 'rgba(255, 255, 255, 0.6)';
  const WAVEFORM_UNPLAYED_COLOR = 'rgba(255, 255, 255, 0.15)';
  const BAR_WIDTH = 3;
  const BAR_GAP = 1;
  const MIN_BAR_HEIGHT = 2;

  // Reset interaction state when peaks change (track changed mid-drag)
  $effect(() => {
    // Track peaks identity change - cancel any ongoing interaction
    void peaks;
    isDragging = false;
    hoverProgress = null;
  });

  function formatTime(ms: number): string {
    if (!ms || ms < 0) return '0:00';
    const mins = Math.floor(ms / 60000);
    const secs = Math.floor((ms % 60000) / 1000);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function drawRawWaveform(
    ctx: CanvasRenderingContext2D,
    peaksData: Uint8Array,
    displayProg: number,
    w: number,
    h: number
  ): void {
    const centerY = h / 2;
    const maxHalfHeight = h * 0.45;
    const peakLen = peaksData.length;

    // Helper to build envelope path
    function buildEnvelopePath(): void {
      ctx.beginPath();
      
      // Top edge: left to right
      for (let x = 0; x <= w; x++) {
        const peakIdx = Math.min(Math.floor((x / w) * peakLen), peakLen - 1);
        const amp = (peaksData[peakIdx] || 0) / 255;
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
        const amp = (peaksData[peakIdx] || 0) / 255;
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
    peaksData: Uint8Array,
    displayProg: number,
    w: number,
    h: number
  ): void {
    const totalBars = Math.floor(w / (BAR_WIDTH + BAR_GAP));
    const step = peaksData.length / totalBars;

    for (let i = 0; i < totalBars; i++) {
      // Calculate peak value for this bar (simple sampling)
      const peakIndex = Math.floor(i * step);
      const rawValue = peaksData[peakIndex] || 0;
      const normalizedValue = rawValue / 255;
      
      // Calculate dimensions
      // Apply a non-linear scaling to make quiet parts more visible but keep peaks distinct
      const visualHeight = Math.max(MIN_BAR_HEIGHT, Math.pow(normalizedValue, 0.8) * h * 0.8);
      
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
      ctx.roundRect(x, y, BAR_WIDTH, visualHeight, 2);
      ctx.fill();
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

  // Drawing Logic
  $effect(() => {
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

    if (!peaks || peaks.length === 0) {
      // Fallback: Simple progress bar
      const centerY = height / 2;
      const trackHeight = 4;
      
      // Track
      ctx.fillStyle = 'rgba(255, 255, 255, 0.1)';
      ctx.beginPath();
      ctx.roundRect(0, centerY - trackHeight/2, width, trackHeight, trackHeight/2);
      ctx.fill();
      
      // Progress
      const progressWidth = width * displayProgress;
      ctx.fillStyle = WAVEFORM_PLAYED_COLOR;
      ctx.beginPath();
      ctx.roundRect(0, centerY - trackHeight/2, progressWidth, trackHeight, trackHeight/2);
      ctx.fill();
      return;
    }

    // Draw waveform based on style
    if (effectiveStyle === 'raw') {
      drawRawWaveform(ctx, peaks, displayProgress, width, height);
    } else {
      drawPillsWaveform(ctx, peaks, displayProgress, width, height);
    }

    // Draw Hover Line
    drawHoverLine(ctx, hoverProgress, width, height);
  });

  // Interaction Handlers
  function getProgress(e: MouseEvent | PointerEvent) {
    if (!container) return 0;
    const rect = container.getBoundingClientRect();
    const x = e.clientX - rect.left;
    return Math.max(0, Math.min(1, x / rect.width));
  }

  function handlePointerDown(e: PointerEvent) {
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
    background: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(4px);
    color: var(--text-primary);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    pointer-events: none;
    white-space: nowrap;
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
    z-index: 10;
  }
  
  /* Focus styles for accessibility */
  .waveform-container:focus-visible {
    box-shadow: 0 0 0 2px rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.5);
    border-radius: 4px;
  }
</style>
