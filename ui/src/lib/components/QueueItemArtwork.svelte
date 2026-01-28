<script lang="ts">
  import { onMount } from 'svelte';
  import { getArtworkBestForTrack, getArtworkBytes } from '../api/artwork';
  import { queueArtworkCache, setQueueArtworkCache } from '../state/queueArtwork';
  import { Fixtures } from '../data/fixtures';
  
  interface Props {
    trackId: number;
    size?: number;
  }
  
  let { trackId, size = 40 }: Props = $props();
  
  let artworkUrl = $state<string | null>(null);
  let loading = $state(true);
  
  // Check cache first, then fetch if needed
  $effect(() => {
    const cached = $queueArtworkCache.get(trackId);
    if (cached !== undefined) {
      // Cache hit (could be url or null for "no artwork")
      artworkUrl = cached;
      loading = false;
    } else {
      // Cache miss - fetch artwork
      loading = true;
      fetchArtwork(trackId);
    }
  });
  
  async function fetchArtwork(id: number) {
    try {
      // Mock mode check
      if (import.meta.env.SERMON_MOCK === '1') {
        // In mock mode, we don't have track-to-album mapping readily available
        // Just set null for now (could be enhanced later)
        setQueueArtworkCache(id, null);
        artworkUrl = null;
        loading = false;
        return;
      }
      
      const best = await getArtworkBestForTrack(id);
      if (best.source !== 'none' && best.cacheKey && best.mime) {
        const bytes = await getArtworkBytes(best.cacheKey, best.mime);
        const url = `data:${bytes.mime};base64,${bytes.bytesBase64}`;
        setQueueArtworkCache(id, url);
        artworkUrl = url;
      } else {
        // No artwork available
        setQueueArtworkCache(id, null);
        artworkUrl = null;
      }
    } catch (e) {
      console.error('Failed to fetch queue item artwork:', e);
      setQueueArtworkCache(id, null);
      artworkUrl = null;
    } finally {
      loading = false;
    }
  }
</script>

{#if artworkUrl}
  <img 
    src={artworkUrl} 
    alt="" 
    class="q-thumb"
    style="width: {size}px; height: {size}px;"
  />
{:else}
  <div 
    class="q-thumb-placeholder"
    style="width: {size}px; height: {size}px;"
  ></div>
{/if}

<style>
  .q-thumb {
    border-radius: 4px;
    object-fit: cover;
    flex-shrink: 0;
    background: #222;
  }
  
  .q-thumb-placeholder {
    border-radius: 4px;
    background: linear-gradient(135deg, #2a2a2a, #1a1a1a);
    flex-shrink: 0;
  }
</style>
