<script lang="ts">
  import { onDestroy, type Snippet } from 'svelte';
  import { X } from '@lucide/svelte';

  interface Props {
    open: boolean;
    title: string;
    onclose: () => void;
    /** Prevent closing via ESC (e.g., during save operation) */
    preventClose?: boolean;
    /** Enable dragging by header (default: false) */
    draggable?: boolean;
    children: Snippet;
  }

  let { open, title, onclose, preventClose = false, draggable = false, children }: Props = $props();

  let dialogElement = $state<HTMLDivElement | null>(null);
  let previouslyFocused: HTMLElement | null = null;
  let titleId = `modal-title-${Math.random().toString(36).slice(2, 9)}`;

  // Drag state
  let isDragging = $state(false);
  let dragOffset = $state({ x: 0, y: 0 });
  let dialogPosition = $state({ x: 0, y: 0 });

  // Drag handlers
  function handlePointerDown(e: PointerEvent) {
    if (!draggable || !dialogElement) return;
    // Don't start drag if clicking the close button
    if ((e.target as HTMLElement).closest('.close-btn')) return;
    
    isDragging = true;
    const rect = dialogElement.getBoundingClientRect();
    dragOffset = {
      x: e.clientX - rect.left - rect.width / 2,
      y: e.clientY - rect.top - rect.height / 2
    };
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
  }

  function handlePointerMove(e: PointerEvent) {
    if (!isDragging || !dialogElement) return;
    
    const rect = dialogElement.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    
    // Calculate new position (centered)
    let newX = e.clientX - dragOffset.x - viewportWidth / 2;
    let newY = e.clientY - dragOffset.y - viewportHeight / 2;
    
    // Clamp to keep dialog within viewport
    const halfWidth = rect.width / 2;
    const halfHeight = rect.height / 2;
    const maxX = viewportWidth / 2 - halfWidth - 20;
    const maxY = viewportHeight / 2 - halfHeight - 20;
    
    newX = Math.max(-maxX, Math.min(maxX, newX));
    newY = Math.max(-maxY, Math.min(maxY, newY));
    
    dialogPosition = { x: newX, y: newY };
  }

  function handlePointerUp(e: PointerEvent) {
    if (isDragging) {
      isDragging = false;
      (e.target as HTMLElement).releasePointerCapture(e.pointerId);
    }
  }

  // Focus trap: get all focusable elements within dialog
  function getFocusableElements(): HTMLElement[] {
    if (!dialogElement) return [];
    const focusable = dialogElement.querySelectorAll<HTMLElement>(
      'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
    );
    return Array.from(focusable);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!open) return;

    if (e.key === 'Escape' && !preventClose) {
      e.preventDefault();
      onclose();
      return;
    }

    if (e.key === 'Tab') {
      const focusables = getFocusableElements();
      if (focusables.length === 0) return;

      const first = focusables[0];
      const last = focusables[focusables.length - 1];

      if (e.shiftKey) {
        // Shift+Tab: if on first, wrap to last
        if (document.activeElement === first) {
          e.preventDefault();
          last.focus();
        }
      } else {
        // Tab: if on last, wrap to first
        if (document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && !preventClose) {
      onclose();
    }
  }

  // Effect: manage focus when open state changes
  $effect(() => {
    if (open) {
      // Store previously focused element
      previouslyFocused = document.activeElement as HTMLElement;
      
      // Prevent body scroll
      document.body.style.overflow = 'hidden';
      
      // Focus first focusable element after render
      requestAnimationFrame(() => {
        const focusables = getFocusableElements();
        if (focusables.length > 0) {
          focusables[0].focus();
        } else if (dialogElement) {
          dialogElement.focus();
        }
      });
    } else {
      // Restore body scroll
      document.body.style.overflow = '';
      
      // Reset drag position when closing
      dialogPosition = { x: 0, y: 0 };
      
      // Restore focus to previously focused element
      if (previouslyFocused && typeof previouslyFocused.focus === 'function') {
        previouslyFocused.focus();
      }
    }
  });

  onDestroy(() => {
    document.body.style.overflow = '';
  });
</script>

<svelte:window on:keydown={handleKeydown} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={handleBackdropClick}>
    <div
      bind:this={dialogElement}
      class="modal-dialog"
      class:draggable
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      tabindex="-1"
      style:transform={draggable ? `translate(${dialogPosition.x}px, ${dialogPosition.y}px)` : undefined}
    >
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="modal-header"
        onpointerdown={handlePointerDown}
        onpointermove={handlePointerMove}
        onpointerup={handlePointerUp}
      >
        <h2 id={titleId}>{title}</h2>
        {#if !preventClose}
          <button class="close-btn" onclick={onclose} aria-label="Close"><X size={18} /></button>
        {/if}
      </div>
      <div class="modal-body">
        {@render children()}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-dialog {
    background: var(--surface-2, rgba(30, 30, 34, 0.95));
    border: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
    border-radius: var(--radius-md, 12px);
    box-shadow: var(--shadow-3, 0 12px 32px rgba(0, 0, 0, 0.6));
    min-width: 400px;
    max-width: 90vw;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    outline: none;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--glass-border);
    user-select: none;
  }

  .draggable .modal-header {
    cursor: move;
  }

  .draggable .modal-header .close-btn {
    cursor: pointer;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 500;
    color: #fff;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: #888;
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0;
    line-height: 1;
    transition: color 0.2s;
  }

  .close-btn:hover {
    color: #fff;
  }

  .modal-body {
    padding: 1.5rem;
    overflow-y: auto;
  }
</style>
