<script lang="ts">
  import { Fixtures, type Track } from '../data/fixtures';

  let tracks: Track[] = [];
  const isMock = import.meta.env.SERMON_MOCK === '1';

  if (isMock) {
    tracks = Fixtures.getTracks();
  }

  function formatDuration(ms: number) {
    const minutes = Math.floor(ms / 60000);
    const seconds = ((ms % 60000) / 1000).toFixed(0);
    return minutes + ":" + (Number(seconds) < 10 ? '0' : '') + seconds;
  }
</script>

<div class="view-container">
  <h1>Tracks</h1>
  <table class="tracks-table">
    <thead>
      <tr>
        <th>#</th>
        <th>Title</th>
        <th>Artist</th>
        <th>Album</th>
        <th>Duration</th>
      </tr>
    </thead>
    <tbody>
      {#if isMock}
        {#each tracks as track, i}
          <tr>
            <td>{i + 1}</td>
            <td>{track.title}</td>
            <td>{Fixtures.getArtist(track.artistId)?.name || 'Unknown'}</td>
            <td>{Fixtures.getAlbum(track.albumId)?.title || 'Unknown'}</td>
            <td>{formatDuration(track.durationMs)}</td>
          </tr>
        {/each}
      {:else}
        {#each Array(15) as _, i}
          <tr>
            <td>{i + 1}</td>
            <td>Track Title {i + 1}</td>
            <td>Artist Name</td>
            <td>Album Name</td>
            <td>3:45</td>
          </tr>
        {/each}
      {/if}
    </tbody>
  </table>
</div>

<style>
  .view-container {
    padding: 2rem;
    color: #fff;
    height: 100%;
    overflow-y: auto;
  }
  .tracks-table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 1rem;
    text-align: left;
    font-size: 0.9rem;
  }
  th {
    border-bottom: 1px solid var(--glass-border);
    padding: 0.8rem;
    color: #888;
    font-weight: normal;
  }
  td {
    padding: 0.8rem;
    border-bottom: 1px solid var(--glass-highlight);
    color: rgba(255,255,255,0.8);
  }
  tr:hover {
    background: var(--glass-highlight);
  }
</style>
