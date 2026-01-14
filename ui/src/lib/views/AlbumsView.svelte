<script lang="ts">
  import { Fixtures, type Album } from '../data/fixtures';

  let albums: Album[] = [];
  const isMock = import.meta.env.SERMON_MOCK === '1';

  if (isMock) {
    albums = Fixtures.getAlbums();
  }
</script>

<div class="view-container">
  <h1>Albums</h1>
  <div class="grid">
    {#if isMock}
      {#each albums as album}
        <div class="card">
          <img src={Fixtures.getArtworkPath(album.artworkFile)} alt={album.title} class="artwork" />
          <div class="info">
            <div class="title">{album.title}</div>
            <div class="year">{album.year}</div>
          </div>
        </div>
      {/each}
    {:else}
      {#each Array(10) as _, i}
        <div class="card placeholder">Album {i + 1}</div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .view-container {
    padding: 2rem;
    color: #fff;
    height: 100%;
    overflow-y: auto;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 1.5rem;
    margin-top: 1rem;
  }
  .card {
    background: var(--glass-highlight); /* Glassy card */
    border-radius: var(--glass-radius);
    overflow: hidden;
    transition: transform 0.2s;
    border: 1px solid var(--glass-border);
  }
  .card:hover {
    transform: translateY(-4px);
    background: var(--glass-border);
  }
  .card.placeholder {
    aspect-ratio: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #222;
  }
  .artwork {
    width: 100%;
    aspect-ratio: 1;
    object-fit: cover;
    display: block;
  }
  .info {
    padding: 0.8rem;
  }
  .title {
    font-weight: 500;
    margin-bottom: 0.2rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .year {
    font-size: 0.8rem;
    color: #888;
  }
</style>
