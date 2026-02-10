import { beforeEach, vi } from 'vitest';

const noopUnlisten = vi.fn(() => {});

function createUnmockedInvokeError(command: unknown): Error {
  const commandName = typeof command === 'string' ? command : String(command);
  return new Error(`Unmocked Tauri invoke command: ${commandName}`);
}

const invokeMock = vi.fn(async (command: unknown) => {
  throw createUnmockedInvokeError(command);
});

const eventListenMock = vi.fn(async () => noopUnlisten);
const eventEmitMock = vi.fn(async () => undefined);
const eventOnceMock = vi.fn(async () => noopUnlisten);

const mockCurrentWindow = {
  label: 'main',
  listen: vi.fn(async () => noopUnlisten),
  once: vi.fn(async () => noopUnlisten),
  emit: vi.fn(async () => undefined),
  close: vi.fn(async () => undefined),
  minimize: vi.fn(async () => undefined),
  maximize: vi.fn(async () => undefined),
  unmaximize: vi.fn(async () => undefined),
  toggleMaximize: vi.fn(async () => undefined),
  isMaximized: vi.fn(async () => false),
  show: vi.fn(async () => undefined),
  hide: vi.fn(async () => undefined),
  unminimize: vi.fn(async () => undefined),
  setFocus: vi.fn(async () => undefined),
  setTitle: vi.fn(async () => undefined),
  startDragging: vi.fn(async () => undefined),
};

const getCurrentWindowMock = vi.fn(() => mockCurrentWindow);

class MockWebviewWindow {
  static getByLabel = vi.fn(async () => null);

  label: string;

  options: unknown;

  constructor(label: string, options?: unknown) {
    this.label = label;
    this.options = options;
  }

  listen = vi.fn(async () => noopUnlisten);

  once = vi.fn(async () => noopUnlisten);

  emit = vi.fn(async () => undefined);

  close = vi.fn(async () => undefined);

  unminimize = vi.fn(async () => undefined);

  show = vi.fn(async () => undefined);

  setFocus = vi.fn(async () => undefined);

  setTitle = vi.fn(async () => undefined);
}

const dialogOpenMock = vi.fn(async () => null);
const dialogSaveMock = vi.fn(async () => null);
const dialogMessageMock = vi.fn(async () => undefined);
const dialogAskMock = vi.fn(async () => false);
const dialogConfirmMock = vi.fn(async () => false);

const fsReadTextFileMock = vi.fn(async () => '');
const fsWriteTextFileMock = vi.fn(async () => undefined);
const fsReadFileMock = vi.fn(async () => new Uint8Array());
const fsWriteFileMock = vi.fn(async () => undefined);
const fsExistsMock = vi.fn(async () => false);
const fsMkdirMock = vi.fn(async () => undefined);
const fsReadDirMock = vi.fn(async () => []);
const fsStatMock = vi.fn(async () => ({ isFile: true, isDirectory: false, size: 0 }));
const fsLstatMock = vi.fn(async () => ({ isFile: true, isDirectory: false, size: 0 }));
const fsRemoveMock = vi.fn(async () => undefined);
const fsRenameMock = vi.fn(async () => undefined);
const fsCopyFileMock = vi.fn(async () => undefined);
const fsCreateMock = vi.fn(async () => ({
  write: vi.fn(async () => undefined),
  read: vi.fn(async () => new Uint8Array()),
  seek: vi.fn(async () => undefined),
  truncate: vi.fn(async () => undefined),
  close: vi.fn(async () => undefined),
}));
const fsWatchMock = vi.fn(async () => noopUnlisten);

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: eventListenMock,
  emit: eventEmitMock,
  once: eventOnceMock,
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: getCurrentWindowMock,
}));

vi.mock('@tauri-apps/api/webviewWindow', () => ({
  WebviewWindow: MockWebviewWindow,
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: dialogOpenMock,
  save: dialogSaveMock,
  message: dialogMessageMock,
  ask: dialogAskMock,
  confirm: dialogConfirmMock,
}));

vi.mock('@tauri-apps/plugin-fs', () => ({
  readTextFile: fsReadTextFileMock,
  writeTextFile: fsWriteTextFileMock,
  readFile: fsReadFileMock,
  writeFile: fsWriteFileMock,
  exists: fsExistsMock,
  mkdir: fsMkdirMock,
  readDir: fsReadDirMock,
  stat: fsStatMock,
  lstat: fsLstatMock,
  remove: fsRemoveMock,
  rename: fsRenameMock,
  copyFile: fsCopyFileMock,
  create: fsCreateMock,
  watch: fsWatchMock,
}));

beforeEach(() => {
  invokeMock.mockClear();
  invokeMock.mockImplementation(async (command: unknown) => {
    throw createUnmockedInvokeError(command);
  });

  eventListenMock.mockClear();
  eventEmitMock.mockClear();
  eventOnceMock.mockClear();

  getCurrentWindowMock.mockClear();
  MockWebviewWindow.getByLabel.mockClear();

  dialogOpenMock.mockClear();
  dialogSaveMock.mockClear();
  dialogMessageMock.mockClear();
  dialogAskMock.mockClear();
  dialogConfirmMock.mockClear();

  fsReadTextFileMock.mockClear();
  fsWriteTextFileMock.mockClear();
  fsReadFileMock.mockClear();
  fsWriteFileMock.mockClear();
  fsExistsMock.mockClear();
  fsMkdirMock.mockClear();
  fsReadDirMock.mockClear();
  fsStatMock.mockClear();
  fsLstatMock.mockClear();
  fsRemoveMock.mockClear();
  fsRenameMock.mockClear();
  fsCopyFileMock.mockClear();
  fsCreateMock.mockClear();
  fsWatchMock.mockClear();
});

const globalScope = globalThis as typeof globalThis & { __TAURI__?: Record<string, unknown> };
globalScope.__TAURI__ = {
  ...(globalScope.__TAURI__ ?? {}),
  invoke: invokeMock,
};
