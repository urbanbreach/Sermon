<script lang="ts">
  // No props needed, pure visual component
</script>

<div class="skeleton-card">
  <div class="skeleton-artwork shimmer"></div>
  <div class="skeleton-info">
    <div class="skeleton-title shimmer"></div>
    <div class="skeleton-artist shimmer"></div>
  </div>
</div>

<style>
  .skeleton-card {
    display: flex;
    flex-direction: column;
    /* Match Album Card dimensions/layout if possible or let parent grid control it */
  }
  
  .skeleton-artwork {
    width: 100%;
    aspect-ratio: 1;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.06);
  }
  
  .skeleton-info {
    padding: 8px 4px 4px 4px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  
  .skeleton-title {
    height: 14px;
    width: 80%;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.06);
  }
  
  .skeleton-artist {
    height: 12px;
    width: 60%;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.04);
  }
  
  .shimmer {
    background: linear-gradient(
      90deg,
      rgba(255, 255, 255, 0.04) 0%,
      rgba(255, 255, 255, 0.08) 50%,
      rgba(255, 255, 255, 0.04) 100%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s ease-in-out infinite;
  }
  
  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }
</style>
