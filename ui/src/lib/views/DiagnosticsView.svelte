<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import {
    Activity,
    AlertTriangle,
    ArrowRight,
    CheckCircle,
    ChevronDown,
    ChevronRight,
    Cpu,
    Server,
    ShieldCheck,
    XCircle,
  } from '@lucide/svelte';
  import Modal from '../components/Modal.svelte';
  import { setViewTitle } from '../state/viewTitle';
  import { audioTelemetry } from '../state/playback';
  import type { AudioTelemetryEvent, TelemetrySignalPathCheck } from '../types/telemetry';

  type HealthTone = 'healthy' | 'warning' | 'idle';

  const isDebugMode = import.meta.env.SERMON_DEBUG === '1';

  let telemetry = $derived($audioTelemetry);
  let nowMs = $state(Date.now());

  // Signal-path selection
  let manualSelectedStage = $state<string | null>(null);
  let signalChecks = $derived(telemetry?.signal_path_checks ?? []);
  let selectedSignalStage = $derived(
    signalChecks.length === 0
      ? null
      : manualSelectedStage && signalChecks.some((check) => check.stage === manualSelectedStage)
        ? manualSelectedStage
        : signalChecks[0].stage,
  );
  let selectedSignalCheck = $derived(
    selectedSignalStage
      ? signalChecks.find((check) => check.stage === selectedSignalStage) ?? null
      : null,
  );

  // Debug harness (DEV only)
  let showDevTools = $state(false);
  let harnessModalOpen = $state(false);
  let harnessInput = $state('');
  let harnessCheckbox = $state(false);
  let activeElementInfo = $state('');

  let clockInterval: ReturnType<typeof setInterval> | null = null;
  let debugInterval: ReturnType<typeof setInterval> | null = null;

  let healthTone = $derived(computeHealthTone(telemetry));
  let healthHeadline = $derived(getHealthHeadline(healthTone));
  let healthDetail = $derived(getHealthDetail(telemetry, healthTone));

  let modeLabel = $derived(getModeLabel(telemetry));
  let backendLabel = $derived(telemetry?.device?.backend?.kind?.toUpperCase() || '—');
  let sourceSummary = $derived(getSourceSummary(telemetry));
  let outputSummary = $derived(getOutputSummary(telemetry));
  let resamplerSummary = $derived(getResamplerSummary(telemetry));
  let telemetryAgeLabel = $derived(getTelemetryAgeLabel(telemetry, nowMs));

  let ringFillPercent = $derived(
    Math.max(0, Math.min(100, telemetry?.stability?.ring_buffer?.fill_percent ?? 0)),
  );
  let dopFillPercent = $derived(
    Math.max(0, Math.min(100, telemetry?.stability?.dop_ring_buffer?.fill_percent ?? 0)),
  );

  let ringUnderrunsTrack = $derived(telemetry?.stability?.ring_buffer?.underruns?.track ?? 0);
  let ringOverflowsTrack = $derived(telemetry?.stability?.ring_buffer?.overflows?.track ?? 0);
  let signalIssueCount = $derived(
    signalChecks.filter((check) => check.status !== 'ok' && check.status !== 'inactive').length,
  );
  let recentEvents = $derived((telemetry?.stability?.recent_events ?? []).slice(0, 6));

  onMount(() => {
    setViewTitle('Audio Diagnostics');

    clockInterval = setInterval(() => {
      nowMs = Date.now();
    }, 1000);

    if (isDebugMode) {
      debugInterval = setInterval(() => {
        if (harnessModalOpen) {
          updateActiveElement();
        }
      }, 120);
    }
  });

  onDestroy(() => {
    setViewTitle('');

    if (clockInterval) {
      clearInterval(clockInterval);
      clockInterval = null;
    }

    if (debugInterval) {
      clearInterval(debugInterval);
      debugInterval = null;
    }
  });

  function updateActiveElement() {
    const active = document.activeElement;

    if (!active) {
      activeElementInfo = 'none';
      return;
    }

    const tag = active.tagName.toLowerCase();
    const id = active.id ? `#${active.id}` : '';
    const testId = active.getAttribute('data-testid');
    activeElementInfo = testId ? `${tag}${id} [${testId}]` : `${tag}${id}`;
  }

  function computeHealthTone(t: AudioTelemetryEvent | null): HealthTone {
    if (!t) {
      return 'idle';
    }

    if (
      t.integrity.pcm_bit_perfect.status === 'no' ||
      t.stability.ring_buffer.underruns.track > 0 ||
      t.stability.ring_buffer.overflows.track > 0 ||
      t.signal_path_checks.some((check) => check.status === 'touching_bits')
    ) {
      return 'warning';
    }

    return 'healthy';
  }

  function getHealthHeadline(tone: HealthTone): string {
    if (tone === 'healthy') {
      return 'Signal path healthy';
    }

    if (tone === 'warning') {
      return 'Attention recommended';
    }

    return 'Waiting for telemetry';
  }

  function getHealthDetail(t: AudioTelemetryEvent | null, tone: HealthTone): string {
    if (!t) {
      return 'Start playback to populate diagnostics.';
    }

    if (tone === 'healthy') {
      return 'No active integrity or stability warnings.';
    }

    const issues: string[] = [];

    if (t.integrity.pcm_bit_perfect.status === 'no') {
      issues.push('bit-perfect path is not preserved');
    }

    if (t.stability.ring_buffer.underruns.track > 0) {
      issues.push(`underruns (${t.stability.ring_buffer.underruns.track})`);
    }

    if (t.stability.ring_buffer.overflows.track > 0) {
      issues.push(`overflows (${t.stability.ring_buffer.overflows.track})`);
    }

    const touchingChecks = t.signal_path_checks.filter((check) => check.status === 'touching_bits').length;
    if (touchingChecks > 0) {
      issues.push(`signal-path touchpoints (${touchingChecks})`);
    }

    return `Detected ${issues.join(', ')}.`;
  }

  function formatSampleRate(hz: number): string {
    if (!hz) {
      return '—';
    }

    return `${(hz / 1000).toFixed(1)} kHz`;
  }

  function formatBitDepth(bits: number, validBits: number): string {
    if (!bits) {
      return '—';
    }

    if (validBits > 0 && validBits < bits) {
      return `${validBits}/${bits} bit`;
    }

    return `${bits} bit`;
  }

  function getModeLabel(t: AudioTelemetryEvent | null): string {
    if (!t) {
      return 'Standby';
    }

    if (t.playback.output_mode === 'exclusive') {
      return 'WASAPI Exclusive';
    }

    if (t.playback.output_mode === 'shared') {
      return 'WASAPI Shared';
    }

    return 'ASIO';
  }

  function getSourceSummary(t: AudioTelemetryEvent | null): string {
    if (!t) {
      return 'No source active';
    }

    const decode = t.format.decode;
    const codec = decode.codec ? decode.codec.toUpperCase() : 'PCM';
    return `${codec} · ${formatSampleRate(decode.sample_rate)} · ${decode.bit_depth} bit · ${decode.channels}ch`;
  }

  function getOutputSummary(t: AudioTelemetryEvent | null): string {
    if (!t) {
      return 'No output active';
    }

    const output = t.format.output;
    return `${formatSampleRate(output.sample_rate)} · ${formatBitDepth(output.bit_depth, output.valid_bits)} · ${output.channels}ch`;
  }

  function getResamplerSummary(t: AudioTelemetryEvent | null): string {
    if (!t) {
      return 'Inactive';
    }

    if (!t.format.resampler.active) {
      return 'Bypassed';
    }

    return `${formatSampleRate(t.format.resampler.source_sample_rate)} → ${formatSampleRate(t.format.resampler.output_sample_rate)}`;
  }

  function getTelemetryAgeLabel(t: AudioTelemetryEvent | null, now: number): string {
    if (!t) {
      return 'No telemetry frame';
    }

    const ageMs = Math.max(0, now - t.timestamp_ms);

    if (ageMs < 1500) {
      return 'Live now';
    }

    const ageSec = Math.floor(ageMs / 1000);
    return `${ageSec}s ago`;
  }

  function getSignalStatusLabel(status: TelemetrySignalPathCheck['status']): string {
    if (status === 'ok') {
      return 'ok';
    }

    if (status === 'touching_bits') {
      return 'touching bits';
    }

    if (status === 'inactive') {
      return 'inactive';
    }

    return 'unknown';
  }

  function getEventAgeLabel(eventTimestampMs: number): string {
    const ageMs = Math.max(0, nowMs - eventTimestampMs);

    if (ageMs < 1000) {
      return 'just now';
    }

    if (ageMs < 60000) {
      return `${Math.floor(ageMs / 1000)}s ago`;
    }

    return `${Math.floor(ageMs / 60000)}m ago`;
  }
</script>

<div class="view-container" data-testid="diag-view">
  <div class="content-width">
    <header class="hero" data-testid="diag-overview">
      <div class="hero-status">
        <div
          class="health-badge"
          class:healthy={healthTone === 'healthy'}
          class:warning={healthTone === 'warning'}
          class:idle={healthTone === 'idle'}
        >
          {#if healthTone === 'healthy'}
            <ShieldCheck size={16} />
          {:else if healthTone === 'warning'}
            <AlertTriangle size={16} />
          {:else}
            <Activity size={16} />
          {/if}
        </div>

        <div class="hero-copy">
          <h2>{healthHeadline}</h2>
          <p>{healthDetail}</p>
        </div>
      </div>

      <div class="hero-device">
        <div class="device-line">
          <Server size={12} />
          <span>{telemetry?.device?.device_name || 'No output device selected'}</span>
        </div>
        <div class="device-sub">
          <span class="mode-pill">{modeLabel}</span>
          <span class="age-label">{telemetryAgeLabel}</span>
        </div>
      </div>
    </header>
    <div class="flow-bar">
      <div class="flow-endpoint">
        <span class="flow-label">SOURCE</span>
        <span class="flow-value">{sourceSummary}</span>
      </div>
      <span class="flow-arrow"><ArrowRight size={14} /></span>
      <div class="flow-endpoint flow-output">
        <span class="flow-label">OUTPUT</span>
        <span class="flow-value">{outputSummary}</span>
      </div>
    </div>
    <section class="section" data-testid="diag-signal-path">
      <div class="section-head">
        <span class="section-title">Signal Path</span>
        <span class="section-meta">{signalChecks.length} stages · Resampler: {resamplerSummary}</span>
      </div>

      {#if signalChecks.length === 0}
        <div class="empty-state">Waiting for signal-path data&hellip;</div>
      {:else}
        <div class="pipeline" data-testid="diag-signal-path-graph">
          {#each signalChecks as check, index (check.stage)}
            <button
              class="stage-node"
              class:selected={check.stage === selectedSignalStage}
              class:ok={check.status === 'ok'}
              class:warn={check.status === 'touching_bits'}
              class:inactive={check.status === 'inactive'}
              class:unknown={check.status === 'unknown'}
              onclick={() => (manualSelectedStage = check.stage)}
            >
              <span class="stage-icon">
                {#if check.status === 'ok'}
                  <CheckCircle size={12} />
                {:else if check.status === 'touching_bits'}
                  <AlertTriangle size={12} />
                {:else if check.status === 'inactive'}
                  <XCircle size={12} />
                {:else}
                  <Activity size={12} />
                {/if}
              </span>
              <span class="stage-name">{check.stage}</span>
            </button>

            {#if index < signalChecks.length - 1}
              <span class="pipe-connector"><ArrowRight size={11} /></span>
            {/if}
          {/each}
        </div>

        {#if selectedSignalCheck}
          <div class="stage-detail">
            <div class="detail-head">
              <span class="detail-stage">{selectedSignalCheck.stage}</span>
              <span class="detail-reason">
                {selectedSignalCheck.reason_code || getSignalStatusLabel(selectedSignalCheck.status)}
              </span>
            </div>
            <p class="detail-body">{selectedSignalCheck.detail || 'No additional detail provided.'}</p>
          </div>
        {/if}
      {/if}
    </section>

    <div class="main-grid">

      <section class="section" data-testid="diag-stability">
        <div class="section-head">
          <span class="section-title">Health & Stability</span>
          <span class="section-meta" class:meta-warn={signalIssueCount > 0}>{signalIssueCount} issues</span>
        </div>

        <div class="health-strip">
          <div class="hs-metric">
            <span class="hs-label">Bit-perfect</span>
            <span class="hs-value">{telemetry?.integrity?.pcm_bit_perfect?.status || 'unknown'}</span>
          </div>
          <div class="hs-metric">
            <span class="hs-label">Anomalies</span>
            <span class="hs-value" class:val-warn={signalIssueCount > 0}>{signalIssueCount}</span>
          </div>
          <div class="hs-metric">
            <span class="hs-label">Underruns</span>
            <span class="hs-value" class:val-warn={ringUnderrunsTrack > 0}>{ringUnderrunsTrack}</span>
          </div>
          <div class="hs-metric">
            <span class="hs-label">Overflows</span>
            <span class="hs-value" class:val-warn={ringOverflowsTrack > 0}>{ringOverflowsTrack}</span>
          </div>
        </div>

        <div class="buffer-section">
          <div class="buf-row">
            <div class="buf-head">
              <span class="buf-name">PCM ring buffer</span>
              <span class="buf-pct">{ringFillPercent.toFixed(0)}%</span>
            </div>
            <div class="buf-track"><div class="buf-fill" style={`width: ${ringFillPercent}%`}></div></div>
          </div>

          <div class="buf-row">
            <div class="buf-head">
              <span class="buf-name">DoP ring buffer</span>
              <span class="buf-pct">{dopFillPercent.toFixed(0)}%</span>
            </div>
            <div class="buf-track"><div class="buf-fill" style={`width: ${dopFillPercent}%`}></div></div>
          </div>
        </div>

        <div class="counter-row">
          <span class:val-warn={(telemetry?.stability?.ring_buffer?.underruns?.track ?? 0) > 0}>
            PCM underruns {telemetry?.stability?.ring_buffer?.underruns?.track ?? 0}
          </span>
          <span class:val-warn={(telemetry?.stability?.ring_buffer?.overflows?.track ?? 0) > 0}>
            PCM overflows {telemetry?.stability?.ring_buffer?.overflows?.track ?? 0}
          </span>
          <span class:val-warn={(telemetry?.stability?.dop_ring_buffer?.underruns?.track ?? 0) > 0}>
            DoP underruns {telemetry?.stability?.dop_ring_buffer?.underruns?.track ?? 0}
          </span>
          <span class:val-warn={(telemetry?.stability?.dop_ring_buffer?.overflows?.track ?? 0) > 0}>
            DoP overflows {telemetry?.stability?.dop_ring_buffer?.overflows?.track ?? 0}
          </span>
          <span class:val-warn={(telemetry?.stability?.asio?.callback_underruns?.track ?? 0) > 0}>
            ASIO cb {telemetry?.stability?.asio?.callback_underruns?.track ?? 0}
          </span>
          <span class:val-warn={(telemetry?.stability?.asio?.dop_drops?.track ?? 0) > 0}>
            ASIO DoP drops {telemetry?.stability?.asio?.dop_drops?.track ?? 0}
          </span>
        </div>

        {#if recentEvents.length > 0}
          <div class="events-section" data-testid="diag-events">
            <div class="events-head">
              <span class="section-title">Recent Events</span>
              <span class="section-meta">{recentEvents.length}</span>
            </div>
            {#each recentEvents as event (`${event.ts_ms}-${event.code}`)}
              <div class="event-row">
                <span class="event-code">{event.code}</span>
                <span class="event-detail">{event.detail || 'No detail provided.'}</span>
                <span class="event-age">{getEventAgeLabel(event.ts_ms)}</span>
              </div>
            {/each}
          </div>
        {:else}
          <div class="empty-subtle">No recent engine events.</div>
        {/if}
      </section>

      <section class="section" data-testid="diag-device-details">
        <div class="section-head">
          <span class="section-title">Engine Snapshot</span>
        </div>

        <div class="engine-list">
          <div class="eng-row">
            <span class="eng-label">Device</span>
            <span class="eng-value">{telemetry?.device?.device_name || '—'}</span>
          </div>
          <div class="eng-row">
            <span class="eng-label">Backend</span>
            <span class="eng-value">{backendLabel}</span>
          </div>
          <div class="eng-row">
            <span class="eng-label">Mode</span>
            <span class="eng-value">{modeLabel}</span>
          </div>
          <div class="eng-row">
            <span class="eng-label">Policy</span>
            <span class="eng-value">{telemetry?.playback?.policy || '—'}</span>
          </div>
          <div class="eng-row">
            <span class="eng-label">Timing</span>
            <span class="eng-value">{telemetry?.playback?.timing_mode || '—'}</span>
          </div>
          <div class="eng-row">
            <span class="eng-label">Gain</span>
            <span class="eng-value">{telemetry?.playback?.gain_mode || '—'}</span>
          </div>
          <div class="eng-row">
            <span class="eng-label">Conversion</span>
            <span class="eng-value">{telemetry?.playback?.conversion || '—'}</span>
          </div>
          <div class="eng-row">
            <span class="eng-label">Volume mode</span>
            <span class="eng-value">{telemetry?.playback?.effective_volume_mode || '—'}</span>
          </div>
          <div class="eng-row">
            <span class="eng-label">Fade</span>
            <span class="eng-value">
              {telemetry?.playback?.fade_enabled ? 'Enabled' : 'Off'}{telemetry?.playback?.fade_active ? ' (active)' : ''}
            </span>
          </div>
          <div class="eng-row">
            <span class="eng-label">Playback state</span>
            <span class="eng-value">{telemetry?.playback?.state || '—'}</span>
          </div>

          {#if telemetry?.device?.backend?.wasapi}
            <div class="eng-row">
              <span class="eng-label">WASAPI buffer</span>
              <span class="eng-value">{telemetry.device.backend.wasapi.buffer_frames} frames</span>
            </div>
            <div class="eng-row">
              <span class="eng-label">Default period</span>
              <span class="eng-value">{(telemetry.device.backend.wasapi.device_period_default_hns / 10000).toFixed(2)} ms</span>
            </div>
          {/if}

          {#if telemetry?.device?.backend?.asio}
            <div class="eng-row">
              <span class="eng-label">ASIO driver</span>
              <span class="eng-value">{telemetry.device.backend.asio.driver_name}</span>
            </div>
            <div class="eng-row">
              <span class="eng-label">ASIO buffer</span>
              <span class="eng-value">{telemetry.device.backend.asio.buffer_size_frames} samples</span>
            </div>
            <div class="eng-row">
              <span class="eng-label">ASIO sample fmt</span>
              <span class="eng-value">{telemetry.device.backend.asio.sample_format}</span>
            </div>
          {/if}
        </div>
      </section>
    </div>

    {#if isDebugMode}
      <section class="section dev-section">
        <button class="collapse-toggle" onclick={() => (showDevTools = !showDevTools)} data-testid="dev-tools-header">
          <div class="toggle-left">
            {#if showDevTools}
              <ChevronDown size={15} />
            {:else}
              <ChevronRight size={15} />
            {/if}
            <Cpu size={15} />
            <span class="section-title">Dev Tools</span>
          </div>
        </button>

        {#if showDevTools}
          <div class="collapse-body">
            <button class="action-btn" onclick={() => (harnessModalOpen = true)} data-testid="harness-open-btn">
              Open focus trap harness
            </button>
          </div>
        {/if}
      </section>
    {/if}
  </div>
</div>

{#if isDebugMode}
  <Modal open={harnessModalOpen} title="Focus Trap Test" onclose={() => (harnessModalOpen = false)}>
    <div class="harness-modal-content">
      <div class="focus-debug">
        <strong>Active element:</strong> <code>{activeElementInfo}</code>
      </div>

      <div class="harness-field">
        <label for="harness-input">Test input</label>
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
          <input type="checkbox" bind:checked={harnessCheckbox} data-testid="harness-checkbox" />
          Test checkbox
        </label>
      </div>

      <div class="harness-actions">
        <button class="harness-btn secondary" onclick={() => (harnessModalOpen = false)} data-testid="harness-cancel-btn">
          Cancel
        </button>

        <button class="harness-btn" onclick={() => (harnessModalOpen = false)} data-testid="harness-apply-btn">
          Apply
        </button>
      </div>
    </div>
  </Modal>
{/if}

<style>
  /* ═══ Layout shell ═══ */

  .view-container {
    height: 100%;
    overflow-y: auto;
    padding: 20px;
    padding-bottom: calc(var(--layout-player-height, 80px) + 28px);
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--scrollbar-track);
  }

  .content-width {
    width: min(1040px, 100%);
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  /* ═══ Section ═══ */

  .section {
    border-radius: var(--radius-md);
    border: 1px solid var(--divider-color);
    background: rgba(255, 255, 255, 0.02);
    overflow: hidden;
  }

  .section-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--divider-color);
  }

  .section-title {
    font-size: 11px;
    font-weight: 650;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .section-meta {
    font-size: 11px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
    font-family: var(--font-mono);
  }

  .section-meta.meta-warn {
    color: #f8b44f;
  }

  /* ═══ Hero ═══ */

  .hero {
    border-radius: var(--radius-md);
    border: 1px solid var(--divider-color);
    background: rgba(255, 255, 255, 0.025);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .hero-status {
    display: flex;
    align-items: flex-start;
    gap: 10px;
  }

  .health-badge {
    width: 32px;
    height: 32px;
    border-radius: 999px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: var(--text-tertiary);
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.06);
  }

  .health-badge.healthy {
    color: #36d77f;
    background: rgba(54, 215, 127, 0.1);
    border-color: rgba(54, 215, 127, 0.25);
  }

  .health-badge.warning {
    color: #f8b44f;
    background: rgba(248, 180, 79, 0.1);
    border-color: rgba(248, 180, 79, 0.25);
  }

  .hero-copy {
    flex: 1;
    min-width: 0;
  }

  .hero-copy h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 650;
    color: var(--text-primary);
    line-height: 1.3;
  }

  .hero-copy p {
    margin: 2px 0 0;
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .hero-device {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding-top: 12px;
    border-top: 1px solid var(--divider-color);
  }

  .device-line {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-secondary);
    font-size: 12px;
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .device-line :global(svg) {
    flex-shrink: 0;
    color: var(--text-tertiary);
  }

  .device-sub {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .mode-pill {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 3px 8px;
    border-radius: var(--radius-pill);
    background: rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.12);
    border: 1px solid rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.28);
    color: var(--text-primary);
  }

  .age-label {
    font-size: 11px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  /* ═══ Flow bar ═══ */

  .flow-bar {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    border-radius: var(--radius-md);
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--divider-color);
  }

  .flow-endpoint {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .flow-output {
    text-align: right;
    align-items: flex-end;
  }

  .flow-label {
    font-size: 9px;
    font-weight: 650;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .flow-value {
    font-size: 12px;
    color: var(--text-primary);
    font-family: var(--font-mono);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .flow-arrow {
    color: var(--text-tertiary);
    display: flex;
    opacity: 0.5;
  }

  /* ═══ Signal Path pipeline ═══ */

  .pipeline {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    padding: 12px 14px;
  }

  .stage-node {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid rgba(255, 255, 255, 0.07);
    background: rgba(255, 255, 255, 0.02);
    color: var(--text-secondary);
    cursor: pointer;
    white-space: nowrap;
    transition:
      background var(--motion-fast) var(--ease-out),
      border-color var(--motion-fast) var(--ease-out);
  }

  .stage-node:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.12);
  }

  .stage-node.selected {
    border-color: var(--accent-medium);
    background: rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.08);
  }

  .stage-node.ok { color: #36d77f; }
  .stage-node.warn { color: #f8b44f; }
  .stage-node.inactive,
  .stage-node.unknown { color: var(--text-tertiary); }

  .stage-icon {
    display: inline-flex;
    flex-shrink: 0;
  }

  .stage-name {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .pipe-connector {
    color: var(--text-disabled);
    display: inline-flex;
    flex-shrink: 0;
  }

  .stage-detail {
    margin: 0 14px 12px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: rgba(0, 0, 0, 0.18);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .detail-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    margin-bottom: 5px;
  }

  .detail-stage {
    font-size: 11px;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-secondary);
  }

  .detail-reason {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-tertiary);
  }

  .detail-body {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.45;
  }

  /* ═══ Main two-column grid ═══ */

  .main-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 10px;
  }

  /* ═══ Health strip ═══ */

  .health-strip {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
  }

  .hs-metric {
    padding: 10px 14px;
    display: flex;
    flex-direction: column;
    gap: 3px;
    border-right: 1px solid var(--divider-color);
  }

  .hs-metric:last-child {
    border-right: none;
  }

  .hs-label {
    font-size: 9px;
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .hs-value {
    font-size: 15px;
    font-weight: 650;
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
    text-transform: capitalize;
    font-family: var(--font-mono);
  }

  .hs-value.val-warn {
    color: #f8b44f;
  }

  /* ═══ Buffer bars ═══ */

  .buffer-section {
    padding: 10px 14px;
    border-top: 1px solid var(--divider-color);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .buf-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .buf-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .buf-name {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .buf-pct {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }

  .buf-track {
    height: 4px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.06);
    overflow: hidden;
  }

  .buf-fill {
    height: 100%;
    border-radius: 999px;
    background: linear-gradient(
      90deg,
      var(--theme-accent) 0%,
      rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.5) 100%
    );
    transition: width var(--motion-medium) var(--ease-out);
  }

  /* ═══ Counter row ═══ */

  .counter-row {
    padding: 8px 14px;
    border-top: 1px solid var(--divider-color);
    display: flex;
    flex-wrap: wrap;
    gap: 4px 12px;
    font-size: 10px;
    color: var(--text-tertiary);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }

  .counter-row .val-warn {
    color: #f8b44f;
  }

  /* ═══ Events ═══ */

  .events-section {
    border-top: 1px solid var(--divider-color);
    padding: 10px 14px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .events-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2px;
  }

  .event-row {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 8px;
    align-items: baseline;
    padding: 4px 0;
  }

  .event-row + .event-row {
    border-top: 1px solid rgba(255, 255, 255, 0.03);
  }

  .event-code {
    font-size: 11px;
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
  }

  .event-detail {
    font-size: 11px;
    color: var(--text-tertiary);
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .event-age {
    font-size: 10px;
    color: var(--text-disabled);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  /* ═══ Engine list ═══ */

  .engine-list {
    padding: 4px 0;
  }

  .eng-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 12px;
    padding: 6px 14px;
  }

  .eng-row + .eng-row {
    border-top: 1px solid rgba(255, 255, 255, 0.035);
  }

  .eng-label {
    font-size: 11px;
    color: var(--text-tertiary);
    flex-shrink: 0;
    white-space: nowrap;
  }

  .eng-value {
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--text-primary);
    text-align: right;
    overflow-wrap: anywhere;
    min-width: 0;
  }

  /* ═══ Empty states ═══ */

  .empty-state {
    padding: 20px 14px;
    color: var(--text-tertiary);
    font-size: 12px;
    text-align: center;
  }

  .empty-subtle {
    padding: 12px 14px;
    border-top: 1px solid var(--divider-color);
    color: var(--text-disabled);
    font-size: 11px;
  }

  /* ═══ Dev tools ═══ */

  .dev-section {
    border-color: rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.25);
  }

  .collapse-toggle {
    width: 100%;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    padding: 10px 14px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
    text-align: left;
  }

  .collapse-toggle:hover {
    background: rgba(255, 255, 255, 0.03);
    color: var(--text-primary);
  }

  .toggle-left {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .collapse-body {
    border-top: 1px solid var(--divider-color);
    padding: 12px 14px;
  }

  .action-btn {
    border-radius: var(--radius-sm);
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.14);
    color: var(--text-primary);
    padding: 7px 12px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition:
      background var(--motion-fast) var(--ease-out),
      border-color var(--motion-fast) var(--ease-out);
  }

  .action-btn:hover {
    background: rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.24);
    border-color: rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.4);
  }

  /* ═══ Harness modal ═══ */

  .harness-modal-content {
    min-width: 350px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .focus-debug {
    border-radius: var(--radius-sm);
    border: 1px solid rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.4);
    background: rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.1);
    padding: 9px 10px;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .focus-debug code {
    color: var(--text-primary);
    font-family: var(--font-mono);
  }

  .harness-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .harness-field label {
    color: var(--text-tertiary);
    font-size: 12px;
  }

  .harness-field input[type='text'] {
    width: 100%;
    border-radius: var(--radius-sm);
    border: 1px solid var(--divider-color);
    background: rgba(255, 255, 255, 0.05);
    color: var(--text-primary);
    font-size: 13px;
    padding: 8px 10px;
    outline: none;
  }

  .harness-field input[type='text']:focus {
    box-shadow: var(--focus-ring);
    border-color: rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.5);
  }

  .checkbox-label {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .harness-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 4px;
  }

  .harness-btn {
    border-radius: var(--radius-sm);
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.14);
    color: var(--text-primary);
    padding: 7px 12px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .harness-btn:hover:not(:disabled) {
    background: rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.24);
    border-color: rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.4);
  }

  .harness-btn.secondary {
    background: rgba(255, 255, 255, 0.05);
    border-color: var(--divider-color);
  }

  /* ═══ Responsive ═══ */

  @media (max-width: 900px) {
    .main-grid {
      grid-template-columns: 1fr;
    }

    .health-strip {
      grid-template-columns: repeat(2, 1fr);
    }

    .hs-metric:nth-child(2) {
      border-right: none;
    }

    .hs-metric:nth-child(3),
    .hs-metric:nth-child(4) {
      border-top: 1px solid var(--divider-color);
    }

    .hero-device {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }
  }

  @media (max-width: 600px) {
    .flow-bar {
      grid-template-columns: 1fr;
      gap: 6px;
    }

    .flow-output {
      text-align: left;
      align-items: flex-start;
    }

    .flow-arrow {
      transform: rotate(90deg);
      justify-self: start;
    }

    .counter-row {
      flex-direction: column;
      gap: 2px;
    }
  }
</style>
