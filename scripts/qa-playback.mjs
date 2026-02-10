#!/usr/bin/env node
/**
 * Playback Transport QA Harness
 *
 * This script provides instructions for MCP-based QA of playback transport controls.
 *
 * For automated testing, use Tauri MCP tools directly:
 * 1. tauri_driver_session({ action: 'start' })
 * 2. tauri_ipc_monitor({ action: 'start' })
 * 3. tauri_webview_wait_for({ type: 'selector', value: '[data-testid="topbar-title"]' })
 * 4. tauri_webview_interact({ action: 'click', selector: '.waveform-play' })
 * 5. tauri_ipc_get_captured() and assert playback/queue command traffic
 */

console.log('╔═══════════════════════════════════════════════════════════╗');
console.log('║          Playback Transport QA Harness                    ║');
console.log('╠═══════════════════════════════════════════════════════════╣');
console.log('║ Prerequisites:                                            ║');
console.log('║ 1. App running: cmd.exe /c "cargo tauri dev"              ║');
console.log('║ 2. MCP bridge connected                                   ║');
console.log('║ 3. Library has at least one playable track                ║');
console.log('╠═══════════════════════════════════════════════════════════╣');
console.log('║ MCP Commands to Execute:                                  ║');
console.log('║                                                           ║');
console.log('║ 1. Start driver + IPC monitor:                            ║');
console.log('║    tauri_driver_session({ action: \'start\' })             ║');
console.log('║    tauri_ipc_monitor({ action: \'start\' })                ║');
console.log('║                                                           ║');
console.log('║ 2. Wait for top bar title:                                ║');
console.log('║    tauri_webview_wait_for({                               ║');
console.log('║      type: \'selector\',                                    ║');
console.log('║      value: \'[data-testid="topbar-title"]\'               ║');
console.log('║    })                                                     ║');
console.log('║                                                           ║');
console.log('║ 3. Verify play button exists and click transport controls:║');
console.log('║    tauri_webview_find_element({                           ║');
console.log("║      selector: '.waveform-play'                           ║");
console.log('║    })                                                     ║');
console.log('║    tauri_webview_interact({                               ║');
console.log("║      action: 'click',                                     ║");
console.log("║      selector: '.waveform-play'                           ║");
console.log('║    })                                                     ║');
console.log("║    tauri_webview_interact({ action: 'click', selector: '.waveform-play' }) ║");
console.log('║                                                           ║');
console.log('║ 4. Inspect IPC traffic:                                   ║');
console.log('║    tauri_ipc_get_captured({})                             ║');
console.log('║    Expect at least one of:                                ║');
console.log('║    cmd_playback_start / cmd_playback_resume / cmd_playback_pause ║');
console.log('║    Optional depending on state: cmd_playback_next         ║');
console.log('║                                                           ║');
console.log('║ 5. Negative-path assertion (required):                    ║');
console.log('║    tauri_ipc_execute_command({                            ║');
console.log('║      command: \'cmd_queue_set_and_play\',                   ║');
console.log('║      args: { trackIds: [] }                               ║');
console.log('║    })                                                     ║');
console.log('║    Expected: error/validation failure, no crash           ║');
console.log('╠═══════════════════════════════════════════════════════════╣');
console.log('║ Success Criteria:                                         ║');
console.log('║ - [data-testid="topbar-title"] is reachable               ║');
console.log('║ - .waveform-play interaction is successful                 ║');
console.log('║ - IPC capture contains playback command dispatches         ║');
console.log('║ - Empty trackIds on cmd_queue_set_and_play is rejected    ║');
console.log('║ - Evidence saved under .sisyphus/evidence/                ║');
console.log('║                                                           ║');
console.log('║ Evidence Paths:                                           ║');
console.log('║ - .sisyphus/evidence/task-8-playback-topbar.png           ║');
console.log('║ - .sisyphus/evidence/task-8-playback-transport.png        ║');
console.log('║ - .sisyphus/evidence/task-8-playback-ipc.json             ║');
console.log('║ - .sisyphus/evidence/task-8-playback-negative.json        ║');
console.log('╚═══════════════════════════════════════════════════════════╝');

// Exit successfully - actual QA done via MCP
process.exit(0);
