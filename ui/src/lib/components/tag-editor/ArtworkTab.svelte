<script lang="ts">
  import { Loader2, Search } from '@lucide/svelte';
  import type { TrackTagSnapshot } from '../../types/library';
  import {
    getArtworkBestForTrack,
    getArtworkBytes,
    searchArtworkCandidates,
    selectArtworkCandidateForAlbum,
    embedArtworkToFile,
  } from '../../api/artwork';
  import type { ArtworkCandidate } from '../../types/artwork';
  import { Fixtures } from '../../data/fixtures';

  interface Props {
    tracks: TrackTagSnapshot[];
    disabled?: boolean;
  }

  let { tracks, disabled = false }: Props = $props();

  let currentArtworkUrl: string | null = $state(null);
  let loadingCurrent = $state(false);

  let pictureType = $state('Album Cover');
  let comments = $state('');
  let primaryPicture = $state(true);

  let isSearchOpen = $state(false);
  let searchQuery = $state('');
  let candidates: ArtworkCandidate[] = $state([]);
  let searching = $state(false);
  let searchError: string | null = $state(null);

  let confirmCandidate: ArtworkCandidate | null = $state(null);
  let embedding = $state(false);
  let embedProgress = $state(0);
  let embedTotal = $state(0);
  let embedError: string | null = $state(null);

  $effect(() => {
    if (tracks.length > 0) {
      void loadCurrentArtwork();
      const first = tracks[0];
      searchQuery = `${first.artist ?? ''} ${first.album ?? ''}`.trim();
    }
  });

  async function loadCurrentArtwork(): Promise<void> {
    if (tracks.length === 0) {
      return;
    }

    loadingCurrent = true;

    try {
      const best = await getArtworkBestForTrack(tracks[0].trackId);
      if (!best.cacheKey) {
        currentArtworkUrl = null;
        return;
      }

      const bytes = await getArtworkBytes(best.cacheKey, best.mime || 'image/jpeg');
      currentArtworkUrl = `data:${bytes.mime};base64,${bytes.bytesBase64}`;
    } catch {
      currentArtworkUrl = null;
    } finally {
      loadingCurrent = false;
    }
  }

  async function handleSearch(): Promise<void> {
    if (!searchQuery.trim()) {
      return;
    }

    searching = true;
    searchError = null;

    try {
      const response = await searchArtworkCandidates(undefined, searchQuery, undefined);
      candidates = response.candidates;
    } catch {
      searchError = 'Failed to search artwork';
      candidates = [];
    } finally {
      searching = false;
    }
  }

  function getCandidateImageUrl(candidate: ArtworkCandidate): string {
    if (import.meta.env.SERMON_MOCK === '1') {
      const fixtureUrl = Fixtures.getArtworkPath(candidate.imageUrl);
      if (fixtureUrl) {
        return fixtureUrl;
      }
    }

    return candidate.imageUrl;
  }

  async function confirmEmbed(): Promise<void> {
    if (!confirmCandidate || tracks.length === 0) {
      return;
    }

    embedding = true;
    embedError = null;
    embedProgress = 0;
    embedTotal = tracks.length;

    try {
      const first = tracks[0];
      const selectRes = await selectArtworkCandidateForAlbum(
        first.albumArtist || first.artist || '',
        first.album || '',
        confirmCandidate.provider,
        confirmCandidate.providerItemId,
        confirmCandidate.imageUrl,
      );

      for (const track of tracks) {
        await embedArtworkToFile(track.trackId, selectRes.cacheKey, 'image/jpeg');
        embedProgress += 1;
      }

      await loadCurrentArtwork();
      isSearchOpen = false;
      candidates = [];
      confirmCandidate = null;
    } catch {
      embedError = 'Failed to embed artwork to files';
    } finally {
      embedding = false;
    }
  }
</script>

<div class="artwork-tab">
  <div class="artwork-item">
    <label class="item-label">
      <input type="checkbox" checked disabled />
      <span>artwork:</span>
    </label>

    <div class="item-body">
      <div class="thumbnail">
        {#if loadingCurrent}
          <div class="placeholder"><Loader2 class="spin" size={26} /></div>
        {:else if currentArtworkUrl}
          <img src={currentArtworkUrl} alt="Artwork" />
        {:else}
          <div class="placeholder">No artwork</div>
        {/if}
      </div>

      <div class="meta">
        <label>
          <span>picture type:</span>
          <select bind:value={pictureType} disabled={disabled}>
            <option>Album Cover</option>
            <option>Artist</option>
            <option>Other</option>
          </select>
        </label>

        <label>
          <span>comments:</span>
          <textarea bind:value={comments} disabled={disabled}></textarea>
        </label>

        <label class="primary-row">
          <input type="checkbox" bind:checked={primaryPicture} disabled={disabled} />
          <span>primary picture</span>
        </label>
      </div>

      <div class="actions">
        <button type="button" disabled>Delete</button>
        <button type="button" disabled>Save To...</button>
        <button type="button" onclick={() => (isSearchOpen = true)} disabled={disabled}>
          Search Online
        </button>
      </div>
    </div>
  </div>

  {#if isSearchOpen}
    <div class="overlay">
      <div class="search-panel">
        <div class="search-head">
          <div class="query-input">
            <input type="text" bind:value={searchQuery} onkeydown={(event) => event.key === 'Enter' && handleSearch()} />
            <button type="button" onclick={handleSearch} disabled={searching}><Search size={14} /></button>
          </div>
          <button type="button" class="close" onclick={() => (isSearchOpen = false)}>Close</button>
        </div>

        {#if searchError}
          <div class="error">{searchError}</div>
        {/if}

        <div class="result-grid">
          {#if searching}
            <div class="result-placeholder"><Loader2 class="spin" size={20} /> Searching...</div>
          {:else if candidates.length === 0}
            <div class="result-placeholder">No results</div>
          {:else}
            {#each candidates as candidate}
              <button type="button" class="candidate" onclick={() => (confirmCandidate = candidate)}>
                <img src={getCandidateImageUrl(candidate)} alt="Candidate artwork" loading="lazy" />
                <span>{candidate.provider}</span>
              </button>
            {/each}
          {/if}
        </div>
      </div>
    </div>
  {/if}

  {#if confirmCandidate}
    <div class="overlay">
      <div class="confirm-panel">
        <h3>Embed selected artwork?</h3>
        <img src={getCandidateImageUrl(confirmCandidate)} alt="Selected artwork" />
        <p>This will write artwork to {tracks.length} file{tracks.length === 1 ? '' : 's'}.</p>

        {#if embedding}
          <p class="progress">Embedding... {embedProgress}/{embedTotal}</p>
        {/if}

        {#if embedError}
          <p class="error">{embedError}</p>
        {/if}

        <div class="confirm-actions">
          <button type="button" onclick={() => (confirmCandidate = null)} disabled={embedding}>Cancel</button>
          <button type="button" onclick={confirmEmbed} disabled={embedding}>Embed Artwork</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .artwork-tab {
    height: 100%;
    position: relative;
    padding: 12px 0;
    overflow: auto;
  }

  .artwork-item {
    border-bottom: 1px solid var(--divider-color);
    padding-bottom: 12px;
  }

  .item-label {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-secondary);
    font-size: 13px;
    margin-bottom: 8px;
  }

  .item-label input {
    width: 14px;
    height: 14px;
    accent-color: var(--theme-accent);
  }

  .item-body {
    display: grid;
    grid-template-columns: 170px 1fr 130px;
    gap: 12px;
    align-items: start;
  }

  .thumbnail {
    width: 162px;
    height: 162px;
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .thumbnail img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .placeholder {
    color: var(--text-tertiary);
    font-size: 12px;
  }

  .meta {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .meta label {
    display: grid;
    gap: 4px;
  }

  .meta span {
    color: var(--text-secondary);
    font-size: 13px;
    text-transform: lowercase;
  }

  .meta select,
  .meta textarea {
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-primary);
    font-size: 13px;
    border-radius: 0;
    padding: 6px 8px;
  }

  .meta textarea {
    min-height: 70px;
    resize: vertical;
  }

  .primary-row {
    display: flex !important;
    align-items: center;
    gap: 6px;
  }

  .primary-row input {
    width: 14px;
    height: 14px;
    accent-color: var(--theme-accent);
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .actions button,
  .close,
  .confirm-actions button,
  .query-input button {
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-primary);
    font-size: 13px;
    padding: 6px 8px;
    border-radius: 0;
    cursor: pointer;
  }

  .actions button:disabled,
  .confirm-actions button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 12px;
    z-index: 5;
  }

  .search-panel,
  .confirm-panel {
    width: min(840px, 100%);
    max-height: 90%;
    overflow: auto;
    border: 1px solid var(--divider-color);
    background: var(--surface-0);
    padding: 10px;
    display: grid;
    gap: 10px;
  }

  .search-head {
    display: flex;
    gap: 10px;
    align-items: center;
    justify-content: space-between;
  }

  .query-input {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 6px;
    width: min(520px, 100%);
  }

  .query-input input {
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-primary);
    font-size: 13px;
    padding: 6px 8px;
    border-radius: 0;
  }

  .result-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: 8px;
  }

  .candidate {
    border: 1px solid var(--divider-color);
    background: var(--surface-1);
    color: var(--text-secondary);
    padding: 4px;
    display: grid;
    gap: 6px;
    cursor: pointer;
    border-radius: 0;
  }

  .candidate img {
    width: 100%;
    aspect-ratio: 1;
    object-fit: cover;
  }

  .candidate span {
    font-size: 11px;
    text-transform: uppercase;
    color: var(--text-tertiary);
  }

  .result-placeholder,
  .error,
  .progress {
    color: var(--text-secondary);
    font-size: 13px;
  }

  .error {
    color: #fca5a5;
  }

  .confirm-panel img {
    width: 180px;
    height: 180px;
    object-fit: cover;
    border: 1px solid var(--divider-color);
  }

  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .spin {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }

    to {
      transform: rotate(360deg);
    }
  }
</style>
