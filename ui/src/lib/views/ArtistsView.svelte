<script lang="ts">
  import { Fixtures, type Artist } from '../data/fixtures';

  let artists: Artist[] = [];
  const isMock = import.meta.env.SERMON_MOCK === '1';

  if (isMock) {
    artists = Fixtures.getArtists();
  }
</script>

<div class="view-container">
  <h1>Artists</h1>
  <div class="list">
    {#if isMock}
      {#each artists as artist}
        <div class="item">
          <div class="avatar">{artist.name[0]}</div>
          <div class="name">{artist.name}</div>
        </div>
      {/each}
    {:else}
      {#each Array(10) as _, i}
        <div class="item">Artist Name {i + 1}</div>
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
  .list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-top: 1rem;
  }
  .item {
    background: var(--glass-highlight);
    padding: 1rem;
    border-radius: var(--glass-radius);
    border: 1px solid transparent;
    display: flex;
    align-items: center;
    gap: 1rem;
    cursor: pointer;
    transition: background 0.2s, border-color 0.2s;
  }
  .item:hover {
    background: var(--glass-border);
    border-color: rgba(255,255,255,0.2);
  }
  .avatar {
    width: 40px;
    height: 40px;
    background: #333;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    color: #fff;
  }
  .name {
    font-size: 1rem;
  }
</style>
