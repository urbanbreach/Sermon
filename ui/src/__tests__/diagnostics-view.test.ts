import { describe, expect, it } from 'vitest';
import DiagnosticsView from '../lib/views/DiagnosticsView.svelte';
import diagnosticsViewSource from '../lib/views/DiagnosticsView.svelte?raw';

describe('DiagnosticsView', () => {
  it('exports a Svelte component', () => {
    expect(DiagnosticsView).toBeDefined();
  });

  it.each([
    'diag-view',
    'diag-overview',
    'diag-signal-path',
    'diag-signal-path-graph',
    'diag-stability',
    'diag-events',
    'diag-device-details',
  ])('contains %s data-testid section hook', (testId) => {
    expect(diagnosticsViewSource).toContain(`data-testid="${testId}"`);
  });
});
