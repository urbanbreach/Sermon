<script lang="ts">
  import { audioDebug } from '../state/playback';
  import { onMount } from 'svelte';
  import { getLibraryStats } from '../api/library';
  import type { LibraryStats } from '../types/library';
  import Modal from '../components/Modal.svelte';
  import { Activity, Settings, Cpu, ArrowRight, Database, CheckCircle, AlertTriangle, XCircle } from '@lucide/svelte';

  let debug = $derived($audioDebug);
  let stats: LibraryStats | null = $state(null);

  // Focus trap harness (DEV only)
  const isDebugMode = import.meta.env.SERMON_DEBUG === '1';
  let harnessModalOpen = $state(false);
  let harnessInput = $state('');
  let harnessCheckbox = $state(false);
  let openButtonRef = $state<HTMLButtonElement | null>(null);

  // Track active element for focus debug display
  let activeElementInfo = $state('');
  
  function updateActiveElement() {
    const el = document.activeElement;
    if (el) {
      const tag = el.tagName.toLowerCase();
      const id = el.id ? `#${el.id}` : '';
      const testId = el.getAttribute('data-testid') || '';
      activeElementInfo = `${tag}${id}${testId ? ` [${testId}]` : ''}`;
    } else {
      activeElementInfo = 'none';
    }
  }

  onMount(() => {
    // Load library stats
    getLibraryStats()
      .then((s) => { stats = s; })
      .catch((e) => { console.error('Failed to load library stats:', e); });

    // Update active element info periodically when harness is open
    const interval = setInterval(() => {
      if (harnessModalOpen) {
        updateActiveElement();
      }
    }, 100);

    return () => clearInterval(interval);
  });

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatDate(ms?: number): string {
    if (!ms) return 'Never';
    return new Date(ms).toLocaleString();
  }
</script>

<div class="view-container">
  <h1><Activity size={24} /> Audio Diagnostics</h1>

  <div class="grid">
    <!-- Status Card -->
    <div class="card status-card">
      <h2><CheckCircle size={16} /> Playback Status</h2>
      <div class="status-indicator" class:bit-perfect={debug?.bit_perfect === 'yes'}>
        <div class="dot"></div>
        <span class="label">Bit-Perfect:</span>
        <span class="value">{debug?.bit_perfect === 'yes' ? 'YES' : 'NO'}</span>
      </div>
      {#if debug?.bit_perfect === 'no'}
        <div class="reason">
          <AlertTriangle size={14} />
          Reason: {debug?.bit_perfect_reason || 'Unknown'}
        </div>
      {/if}
    </div>

    <!-- Configuration -->
    <div class="card">
      <h2><Settings size={16} /> Configuration</h2>
      <div class="row">
        <span class="label">Output Mode</span>
        <span class="value">{debug?.output_mode || '—'}</span>
      </div>
      <div class="row">
        <span class="label">Policy</span>
        <span class="value">{debug?.policy || '—'}</span>
      </div>
      <div class="row">
        <span class="label">Exclusive Active</span>
        <span class="value highlight">{debug?.exclusive_active ? 'YES' : 'NO'}</span>
      </div>
    </div>

    <!-- Processing -->
    <div class="card">
      <h2><Cpu size={16} /> Processing</h2>
      <div class="row">
        <span class="label">Conversion</span>
        <span class="value" class:warn={debug?.conversion !== 'none'}>
          {debug?.conversion || 'none'}
        </span>
      </div>
      <div class="row">
        <span class="label">Gain Mode</span>
        <span class="value">{debug?.gain_mode || '—'}</span>
      </div>
      <div class="row">
        <span class="label">Fade Enabled</span>
        <span class="value">{debug?.fade_enabled ? 'Yes' : 'No'}</span>
      </div>
    </div>

    <!-- Formats -->
    <div class="card full-width">
      <h2><Activity size={16} /> Format Pipeline</h2>
      <div class="pipeline">
        <div class="stage">
          <h3>Source</h3>
          <div class="details">
            <div>{debug?.decode_format?.sample_rate || 0} Hz</div>
            <div>{debug?.decode_format?.bit_depth || 0}-bit</div>
            <div>{debug?.decode_format?.channels || 0} ch</div>
            <div class="sub">{debug?.decode_format?.codec || '—'}</div>
          </div>
        </div>

        <div class="arrow"><ArrowRight size={24} /></div>

        <div class="stage">
          <h3>Output</h3>
          <div class="details">
            <div>{debug?.output_format?.sample_rate || 0} Hz</div>
            <div>
              {#if debug?.output_format?.valid_bits && debug?.output_format?.valid_bits !== debug?.output_format?.bit_depth}
                {debug?.output_format?.valid_bits}-bit (in {debug?.output_format?.bit_depth}-bit container)
              {:else}
                {debug?.output_format?.bit_depth || 0}-bit
              {/if}
            </div>
            <div>{debug?.output_format?.channels || 0} ch</div>
            <div class="sub">WASAPI</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Library Stats -->
    <div class="card">
      <h2><Database size={16} /> Library Stats</h2>
      {#if stats}
        <div class="row">
          <span class="label">Tracks</span>
          <span class="value">{stats.trackCount.toLocaleString()}</span>
        </div>
        <div class="row">
          <span class="label">Albums</span>
          <span class="value">{stats.albumCount.toLocaleString()}</span>
        </div>
        <div class="row">
          <span class="label">Artists</span>
          <span class="value">{stats.artistCount.toLocaleString()}</span>
        </div>
        <div class="row">
          <span class="label">Database Size</span>
          <span class="value">{formatBytes(stats.dbSizeBytes)}</span>
        </div>
        <div class="row">
          <span class="label">Last Scan</span>
          <span class="value">{formatDate(stats.lastScanCompletedMs)}</span>
        </div>
      {:else}
        <div class="loading">Loading...</div>
      {/if}
    </div>

    <!-- Focus Trap Harness (DEV only) -->
    {#if isDebugMode}
      <div class="card full-width">
        <h2>Focus Trap Harness (DEV)</h2>
        <p class="harness-note">Test modal focus trap behavior. Run with <code>SERMON_DEBUG=1 cargo tauri dev</code></p>
        
        <button 
          bind:this={openButtonRef}
          class="harness-btn"
          onclick={() => harnessModalOpen = true}
          data-testid="harness-open-btn"
        >
          Open Test Modal
        </button>

        <div class="checklist">
          <strong>Manual Checklist:</strong>
          <ol>
            <li>Open modal → focus moves into modal (first focusable)</li>
            <li>Tab cycles within modal; does not escape</li>
            <li>Shift+Tab cycles backwards within modal</li>
            <li>ESC closes modal</li>
            <li>Focus returns to the opening button</li>
            <li>Background elements are not clickable while modal open</li>
          </ol>
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- Focus Trap Test Modal (DEV only) -->
{#if isDebugMode}
  <Modal open={harnessModalOpen} title="Focus Trap Test" onclose={() => harnessModalOpen = false}>
    <div class="harness-modal-content">
      <div class="focus-debug">
        <strong>Active Element:</strong> <code>{activeElementInfo}</code>
      </div>

      <div class="harness-field">
        <label for="harness-input">Test Input</label>
        <input 
          id="harness-input" 
          type="text" 
          bind:value={harnessInput}
          placeholder="Type something..."
          data-testid="harness-input"
        />
      </div>

      <div class="harness-field">
        <label class="checkbox-label">
          <input 
            type="checkbox" 
            bind:checked={harnessCheckbox}
            data-testid="harness-checkbox"
          />
          Test Checkbox
        </label>
      </div>

      <div class="harness-actions">
        <button 
          class="harness-btn secondary"
          onclick={() => harnessModalOpen = false}
          data-testid="harness-cancel-btn"
        >
          Cancel
        </button>
        <button 
          class="harness-btn primary"
          onclick={() => harnessModalOpen = false}
          data-testid="harness-apply-btn"
        >
          Apply
        </button>
      </div>
    </div>
  </Modal>
{/if}

<style>
  .view-container {
    padding: 2rem;
    color: #fff;
    height: 100%;
    overflow-y: auto;
    background: transparent;
  }

  h1 {
    margin-bottom: 2rem;
    font-size: var(--text-view-title, 22px);
    font-weight: 600;
    text-shadow: 0 2px 4px rgba(0,0,0,0.5);
    display: flex;
    align-items: center;
    gap: var(--space-3, 12px);
  }

  h2 {
    font-size: var(--text-body, 14px);
    font-weight: 500;
    color: rgba(255, 255, 255, 0.7);
    margin-bottom: var(--space-4, 16px);
    border-bottom: 1px solid var(--glass-border, rgba(255, 255, 255, 0.08));
    padding-bottom: var(--space-2, 8px);
    display: flex;
    align-items: center;
    gap: var(--space-2, 8px);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1.5rem;
  }

  .card {
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    border-radius: 8px;
    padding: 1.5rem;
    box-shadow: var(--glass-shadow);
  }

  .full-width {
    grid-column: 1 / -1;
  }

  .row {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.75rem;
    font-size: 0.95rem;
  }

  .label {
    color: #aaa;
  }

  .value {
    font-family: monospace;
    font-weight: bold;
  }

  .value.highlight {
    color: #4af;
  }

  .value.warn {
    color: #fa4;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }

  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #444;
  }

  .status-indicator.bit-perfect .dot {
    background: #4f4;
    box-shadow: 0 0 10px rgba(68, 255, 68, 0.4);
  }

  .status-indicator:not(.bit-perfect) .dot {
    background: #fa4;
  }

  .reason {
    font-size: var(--text-meta, 12px);
    color: #fa4;
    margin-top: var(--space-2, 8px);
    padding: var(--space-2, 8px) var(--space-3, 12px);
    background: rgba(255, 170, 68, 0.1);
    border-radius: var(--radius-sm, 8px);
    display: flex;
    align-items: center;
    gap: var(--space-2, 8px);
  }

  .pipeline {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-6, 24px);
    padding: var(--space-4, 16px) 0;
  }

  .stage {
    text-align: center;
    background: rgba(255, 255, 255, 0.05);
    padding: var(--space-4, 16px) var(--space-6, 24px);
    border-radius: var(--radius-md, 12px);
    min-width: 120px;
    border: 1px solid var(--glass-border, rgba(255, 255, 255, 0.08));
  }

  .stage h3 {
    font-size: var(--text-meta, 12px);
    color: rgba(255, 255, 255, 0.5);
    margin-bottom: var(--space-2, 8px);
    text-transform: uppercase;
    letter-spacing: 1px;
    font-weight: 500;
  }

  .details {
    font-family: 'Inter Variable', monospace;
    font-size: var(--text-body, 14px);
    line-height: 1.5;
    font-variant-numeric: tabular-nums;
  }

  .sub {
    font-size: var(--text-meta, 12px);
    color: rgba(255, 255, 255, 0.4);
    margin-top: var(--space-1, 4px);
  }

  .arrow {
    color: rgba(255, 255, 255, 0.3);
  }

  /* Focus Trap Harness Styles */
  .harness-note {
    font-size: 0.85rem;
    color: #888;
    margin-bottom: 1rem;
  }

  .harness-note code {
    background: rgba(255, 255, 255, 0.1);
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    font-size: 0.8rem;
  }

  .harness-btn {
    background: rgba(68, 170, 255, 0.2);
    border: 1px solid rgba(68, 170, 255, 0.4);
    color: #4af;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.2s;
    backdrop-filter: blur(var(--glass-blur));
  }

  .harness-btn:hover {
    background: rgba(68, 170, 255, 0.3);
  }

  .harness-btn.secondary {
    background: rgba(255, 255, 255, 0.08);
    border-color: var(--glass-border);
    color: #ccc;
  }

  .harness-btn.secondary:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }

  .harness-btn.primary {
    background: rgba(68, 170, 255, 0.2);
    border-color: rgba(68, 170, 255, 0.4);
    color: #4af;
  }

  .checklist {
    margin-top: 1.5rem;
    padding: 1rem;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 6px;
    font-size: 0.9rem;
  }

  .checklist ol {
    margin: 0.5rem 0 0 1.5rem;
    padding: 0;
    color: #aaa;
  }

  .checklist li {
    margin-bottom: 0.25rem;
  }

  .harness-modal-content {
    min-width: 350px;
  }

  .focus-debug {
    background: rgba(68, 170, 255, 0.1);
    border: 1px solid rgba(68, 170, 255, 0.3);
    border-radius: 6px;
    padding: 0.75rem;
    margin-bottom: 1.5rem;
    font-size: 0.9rem;
  }

  .focus-debug code {
    font-family: monospace;
    color: #4af;
  }

  .harness-field {
    margin-bottom: 1rem;
  }

  .harness-field label {
    display: block;
    font-size: 0.85rem;
    color: #888;
    margin-bottom: 0.25rem;
  }

  .harness-field input[type="text"] {
    width: 100%;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--glass-border);
    border-radius: 6px;
    color: #fff;
    padding: 0.5rem 0.75rem;
    font-size: 0.95rem;
    outline: none;
  }

  .harness-field input[type="text"]:focus {
    border-color: rgba(68, 170, 255, 0.5);
  }

  .harness-field .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    color: #ccc;
  }

  .harness-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--glass-border);
  }
</style>
