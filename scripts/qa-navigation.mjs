#!/usr/bin/env node
/**
 * Navigation + Diagnostics QA Harness
 *
 * This script provides instructions for MCP-based QA of app navigation and diagnostics view.
 *
 * For automated testing, use Tauri MCP tools directly:
 * 1. tauri_driver_session({ action: 'start' })
 * 2. Navigate core routes (albums/artists/tracks)
 * 3. Open diagnostics via .diagnostics-pill
 * 4. Assert diagnostics data-testid selectors resolve
 */

console.log('╔═══════════════════════════════════════════════════════════╗');
console.log('║          Navigation + Diagnostics QA Harness              ║');
console.log('╠═══════════════════════════════════════════════════════════╣');
console.log('║ Prerequisites:                                            ║');
console.log('║ 1. App running: cmd.exe /c "cargo tauri dev"              ║');
console.log('║ 2. MCP bridge connected                                   ║');
console.log('║ 3. Telemetry stream active (for diagnostics content)      ║');
console.log('╠═══════════════════════════════════════════════════════════╣');
console.log('║ MCP Commands to Execute:                                  ║');
console.log('║                                                           ║');
console.log('║ 1. Start session + confirm app shell:                     ║');
console.log('║    tauri_driver_session({ action: \'start\' })             ║');
console.log('║    tauri_webview_wait_for({ type: \'selector\', value: \'.top-bar\' }) ║');
console.log('║                                                           ║');
console.log('║ 2. Navigate primary views from top bar controls:          ║');
console.log('║    Use tauri_webview_execute_js to click Albums/Artists/Tracks buttons by text ║');
console.log('║    After each click, verify view change via DOM snapshot  ║');
console.log('║                                                           ║');
console.log('║ 3. Open diagnostics view:                                 ║');
console.log('║    tauri_webview_find_element({ selector: \'.diagnostics-pill\' }) ║');
console.log('║    tauri_webview_interact({ action: \'click\', selector: \'.diagnostics-pill\' }) ║');
console.log('║    tauri_webview_wait_for({                               ║');
console.log('║      type: \'selector\', value: \'[data-testid="diag-view"]\'   ║');
console.log('║    })                                                     ║');
console.log('║                                                           ║');
console.log('║ 4. Assert diagnostics selectors are present:              ║');
console.log('║    [data-testid="diag-view"]                              ║');
console.log('║    [data-testid="diag-overview"]                          ║');
console.log('║    [data-testid="diag-signal-path"]                       ║');
console.log('║    [data-testid="diag-stability"]                         ║');
console.log('║    [data-testid="diag-device-details"]                    ║');
console.log('║                                                           ║');
console.log('║ 5. Assert top bar title selector in diagnostics route:    ║');
console.log('║    tauri_webview_find_element({                           ║');
console.log('║      selector: \'[data-testid="topbar-title"]\'           ║');
console.log('║    })                                                     ║');
console.log('║    Expected title text contains \'Audio Diagnostics\'     ║');
console.log('╠═══════════════════════════════════════════════════════════╣');
console.log('║ Success Criteria:                                         ║');
console.log('║ - Core route navigation responds without dead clicks       ║');
console.log('║ - Diagnostics opens via .diagnostics-pill                 ║');
console.log('║ - Diagnostics data-testid selectors all resolve           ║');
console.log('║ - [data-testid="topbar-title"] is visible in diagnostics  ║');
console.log('║ - Evidence saved under .sisyphus/evidence/                ║');
console.log('║                                                           ║');
console.log('║ Evidence Paths:                                           ║');
console.log('║ - .sisyphus/evidence/task-8-navigation-albums.png         ║');
console.log('║ - .sisyphus/evidence/task-8-navigation-diagnostics.png    ║');
console.log('║ - .sisyphus/evidence/task-8-navigation-dom.yaml           ║');
console.log('║ - .sisyphus/evidence/task-8-navigation-selectors.json     ║');
console.log('╚═══════════════════════════════════════════════════════════╝');

// Exit successfully - actual QA done via MCP
process.exit(0);
