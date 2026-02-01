<script lang="ts">
  import { audioTelemetry } from '../state/playback';
  import { 
    probeOutputCapabilities, 
    type ProbeCapabilitiesResult, 
    type ProbeCell 
  } from '../api/playback';
  import { onMount, onDestroy } from 'svelte';
  import { setViewTitle } from '../state/viewTitle';
  import Modal from '../components/Modal.svelte';
  import { 
    Activity, 
    CheckCircle, 
    AlertTriangle, 
    XCircle, 
    ChevronRight, 
    ChevronDown, 
    Cpu, 
    Server, 
    ShieldCheck, 
    Zap,
    ArrowRight,
    Play
  } from '@lucide/svelte';
  import type { AudioTelemetryEvent } from '../types/telemetry';

  let telemetry = $derived($audioTelemetry);
  
  // Probe state
  let probeResults = $state<ProbeCapabilitiesResult | null>(null);
  let isProbing = $state(false);

  function isProbeValid(results: ProbeCapabilitiesResult | null, t: AudioTelemetryEvent | null): boolean {
    if (!results || !t || !t.device) return false;
    
    const k = results.key;
    const backend = t.device.backend.kind;
    
    if (k.backend !== backend) return false;
    
    if (backend === 'asio') {
      return k.asioDriver === (t.device.backend.asio?.driver_name ?? null);
    } else {
      return k.deviceId === t.device.device_id;
    }
  }

  let activeProbeResults = $derived(isProbeValid(probeResults, telemetry) ? probeResults : null);

  async function handleProbe() {
    if (!telemetry) return;
    
    const backend = telemetry.device?.backend?.kind || 'wasapi';
    const isAsio = backend === 'asio';
    const isPlaying = telemetry.playback?.state !== 'stopped';
    
    if (isAsio && isPlaying) {
      console.error('Cannot probe ASIO while playing');
      return;
    }
    
    isProbing = true;
    try {
      const result = await probeOutputCapabilities(
        backend as 'wasapi' | 'asio',
        telemetry.device?.device_id || null,
        telemetry.device?.backend?.asio?.driver_name || null,
        2,
        [44100, 48000, 88200, 96000, 176400, 192000, 352800, 384000],
        [16, 24, 32]
      );
      probeResults = result;
    } catch (e) {
      console.error('Probe failed:', e);
    } finally {
      isProbing = false;
    }
  }

  function getProbeCell(sampleRate: number, bitDepth: number): ProbeCell | undefined {
    if (!activeProbeResults?.cells) return undefined;
    return activeProbeResults.cells.find(
      c => c.sampleRate === sampleRate && c.bitDepth === bitDepth
    );
  }

  // Section visibility state
  let showSignalPath = $state(true);
  let showDeviceDetails = $state(false);
  let showStability = $state(false);
  let showCapabilities = $state(false);
  let showDevTools = $state(false);

  // Focus trap harness (DEV only)
  const isDebugMode = import.meta.env.SERMON_DEBUG === '1';
  let harnessModalOpen = $state(false);
  let harnessInput = $state('');
  let harnessCheckbox = $state(false);
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
    setViewTitle('Audio Diagnostics');

    if (isDebugMode) {
      const interval = setInterval(() => {
        if (harnessModalOpen) {
          updateActiveElement();
        }
      }, 100);
      return () => clearInterval(interval);
    }
  });

  onDestroy(() => {
    setViewTitle('');
  });

  // Helpers for display
  function formatSampleRate(hz: number): string {
    if (!hz) return '—';
    return (hz / 1000).toFixed(1) + ' kHz';
  }

  function formatBitDepth(bits: number, validBits?: number): string {
    if (!bits) return '—';
    if (validBits && validBits < bits) {
      return `${validBits}/${bits} bit`;
    }
    return `${bits} bit`;
  }

  function getFormatSummary(t: AudioTelemetryEvent | null) {
    if (!t) return { source: '—', output: 'Waiting for playback...' };
    
    const decode = t.format?.decode;
    const output = t.format?.output;
    const backend = t.device?.backend;
    const isExclusive = t.device?.exclusive_active;

    const sourceStr = decode 
      ? `${decode.codec?.toUpperCase() || 'PCM'} ${formatSampleRate(decode.sample_rate)}/${decode.bit_depth}bit`
      : 'No Source';

    const backendStr = backend?.kind === 'asio' 
      ? 'ASIO' 
      : (isExclusive ? 'WASAPI Exclusive' : 'WASAPI Shared');

    const outputStr = output
      ? `${backendStr} ${formatSampleRate(output.sample_rate)}/${formatBitDepth(output.bit_depth, output.valid_bits)}`
      : 'No Output';

    return { source: sourceStr, output: outputStr };
  }

  let summary = $derived(getFormatSummary(telemetry));

  // Capability probe logic
  let isAsio = $derived(telemetry?.device?.backend?.kind === 'asio');
  let isPlaying = $derived(telemetry?.playback?.state !== 'stopped');
  let canProbe = $derived(!isAsio || !isPlaying);
</script>

<div class="view-container" data-testid="diag-view">
  <div class="content-width">
    
    <!-- Compact Overview Panel -->
    <div class="overview-panel" data-testid="diag-overview">
      <div class="overview-header">
        <div class="badges">
          <!-- Bit Perfect Badge -->
          <div class="integrity-badge" 
               class:success={telemetry?.integrity?.pcm_bit_perfect?.status === 'yes'}
               class:warning={telemetry?.integrity?.pcm_bit_perfect?.status === 'no'}
               class:unknown={!telemetry || telemetry?.integrity?.pcm_bit_perfect?.status === 'unknown'}
          >
            {#if telemetry?.integrity?.pcm_bit_perfect?.status === 'yes'}
              <ShieldCheck size={16} />
              <span>Bit-Perfect</span>
            {:else if telemetry?.integrity?.pcm_bit_perfect?.status === 'no'}
              <AlertTriangle size={16} />
              <span>Resampled</span>
            {:else}
              <Activity size={16} />
              <span>Standby</span>
            {/if}
          </div>

          <!-- DoP Integrity Badge (Only if DSD/DoP active or relevant) -->
          {#if telemetry?.format?.decode?.is_dsd || telemetry?.integrity?.dop_payload_integrity?.status !== 'unknown'}
            <div class="integrity-badge"
                 class:success={telemetry?.integrity?.dop_payload_integrity?.status === 'ok'}
                 class:error={telemetry?.integrity?.dop_payload_integrity?.status === 'degraded'}
            >
              <Zap size={16} />
              <span>DoP Integrity</span>
            </div>
          {/if}
        </div>
        
        <div class="device-info">
          <Server size={14} class="icon-muted" />
          <span class="device-name">{telemetry?.device?.device_name || 'No Device Selected'}</span>
        </div>
      </div>

      <div class="format-flow">
        <div class="flow-node source">
          <span class="label">Source</span>
          <span class="value">{summary.source}</span>
        </div>
        <div class="flow-arrow">
          <ArrowRight size={16} />
        </div>
        <div class="flow-node output">
          <span class="label">Output</span>
          <span class="value">{summary.output}</span>
        </div>
      </div>
    </div>

    <!-- Expandable: Signal Path -->
    <div class="section-container" data-testid="diag-signal-path">
      <button class="section-header" onclick={() => showSignalPath = !showSignalPath}>
        <div class="header-left">
          {#if showSignalPath}
            <ChevronDown size={18} />
          {:else}
            <ChevronRight size={18} />
          {/if}
          <h3>Signal Path</h3>
        </div>
        <div class="header-right">
          <!-- Summary/Status icon could go here -->
        </div>
      </button>
      
      {#if showSignalPath}
        <div class="section-content">
          <div class="signal-path-graph" data-testid="diag-signal-path-graph">
            {#if telemetry?.signal_path_checks}
              <div class="graph-scroll-container">
                {#each telemetry.signal_path_checks as check, i}
                  <div class="graph-node-wrapper">
                    <div 
                      class="graph-node" 
                      class:ok={check.status === 'ok'}
                      class:touching={check.status === 'touching_bits'}
                      class:unknown={check.status === 'unknown'}
                      class:inactive={check.status === 'inactive'}
                    >
                      <div class="node-icon">
                        {#if check.status === 'ok'}
                          <CheckCircle size={18} />
                        {:else if check.status === 'touching_bits'}
                          <AlertTriangle size={18} />
                        {:else if check.status === 'inactive'}
                          <XCircle size={18} />
                        {:else}
                          <Activity size={18} />
                        {/if}
                      </div>
                      <div class="node-content">
                        <span class="node-stage">{check.stage}</span>
                        <span class="node-reason">{check.reason_code || check.status}</span>
                      </div>
                      
                      <div class="node-tooltip">
                        <strong>{check.stage.toUpperCase()}</strong>
                        <div class="tooltip-status"
                             class:text-ok={check.status === 'ok'}
                             class:text-warn={check.status === 'touching_bits'}
                        >
                          Status: {check.status.replace('_', ' ')}
                        </div>
                        <p>{check.detail}</p>
                      </div>
                    </div>
                    
                    {#if i < telemetry.signal_path_checks.length - 1}
                      <div class="graph-connector">
                        <ArrowRight size={16} />
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            {:else}
               <div class="empty-graph">Waiting for signal path data...</div>
            {/if}
          </div>
          
          <div class="signal-checks">
            {#if telemetry?.signal_path_checks}
              {#each telemetry.signal_path_checks as check}
                <div class="check-item">
                  <div class="check-status" class:ok={check.status === 'ok'} class:warn={check.status !== 'ok'}>
                    {#if check.status === 'ok'}
                      <CheckCircle size={14} />
                    {:else}
                      <AlertTriangle size={14} />
                    {/if}
                  </div>
                  <div class="check-info">
                    <span class="check-stage">{check.stage}</span>
                    <span class="check-detail">{check.detail}</span>
                  </div>
                </div>
              {/each}
            {:else}
              <div class="empty-state">No signal path data available</div>
            {/if}
          </div>
        </div>
      {/if}
    </div>

    <!-- Expandable: Device Details -->
    <div class="section-container" data-testid="diag-device-details">
      <button class="section-header" onclick={() => showDeviceDetails = !showDeviceDetails}>
        <div class="header-left">
          {#if showDeviceDetails}
            <ChevronDown size={18} />
          {:else}
            <ChevronRight size={18} />
          {/if}
          <h3>Device & Backend</h3>
        </div>
      </button>

      {#if showDeviceDetails}
        <div class="section-content">
          <div class="detail-grid">
            <div class="detail-row">
              <span class="label">Backend Type</span>
              <span class="value">{telemetry?.device?.backend?.kind?.toUpperCase() || '—'}</span>
            </div>
            
            {#if telemetry?.device?.backend?.wasapi}
               <div class="detail-row">
                 <span class="label">Buffer Size</span>
                 <span class="value">{telemetry.device.backend.wasapi.buffer_frames} frames</span>
               </div>
               <div class="detail-row">
                 <span class="label">Device Period</span>
                 <span class="value">{(telemetry.device.backend.wasapi.device_period_default_hns / 10000).toFixed(2)} ms</span>
               </div>
            {/if}

            {#if telemetry?.device?.backend?.asio}
               <div class="detail-row">
                 <span class="label">Driver</span>
                 <span class="value">{telemetry.device.backend.asio.driver_name}</span>
               </div>
               <div class="detail-row">
                 <span class="label">Buffer Size</span>
                 <span class="value">{telemetry.device.backend.asio.buffer_size_frames} samples</span>
               </div>
               <div class="detail-row">
                 <span class="label">Sample Format</span>
                 <span class="value">{telemetry.device.backend.asio.sample_format}</span>
               </div>
            {/if}
            
            <div class="detail-row">
               <span class="label">Exclusive Mode</span>
               <span class="value highlight">{telemetry?.device?.exclusive_active ? 'Active' : 'Inactive'}</span>
            </div>
          </div>
        </div>
      {/if}
    </div>

    <!-- Expandable: Stability -->
    <div class="section-container" data-testid="diag-stability">
      <button class="section-header" onclick={() => showStability = !showStability}>
        <div class="header-left">
          {#if showStability}
            <ChevronDown size={18} />
          {:else}
            <ChevronRight size={18} />
          {/if}
          <h3>Stability & Telemetry</h3>
        </div>
      </button>

      {#if showStability}
        <div class="section-content">
          <div class="detail-grid">
             <div class="detail-row">
               <span class="label">Ring Buffer Fill</span>
               <div class="bar-container">
                  <div class="bar-fill" style="width: {telemetry?.stability?.ring_buffer?.fill_percent || 0}%"></div>
               </div>
               <span class="value-mini">{telemetry?.stability?.ring_buffer?.fill_percent?.toFixed(1) || 0}%</span>
             </div>
             
             <div class="detail-row">
                <span class="label">Underruns (Track)</span>
                <span class="value" class:warn={(telemetry?.stability?.ring_buffer?.underruns?.track || 0) > 0}>
                  {telemetry?.stability?.ring_buffer?.underruns?.track || 0}
                </span>
             </div>

             <div class="detail-row">
                <span class="label">Overflows (Track)</span>
                <span class="value" class:warn={(telemetry?.stability?.ring_buffer?.overflows?.track || 0) > 0}>
                  {telemetry?.stability?.ring_buffer?.overflows?.track || 0}
                </span>
             </div>
          </div>
        </div>
      {/if}
    </div>

    <!-- Expandable: Capabilities -->
    <div class="section-container" data-testid="diag-capabilities">
      <button class="section-header" onclick={() => showCapabilities = !showCapabilities}>
        <div class="header-left">
          {#if showCapabilities}
            <ChevronDown size={18} />
          {:else}
            <ChevronRight size={18} />
          {/if}
          <h3>Format Capabilities</h3>
        </div>
      </button>

      {#if showCapabilities}
        <div class="section-content">
          {#if !activeProbeResults}
            <div class="capabilities-placeholder" data-testid="diag-capability-matrix">
              <p>Probe device capabilities to populate matrix.</p>
              
              <div class="probe-action">
                <button 
                  class="probe-btn" 
                  disabled={!canProbe || isProbing || !telemetry}
                  onclick={handleProbe}
                  data-testid="diag-probe-btn"
                  title={!canProbe ? "Stop playback to probe ASIO device" : "Check supported formats"}
                >
                  {#if isProbing}
                    <Activity size={16} class="spin" />
                    Probing...
                  {:else}
                    <Play size={16} />
                    Run Probe
                  {/if}
                </button>
                {#if !canProbe}
                  <span class="probe-hint">
                    <AlertTriangle size={14} />
                    ASIO requires exclusive access. Stop playback to probe.
                  </span>
                {/if}
              </div>
            </div>
          {:else}
            <div class="capability-results" data-testid="diag-capability-matrix">
              <div class="matrix-header">
                <div class="matrix-meta">
                  <span class="label">Last Probed</span>
                  <span class="value">{new Date(activeProbeResults.probedAtMs).toLocaleTimeString()}</span>
                </div>
                <button 
                   class="probe-retry-btn" 
                   onclick={handleProbe}
                   disabled={isProbing}
                >
                  Refresh
                </button>
              </div>

              <div class="capability-matrix">
                <!-- Header Row -->
                <div class="matrix-row header">
                  <div class="matrix-cell label">Hz \ Bit</div>
                  {#each [16, 24, 32] as bit}
                    <div class="matrix-cell header">{bit} bit</div>
                  {/each}
                </div>

                <!-- Data Rows -->
                {#each [44100, 48000, 88200, 96000, 176400, 192000, 352800, 384000] as sr}
                  <div class="matrix-row">
                    <div class="matrix-cell label">{(sr / 1000).toFixed(1)}k</div>
                    {#each [16, 24, 32] as bit}
                      {@const cell = getProbeCell(sr, bit)}
                      <div class="matrix-cell data" 
                           class:supported={cell?.supported}
                           class:unsupported={cell && !cell.supported}
                           title={cell?.supported ? 'Supported' : (cell?.reasonCode || 'Unsupported')}
                      >
                        {#if cell?.supported}
                          <div class="dot supported"></div>
                        {:else if cell}
                          <div class="dot unsupported"></div>
                        {:else}
                          <span class="dash">—</span>
                        {/if}
                      </div>
                    {/each}
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Dev Tools (only when SERMON_DEBUG=1) -->
    {#if isDebugMode}
      <div class="section-container dev-tools-section">
        <button class="section-header" onclick={() => showDevTools = !showDevTools} data-testid="dev-tools-header">
           <div class="header-left">
             {#if showDevTools}
               <ChevronDown size={18} />
             {:else}
               <ChevronRight size={18} />
             {/if}
             <Cpu size={18} />
             <h3>Dev Tools</h3>
           </div>
        </button>

        {#if showDevTools}
          <div class="section-content">
            <div class="dev-tools-grid">
              <div class="dev-tool-item">
                <span class="tool-label">Focus Management</span>
                <button 
                  class="harness-btn primary" 
                  onclick={() => harnessModalOpen = true} 
                  data-testid="harness-open-btn"
                >
                  Open Focus Trap Harness
                </button>
              </div>
            </div>
          </div>
        {/if}
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
    padding: 1.5rem;
    padding-bottom: calc(var(--layout-player-height, 80px) + 2rem);
    color: var(--text-primary);
    height: 100%;
    overflow-y: auto;
  }

  .content-width {
    max-width: 800px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  /* Overview Panel */
  .overview-panel {
    background: var(--surface-1);
    border: 1px solid var(--divider-color);
    border-radius: var(--radius-md);
    padding: 1.5rem;
    backdrop-filter: none;
    box-shadow: var(--shadow-2);
  }

  .overview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .badges {
    display: flex;
    gap: 0.75rem;
  }

  .integrity-badge {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.35rem 0.75rem;
    border-radius: var(--radius-pill);
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    font-size: var(--text-meta);
    font-weight: 600;
    color: var(--text-secondary);
  }

  .integrity-badge.success {
    background: rgba(76, 175, 80, 0.15);
    border-color: rgba(76, 175, 80, 0.3);
    color: #4caf50;
  }

  .integrity-badge.warning {
    background: rgba(255, 152, 0, 0.15);
    border-color: rgba(255, 152, 0, 0.3);
    color: #ff9800;
  }
  
  .integrity-badge.error {
    background: rgba(244, 67, 54, 0.15);
    border-color: rgba(244, 67, 54, 0.3);
    color: #f44336;
  }

  .device-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: var(--text-meta);
    color: var(--text-secondary);
  }

  .icon-muted {
    opacity: 0.5;
  }

  .format-flow {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: rgba(0, 0, 0, 0.2);
    border-radius: var(--radius-md);
    padding: 1rem 1.5rem;
    border: 1px solid var(--divider-color);
  }

  .flow-node {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .flow-node.output {
    text-align: right;
    align-items: flex-end;
  }

  .flow-node .label {
    font-size: var(--text-meta);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .flow-node .value {
    font-family: 'Inter Variable', monospace;
    font-weight: 500;
    font-size: var(--text-body);
    color: var(--text-primary);
  }

  .flow-arrow {
    color: var(--text-tertiary);
    opacity: 0.5;
  }

  /* Section Styles */
  .section-container {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--divider-color);
    border-radius: var(--radius-md);
    overflow: hidden;
    transition: background 0.2s;
  }

  .section-container:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .section-header {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.25rem;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    transition: color 0.2s;
    text-align: left;
  }

  .section-header:hover {
    color: var(--text-primary);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .section-header h3 {
    margin: 0;
    font-size: var(--text-body);
    font-weight: 500;
  }

  .section-content {
    padding: 0 1.5rem 1.5rem 1.5rem;
    border-top: 1px solid var(--divider-color);
    margin-top: -1px; /* Align border */
    padding-top: 1.5rem;
  }

  /* Signal Path Specifics */
  .signal-path-graph {
    margin-bottom: 2rem;
    position: relative;
    z-index: 1;
  }

  .graph-scroll-container {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    overflow-x: auto;
    padding: 1rem 0.5rem;
    /* Hide scrollbar but keep functionality */
    scrollbar-width: thin;
    scrollbar-color: var(--divider-color) transparent;
  }

  .graph-scroll-container::-webkit-scrollbar {
    height: 6px;
  }

  .graph-scroll-container::-webkit-scrollbar-track {
    background: transparent;
  }

  .graph-scroll-container::-webkit-scrollbar-thumb {
    background-color: var(--divider-color);
    border-radius: 3px;
  }

  .graph-node-wrapper {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .graph-node {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 110px;
    height: 80px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--divider-color);
    border-radius: var(--radius-md);
    padding: 0.75rem;
    transition: all 0.2s ease;
    cursor: default;
    user-select: none;
  }

  .graph-node:hover {
    transform: translateY(-2px);
    background: rgba(255, 255, 255, 0.07);
    box-shadow: var(--shadow-2);
    z-index: 10;
  }

  .graph-node.ok {
    border-color: rgba(76, 175, 80, 0.3);
    background: linear-gradient(180deg, rgba(76, 175, 80, 0.05) 0%, rgba(76, 175, 80, 0.1) 100%);
  }
  .graph-node.ok .node-icon { color: #4ade80; }

  .graph-node.touching {
    border-color: rgba(251, 146, 60, 0.3);
    background: linear-gradient(180deg, rgba(251, 146, 60, 0.05) 0%, rgba(251, 146, 60, 0.1) 100%);
  }
  .graph-node.touching .node-icon { color: #fb923c; }

  .graph-node.unknown {
    border-color: var(--divider-color);
    opacity: 0.8;
  }
  .graph-node.unknown .node-icon { color: var(--text-tertiary); }

  .graph-node.inactive {
    border-color: rgba(255, 255, 255, 0.05);
    background: rgba(0, 0, 0, 0.2);
    opacity: 0.5;
  }
  .graph-node.inactive .node-icon { color: var(--text-disabled); }

  .node-icon {
    margin-bottom: 0.5rem;
  }

  .node-content {
    text-align: center;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .node-stage {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-secondary);
  }

  .node-reason {
    font-size: 0.65rem;
    color: var(--text-tertiary);
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .graph-connector {
    color: var(--text-tertiary);
    opacity: 0.3;
    display: flex;
    align-items: center;
  }

  .empty-graph {
    text-align: center;
    padding: 2rem;
    color: var(--text-tertiary);
    font-style: italic;
    background: rgba(0, 0, 0, 0.1);
    border-radius: var(--radius-md);
  }

  .node-tooltip {
    position: absolute;
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%) translateY(-10px);
    width: 220px;
    background: #1a1a1a;
    border: 1px solid var(--divider-color);
    padding: 0.75rem;
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-3);
    pointer-events: none;
    opacity: 0;
    visibility: hidden;
    transition: all 0.2s ease;
    z-index: 100;
  }

  .graph-node:hover .node-tooltip {
    opacity: 1;
    visibility: visible;
    transform: translateX(-50%) translateY(-5px);
  }

  .node-tooltip strong {
    display: block;
    color: var(--text-primary);
    font-size: 0.8rem;
    margin-bottom: 0.25rem;
    border-bottom: 1px solid rgba(255,255,255,0.1);
    padding-bottom: 0.25rem;
  }

  .tooltip-status {
    font-size: 0.75rem;
    font-weight: 600;
    margin-bottom: 0.25rem;
    text-transform: capitalize;
    color: var(--text-tertiary);
  }

  .tooltip-status.text-ok { color: #4ade80; }
  .tooltip-status.text-warn { color: #fb923c; }

  .node-tooltip p {
    font-size: 0.75rem;
    color: var(--text-secondary);
    margin: 0;
    line-height: 1.4;
  }

  .node-tooltip::after {
    content: '';
    position: absolute;
    top: 100%;
    left: 50%;
    margin-left: -6px;
    border-width: 6px;
    border-style: solid;
    border-color: #1a1a1a transparent transparent transparent;
  }

  .signal-checks {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .check-item {
    display: flex;
    gap: 1rem;
    align-items: flex-start;
  }

  .check-status {
    margin-top: 2px;
    color: var(--text-tertiary);
  }

  .check-status.ok { color: #4caf50; }
  .check-status.warn { color: #ff9800; }

  .check-info {
    display: flex;
    flex-direction: column;
  }

  .check-stage {
    font-size: var(--text-body);
    font-weight: 500;
  }

  .check-detail {
    font-size: var(--text-meta);
    color: var(--text-secondary);
  }

  /* Details Grid */
  .detail-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 1.5rem;
  }

  .detail-row {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .detail-row .label {
    font-size: var(--text-meta);
    color: var(--text-secondary);
  }

  .detail-row .value {
    font-family: 'Inter Variable', monospace;
    font-size: var(--text-body);
    color: var(--text-primary);
  }

  .detail-row .value.highlight {
    color: var(--theme-accent);
  }

  .detail-row .value.warn {
    color: #ff9800;
  }

  /* Stability Bars */
  .bar-container {
    height: 6px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
    width: 100%;
    margin: 4px 0;
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    background: var(--theme-accent);
    transition: width 0.3s ease-out;
  }

  .value-mini {
    font-size: 11px;
    color: var(--text-tertiary);
    font-family: monospace;
  }

  /* Capabilities */
  .capabilities-placeholder {
    text-align: center;
    padding: 2rem;
    color: var(--text-secondary);
  }

  .probe-action {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    margin-top: 1rem;
  }

  .probe-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background: var(--theme-accent);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #fff;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s;
  }

  .probe-btn:hover:not(:disabled) {
    background: var(--theme-accent-hover);
    transform: translateY(-1px);
  }

  .probe-btn:disabled {
    background: rgba(255, 255, 255, 0.05);
    color: var(--text-disabled);
    cursor: not-allowed;
    border-color: transparent;
  }

  .probe-hint {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.8rem;
    color: #ff9800;
  }

  .spin {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  /* Capability Matrix */
  .capability-results {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .matrix-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .matrix-meta {
    display: flex;
    gap: 0.5rem;
    font-size: var(--text-meta);
  }

  .matrix-meta .label {
    color: var(--text-tertiary);
  }

  .probe-retry-btn {
    background: transparent;
    border: none;
    color: var(--theme-accent);
    font-size: var(--text-meta);
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    border-radius: var(--radius-sm);
  }

  .probe-retry-btn:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .capability-matrix {
    display: grid;
    gap: 2px;
    background: rgba(255, 255, 255, 0.05); /* Grid lines */
    border: 1px solid var(--divider-color);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .matrix-row {
    display: grid;
    grid-template-columns: 80px repeat(3, 1fr);
    background: transparent;
  }

  .matrix-cell {
    background: var(--surface-1); /* Reset bg for cells to create gaps */
    padding: 0.75rem;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.9rem;
  }

  .matrix-cell.label {
    font-size: 0.75rem;
    color: var(--text-secondary);
    background: rgba(0, 0, 0, 0.2);
    font-weight: 600;
  }

  .matrix-cell.header {
    font-size: 0.75rem;
    color: var(--text-secondary);
    background: rgba(0, 0, 0, 0.2);
    font-weight: 600;
  }

  .matrix-cell.data {
    transition: background 0.2s;
  }

  .matrix-cell.data:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
  }

  .dot.supported {
    background: #4ade80;
    box-shadow: 0 0 8px rgba(74, 222, 128, 0.4);
  }

  .dot.unsupported {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid var(--divider-color);
    width: 8px;
    height: 8px;
  }

  .dash {
    color: var(--text-disabled);
  }

  /* Dev Tools Styles */
  .dev-tools-section {
    opacity: 0.9;
    margin-top: 2rem;
    border-color: rgba(68, 170, 255, 0.3);
  }

  .dev-tools-grid {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .dev-tool-item {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .tool-label {
    font-size: var(--text-meta);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
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
    border: 1px solid var(--divider-color);
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
    border-top: 1px solid var(--divider-color);
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
    backdrop-filter: none;
  }

  .harness-btn:hover {
    background: rgba(68, 170, 255, 0.3);
  }

  .harness-btn.secondary {
    background: rgba(255, 255, 255, 0.08);
    border-color: var(--divider-color);
    color: #ccc;
  }

  .harness-btn.secondary:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }
</style>
