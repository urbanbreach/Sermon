<script lang="ts">
  import * as DropdownMenu from './DropdownMenu.svelte';
  import { MoreVertical } from '@lucide/svelte';

  let open = $state(false);
  
  function handlePlayNow() {
    console.log('Play Now clicked');
  }
  
  function handleAddToQueue() {
    console.log('Add to Queue clicked');
  }
  
  function handleEditTags() {
    console.log('Edit Tags clicked');
  }
</script>

<div class="demo-container">
  <DropdownMenu.Root bind:open>
    <DropdownMenu.Trigger>
      <button class="trigger-button" aria-label="Open track menu">
        <MoreVertical size={16} />
      </button>
    </DropdownMenu.Trigger>
    <DropdownMenu.Portal>
      <DropdownMenu.Content class="dropdown-content" sideOffset={4}>
        <DropdownMenu.Item class="dropdown-item" onSelect={handlePlayNow}>
          Play Now
        </DropdownMenu.Item>
        <DropdownMenu.Item class="dropdown-item" onSelect={handleAddToQueue}>
          Add to Queue
        </DropdownMenu.Item>
        <DropdownMenu.Separator class="dropdown-separator" />
        <DropdownMenu.Item class="dropdown-item" onSelect={handleEditTags}>
          Edit Tags...
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Portal>
  </DropdownMenu.Root>
</div>

<style>
  .demo-container {
    padding: 100px;
    background: #0a0a0a;
  }
  
  .trigger-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm, 4px);
    color: rgba(255, 255, 255, 0.70);
    cursor: pointer;
  }
  
  .trigger-button:hover {
    background: rgba(255, 255, 255, 0.10);
    color: rgba(255, 255, 255, 0.92);
  }
</style>
