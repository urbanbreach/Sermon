<script lang="ts">
  import { currentRoute } from '../state/route';
  
  function goBack() {
    currentRoute.set('albums'); // Or previous route if tracked
  }
</script>

<div class="now-playing-view">
  <button class="back-btn" on:click={goBack}>&larr; Back to Library</button>
  
  <div class="content">
    <div class="art-large"></div>
    <div class="info-large">
      <h1>Track Title</h1>
      <h2>Artist Name</h2>
      <h3>Album Title</h3>
    </div>
  </div>
</div>

<style>
  .now-playing-view {
    position: absolute;
    top: 40px; /* Below top bar */
    left: 0;
    right: 0;
    bottom: 80px; /* Above bottom bar */
    background: #0a0a0a;
    z-index: 100;
    padding: 2rem;
    display: flex;
    flex-direction: column;
    color: #fff;
  }

  .back-btn {
    align-self: flex-start;
    background: none;
    border: none;
    color: #888;
    cursor: pointer;
    font-size: 1rem;
    margin-bottom: 2rem;
  }
  .back-btn:hover {
    color: #fff;
  }

  .content {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 2rem;
  }

  .art-large {
    width: 300px;
    height: 300px;
    background: #222;
    border-radius: 8px;
    box-shadow: 0 10px 30px rgba(0,0,0,0.5);
  }

  .info-large {
    text-align: center;
  }

  h1 { font-size: 2rem; margin: 0 0 0.5rem 0; }
  h2 { font-size: 1.5rem; color: #ccc; margin: 0 0 0.5rem 0; font-weight: normal; }
  h3 { font-size: 1.2rem; color: #888; margin: 0; font-weight: normal; }
</style>
