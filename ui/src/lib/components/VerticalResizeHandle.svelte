<script lang="ts">
  import { railSplitRatio, setRailSplitRatio } from '../state/effects';

  interface Props {
    containerHeight: number;
    minRatio?: number;
    maxRatio?: number;
  }

  let { containerHeight, minRatio = 0.2, maxRatio = 0.7 }: Props = $props();

  let isDragging = $state(false);
  let startY = 0;
  let startRatio = 0;

  function handleMouseDown(e: MouseEvent) {
    e.preventDefault();
    isDragging = true;
    startY = e.clientY;
    startRatio = $railSplitRatio;

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = 'row-resize';
    document.body.style.userSelect = 'none';
  }

  function handleMouseMove(e: MouseEvent) {
    if (!containerHeight) return;
    
    const deltaY = e.clientY - startY;
    const deltaRatio = deltaY / containerHeight;
    const newRatio = Math.min(maxRatio, Math.max(minRatio, startRatio + deltaRatio));
    
    railSplitRatio.set(newRatio);
  }

  function handleMouseUp() {
    isDragging = false;
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';

    setRailSplitRatio($railSplitRatio);
  }
</script>

<div 
  class="vertical-resize-handle"
  class:dragging={isDragging}
  onmousedown={handleMouseDown}
  role="separator"
  aria-orientation="horizontal"
  aria-label="Resize panels"
></div>

<style>
  .vertical-resize-handle {
    height: 9px;
    margin: -4px 0;
    cursor: row-resize;
    background: transparent;
    position: relative;
    z-index: 100;
    flex-shrink: 0;
    -webkit-app-region: no-drag;
  }

  .vertical-resize-handle::after {
    content: '';
    position: absolute;
    left: 0;
    right: 0;
    top: 4px;
    height: 1px;
    background: var(--divider-color, rgba(255, 255, 255, 0.07));
    transition: background 0.15s ease, height 0.15s ease;
  }

  .vertical-resize-handle:hover::after,
  .vertical-resize-handle.dragging::after {
    background: var(--accent-medium, rgba(255, 255, 255, 0.25));
    height: 2px;
    top: 3.5px;
  }

  @media (prefers-reduced-motion: reduce) {
    .vertical-resize-handle::after {
      transition: none;
    }
  }
</style>
