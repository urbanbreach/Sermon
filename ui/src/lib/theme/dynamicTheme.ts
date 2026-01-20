/**
 * Dynamic Theme Engine
 * 
 * Derives accent color and background gradient from album artwork.
 * Algorithm is deterministic for snapshot reproducibility.
 */

export interface ThemeColors {
  accent: [number, number, number];
  bg0: [number, number, number];
  bg1: [number, number, number];
}

// Default fallback theme (used when no pixels remain after filtering)
const FALLBACK_THEME: ThemeColors = {
  accent: [74, 175, 255],
  bg0: [24, 32, 44],
  bg1: [10, 10, 10],
};

/**
 * RGB to HSV conversion (exact algorithm per spec)
 */
function rgbToHsv(r: number, g: number, b: number): { h: number; s: number; v: number } {
  const rNorm = r / 255;
  const gNorm = g / 255;
  const bNorm = b / 255;

  const max = Math.max(rNorm, gNorm, bNorm);
  const min = Math.min(rNorm, gNorm, bNorm);
  const delta = max - min;

  const v = max;
  const s = max === 0 ? 0 : delta / max;

  let h = 0;
  if (delta !== 0) {
    if (max === rNorm) {
      h = 60 * (((gNorm - bNorm) / delta) % 6);
    } else if (max === gNorm) {
      h = 60 * ((bNorm - rNorm) / delta + 2);
    } else {
      h = 60 * ((rNorm - gNorm) / delta + 4);
    }
    if (h < 0) h += 360;
  }

  return { h, s, v };
}

/**
 * HSV to RGB conversion (exact algorithm per spec)
 */
function hsvToRgb(h: number, s: number, v: number): [number, number, number] {
  const c = v * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = v - c;

  let r1 = 0, g1 = 0, b1 = 0;

  if (h >= 0 && h < 60) {
    r1 = c; g1 = x; b1 = 0;
  } else if (h >= 60 && h < 120) {
    r1 = x; g1 = c; b1 = 0;
  } else if (h >= 120 && h < 180) {
    r1 = 0; g1 = c; b1 = x;
  } else if (h >= 180 && h < 240) {
    r1 = 0; g1 = x; b1 = c;
  } else if (h >= 240 && h < 300) {
    r1 = x; g1 = 0; b1 = c;
  } else {
    r1 = c; g1 = 0; b1 = x;
  }

  return [
    Math.round(Math.min(255, Math.max(0, (r1 + m) * 255))),
    Math.round(Math.min(255, Math.max(0, (g1 + m) * 255))),
    Math.round(Math.min(255, Math.max(0, (b1 + m) * 255))),
  ];
}

/**
 * Compute theme from image source URL
 * Works with both asset URLs (snapshot mode) and data URLs (runtime mode)
 */
export async function computeThemeFromImageSrc(src: string): Promise<ThemeColors> {
  // Load image
  const img = new Image();
  img.crossOrigin = 'anonymous';
  
  await new Promise<void>((resolve, reject) => {
    img.onload = () => resolve();
    img.onerror = () => reject(new Error('Failed to load image'));
    img.src = src;
  });

  // Wait for decode to ensure deterministic rendering
  if (img.decode) {
    await img.decode();
  }

  // Create offscreen canvas at fixed working size: 48x48
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d');
  if (!ctx) {
    return FALLBACK_THEME;
  }

  const size = 48;
  canvas.width = size;
  canvas.height = size;
  ctx.drawImage(img, 0, 0, size, size);

  // Get pixel data
  const imageData = ctx.getImageData(0, 0, size, size);
  const data = imageData.data;

  // Sample pixels on fixed grid: every 3px (16x16 samples)
  interface PixelSample {
    r: number;
    g: number;
    b: number;
    h: number;
    s: number;
  }

  const samples: PixelSample[] = [];
  const step = 3;

  for (let y = 0; y < size; y += step) {
    for (let x = 0; x < size; x += step) {
      const i = (y * size + x) * 4;
      const r = data[i];
      const g = data[i + 1];
      const b = data[i + 2];
      const a = data[i + 3];

      // Alpha filter: discard pixels where alpha < 204 (0.8)
      if (a < 204) continue;

      // Relative luminance filter
      const lum = 0.2126 * (r / 255) + 0.7152 * (g / 255) + 0.0722 * (b / 255);
      if (lum < 0.05 || lum > 0.95) continue;

      const { h, s } = rgbToHsv(r, g, b);
      samples.push({ r, g, b, h, s });
    }
  }

  // If no pixels remain after filtering, use fallback
  if (samples.length === 0) {
    return FALLBACK_THEME;
  }

  // Build hue histogram with 36 buckets (10 degrees each)
  const bucketCount = 36;
  const buckets: { samples: PixelSample[]; sumS: number }[] = [];
  for (let i = 0; i < bucketCount; i++) {
    buckets.push({ samples: [], sumS: 0 });
  }

  for (const sample of samples) {
    const bucketIndex = Math.min(Math.floor(sample.h / 10), bucketCount - 1);
    buckets[bucketIndex].samples.push(sample);
    buckets[bucketIndex].sumS += sample.s;
  }

  // Pick dominant bucket by count; tie-break by higher mean S; then lower index
  let dominantBucket = 0;
  let maxCount = 0;
  let maxMeanS = 0;

  for (let i = 0; i < bucketCount; i++) {
    const count = buckets[i].samples.length;
    if (count === 0) continue;

    const meanS = buckets[i].sumS / count;

    if (count > maxCount || (count === maxCount && meanS > maxMeanS)) {
      dominantBucket = i;
      maxCount = count;
      maxMeanS = meanS;
    }
  }

  // Compute accent RGB as mean of original RGB bytes in dominant bucket
  const dominantSamples = buckets[dominantBucket].samples;
  if (dominantSamples.length === 0) {
    return FALLBACK_THEME;
  }

  let sumR = 0, sumG = 0, sumB = 0;
  for (const s of dominantSamples) {
    sumR += s.r;
    sumG += s.g;
    sumB += s.b;
  }

  let accentR = Math.round(sumR / dominantSamples.length);
  let accentG = Math.round(sumG / dominantSamples.length);
  let accentB = Math.round(sumB / dominantSamples.length);

  // Saturation clamp: if S > 0.85, set S = 0.85 and recompute RGB
  const accentHsv = rgbToHsv(accentR, accentG, accentB);
  if (accentHsv.s > 0.85) {
    [accentR, accentG, accentB] = hsvToRgb(accentHsv.h, 0.85, accentHsv.v);
  }

  // Gradient stops: multiply RGB bytes by factors
  const bg0: [number, number, number] = [
    Math.round(accentR * 0.65),
    Math.round(accentG * 0.65),
    Math.round(accentB * 0.65),
  ];

  const bg1: [number, number, number] = [
    Math.round(accentR * 0.30),
    Math.round(accentG * 0.30),
    Math.round(accentB * 0.30),
  ];

  return {
    accent: [accentR, accentG, accentB],
    bg0,
    bg1,
  };
}

/**
 * Apply theme colors to document via CSS variables
 */
export function applyThemeToDocument(theme: ThemeColors): void {
  const root = document.documentElement;
  
  root.style.setProperty('--theme-accent', `rgb(${theme.accent.join(',')})`);
  root.style.setProperty('--theme-bg-0', `rgb(${theme.bg0.join(',')})`);
  root.style.setProperty('--theme-bg-1', `rgb(${theme.bg1.join(',')})`);
  
  // Also set individual RGB values for alpha variations
  root.style.setProperty('--theme-accent-r', String(theme.accent[0]));
  root.style.setProperty('--theme-accent-g', String(theme.accent[1]));
  root.style.setProperty('--theme-accent-b', String(theme.accent[2]));
}

/**
 * Reset theme to defaults
 */
export function resetTheme(): void {
  applyThemeToDocument(FALLBACK_THEME);
}
