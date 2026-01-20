<script lang="ts">
  import { currentArtworkUrl } from '../state/artwork';
  import { computeThemeFromImageSrc, applyThemeToDocument, resetTheme } from '../theme/dynamicTheme';
  import { reduceEffects } from '../state/effects';
  
  let prevArtwork = $state('');
  let isTransitioning = $state(false);
  
  $effect(() => {
    if ($currentArtworkUrl && $currentArtworkUrl !== prevArtwork) {
      handleArtworkChange($currentArtworkUrl);
    } else if (!$currentArtworkUrl && prevArtwork) {
      resetTheme();
      prevArtwork = '';
    }
  });
  
  async function handleArtworkChange(url: string) {
    isTransitioning = true;
    try {
      const theme = await computeThemeFromImageSrc(url);
      applyThemeToDocument(theme);
    } catch (e) {
      console.error('Theme extraction failed:', e);
      resetTheme();
    }
    prevArtwork = url;
    // Allow crossfade to complete
    setTimeout(() => isTransitioning = false, 1200);
  }
</script>

<div class="background-container" class:transitioning={isTransitioning} class:reduce-effects={$reduceEffects}>
  <!-- Base dark layer -->
  <div class="bg-base"></div>
  
  <!-- Primary glow (center) -->
  <div class="bg-glow bg-glow-primary"></div>
  
  <!-- Secondary glow (top-left) -->
  <div class="bg-glow bg-glow-secondary"></div>
  
  <!-- Tertiary glow (bottom-right) -->
  <div class="bg-glow bg-glow-tertiary"></div>
  
  <!-- Noise overlay -->
  <div class="bg-noise"></div>
  
  <!-- Dark gradient overlay for readability -->
  <div class="bg-overlay"></div>
</div>

<style>
  .background-container {
    position: fixed;
    inset: 0;
    z-index: -1;
    pointer-events: none;
    overflow: hidden;
  }
  
  .bg-base {
    position: absolute;
    inset: 0;
    background: #0a0a0a;
  }
  
  .bg-glow {
    position: absolute;
    inset: -50%;
    opacity: var(--bg-intensity, 0.35);
    transition: opacity calc(var(--bg-crossfade-ms, 1200) * 1ms) ease-in-out,
                background calc(var(--bg-crossfade-ms, 1200) * 1ms) ease-in-out;
  }
  
  .bg-glow-primary {
    background: radial-gradient(
      ellipse 80% 60% at 50% 40%,
      rgba(var(--theme-accent-r, 74), var(--theme-accent-g, 175), var(--theme-accent-b, 255), 0.4) 0%,
      transparent 70%
    );
  }
  
  .bg-glow-secondary {
    background: radial-gradient(
      ellipse 60% 50% at 20% 20%,
      rgba(var(--theme-accent-2-r, 100), var(--theme-accent-2-g, 180), var(--theme-accent-2-b, 255), 0.25) 0%,
      transparent 60%
    );
  }
  
  .bg-glow-tertiary {
    background: radial-gradient(
      ellipse 50% 40% at 80% 80%,
      rgba(var(--theme-accent-3-r, 50), var(--theme-accent-3-g, 150), var(--theme-accent-3-b, 230), 0.2) 0%,
      transparent 50%
    );
  }
  
  .bg-noise {
    position: absolute;
    inset: 0;
    opacity: var(--bg-noise-opacity, 0.18);
    background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.8' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)'/%3E%3C/svg%3E");
    background-repeat: repeat;
    mix-blend-mode: overlay;
    pointer-events: none;
  }
  
  .bg-overlay {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      to bottom,
      rgba(0, 0, 0, 0.3) 0%,
      rgba(0, 0, 0, 0.6) 100%
    );
  }
  
  .reduce-effects .bg-glow {
    display: none;
  }
  
  .reduce-effects .bg-noise {
    display: none;
  }
</style>
