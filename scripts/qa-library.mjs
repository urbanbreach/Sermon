#!/usr/bin/env node
/**
 * Library Scan + Browse QA Harness
 *
 * This script provides instructions for MCP-based QA of library scanning and view navigation.
 *
 * For automated testing, use Tauri MCP tools directly:
 * 1. tauri_driver_session({ action: 'start' })
 * 2. tauri_ipc_monitor({ action: 'start' })
 * 3. Navigate Albums/Tracks via top segmented control
 * 4. tauri_ipc_get_captured() and assert library + scan commands
 */

console.log('╔═══════════════════════════════════════════════════════════╗');
console.log('║          Library Scan + Browse QA Harness                ║');
console.log('╠═══════════════════════════════════════════════════════════╣');
console.log('║ Prerequisites:                                            ║');
console.log('║ 1. App running: cmd.exe /c "cargo tauri dev"              ║');
console.log('║ 2. MCP bridge connected                                   ║');
console.log('║ 3. Test library folder configured locally                 ║');
console.log('╠═══════════════════════════════════════════════════════════╣');
console.log('║ MCP Commands to Execute:                                  ║');
console.log('║                                                           ║');
console.log('║ 1. Start driver + IPC monitor:                            ║');
console.log('║    tauri_driver_session({ action: \'start\' })             ║');
console.log('║    tauri_ipc_monitor({ action: \'start\' })                ║');
console.log('║                                                           ║');
console.log('║ 2. Trigger a scan command path:                           ║');
console.log('║    tauri_ipc_execute_command({ command: \'cmd_scan_start\' }) ║');
console.log('║    Then inspect captured IPC for cmd_scan_start           ║');
console.log('║                                                           ║');
console.log('║ 3. Navigate Albums and confirm data load call:            ║');
console.log('║    tauri_webview_find_element({                           ║');
console.log('║      strategy: \'text\', selector: \'Albums\'              ║');
console.log('║    })                                                     ║');
console.log('║    tauri_webview_execute_js({                             ║');
console.log('║      script: "Array.from(document.querySelectorAll(\'button\')).find(b => b.textContent?.trim() === \'Albums\')?.click()" ║');
console.log('║    })                                                     ║');
console.log('║    tauri_ipc_get_captured({}) -> expect cmd_library_list_albums_page ║');
console.log('║                                                           ║');
console.log('║ 4. Navigate Tracks and confirm data load call:            ║');
console.log('║    tauri_webview_find_element({                           ║');
console.log('║      strategy: \'text\', selector: \'Tracks\'              ║');
console.log('║    })                                                     ║');
console.log('║    tauri_webview_execute_js({                             ║');
console.log('║      script: "Array.from(document.querySelectorAll(\'button\')).find(b => b.textContent?.trim() === \'Tracks\')?.click()" ║');
console.log('║    })                                                     ║');
console.log('║    tauri_ipc_get_captured({}) -> expect cmd_library_list_tracks_page ║');
console.log('║                                                           ║');
console.log('║ 5. Optional library health assertions from IPC capture:   ║');
console.log('║    cmd_library_get_stats                                  ║');
console.log('║    cmd_library_search_suggest (when search input is used) ║');
console.log('╠═══════════════════════════════════════════════════════════╣');
console.log('║ Success Criteria:                                         ║');
console.log('║ - Scan path dispatches cmd_scan_start                     ║');
console.log('║ - Albums navigation dispatches cmd_library_list_albums_page ║');
console.log('║ - Tracks navigation dispatches cmd_library_list_tracks_page ║');
console.log('║ - UI remains responsive while data populates              ║');
console.log('║ - Evidence saved under .sisyphus/evidence/                ║');
console.log('║                                                           ║');
console.log('║ Evidence Paths:                                           ║');
console.log('║ - .sisyphus/evidence/task-8-library-albums.png            ║');
console.log('║ - .sisyphus/evidence/task-8-library-tracks.png            ║');
console.log('║ - .sisyphus/evidence/task-8-library-scan.json             ║');
console.log('║ - .sisyphus/evidence/task-8-library-ipc.json              ║');
console.log('╚═══════════════════════════════════════════════════════════╝');

// Exit successfully - actual QA done via MCP
process.exit(0);
