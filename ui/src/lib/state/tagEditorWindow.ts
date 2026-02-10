import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

const TAG_EDITOR_WINDOW_LABEL = 'tag-editor';
const MAIN_WINDOW_LABEL = 'main';

function normalizeTrackIds(trackIds: number[]): number[] {
  const deduped = new Set<number>();
  for (const id of trackIds) {
    if (Number.isInteger(id) && id > 0) {
      deduped.add(id);
    }
  }
  return Array.from(deduped);
}

function buildEditorUrl(trackIds: number[]): string {
  const params = new URLSearchParams();
  params.set('window', 'tag-editor');
  params.set('trackIds', trackIds.join(','));
  return `/?${params.toString()}`;
}

export async function openTagEditorWindow(trackIds: number[]): Promise<void> {
  const normalizedTrackIds = normalizeTrackIds(trackIds);
  if (normalizedTrackIds.length === 0) {
    return;
  }

  const existingWindow = await WebviewWindow.getByLabel(TAG_EDITOR_WINDOW_LABEL);
  if (existingWindow) {
    await existingWindow.emit('tag-editor://open', normalizedTrackIds);
    await existingWindow.unminimize();
    await existingWindow.show();
    await existingWindow.setFocus();
    return;
  }

  const tagEditorWindow = new WebviewWindow(TAG_EDITOR_WINDOW_LABEL, {
    title: normalizedTrackIds.length > 1 ? `Edit ${normalizedTrackIds.length} Tracks` : 'Edit Tags',
    url: buildEditorUrl(normalizedTrackIds),
    width: 980,
    height: 760,
    minWidth: 860,
    minHeight: 620,
    center: true,
    focus: true,
    resizable: true,
    decorations: false,
    transparent: false,
    backgroundColor: '#0a0a0a',
  });

  tagEditorWindow.once('tauri://error', (event) => {
    console.error('Failed to create tag editor window', event);
  });
}

export async function showTagEditorInMainPanel(trackIds: number[]): Promise<void> {
  const normalizedTrackIds = normalizeTrackIds(trackIds);
  if (normalizedTrackIds.length === 0) {
    return;
  }

  const mainWindow = await WebviewWindow.getByLabel(MAIN_WINDOW_LABEL);
  if (!mainWindow) {
    return;
  }

  await mainWindow.emit('tag-editor://show-in-main-panel', normalizedTrackIds);
  await mainWindow.unminimize();
  await mainWindow.show();
  await mainWindow.setFocus();
}
