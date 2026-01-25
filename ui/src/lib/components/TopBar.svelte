<script lang="ts">
  import { canGoBack, canGoForward, goBack, goForward } from '../state/route';
  import { toggleRail, isRailOpen } from '../state/rightRail';
  import { viewTitle } from '../state/viewTitle';
  import { pressScale } from '../utils/animations';
  import { ChevronLeft, ChevronRight, PanelRight } from '@lucide/svelte';
</script>

<div class="top-bar">
  <div class="nav-buttons">
    <button 
      class="nav-btn" 
      onclick={goBack} 
      disabled={!$canGoBack} 
      title="Go Back" 
      use:pressScale
    >
      <ChevronLeft size={20} strokeWidth={2} />
    </button>
    <button 
      class="nav-btn" 
      onclick={goForward} 
      disabled={!$canGoForward} 
      title="Go Forward" 
      use:pressScale
    >
      <ChevronRight size={20} strokeWidth={2} />
    </button>
  </div>

  {#if $viewTitle}
    <h1 class="view-title">{$viewTitle}</h1>
  {/if}

  <button 
    class="nav-btn rail-toggle" 
    class:active={$isRailOpen}
    onclick={toggleRail} 
    title="Toggle Queue" 
    use:pressScale
  >
    <PanelRight size={20} strokeWidth={1.5} />
  </button>
</div>

<style>
  .top-bar {
    height: var(--layout-header-height);
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border-bottom: 1px solid var(--glass-border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    -webkit-app-region: drag;
    user-select: none;
    position: relative;
    z-index: 100;
  }



  .nav-buttons {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    -webkit-app-region: no-drag;
  }

  .view-title {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
    white-space: nowrap;
    pointer-events: none;
  }

  .nav-btn {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    background: var(--surface-1);
    border: 1px solid var(--glass-border);
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--motion-fast) var(--ease-out);
    padding: 0;
    -webkit-app-region: no-drag;
    box-shadow: var(--shadow-1);
  }

  .nav-btn:hover:not(:disabled) {
    background: var(--surface-hover);
    border-color: rgba(255, 255, 255, 0.12);
    box-shadow: var(--shadow-2);
    color: var(--text-primary);
  }

  .nav-btn:active:not(:disabled) {
    background: var(--surface-active);
    box-shadow: var(--shadow-inset);
    border-color: rgba(255, 255, 255, 0.08);
    transform: scale(0.97);
  }

  .nav-btn.active {
    background: var(--accent-weak);
    color: var(--theme-accent);
    border-color: var(--accent-medium);
    box-shadow: var(--shadow-glow);
  }

  .nav-btn:disabled {
    color: var(--text-disabled);
    cursor: default;
    background: rgba(255, 255, 255, 0.02);
    border-color: transparent;
    opacity: 0.6;
    box-shadow: none;
  }
</style>
