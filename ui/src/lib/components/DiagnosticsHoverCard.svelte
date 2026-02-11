<script lang="ts">
  import { Activity, AlertTriangle, ShieldCheck } from '@lucide/svelte';
  import type { AudioTelemetryEvent } from '../types/telemetry';
  import { audioTelemetry } from '../state/playback';
  import { currentRouteName, navigate } from '../state/route';
  import { pressScale } from '../utils/animations';

  type HealthTone = 'healthy' | 'attention' | 'idle';

  let telemetry = $derived($audioTelemetry);
  let isHovered = $state(false);
  let isFocused = $state(false);

  let isOpen = $derived(isHovered || isFocused);
  let isActive = $derived($currentRouteName === 'diagnostics');
  let healthTone = $derived(getHealthTone(telemetry));
  let bitPerfectText = $derived(getBitPerfectText(telemetry));
  let outputMode = $derived(getOutputMode(telemetry));
  let outputFormat = $derived(getOutputFormat(telemetry));
  let issueCount = $derived(getIssueCount(telemetry));
  let ringFillPercent = $derived(Math.max(0, Math.min(100, telemetry?.stability?.ring_buffer?.fill_percent ?? 0)));
  let trackUnderruns = $derived(telemetry?.stability?.ring_buffer?.underruns?.track ?? 0);

  function openDiagnostics() {
    navigate({ name: 'diagnostics' });
  }

  function getHealthTone(t: AudioTelemetryEvent | null): HealthTone {
    if (!t) {
      return 'idle';
    }

    if (t.integrity.pcm_bit_perfect.status === 'no') {
      return 'attention';
    }

    if (t.stability.ring_buffer.underruns.track > 0 || t.stability.ring_buffer.overflows.track > 0) {
      return 'attention';
    }

    if (t.signal_path_checks.some((check) => check.status === 'touching_bits')) {
      return 'attention';
    }

    return 'healthy';
  }

  function getBitPerfectText(t: AudioTelemetryEvent | null): string {
    if (!t) {
      return 'Standby';
    }

    if (t.integrity.pcm_bit_perfect.status === 'yes') {
      return 'Bit-perfect';
    }

    if (t.integrity.pcm_bit_perfect.status === 'no') {
      return 'Path altered';
    }

    return 'Unknown path';
  }

  function getOutputMode(t: AudioTelemetryEvent | null): string {
    if (!t) {
      return 'No output';
    }

    if (t.playback.output_mode === 'exclusive') {
      return 'WASAPI Excl';
    }

    if (t.playback.output_mode === 'shared') {
      return 'WASAPI Shared';
    }

    return 'ASIO';
  }

  function getOutputFormat(t: AudioTelemetryEvent | null): string {
    if (!t) {
      return '—';
    }

    const output = t.format.output;
    return `${(output.sample_rate / 1000).toFixed(1)}k · ${output.valid_bits || output.bit_depth}bit`;
  }

  function getIssueCount(t: AudioTelemetryEvent | null): number {
    if (!t) {
      return 0;
    }

    let count = 0;

    if (t.integrity.pcm_bit_perfect.status === 'no') {
      count += 1;
    }

    count += t.signal_path_checks.filter((check) => check.status !== 'ok' && check.status !== 'inactive').length;

    if (t.stability.ring_buffer.underruns.track > 0 || t.stability.ring_buffer.overflows.track > 0) {
      count += 1;
    }

    return count;
  }
</script>

<div
  class="diagnostics-hover"
  role="group"
  aria-label="Diagnostics status"
  onmouseenter={() => (isHovered = true)}
  onmouseleave={() => (isHovered = false)}
>
  <button
    class="diagnostics-pill"
    class:active={isActive}
    class:healthy={healthTone === 'healthy'}
    class:attention={healthTone === 'attention'}
    class:idle={healthTone === 'idle'}
    onclick={openDiagnostics}
    onfocus={() => (isFocused = true)}
    onblur={() => (isFocused = false)}
    title="Open full diagnostics"
    use:pressScale={{ scale: 0.985 }}
  >
    <span class="status-dot" aria-hidden="true"></span>
    <span class="pill-label">{bitPerfectText}</span>
  </button>

  <div class="preview-popover" class:open={isOpen} aria-hidden={!isOpen}>
    <div
      class="popover-health"
      class:healthy={healthTone === 'healthy'}
      class:attention={healthTone === 'attention'}
    >
      {#if healthTone === 'healthy'}
        <ShieldCheck size={14} strokeWidth={1.8} />
        <span>Signal path healthy</span>
      {:else if healthTone === 'attention'}
        <AlertTriangle size={14} strokeWidth={1.8} />
        <span>Attention recommended</span>
      {:else}
        <Activity size={14} strokeWidth={1.8} />
        <span>Waiting for telemetry</span>
      {/if}
    </div>

    <div class="popover-metrics">
      <div class="pm-row">
        <span class="pm-label">Output</span>
        <span class="pm-value">{outputMode}</span>
      </div>
      <div class="pm-row">
        <span class="pm-label">Resolution</span>
        <span class="pm-value">{outputFormat}</span>
      </div>
      <div class="pm-row">
        <span class="pm-label">Underruns</span>
        <span class="pm-value" class:pm-warn={trackUnderruns > 0}>{trackUnderruns}</span>
      </div>
      <div class="pm-row">
        <span class="pm-label">Issues</span>
        <span class="pm-value" class:pm-warn={issueCount > 0}>{issueCount}</span>
      </div>
    </div>

    <div class="popover-buffer">
      <div class="pb-track">
        <div class="pb-fill" style={`width: ${ringFillPercent}%`}></div>
      </div>
      <span class="pb-meta">Buffer {ringFillPercent.toFixed(0)}%</span>
    </div>
  </div>
</div>

<style>
  .diagnostics-hover {
    --diag-healthy: #32d178;
    --diag-healthy-dim: rgba(50, 209, 120, 0.14);
    --diag-warning: #f8b44f;
    --diag-warning-dim: rgba(248, 180, 79, 0.14);

    position: relative;
    -webkit-app-region: no-drag;
    display: flex;
    align-items: center;
  }

  /* ── Pill ── */

  .diagnostics-pill {
    height: 30px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-primary);
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 0 12px 0 10px;
    cursor: pointer;
    white-space: nowrap;
    transition:
      background var(--motion-fast) var(--ease-out),
      border-color var(--motion-fast) var(--ease-out),
      box-shadow var(--motion-fast) var(--ease-out);
  }

  .diagnostics-pill:hover {
    background: var(--surface-hover);
    border-color: rgba(255, 255, 255, 0.14);
  }

  .diagnostics-pill.active {
    border-color: var(--accent-medium);
    background: linear-gradient(
      180deg,
      rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.16) 0%,
      rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.06) 100%
    );
  }

  .status-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--text-tertiary);
    flex-shrink: 0;
    transition:
      background var(--motion-fast) var(--ease-out),
      box-shadow var(--motion-fast) var(--ease-out);
  }

  .diagnostics-pill.healthy .status-dot {
    background: var(--diag-healthy);
    box-shadow: 0 0 8px rgba(50, 209, 120, 0.5);
  }

  .diagnostics-pill.attention .status-dot {
    background: var(--diag-warning);
    box-shadow: 0 0 8px rgba(248, 180, 79, 0.45);
  }

  .pill-label {
    font-size: 11px;
    font-weight: 550;
    letter-spacing: 0.01em;
    color: var(--text-secondary);
    line-height: 1;
  }

  .diagnostics-pill.healthy .pill-label,
  .diagnostics-pill.attention .pill-label {
    color: var(--text-primary);
  }

  /* ── Popover ── */

  .preview-popover {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 0;
    width: 256px;
    border-radius: var(--radius-md);
    border: 1px solid var(--divider-color);
    background: var(--surface-floating);
    box-shadow: var(--shadow-3);
    padding: 14px;
    opacity: 0;
    transform: translateY(5px) scale(0.98);
    transform-origin: bottom left;
    pointer-events: none;
    transition:
      opacity var(--motion-fast) var(--ease-out),
      transform var(--motion-fast) var(--ease-out);
    z-index: 300;
    backdrop-filter: blur(20px);
  }

  .preview-popover.open {
    opacity: 1;
    transform: translateY(0) scale(1);
    pointer-events: auto;
  }

  /* ── Health indicator ── */

  .popover-health {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 12px;
    font-weight: 550;
    color: var(--text-secondary);
    margin-bottom: 12px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--divider-color);
  }

  .popover-health.healthy {
    color: var(--diag-healthy);
  }

  .popover-health.attention {
    color: var(--diag-warning);
  }

  /* ── Metric rows ── */

  .popover-metrics {
    display: flex;
    flex-direction: column;
    margin-bottom: 12px;
  }

  .pm-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 5px 0;
  }

  .pm-row + .pm-row {
    border-top: 1px solid rgba(255, 255, 255, 0.04);
  }

  .pm-label {
    font-size: 10px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--text-tertiary);
  }

  .pm-value {
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }

  .pm-value.pm-warn {
    color: var(--diag-warning);
  }

  /* ── Buffer bar ── */

  .popover-buffer {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .pb-track {
    height: 4px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.07);
    overflow: hidden;
  }

  .pb-fill {
    height: 100%;
    border-radius: 999px;
    background: linear-gradient(
      90deg,
      var(--theme-accent) 0%,
      rgba(var(--theme-accent-r), var(--theme-accent-g), var(--theme-accent-b), 0.55) 100%
    );
    transition: width var(--motion-medium) var(--ease-out);
  }

  .pb-meta {
    font-size: 10px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.02em;
  }
</style>
