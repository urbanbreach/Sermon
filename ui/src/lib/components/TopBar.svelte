<div class="top-bar">
  <div class="search-placeholder">
    Search...
  </div>
  <div class="window-controls">
    <!-- Window controls placeholder -->
    <span>_</span>
    <span>□</span>
    <span>×</span>
  </div>
</div>

<style>
  .top-bar {
    height: 40px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    border-bottom: 1px solid var(--glass-border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1rem;
    -webkit-app-region: drag; /* For Electron later */
    user-select: none;
  }

  .search-placeholder {
    color: #666;
    font-size: 0.9rem;
    background: #222;
    padding: 4px 12px;
    border-radius: 12px;
    width: 200px;
  }

  .window-controls {
    display: flex;
    gap: 1rem;
    color: #888;
    -webkit-app-region: no-drag;
  }
  
  span {
    cursor: pointer;
  }
  
  span:hover {
    color: #fff;
  }
</style>
