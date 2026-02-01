<script lang="ts">
  import { currentRouteName } from '../state/route';
  import { currentArtworkUrl } from '../state/artwork';

  const showWash = $derived($currentRouteName === 'now-playing');
</script>

<div class="background-layer">
  {#if showWash && $currentArtworkUrl}
    <div 
      class="artwork-wash" 
      style:background-image="url('{$currentArtworkUrl}')"
    ></div>
    <div class="vignette"></div>
  {/if}
</div>

<style>
  .background-layer {
    position: fixed;
    inset: 0;
    z-index: -1;
    background: #0a0a0a;
  }
  
  .artwork-wash {
    position: absolute;
    inset: 0;
    background-size: cover;
    background-position: center;
    filter: blur(60px) saturate(0.5);
    opacity: 0.15;
    transform: scale(1.2);
  }
  
  .vignette {
    position: absolute;
    inset: 0;
    background: radial-gradient(
      ellipse at center,
      transparent 0%,
      rgba(10, 10, 10, 0.8) 70%,
      #0a0a0a 100%
    );
  }
</style>
