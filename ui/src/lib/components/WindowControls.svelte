<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { Minus, Square, X } from '@lucide/svelte';

  const appWindow = getCurrentWindow();

  async function minimize() {
    await appWindow.minimize();
  }

  async function toggleMaximize() {
    await appWindow.toggleMaximize();
  }

  async function close() {
    await appWindow.close();
  }
</script>

<div class="window-controls">
  <button onclick={minimize} aria-label="Minimize" class="control-btn">
    <Minus size={12} />
  </button>
  <button onclick={toggleMaximize} aria-label="Maximize" class="control-btn">
    <Square size={10} />
  </button>
  <button onclick={close} aria-label="Close" class="control-btn close">
    <X size={14} />
  </button>
</div>

<style>
  .window-controls {
    display: flex;
    gap: 0;
    height: 100%;
    -webkit-app-region: no-drag;
  }
  .control-btn {
    width: 46px;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    transition: background-color var(--motion-fast) var(--ease-out);
  }
  .control-btn:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }
  .control-btn.close:hover {
    background: #c42b1c;
    color: white;
  }
</style>
