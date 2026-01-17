#!/usr/bin/env node
/**
 * UI Snapshots CLI wrapper
 * 
 * Translates `--milestone XX` to PowerShell's `-Milestone XX` parameter format.
 * Usage: node ./scripts/ui-snapshots.mjs --milestone 04
 */

import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const args = process.argv.slice(2);

// Parse --milestone argument
let milestone = '00';
for (let i = 0; i < args.length; i++) {
  if (args[i] === '--milestone' && args[i + 1]) {
    milestone = args[i + 1];
    break;
  }
}

// Get script directory
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const scriptPath = join(__dirname, 'snapshot.ps1');

// Spawn PowerShell with the correct parameter
const ps = spawn('pwsh', [scriptPath, '-Milestone', milestone], {
  stdio: 'inherit',
  shell: false
});

ps.on('close', (code) => {
  process.exit(code ?? 0);
});

ps.on('error', (err) => {
  console.error('Failed to start PowerShell:', err.message);
  process.exit(1);
});
