<script lang="ts">
  // No props needed
</script>

<tr class="skeleton-row">
  <td class="col-index">
    <div class="skeleton-text shimmer" style="width: 16px"></div>
  </td>
  <td class="col-name">
    <div class="skeleton-text shimmer" style="width: 60%"></div>
  </td>
  <td>
    <div class="skeleton-text shimmer" style="width: 70%"></div>
  </td>
  <td>
    <div class="skeleton-text shimmer" style="width: 50%"></div>
  </td>
  <td>
    <div class="skeleton-text shimmer" style="width: 40%"></div>
  </td>
  <td class="col-time">
    <div class="skeleton-text shimmer" style="width: 80%; margin-left: auto;"></div>
  </td>
  <td class="col-actions">
    <!-- Empty -->
  </td>
</tr>

<style>
  td {
    padding: 0 var(--table-cell-gap, 12px);
    height: 40px;
    border-bottom: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
    vertical-align: middle;
  }

  .col-index {
    width: 40px;
    text-align: center;
  }
  
  .col-name {
    width: 30%;
    min-width: 200px;
  }
  
  .col-time {
    width: 60px;
  }
  
  .col-actions {
    width: 40px;
  }

  .skeleton-text {
    height: 14px;
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
