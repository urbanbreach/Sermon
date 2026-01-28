<script lang="ts">
  import { sidebarWidth, setSidebarWidth, railWidth, setRailWidth } from '../state/effects';

  interface Props {
    side: 'left' | 'right';
    minWidth?: number;
    maxWidth?: number;
  }

  let { side, minWidth = 180, maxWidth = 450 }: Props = $props();

  let isDragging = $state(false);
  let startX = 0;
  let startWidth = 0;

  function handleMouseDown(e: MouseEvent) {
    e.preventDefault();
    isDragging = true;
    startX = e.clientX;
    // @ts-ignore - stores are readable with $ prefix in .svelte
    startWidth = side === 'left' ? $sidebarWidth : $railWidth;

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  }

  function handleMouseMove(e: MouseEvent) {
    // For left sidebar: dragging right = increase width
    // For right rail: dragging left = increase width
    const delta = side === 'left' 
      ? e.clientX - startX 
      : startX - e.clientX;

    const newWidth = Math.min(maxWidth, Math.max(minWidth, startWidth + delta));

    // Update CSS variable immediately for smooth feel
    const cssVar = side === 'left' ? '--layout-sidebar-width' : '--layout-rail-width';
    document.documentElement.style.setProperty(cssVar, `${newWidth}px`);
  }

  function handleMouseUp(e: MouseEvent) {
    isDragging = false;
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';

    // Calculate final width from CSS var
    const cssVar = side === 'left' ? '--layout-sidebar-width' : '--layout-rail-width';
    const currentValue = getComputedStyle(document.documentElement).getPropertyValue(cssVar);
    const finalWidth = parseInt(currentValue, 10) || (side === 'left' ? 250 : 320);

    // Persist to backend
    if (side === 'left') {
      setSidebarWidth(finalWidth);
    } else {
      setRailWidth(finalWidth);
    }
  }
</script>

<div 
  class="resize-handle"
  class:dragging={isDragging}
  onmousedown={handleMouseDown}
  role="separator"
  aria-orientation="vertical"
  aria-label={side === 'left' ? 'Resize sidebar' : 'Resize panel'}
></div>

<style>
  .resize-handle {
    width: 9px;
    margin: 0 -4px;
    cursor: col-resize;
    background: transparent;
    position: relative;
    z-index: 100;
    flex-shrink: 0;
    -webkit-app-region: no-drag;
  }

  .resize-handle::after {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    left: 4px;
    width: 1px;
    background: var(--divider-color, rgba(255, 255, 255, 0.07));
    transition: background 0.15s ease, width 0.15s ease;
  }

  .resize-handle:hover::after,
  .resize-handle.dragging::after {
    background: var(--accent-medium, rgba(255, 255, 255, 0.25));
    width: 2px;
    left: 3.5px;
  }

  @media (prefers-reduced-motion: reduce) {
    .resize-handle::after {
      transition: none;
    }
  }
</style>
