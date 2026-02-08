#!/usr/bin/env node
/**
 * Albums Grid Deterministic Memory Probe Harness (Documentation Script)
 *
 * Purpose
 * -------
 * Document the exact MCP probe sequence used to capture memory baseline evidence
 * before albums-grid memory stabilization changes.
 *
 * This script is intentionally non-invasive: it does not mutate app source code and
 * does not call MCP directly. It prints the sequence, payload schema, and an example
 * evidence envelope expected by:
 *   .sisyphus/evidence/task-1-dev-baseline-memory.json
 */

const PROBE_SEQUENCE = [
  {
    step: 1,
    tool: 'tauri_driver_session',
    action: 'status',
    detail: 'Verify Tauri MCP connection is active.',
  },
  {
    step: 2,
    tool: 'tauri_webview_find_element / tauri_webview_interact',
    action: 'verify Albums tab active (click only if inactive)',
    detail: 'Albums probe must run from Albums tab.',
  },
  {
    step: 3,
    tool: 'tauri_webview_execute_js',
    action: 'window.__artworkMetrics.reset()',
    detail: 'Reset artwork timing/error counters for deterministic baseline.',
  },
  {
    step: 4,
    tool: 'tauri_webview_execute_js',
    action: 'Inject blob URL create/revoke shim (URL.createObjectURL / URL.revokeObjectURL)',
    detail: 'Capture created/revoked/live blob counts via window.__blobUrlStats.get().',
  },
  {
    step: 5,
    tool: 'tauri_webview_execute_js',
    action: 'Run 3 down/up scroll cycles in div.list-wrapper > :first-child',
    detail: 'Step size = max(420, clientHeight * 1.2), wait 70ms between scroll steps.',
  },
  {
    step: 6,
    tool: 'tauri_webview_execute_js',
    action: 'Wait 10s idle, capture idle blob stats and visible image count',
    detail: 'Idle window is mandatory for recovery measurement.',
  },
  {
    step: 7,
    tool: 'tauri_webview_screenshot',
    action: 'Capture Albums view screenshot',
    detail: 'Persist evidence image artifact.',
  },
];

const PROBE_JSON_SCHEMA = {
  type: 'object',
  required: ['cycles', 'idleBlob', 'visibleImages'],
  properties: {
    startedAt: { type: 'string', format: 'date-time' },
    completedAt: { type: 'string', format: 'date-time' },
    scroller: {
      type: 'object',
      required: ['selector', 'stepPx', 'waitMs'],
      properties: {
        selector: { type: 'string' },
        stepPx: { type: 'number' },
        waitMs: { type: 'number' },
      },
    },
    cycles: {
      type: 'array',
      minItems: 3,
      items: {
        type: 'object',
        required: ['cycle', 'markers', 'blob', 'metrics'],
        properties: {
          cycle: { type: 'number', minimum: 1 },
          markers: {
            type: 'object',
            required: ['downStart', 'downEnd', 'upStart', 'upEnd', 'snapshotAt'],
            properties: {
              downStart: { type: 'number' },
              downEnd: { type: 'number' },
              upStart: { type: 'number' },
              upEnd: { type: 'number' },
              snapshotAt: { type: 'number' },
            },
          },
          blob: {
            type: 'object',
            required: ['created', 'revoked', 'live'],
            properties: {
              created: { type: 'number', minimum: 0 },
              revoked: { type: 'number', minimum: 0 },
              live: { type: 'number', minimum: 0 },
            },
          },
          metrics: {
            type: 'object',
            required: ['count', 'errorCount', 'p95ThumbMs'],
            properties: {
              count: { type: 'number', minimum: 0 },
              errorCount: { type: 'number', minimum: 0 },
              p95ThumbMs: { type: 'number' },
            },
          },
        },
      },
    },
    idleBlob: {
      type: 'object',
      required: ['created', 'revoked', 'live'],
      properties: {
        created: { type: 'number', minimum: 0 },
        revoked: { type: 'number', minimum: 0 },
        live: { type: 'number', minimum: 0 },
      },
    },
    visibleImages: { type: 'number', minimum: 0 },
  },
};

const EXAMPLE_EVIDENCE = {
  startedAt: '2026-02-08T00:00:00.000Z',
  completedAt: '2026-02-08T00:00:26.000Z',
  scroller: {
    selector: 'div.list-wrapper > :first-child',
    stepPx: 820,
    waitMs: 70,
  },
  cycles: [
    {
      cycle: 1,
      markers: {
        downStart: 0,
        downEnd: 9860,
        upStart: 9860,
        upEnd: 0,
        snapshotAt: 0,
      },
      blob: { created: 620, revoked: 138, live: 482 },
      metrics: { count: 310, errorCount: 0, p95ThumbMs: 92.5 },
    },
  ],
  idleBlob: { created: 1712, revoked: 1590, live: 122 },
  visibleImages: 38,
};

console.log('\nAlbums Grid Memory Probe Harness (Documentation)\n');
console.log('MCP probe sequence:\n');
for (const step of PROBE_SEQUENCE) {
  console.log(`${step.step}. [${step.tool}] ${step.action}`);
  console.log(`   - ${step.detail}`);
}

console.log('\nExpected evidence JSON schema:\n');
console.log(JSON.stringify(PROBE_JSON_SCHEMA, null, 2));

console.log('\nExample evidence envelope:\n');
console.log(JSON.stringify(EXAMPLE_EVIDENCE, null, 2));

console.log('\nUsage:\n  node scripts/memory-probe.mjs\n');
