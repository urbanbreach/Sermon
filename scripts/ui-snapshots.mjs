import { spawn } from 'node:child_process';
import { parseArgs } from 'node:util';

const { values } = parseArgs({
  options: {
    milestone: {
      type: 'string',
      short: 'm',
    },
  },
});

const milestone = values.milestone || 'default';

console.log(`\x1b[34m[Sermon UI Snapshot Mode]\x1b[0m`);
console.log(`Milestone: ${milestone}`);
console.log(`----------------------------------------`);
console.log(`\x1b[33mInstructions:\x1b[0m`);
console.log(`1. Set Window Resolution: 1440x900`);
console.log(`2. Set Windows Scaling: 100%`);
console.log(`3. Navigate to the desired state.`);
console.log(`4. Capture with Win+Shift+S or Alt+PrntScrn.`);
console.log(`5. Save to artifacts/ui/<milestone>/`);
console.log(`----------------------------------------`);

const env = {
  ...process.env,
  SERMON_MOCK: '1',
  SERMON_SNAPSHOT: '1',
  SERMON_SNAPSHOT_MILESTONE: milestone,
};

// Start Tauri dev
const child = spawn('pnpm', ['tauri', 'dev'], {
  stdio: 'inherit',
  env,
  shell: true,
});

child.on('exit', (code) => {
  process.exit(code ?? 0);
});
