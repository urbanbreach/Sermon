<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { getArtworkBestForAlbum } from '../api/artwork';
  import {
    getDevArtworkUrl,
    releaseDevArtworkUrl
  } from '../utils/artworkDevUrls';
  import { enqueueArtworkDecode, hasReadyThumb, markThumbReady } from '../utils/artworkDecodeQueue';
  import {
    getArtworkSlotsSnapshot,
    subscribeArtworkSlots,
    type ArtworkSlotRecord
  } from '../utils/albumsArtworkSlots';
  import { recordLqipLoaded, recordThumbLoaded, recordArtworkError } from '../utils/artworkMetrics';

  interface Props {
    wrapperEl?: HTMLDivElement | null;
    enabled?: boolean;
    deferHighRes?: boolean;
    ultraFastMode?: boolean;
    artworkRadiusPx?: number;
  }

  let {
    wrapperEl = null,
    enabled = true,
    deferHighRes = false,
    ultraFastMode = false,
    artworkRadiusPx = 10
  }: Props = $props();

  const dispatch = createEventDispatcher<{ unsupported: void }>();

  let canvasEl = $state<HTMLCanvasElement | null>(null);

  const LQIP_SIZE = 32;
  const THUMB_SIZE = 256;
  const MAX_TEXTURES = 512;
  const PRELOAD_MARGIN_IDLE_PX = 1800;
  const PRELOAD_MARGIN_DEFER_PX = 900;
  const PRELOAD_MARGIN_ULTRA_PX = 760;
  const FULL_RES_MARGIN_IDLE_PX = 1100;
  const FULL_RES_MARGIN_DEFER_PX = 280;
  const FULL_RES_MARGIN_ULTRA_PX = 0;
  const STALE_SLOT_FRAMES = 120;
  const ULTRA_FAST_FRAME_INTERVAL_MS = 28;
  const MAX_CONSECUTIVE_TEXTURE_ERRORS = 16;
  const MAX_TOTAL_TEXTURE_ERRORS = 40;
  const MAX_RESOLVED_ALBUM_KEYS = 2000;
  const useDevThumbFallback = import.meta.env.DEV;

  type SlotState = {
    id: string;
    cacheKey: string;
    artistSort: string;
    titleSort: string;
    resolvedKey: string | null;
    lqipUrl: string | null;
    thumbUrl: string | null;
    urlsLoading: boolean;
    stage: 'none' | 'lqip' | 'full';
    x: number;
    y: number;
    width: number;
    height: number;
    lastSeen: number;
    inViewport: boolean;
    inPreloadZone: boolean;
    inFullResZone: boolean;
    element: HTMLElement;
    fullLoadCancel: (() => void) | null;
  };

  type TextureRecord = {
    status: 'loading' | 'ready' | 'error';
    texture: WebGLTexture | null;
    promise: Promise<void> | null;
    lastUsed: number;
  };

  const slots = new Map<string, SlotState>();
  const textures = new Map<string, TextureRecord>();
  const pendingFullLoads = new Set<string>();

  let rafId = 0;
  let frameIndex = 0;
  let textureClock = 0;
  let lastRenderAt = 0;
  let layoutDirty = true;
  let consecutiveTextureErrors = 0;
  let totalTextureErrors = 0;
  let unsupportedDispatched = false;
  let scrollerEl: HTMLElement | null = null;
  let slotRegistrySnapshot = new Map<string, ArtworkSlotRecord>();
  let unsubscribeSlotRegistry: (() => void) | null = null;
  let resizeObserver: ResizeObserver | null = null;

  let gl: WebGLRenderingContext | null = null;
  let program: WebGLProgram | null = null;
  let positionBuffer: WebGLBuffer | null = null;
  let texCoordBuffer: WebGLBuffer | null = null;

  let positionLocation = -1;
  let texCoordLocation = -1;
  let resolutionLocation: WebGLUniformLocation | null = null;
  let sizeLocation: WebGLUniformLocation | null = null;
  let radiusLocation: WebGLUniformLocation | null = null;
  let alphaLocation: WebGLUniformLocation | null = null;

  const vertexShaderSource = `
    attribute vec2 a_position;
    attribute vec2 a_texCoord;
    uniform vec2 u_resolution;
    varying vec2 v_texCoord;

    void main() {
      vec2 zeroToOne = a_position / u_resolution;
      vec2 zeroToTwo = zeroToOne * 2.0;
      vec2 clipSpace = zeroToTwo - 1.0;
      gl_Position = vec4(clipSpace * vec2(1.0, -1.0), 0.0, 1.0);
      v_texCoord = a_texCoord;
    }
  `;

  const fragmentShaderSource = `
    precision mediump float;

    varying vec2 v_texCoord;
    uniform sampler2D u_texture;
    uniform vec2 u_size;
    uniform float u_radius;
    uniform float u_alpha;

    float roundedRectSDF(vec2 p, vec2 b, float r) {
      vec2 q = abs(p) - b + vec2(r);
      return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
    }

    void main() {
      vec4 color = texture2D(u_texture, v_texCoord);
      vec2 size = max(u_size, vec2(1.0));
      float radius = clamp(u_radius, 0.0, min(size.x, size.y) * 0.5 - 0.5);
      vec2 center = size * 0.5;
      vec2 p = v_texCoord * size - center;

      float sdf = roundedRectSDF(p, center, radius);
      float mask = 1.0 - smoothstep(0.0, 1.0, sdf);
      gl_FragColor = vec4(color.rgb, color.a * mask * u_alpha);
    }
  `;

  function metricsKey(slot: SlotState): string {
    return slot.cacheKey || `album:${slot.artistSort}||${slot.titleSort}`;
  }

  function lookupKeyForSlot(slot: Pick<SlotState, 'artistSort' | 'titleSort'>): string {
    return `${slot.artistSort}||${slot.titleSort}`;
  }

  const resolvedAlbumKeyCache = new Map<string, string>();
  const pendingAlbumKeyLookup = new Map<string, Promise<string | null>>();

  function getResolvedAlbumKey(lookupKey: string): string | null {
    const cached = resolvedAlbumKeyCache.get(lookupKey);
    if (!cached) {
      return null;
    }

    resolvedAlbumKeyCache.delete(lookupKey);
    resolvedAlbumKeyCache.set(lookupKey, cached);
    return cached;
  }

  function setResolvedAlbumKey(lookupKey: string, cacheKey: string): void {
    if (resolvedAlbumKeyCache.has(lookupKey)) {
      resolvedAlbumKeyCache.delete(lookupKey);
    }
    resolvedAlbumKeyCache.set(lookupKey, cacheKey);

    while (resolvedAlbumKeyCache.size > MAX_RESOLVED_ALBUM_KEYS) {
      const oldestKey = resolvedAlbumKeyCache.keys().next().value;
      if (!oldestKey) {
        break;
      }
      resolvedAlbumKeyCache.delete(oldestKey);
    }
  }

  function markLayoutDirty(): void {
    layoutDirty = true;
  }

  function refreshSlotRegistrySnapshot(): void {
    slotRegistrySnapshot = getArtworkSlotsSnapshot();
    markLayoutDirty();
  }

  function isDevBlobUrl(url: string | null | undefined): url is string {
    return typeof url === 'string' && url.startsWith('blob:');
  }

  function releaseIfDevBlobUrl(url: string | null | undefined): void {
    if (!isDevBlobUrl(url)) return;
    releaseDevArtworkUrl(url);
  }

  function buildProtocolThumbUrl(cacheKey: string, size: number): string {
    const encodedKey = encodeURIComponent(cacheKey);
    return `sermon-artwork://localhost/thumb/${encodedKey}?s=${size}`;
  }

  function buildProtocolLqipUrl(cacheKey: string): string {
    const encodedKey = encodeURIComponent(cacheKey);
    return `sermon-artwork://localhost/lqip/${encodedKey}`;
  }

  function disableSurface(reason: string): void {
    if (unsupportedDispatched) return;
    unsupportedDispatched = true;
    console.warn('Album artwork surface disabled:', reason);
    cleanupWebGL();
    dispatch('unsupported');
  }

  function compileShader(context: WebGLRenderingContext, type: number, source: string): WebGLShader | null {
    const shader = context.createShader(type);
    if (!shader) return null;

    context.shaderSource(shader, source);
    context.compileShader(shader);

    if (context.getShaderParameter(shader, context.COMPILE_STATUS)) {
      return shader;
    }

    console.warn('Album artwork surface shader compile failed:', context.getShaderInfoLog(shader));
    context.deleteShader(shader);
    return null;
  }

  function initializeWebGL(): boolean {
    if (!canvasEl) return false;

    gl = canvasEl.getContext('webgl', {
      alpha: true,
      antialias: false,
      depth: false,
      stencil: false,
      preserveDrawingBuffer: false,
      powerPreference: 'high-performance'
    });

    if (!gl) return false;

    const vertexShader = compileShader(gl, gl.VERTEX_SHADER, vertexShaderSource);
    const fragmentShader = compileShader(gl, gl.FRAGMENT_SHADER, fragmentShaderSource);
    if (!vertexShader || !fragmentShader) {
      return false;
    }

    program = gl.createProgram();
    if (!program) return false;

    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);

    gl.deleteShader(vertexShader);
    gl.deleteShader(fragmentShader);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      console.warn('Album artwork surface program link failed:', gl.getProgramInfoLog(program));
      return false;
    }

    positionLocation = gl.getAttribLocation(program, 'a_position');
    texCoordLocation = gl.getAttribLocation(program, 'a_texCoord');
    resolutionLocation = gl.getUniformLocation(program, 'u_resolution');
    sizeLocation = gl.getUniformLocation(program, 'u_size');
    radiusLocation = gl.getUniformLocation(program, 'u_radius');
    alphaLocation = gl.getUniformLocation(program, 'u_alpha');

    positionBuffer = gl.createBuffer();
    texCoordBuffer = gl.createBuffer();

    if (!positionBuffer || !texCoordBuffer || positionLocation < 0 || texCoordLocation < 0) {
      return false;
    }

    gl.bindBuffer(gl.ARRAY_BUFFER, texCoordBuffer);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([
        0, 0,
        1, 0,
        0, 1,
        0, 1,
        1, 0,
        1, 1
      ]),
      gl.STATIC_DRAW
    );

    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    gl.clearColor(0, 0, 0, 0);

    return true;
  }

  function releaseSlotUrls(slot: SlotState): void {
    if (slot.fullLoadCancel) {
      slot.fullLoadCancel();
      slot.fullLoadCancel = null;
    }

    if (slot.thumbUrl) {
      pendingFullLoads.delete(slot.thumbUrl);
    }

    if (slot.lqipUrl) {
      releaseIfDevBlobUrl(slot.lqipUrl);
      slot.lqipUrl = null;
    }
    if (slot.thumbUrl) {
      releaseIfDevBlobUrl(slot.thumbUrl);
      slot.thumbUrl = null;
    }
  }

  function detachLayoutObservers(): void {
    if (scrollerEl) {
      scrollerEl.removeEventListener('scroll', markLayoutDirty);
      scrollerEl = null;
    }
    if (resizeObserver) {
      resizeObserver.disconnect();
      resizeObserver = null;
    }
    if (unsubscribeSlotRegistry) {
      unsubscribeSlotRegistry();
      unsubscribeSlotRegistry = null;
    }
  }

  function cleanupWebGL(): void {
    if (rafId) {
      cancelAnimationFrame(rafId);
      rafId = 0;
    }

    detachLayoutObservers();

    for (const slot of slots.values()) {
      releaseSlotUrls(slot);
    }
    slots.clear();
    pendingFullLoads.clear();

    if (gl) {
      for (const record of textures.values()) {
        if (record.texture) {
          gl.deleteTexture(record.texture);
        }
      }

      if (positionBuffer) {
        gl.deleteBuffer(positionBuffer);
      }
      if (texCoordBuffer) {
        gl.deleteBuffer(texCoordBuffer);
      }
      if (program) {
        gl.deleteProgram(program);
      }
    }

    textures.clear();
    resolvedAlbumKeyCache.clear();
    pendingAlbumKeyLookup.clear();
    slotRegistrySnapshot.clear();
    gl = null;
    program = null;
    positionBuffer = null;
    texCoordBuffer = null;
    lastRenderAt = 0;
    layoutDirty = true;
    consecutiveTextureErrors = 0;
    totalTextureErrors = 0;
  }

  async function requestTexture(url: string, decode: boolean): Promise<void> {
    const existing = textures.get(url);
    if (existing?.status === 'ready') {
      existing.lastUsed = ++textureClock;
      return;
    }

    if (existing?.status === 'error') {
      return;
    }

    if (existing?.status === 'loading' && existing.promise) {
      return existing.promise;
    }

    const record: TextureRecord = {
      status: 'loading',
      texture: null,
      promise: null,
      lastUsed: textureClock
    };

    const promise = (async () => {
      const image = new Image();
      image.src = url;

      await new Promise<void>((resolve, reject) => {
        image.onload = () => resolve();
        image.onerror = () => reject(new Error(`Failed to load texture: ${url}`));
      });

      if (decode && 'decode' in image) {
        try {
          await image.decode();
        } catch {
          // ignore decode rejections after successful load
        }
      }

      if (!gl) {
        throw new Error('WebGL unavailable while uploading texture');
      }

      const texture = gl.createTexture();
      if (!texture) {
        throw new Error('Failed to create WebGL texture');
      }

      gl.bindTexture(gl.TEXTURE_2D, texture);
      gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, 0);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);

      record.texture = texture;
      record.status = 'ready';
      record.lastUsed = ++textureClock;
      consecutiveTextureErrors = 0;
    })()
      .catch((err) => {
        record.status = 'error';
        const reason = err instanceof Error ? err.message : String(err);
        console.warn('Album artwork surface texture load failed:', reason);
        consecutiveTextureErrors += 1;
        totalTextureErrors += 1;

        if (
          consecutiveTextureErrors >= MAX_CONSECUTIVE_TEXTURE_ERRORS ||
          totalTextureErrors >= MAX_TOTAL_TEXTURE_ERRORS
        ) {
          disableSurface(
            `repeated texture failures (${consecutiveTextureErrors} consecutive, ${totalTextureErrors} total)`
          );
        }
      })
      .finally(() => {
        record.promise = null;
      });

    record.promise = promise;
    textures.set(url, record);
    return promise;
  }

  function textureReady(url: string | null): boolean {
    if (!url) return false;
    return textures.get(url)?.status === 'ready';
  }

  async function resolveArtworkCacheKey(slot: SlotState): Promise<string | null> {
    if (slot.cacheKey) {
      setResolvedAlbumKey(lookupKeyForSlot(slot), slot.cacheKey);
      return slot.cacheKey;
    }

    if (slot.resolvedKey) {
      setResolvedAlbumKey(lookupKeyForSlot(slot), slot.resolvedKey);
      return slot.resolvedKey;
    }

    const lookupKey = lookupKeyForSlot(slot);
    const cached = getResolvedAlbumKey(lookupKey);
    if (cached) {
      return cached;
    }

    const existingLookup = pendingAlbumKeyLookup.get(lookupKey);
    if (existingLookup) {
      return existingLookup;
    }

    const task = (async () => {
      const best = await getArtworkBestForAlbum(slot.artistSort, slot.titleSort);
      const key = best.cacheKey ?? null;
      if (key) {
        setResolvedAlbumKey(lookupKey, key);
      }
      return key;
    })().finally(() => {
      pendingAlbumKeyLookup.delete(lookupKey);
    });

    pendingAlbumKeyLookup.set(lookupKey, task);
    return task;
  }

  async function ensureSlotUrls(slot: SlotState): Promise<void> {
    if (
      slot.urlsLoading ||
      (!slot.cacheKey && !slot.artistSort && !slot.titleSort) ||
      (slot.lqipUrl && slot.thumbUrl) ||
      (slot.lqipUrl && ultraFastMode && deferHighRes)
    ) {
      return;
    }

    slot.urlsLoading = true;

    try {
      const resolvedKey = await resolveArtworkCacheKey(slot);

      if (!resolvedKey) {
        slot.stage = 'none';
        recordArtworkError(metricsKey(slot));
        return;
      }

      const shouldResolveThumbUrl = !ultraFastMode;

      let lqipUrl: string | null = slot.lqipUrl;
      let thumbUrl: string | null = slot.thumbUrl;

      if (useDevThumbFallback) {
        if (!lqipUrl) {
          lqipUrl = await getDevArtworkUrl(resolvedKey, LQIP_SIZE);
        }
        if (!thumbUrl && shouldResolveThumbUrl) {
          thumbUrl = await getDevArtworkUrl(resolvedKey, THUMB_SIZE);
        }
      } else {
        lqipUrl = lqipUrl ?? buildProtocolLqipUrl(resolvedKey);
        thumbUrl = thumbUrl ?? (shouldResolveThumbUrl ? buildProtocolThumbUrl(resolvedKey, THUMB_SIZE) : null);
      }

      const liveSlot = slots.get(slot.id);
      if (!liveSlot) {
        if (lqipUrl && lqipUrl !== slot.lqipUrl) {
          releaseIfDevBlobUrl(lqipUrl);
        }
        if (thumbUrl && thumbUrl !== slot.thumbUrl) {
          releaseIfDevBlobUrl(thumbUrl);
        }
        return;
      }

      if (lqipUrl && liveSlot.lqipUrl !== lqipUrl) {
        if (liveSlot.lqipUrl) {
          releaseIfDevBlobUrl(liveSlot.lqipUrl);
        }
        liveSlot.lqipUrl = lqipUrl;
      }

      if (thumbUrl && liveSlot.thumbUrl !== thumbUrl) {
        if (liveSlot.thumbUrl) {
          releaseIfDevBlobUrl(liveSlot.thumbUrl);
        }
        liveSlot.thumbUrl = thumbUrl;
      }

      liveSlot.resolvedKey = resolvedKey;

      if (liveSlot.stage === 'none') {
        liveSlot.stage = 'lqip';
      }

      if (liveSlot.stage !== 'full') {
        recordLqipLoaded(metricsKey(liveSlot));
      }
    } catch (err) {
      slot.stage = 'none';
      recordArtworkError(metricsKey(slot));
      const reason = err instanceof Error ? err.message : String(err);
      console.warn('Album artwork surface URL resolve failed:', reason);
    } finally {
      slot.urlsLoading = false;
    }
  }

  function ensureLqipTexture(slot: SlotState): void {
    if (!slot.lqipUrl || textureReady(slot.lqipUrl)) return;
    void requestTexture(slot.lqipUrl, false);
  }

  function ensureFullTexture(slot: SlotState, allowWhileDeferred = false): void {
    const thumbUrl = slot.thumbUrl;
    if (!thumbUrl || textureReady(thumbUrl)) {
      if (thumbUrl && textureReady(thumbUrl)) {
        slot.stage = 'full';
      }
      return;
    }

    if ((!allowWhileDeferred && deferHighRes) || pendingFullLoads.has(thumbUrl) || slot.fullLoadCancel) {
      return;
    }

    pendingFullLoads.add(thumbUrl);
    const key = metricsKey(slot);
    let cancelled = false;
    let cancelTask: (() => void) | null = null;

    const clearTaskState = () => {
      pendingFullLoads.delete(thumbUrl);
      if (cancelTask && slot.fullLoadCancel === cancelTask) {
        slot.fullLoadCancel = null;
      }
    };

    const run = async () => {
      try {
        if (cancelled) {
          return;
        }

        await requestTexture(thumbUrl, true);
        if (cancelled) {
          return;
        }

        const liveSlot = slots.get(slot.id);
        if (!liveSlot || liveSlot.thumbUrl !== thumbUrl) {
          return;
        }

        if (!textureReady(thumbUrl)) {
          recordArtworkError(key);
          return;
        }

        markThumbReady(thumbUrl);
        liveSlot.stage = 'full';
        recordThumbLoaded(key);
      } finally {
        clearTaskState();
      }
    };

    const cancelDecode = enqueueArtworkDecode(run);
    cancelTask = () => {
      cancelled = true;
      cancelDecode();
      clearTaskState();
    };

    slot.fullLoadCancel = cancelTask;
  }

  function preloadMarginPx(): number {
    if (ultraFastMode) return PRELOAD_MARGIN_ULTRA_PX;
    if (deferHighRes) return PRELOAD_MARGIN_DEFER_PX;
    return PRELOAD_MARGIN_IDLE_PX;
  }

  function fullResMarginPx(): number {
    if (ultraFastMode) return FULL_RES_MARGIN_ULTRA_PX;
    if (deferHighRes) return FULL_RES_MARGIN_DEFER_PX;
    return FULL_RES_MARGIN_IDLE_PX;
  }

  function intersects(
    x: number,
    y: number,
    width: number,
    height: number,
    maxWidth: number,
    maxHeight: number,
    marginPx = 0
  ): boolean {
    return !(
      x + width < -marginPx ||
      y + height < -marginPx ||
      x > maxWidth + marginPx ||
      y > maxHeight + marginPx
    );
  }

  function syncSlots(): { visible: SlotState[]; keepUrls: Set<string> } {
    if (!wrapperEl) return { visible: [], keepUrls: new Set<string>() };

    frameIndex += 1;
    const wrapperRect = wrapperEl.getBoundingClientRect();
    const preloadMargin = preloadMarginPx();
    const fullResMargin = fullResMarginPx();
    const visible: SlotState[] = [];
    const keepUrls = new Set<string>();
    const seenIds = new Set<string>();

    for (const [id, registeredSlot] of slotRegistrySnapshot) {
      seenIds.add(id);

      let slot = slots.get(id);
      if (!slot) {
        slot = {
          id,
          cacheKey: registeredSlot.cacheKey,
          artistSort: registeredSlot.artistSort,
          titleSort: registeredSlot.titleSort,
          resolvedKey: registeredSlot.cacheKey || null,
          lqipUrl: null,
          thumbUrl: null,
          urlsLoading: false,
          stage: 'none',
          x: 0,
          y: 0,
          width: 0,
          height: 0,
          lastSeen: frameIndex,
           inViewport: false,
           inPreloadZone: false,
           inFullResZone: false,
           element: registeredSlot.element,
           fullLoadCancel: null
         };
        slots.set(id, slot);
      }

      const cacheKeyChanged = slot.cacheKey !== registeredSlot.cacheKey;
      const artistChanged = slot.artistSort !== registeredSlot.artistSort;
      const titleChanged = slot.titleSort !== registeredSlot.titleSort;
      if (cacheKeyChanged || artistChanged || titleChanged) {
        const previousThumbUrl = slot.thumbUrl;
        slot.cacheKey = registeredSlot.cacheKey;
        slot.artistSort = registeredSlot.artistSort;
        slot.titleSort = registeredSlot.titleSort;
        slot.resolvedKey = registeredSlot.cacheKey || null;
        slot.stage = 'none';
        releaseSlotUrls(slot);
        if (previousThumbUrl) {
          pendingFullLoads.delete(previousThumbUrl);
        }
      }

      slot.lastSeen = frameIndex;
      const elementChanged = slot.element !== registeredSlot.element;
      slot.element = registeredSlot.element;

      if (layoutDirty || elementChanged || slot.width <= 1 || slot.height <= 1) {
        const rect = slot.element.getBoundingClientRect();
        const x = rect.left - wrapperRect.left;
        const y = rect.top - wrapperRect.top;

        slot.x = x;
        slot.y = y;
        slot.width = rect.width;
        slot.height = rect.height;

        if (rect.width <= 1 || rect.height <= 1) {
          slot.inViewport = false;
          slot.inPreloadZone = false;
          slot.inFullResZone = false;
        } else {
          slot.inViewport = intersects(
            x,
            y,
            rect.width,
            rect.height,
            wrapperRect.width,
            wrapperRect.height
          );
          slot.inPreloadZone = intersects(
            x,
            y,
            rect.width,
            rect.height,
            wrapperRect.width,
            wrapperRect.height,
            preloadMargin
          );
          slot.inFullResZone = intersects(
            x,
            y,
            rect.width,
            rect.height,
            wrapperRect.width,
            wrapperRect.height,
            fullResMargin
          );
        }
      }

      if (!slot.inPreloadZone) {
        slot.element.dataset.artworkStage = slot.stage;
        continue;
      }

      void ensureSlotUrls(slot);
      ensureLqipTexture(slot);

      if (slot.thumbUrl && hasReadyThumb(slot.thumbUrl) && textureReady(slot.thumbUrl)) {
        slot.stage = 'full';
      }

      if (slot.inViewport) {
        if (!ultraFastMode) {
          ensureFullTexture(slot, true);
        }
      } else if (!deferHighRes && slot.inFullResZone) {
        ensureFullTexture(slot);
      }

      if (slot.lqipUrl) {
        keepUrls.add(slot.lqipUrl);
      }
      if (slot.thumbUrl) {
        keepUrls.add(slot.thumbUrl);
      }

      slot.element.dataset.artworkStage = slot.stage;
      if (slot.inViewport) {
        visible.push(slot);
      }
    }

    for (const [id, slot] of slots) {
      if (seenIds.has(id)) continue;
      if (frameIndex - slot.lastSeen <= STALE_SLOT_FRAMES) continue;
      const previousThumbUrl = slot.thumbUrl;
      releaseSlotUrls(slot);
      if (previousThumbUrl) {
        pendingFullLoads.delete(previousThumbUrl);
      }
      slots.delete(id);
    }

    layoutDirty = false;

    return { visible, keepUrls };
  }

  function evictTextures(keepUrls: Set<string>): void {
    if (!gl || textures.size <= MAX_TEXTURES) return;

    const candidates = [...textures.entries()]
      .filter(([url, record]) => !keepUrls.has(url) && record.status !== 'loading')
      .sort((a, b) => a[1].lastUsed - b[1].lastUsed);

    while (textures.size > MAX_TEXTURES && candidates.length > 0) {
      const [url, record] = candidates.shift()!;
      if (record.texture) {
        gl.deleteTexture(record.texture);
      }
      textures.delete(url);
      pendingFullLoads.delete(url);
    }
  }

  function drawTexture(slot: SlotState, url: string, dpr: number): void {
    if (!gl || !program || !positionBuffer || !texCoordBuffer) return;

    const record = textures.get(url);
    if (!record || record.status !== 'ready' || !record.texture) return;

    const x = slot.x * dpr;
    const y = slot.y * dpr;
    const width = slot.width * dpr;
    const height = slot.height * dpr;
    if (width < 1 || height < 1) return;

    gl.useProgram(program);

    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([
        x, y,
        x + width, y,
        x, y + height,
        x, y + height,
        x + width, y,
        x + width, y + height
      ]),
      gl.STREAM_DRAW
    );

    gl.enableVertexAttribArray(positionLocation);
    gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, 0, 0);

    gl.bindBuffer(gl.ARRAY_BUFFER, texCoordBuffer);
    gl.enableVertexAttribArray(texCoordLocation);
    gl.vertexAttribPointer(texCoordLocation, 2, gl.FLOAT, false, 0, 0);

    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, record.texture);

    gl.uniform2f(resolutionLocation, gl.canvas.width, gl.canvas.height);
    gl.uniform2f(sizeLocation, width, height);
    gl.uniform1f(radiusLocation, artworkRadiusPx * dpr);
    gl.uniform1f(alphaLocation, slot.stage === 'lqip' ? 0.86 : 1);

    gl.drawArrays(gl.TRIANGLES, 0, 6);
    record.lastUsed = ++textureClock;
  }

  function renderFrame(): void {
    if (!enabled || !canvasEl || !wrapperEl || !gl) {
      rafId = requestAnimationFrame(renderFrame);
      return;
    }

    const now = performance.now();
    if (ultraFastMode && now - lastRenderAt < ULTRA_FAST_FRAME_INTERVAL_MS) {
      rafId = requestAnimationFrame(renderFrame);
      return;
    }
    lastRenderAt = now;

    const deviceDpr = window.devicePixelRatio || 1;
    const dpr = ultraFastMode ? Math.max(0.75, deviceDpr * 0.82) : deviceDpr;
    const width = Math.max(1, Math.round(wrapperEl.clientWidth * dpr));
    const height = Math.max(1, Math.round(wrapperEl.clientHeight * dpr));

    if (canvasEl.width !== width || canvasEl.height !== height) {
      canvasEl.width = width;
      canvasEl.height = height;
    }

    gl.viewport(0, 0, width, height);
    gl.clear(gl.COLOR_BUFFER_BIT);

    const { visible: visibleSlots, keepUrls } = syncSlots();

    for (const slot of visibleSlots) {
      let drawUrl: string | null = null;

      if (slot.stage === 'full' && textureReady(slot.thumbUrl)) {
        drawUrl = slot.thumbUrl;
      } else if (textureReady(slot.lqipUrl)) {
        drawUrl = slot.lqipUrl;
      } else if (textureReady(slot.thumbUrl)) {
        drawUrl = slot.thumbUrl;
      }

      if (!drawUrl) continue;

      drawTexture(slot, drawUrl, dpr);
    }

    evictTextures(keepUrls);
    rafId = requestAnimationFrame(renderFrame);
  }

  $effect(() => {
    if (!canvasEl || !wrapperEl || !enabled) return;

    unsupportedDispatched = false;

    refreshSlotRegistrySnapshot();
    unsubscribeSlotRegistry = subscribeArtworkSlots(refreshSlotRegistrySnapshot);

    const nextScroller = wrapperEl.firstElementChild;
    if (nextScroller instanceof HTMLElement) {
      scrollerEl = nextScroller;
      scrollerEl.addEventListener('scroll', markLayoutDirty, { passive: true });
    }

    resizeObserver = new ResizeObserver(() => {
      markLayoutDirty();
    });
    resizeObserver.observe(wrapperEl);
    if (scrollerEl) {
      resizeObserver.observe(scrollerEl);
    }
    markLayoutDirty();

    if (!initializeWebGL()) {
      disableSurface('webgl initialization failed');
      return;
    }

    rafId = requestAnimationFrame(renderFrame);

    return () => {
      cleanupWebGL();
    };
  });
</script>

<canvas bind:this={canvasEl} class="artwork-surface" aria-hidden="true"></canvas>

<style>
  .artwork-surface {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 2;
  }
</style>
