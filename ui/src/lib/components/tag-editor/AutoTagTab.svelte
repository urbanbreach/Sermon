<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { AlertTriangle, Check, ExternalLink, Fingerprint, Loader2, Search, Wand2, X } from '@lucide/svelte';
  import Button from '../primitives/Button.svelte';
  import { internetSettings, loadCategorySettings } from '../../state/preferences';

  type CandidateSource = 'acoustid' | 'musicbrainz';
  interface TagCandidate {
    source: CandidateSource;
    title: string;
    artist: string;
    album?: string;
    year?: string;
    trackNo?: number;
    discNo?: number;
    recordingId?: string;
    releaseId?: string;
    score?: number;
  }

  const isMock = import.meta.env.SERMON_MOCK === '1';
  const dispatch = createEventDispatcher<{ apply: TagCandidate }>();

  let acoustidFingerprint = $state('');
  let acoustidDuration = $state('');
  let acoustidResults = $state<TagCandidate[]>([]);
  let acoustidLoading = $state(false);
  let acoustidError = $state<string | null>(null);

  let mbSearchType = $state<'recording' | 'release'>('recording');
  let mbTitle = $state('');
  let mbArtist = $state('');
  let mbAlbum = $state('');
  let mbResults = $state<TagCandidate[]>([]);
  let mbLoading = $state(false);
  let mbError = $state<string | null>(null);

  let selectedCandidate = $state<TagCandidate | null>(null);
  let infoMessage = $state<string | null>(null);

  let acoustidKey = $derived($internetSettings?.['internet.acoustid_key'] ?? '');
  let hasAcoustidKey = $derived(acoustidKey.trim().length > 0);

  onMount(async () => {
    if (!isMock) {
      await loadCategorySettings('internet');
    }
  });

  function resetMessages() {
    acoustidError = null;
    mbError = null;
    infoMessage = null;
  }

  function normalizeArtistCredit(credit?: Array<{ name: string; artist?: { name: string } }>): string {
    if (!credit || credit.length === 0) return '';
    return credit.map(part => part.name || part.artist?.name).filter(Boolean).join('');
  }

  function formatScore(score?: number): string {
    if (score === undefined || Number.isNaN(score)) return '—';
    return `${Math.round(score)}%`;
  }

  function extractYear(date?: string): string {
    if (!date) return '';
    return date.slice(0, 4);
  }

  async function lookupAcoustId() {
    resetMessages();
    selectedCandidate = null;
    acoustidResults = [];

    if (!hasAcoustidKey) {
      acoustidError = 'Add your AcoustID API key in Preferences → Internet to use lookup.';
      return;
    }

    if (!acoustidFingerprint.trim() || !acoustidDuration.trim()) {
      acoustidError = 'Fingerprint and duration are required for AcoustID lookup.';
      return;
    }

    const duration = Number(acoustidDuration);
    if (Number.isNaN(duration) || duration <= 0) {
      acoustidError = 'Duration must be a positive number (seconds).';
      return;
    }

    acoustidLoading = true;
    try {
      const url = new URL('https://api.acoustid.org/v2/lookup');
      url.searchParams.set('client', acoustidKey.trim());
      url.searchParams.set('meta', 'recordings+releases+releasegroups');
      url.searchParams.set('fingerprint', acoustidFingerprint.trim());
      url.searchParams.set('duration', String(duration));

      const response = await fetch(url.toString());
      if (!response.ok) {
        throw new Error(`AcoustID request failed (${response.status})`);
      }

      const data = await response.json();
      if (data.status !== 'ok') {
        throw new Error(data.error?.message || 'AcoustID lookup failed');
      }

      const candidates: TagCandidate[] = [];
      for (const result of data.results ?? []) {
        const recordings = result.recordings ?? [];
        for (const recording of recordings) {
          const artist = (recording.artists ?? []).map((a: { name: string }) => a.name).join(', ');
          const release = (recording.releases ?? recording.releasegroups ?? [])[0];
          candidates.push({
            source: 'acoustid',
            title: recording.title ?? '',
            artist,
            album: release?.title ?? '',
            year: extractYear(release?.date),
            recordingId: recording.id,
            releaseId: release?.id,
            score: result.score ? result.score * 100 : undefined
          });
        }
      }

      acoustidResults = candidates;
      if (candidates.length === 0) {
        acoustidError = 'No matches found for this fingerprint.';
      }
    } catch (e) {
      acoustidError = String(e);
    } finally {
      acoustidLoading = false;
    }
  }

  function buildMusicBrainzQuery(): string {
    const parts: string[] = [];
    if (mbTitle.trim()) {
      parts.push(`${mbSearchType}:"${mbTitle.trim()}"`);
    }
    if (mbArtist.trim()) {
      parts.push(`artist:"${mbArtist.trim()}"`);
    }
    if (mbAlbum.trim()) {
      parts.push(`release:"${mbAlbum.trim()}"`);
    }
    return parts.length > 0 ? parts.join(' AND ') : '';
  }

  async function searchMusicBrainz() {
    resetMessages();
    selectedCandidate = null;
    mbResults = [];

    const query = buildMusicBrainzQuery();
    if (!query) {
      mbError = 'Enter at least one field to search.';
      return;
    }

    mbLoading = true;
    try {
      const endpoint = mbSearchType === 'recording' ? 'recording' : 'release';
      const url = new URL(`https://musicbrainz.org/ws/2/${endpoint}/`);
      url.searchParams.set('query', query);
      url.searchParams.set('fmt', 'json');
      url.searchParams.set('limit', '12');

      const response = await fetch(url.toString(), {
        headers: { 'Accept': 'application/json' }
      });
      if (!response.ok) {
        throw new Error(`MusicBrainz request failed (${response.status})`);
      }

      const data = await response.json();
      const list = mbSearchType === 'recording' ? data.recordings : data.releases;
      const candidates: TagCandidate[] = [];

      for (const item of list ?? []) {
        const artist = normalizeArtistCredit(item['artist-credit']);
        const release = item.releases?.[0];
        candidates.push({
          source: 'musicbrainz',
          title: item.title ?? '',
          artist,
          album: mbSearchType === 'release' ? item.title ?? '' : release?.title ?? '',
          year: extractYear((mbSearchType === 'release' ? item.date : release?.date) ?? ''),
          recordingId: item.id,
          releaseId: mbSearchType === 'release' ? item.id : release?.id,
          score: item.score
        });
      }

      mbResults = candidates;
      if (candidates.length === 0) {
        mbError = 'No matches found for this search.';
      }
    } catch (e) {
      mbError = String(e);
    } finally {
      mbLoading = false;
    }
  }

  function selectCandidate(candidate: TagCandidate) {
    selectedCandidate = candidate;
    infoMessage = null;
  }

  async function copyTags() {
    if (!selectedCandidate) return;
    const payload = {
      title: selectedCandidate.title,
      artist: selectedCandidate.artist,
      album: selectedCandidate.album,
      year: selectedCandidate.year,
      trackNo: selectedCandidate.trackNo,
      discNo: selectedCandidate.discNo,
      recordingId: selectedCandidate.recordingId,
      releaseId: selectedCandidate.releaseId
    };
    await navigator.clipboard.writeText(JSON.stringify(payload, null, 2));
    infoMessage = 'Tag preview copied to clipboard.';
  }

  function applyCandidate() {
    if (!selectedCandidate) return;
    dispatch('apply', selectedCandidate);
    infoMessage = 'Apply event sent to Tag Editor.';
  }

  function clearSelection() {
    selectedCandidate = null;
    infoMessage = null;
  }
</script>

<div class="auto-tag-tab">
  <div class="header">
    <Wand2 size={18} />
    <div>
      <h3>Auto-Tagging</h3>
      <p>Identify tracks with AcoustID or search MusicBrainz for official metadata.</p>
    </div>
  </div>

  {#if !hasAcoustidKey}
    <div class="callout warning">
      <AlertTriangle size={16} />
      <div>
        <strong>AcoustID key missing.</strong>
        <span>Add your API key in Preferences → Internet to enable fingerprint lookup.</span>
      </div>
    </div>
  {/if}

  <div class="grid">
    <section class="panel">
      <div class="panel-header">
        <div class="panel-title">
          <Fingerprint size={16} />
          <span>AcoustID Identification</span>
        </div>
        <a class="panel-link" href="https://acoustid.org" target="_blank" rel="noreferrer">
          AcoustID <ExternalLink size={12} />
        </a>
      </div>

      <div class="field">
        <label for="fingerprint">Fingerprint</label>
        <textarea
          id="fingerprint"
          rows={3}
          bind:value={acoustidFingerprint}
          placeholder="Paste an AcoustID fingerprint..."
        ></textarea>
      </div>

      <div class="field">
        <label for="duration">Duration (seconds)</label>
        <input
          id="duration"
          type="number"
          min="1"
          bind:value={acoustidDuration}
          placeholder="e.g. 213"
        />
      </div>

      {#if acoustidError}
        <div class="message error">{acoustidError}</div>
      {/if}

      <div class="actions">
        <Button variant="primary" onclick={lookupAcoustId} disabled={acoustidLoading}>
          {#if acoustidLoading}
            <Loader2 class="spin" size={14} />
            Identifying...
          {:else}
            Identify Track
          {/if}
        </Button>
      </div>

      <div class="results">
        {#if acoustidResults.length > 0}
          {#each acoustidResults as candidate, index (candidate.recordingId ?? candidate.releaseId ?? `${candidate.title}-${index}`)}
            <button class="result-card" onclick={() => selectCandidate(candidate)}>
              <div class="result-meta">
                <span class="source-badge">AcoustID</span>
                <span class="score">{formatScore(candidate.score)}</span>
              </div>
              <div class="title" title={candidate.title}>{candidate.title || 'Untitled recording'}</div>
              <div class="subtitle">{candidate.artist || 'Unknown artist'}</div>
              <div class="subtitle">{candidate.album || 'Unknown release'} {candidate.year ? `(${candidate.year})` : ''}</div>
            </button>
          {/each}
        {:else if !acoustidLoading && !acoustidError}
          <div class="empty">Paste a fingerprint to identify a track.</div>
        {/if}
      </div>
    </section>

    <section class="panel">
      <div class="panel-header">
        <div class="panel-title">
          <Search size={16} />
          <span>MusicBrainz Search</span>
        </div>
        <a class="panel-link" href="https://musicbrainz.org" target="_blank" rel="noreferrer">
          MusicBrainz <ExternalLink size={12} />
        </a>
      </div>

      <div class="field-row">
        <span class="field-label">Search Type</span>
        <div class="segmented">
          <button
            class:active={mbSearchType === 'recording'}
            onclick={() => mbSearchType = 'recording'}
          >Recording</button>
          <button
            class:active={mbSearchType === 'release'}
            onclick={() => mbSearchType = 'release'}
          >Release</button>
        </div>
      </div>

      <div class="field">
        <label for="mb-title">Title</label>
        <input id="mb-title" type="text" bind:value={mbTitle} placeholder="Track or release title" />
      </div>

      <div class="field">
        <label for="mb-artist">Artist</label>
        <input id="mb-artist" type="text" bind:value={mbArtist} placeholder="Artist name" />
      </div>

      <div class="field">
        <label for="mb-album">Album</label>
        <input id="mb-album" type="text" bind:value={mbAlbum} placeholder="Album (optional)" />
      </div>

      {#if mbError}
        <div class="message error">{mbError}</div>
      {/if}

      <div class="actions">
        <Button variant="primary" onclick={searchMusicBrainz} disabled={mbLoading}>
          {#if mbLoading}
            <Loader2 class="spin" size={14} />
            Searching...
          {:else}
            Search
          {/if}
        </Button>
      </div>

      <div class="results">
        {#if mbResults.length > 0}
          {#each mbResults as candidate, index (candidate.recordingId ?? candidate.releaseId ?? `${candidate.title}-${index}`)}
            <button class="result-card" onclick={() => selectCandidate(candidate)}>
              <div class="result-meta">
                <span class="source-badge">MusicBrainz</span>
                <span class="score">{formatScore(candidate.score)}</span>
              </div>
              <div class="title" title={candidate.title}>{candidate.title || 'Untitled recording'}</div>
              <div class="subtitle">{candidate.artist || 'Unknown artist'}</div>
              <div class="subtitle">{candidate.album || 'Unknown release'} {candidate.year ? `(${candidate.year})` : ''}</div>
            </button>
          {/each}
        {:else if !mbLoading && !mbError}
          <div class="empty">Enter metadata to search MusicBrainz.</div>
        {/if}
      </div>
    </section>
  </div>

  <section class="preview">
    <div class="preview-header">
      <h4>Tag Preview</h4>
      {#if selectedCandidate}
        <div class="preview-actions">
          <Button variant="ghost" onclick={clearSelection}><X size={14} /> Clear</Button>
          <Button variant="secondary" onclick={copyTags}><Check size={14} /> Copy JSON</Button>
          <Button variant="primary" onclick={applyCandidate}>Apply to Tag Editor</Button>
        </div>
      {/if}
    </div>

    {#if selectedCandidate}
      <div class="preview-grid">
        <div class="label">Title</div>
        <div class="value">{selectedCandidate.title || '—'}</div>

        <div class="label">Artist</div>
        <div class="value">{selectedCandidate.artist || '—'}</div>

        <div class="label">Album</div>
        <div class="value">{selectedCandidate.album || '—'}</div>

        <div class="label">Year</div>
        <div class="value">{selectedCandidate.year || '—'}</div>

        <div class="label">Recording ID</div>
        <div class="value mono">{selectedCandidate.recordingId || '—'}</div>

        <div class="label">Release ID</div>
        <div class="value mono">{selectedCandidate.releaseId || '—'}</div>
      </div>
    {:else}
      <div class="empty">Select a match to preview tags.</div>
    {/if}

    {#if infoMessage}
      <div class="message success">{infoMessage}</div>
    {/if}
  </section>
</div>

<style>
  .auto-tag-tab {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    padding: 0.5rem 0 1rem;
    min-height: 360px;
  }

  .header {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
  }

  h3 {
    margin: 0;
    color: #fff;
    font-weight: 600;
    font-size: 1rem;
  }

  .header p {
    margin: 0.25rem 0 0;
    color: #999;
    font-size: 0.85rem;
  }

  .callout {
    display: flex;
    gap: 0.75rem;
    align-items: flex-start;
    background: rgba(255, 193, 7, 0.08);
    border: 1px solid rgba(255, 193, 7, 0.2);
    border-radius: 8px;
    padding: 0.75rem 1rem;
    color: #facc15;
    font-size: 0.85rem;
  }

  .callout span {
    display: block;
    color: #d1a926;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: 1rem;
  }

  .panel {
    border: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
    border-radius: 10px;
    padding: 1rem;
    background: rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    min-height: 320px;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
  }

  .panel-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    color: #fff;
  }

  .panel-link {
    font-size: 0.75rem;
    color: #88baff;
    text-decoration: none;
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }

  .panel-link:hover {
    color: #b7d8ff;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .field-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
  }

  .field-label {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: #aaa;
  }

  label {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: #aaa;
  }

  input,
  textarea {
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    padding: 0.5rem 0.75rem;
    color: #fff;
    font-size: 0.85rem;
    font-family: inherit;
    outline: none;
  }

  textarea {
    resize: vertical;
  }

  input:focus,
  textarea:focus {
    border-color: rgba(68, 170, 255, 0.5);
  }

  .segmented {
    display: inline-flex;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    overflow: hidden;
  }

  .segmented button {
    padding: 0.35rem 0.75rem;
    font-size: 0.75rem;
    border: none;
    background: transparent;
    color: #999;
    cursor: pointer;
  }

  .segmented button.active {
    background: rgba(68, 170, 255, 0.25);
    color: #d6ecff;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
  }

  .results {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    overflow-y: auto;
    padding-right: 4px;
  }

  .result-card {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 0.75rem;
    text-align: left;
    background: rgba(255, 255, 255, 0.04);
    cursor: pointer;
    transition: border-color 0.2s, transform 0.2s;
  }

  .result-card:hover {
    border-color: rgba(68, 170, 255, 0.5);
    transform: translateY(-1px);
  }

  .result-meta {
    display: flex;
    justify-content: space-between;
    font-size: 0.7rem;
    color: #aaa;
    margin-bottom: 0.35rem;
  }

  .source-badge {
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .score {
    color: #6ee7ff;
  }

  .title {
    font-size: 0.9rem;
    color: #fff;
    margin-bottom: 0.25rem;
  }

  .subtitle {
    font-size: 0.8rem;
    color: #aaa;
  }

  .preview {
    border: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
    border-radius: 10px;
    padding: 1rem;
    background: rgba(0, 0, 0, 0.15);
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  h4 {
    margin: 0;
    color: #fff;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .preview-actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .preview-actions :global(button) {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
  }

  .preview-grid {
    display: grid;
    grid-template-columns: 140px 1fr;
    gap: 0.5rem 1rem;
    font-size: 0.85rem;
  }

  .preview-grid .label {
    color: #999;
  }

  .preview-grid .value {
    color: #fff;
  }

  .preview-grid .mono {
    font-family: var(--font-mono, monospace);
    font-size: 0.8rem;
    color: #bbb;
  }

  .message {
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    font-size: 0.8rem;
  }

  .message.error {
    background: rgba(239, 68, 68, 0.15);
    color: #f88;
    border: 1px solid rgba(239, 68, 68, 0.25);
  }

  .message.success {
    background: rgba(34, 197, 94, 0.15);
    color: #86efac;
    border: 1px solid rgba(34, 197, 94, 0.25);
  }

  .empty {
    color: #777;
    font-size: 0.8rem;
    text-align: center;
    padding: 0.5rem 0;
  }

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
