import { describe, expect, test } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('global Tauri invoke fallback', () => {
  test('throws loudly for unmocked commands', async () => {
    await expect(invoke('cmd_test_unmocked')).rejects.toThrow(
      'Unmocked Tauri invoke command: cmd_test_unmocked',
    );
  });
});
