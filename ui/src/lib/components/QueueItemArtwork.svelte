<script lang="ts">
  import ArtworkImage from './ArtworkImage.svelte';

  interface Props {
    trackId?: number;
    cacheKey?: string | null;
    size?: number;
  }
  
  let { trackId, cacheKey, size = 40 }: Props = $props();
</script>

<div class="q-thumb-wrapper" style="width: {size}px; height: {size}px;">
  <ArtworkImage {cacheKey} {size} class="q-thumb-inner" />
</div>

<style>
  .q-thumb-wrapper {
    flex-shrink: 0;
    border-radius: 4px;
    overflow: hidden;
    background: #222;
  }

  :global(.q-thumb-inner) {
    border-radius: 4px;
  }
</style>
