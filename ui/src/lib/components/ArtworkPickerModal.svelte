<script lang="ts">
  import Modal from './Modal.svelte';
  import { searchArtworkCandidates, selectArtworkCandidateForAlbum } from '../api/artwork';
  import { Fixtures } from '../data/fixtures';
  import type { ArtworkCandidate } from '../types/artwork';

  interface Props {
    open: boolean;
    albumArtistSort: string;
    albumTitleSort: string;
    albumArtistDisplay: string;
    albumTitleDisplay: string;
    onclose: () => void;
    onselect: (cacheKey: string) => void;
  }

  let { open, albumArtistSort, albumTitleSort, albumArtistDisplay, albumTitleDisplay, onclose, onselect }: Props = $props();

  let candidates: ArtworkCandidate[] = $state([]);
  let loading = $state(false);
  let selecting = $state(false);
  let error: string | null = $state(null);
  let selectedIndex: number | null = $state(null);

  // Load candidates when modal opens
  $effect(() => {
    if (open) {
      loadCandidates();
    } else {
      // Reset state when closed
      candidates = [];
      error = null;
      selectedIndex = null;
    }
  });

  async function loadCandidates() {
    loading = true;
    error = null;
    
    try {
      const response = await searchArtworkCandidates(albumArtistDisplay, albumTitleDisplay);
      candidates = response.candidates;
    } catch (e) {
      console.error('Failed to search artwork candidates:', e);
      error = 'Failed to load artwork options';
    } finally {
      loading = false;
    }
  }

  function getCandidateImageUrl(candidate: ArtworkCandidate): string {
    // In snapshot mode, backend returns simple filenames that need fixture resolution
    if (import.meta.env.SERMON_MOCK === '1') {
      const fixtureUrl = Fixtures.getArtworkPath(candidate.imageUrl);
      if (fixtureUrl) return fixtureUrl;
    }
    return candidate.imageUrl;
  }

  async function handleSelect(candidate: ArtworkCandidate, index: number) {
    selectedIndex = index;
    selecting = true;
    error = null;

    try {
      const result = await selectArtworkCandidateForAlbum(
        albumArtistSort,
        albumTitleSort,
        candidate.provider,
        candidate.providerItemId,
        candidate.imageUrl
      );
      onselect(result.cacheKey);
      onclose();
    } catch (e) {
      console.error('Failed to select artwork:', e);
      error = 'Failed to save artwork selection';
      selectedIndex = null;
    } finally {
      selecting = false;
    }
  }
</script>

<Modal {open} title="Choose Artwork" {onclose} preventClose={selecting}>
  <div class="picker-content">
    <div class="album-info">
      <strong>{albumTitleDisplay}</strong>
      <span class="artist">{albumArtistDisplay}</span>
    </div>

    {#if loading}
      <div class="loading">Searching for artwork...</div>
    {:else if error}
      <div class="error">{error}</div>
    {:else if candidates.length === 0}
      <div class="empty">No artwork found</div>
    {:else}
      <div class="candidates-grid">
        {#each candidates as candidate, i}
          <button
            class="candidate"
            class:selected={selectedIndex === i}
            onclick={() => handleSelect(candidate, i)}
            disabled={selecting}
          >
            <img src={getCandidateImageUrl(candidate)} alt="Artwork option" loading="lazy" />
            <div class="provider-badge">{candidate.provider}</div>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</Modal>

<style>
  .picker-content {
    min-width: 400px;
    max-width: 600px;
  }

  .album-info {
    margin-bottom: 1.5rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--glass-border);
  }

  .album-info strong {
    display: block;
    font-size: 1.1rem;
    margin-bottom: 0.25rem;
  }

  .album-info .artist {
    color: #888;
    font-size: 0.9rem;
  }

  .loading, .error, .empty {
    text-align: center;
    padding: 3rem;
    color: #888;
  }

  .error {
    color: #f55;
  }

  .candidates-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 1rem;
  }

  .candidate {
    position: relative;
    background: transparent;
    border: 2px solid transparent;
    border-radius: 8px;
    padding: 0;
    cursor: pointer;
    overflow: hidden;
    aspect-ratio: 1;
    transition: border-color 0.2s, transform 0.2s;
  }

  .candidate:hover:not(:disabled) {
    border-color: rgba(74, 175, 255, 0.5);
    transform: scale(1.02);
  }

  .candidate.selected {
    border-color: #4af;
  }

  .candidate:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .candidate img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 6px;
  }

  .provider-badge {
    position: absolute;
    bottom: 4px;
    right: 4px;
    background: rgba(0, 0, 0, 0.8);
    color: #aaa;
    font-size: 0.65rem;
    padding: 2px 6px;
    border-radius: 4px;
    text-transform: uppercase;
  }
</style>
