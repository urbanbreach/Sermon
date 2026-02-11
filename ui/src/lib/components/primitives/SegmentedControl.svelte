<script lang="ts">
  import type { Component } from 'svelte';
  import { fade } from 'svelte/transition';

  interface Item {
    id: string;
    label: string;
    icon?: Component;
    tabId?: string;
    controlsId?: string;
  }

  interface Props {
    items: Item[];
    value: string;
    onchange: (id: string) => void;
    ariaLabel?: string;
  }

  let { items, value, onchange, ariaLabel }: Props = $props();

  function handleKeydown(event: KeyboardEvent, id: string) {
    const currentIndex = items.findIndex(item => item.id === id);
    
    if (event.key === 'ArrowRight') {
      event.preventDefault();
      const nextIndex = (currentIndex + 1) % items.length;
      onchange(items[nextIndex].id);
      // In a full implementation we would focus the new element, 
      // but since we're using value-based tabindex, the focus will naturally 
      // land on the active element if we re-render or the user tabs.
      // For now, this updates the selection.
    } else if (event.key === 'ArrowLeft') {
      event.preventDefault();
      const prevIndex = (currentIndex - 1 + items.length) % items.length;
      onchange(items[prevIndex].id);
    } else if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onchange(id);
    }
  }
</script>

<div class="segmented-control" role="tablist" aria-label={ariaLabel}>
  {#each items as item}
    <button
      role="tab"
      id={item.tabId}
      aria-selected={value === item.id}
      aria-controls={item.controlsId}
      tabindex={value === item.id ? 0 : -1}
      class:active={value === item.id}
      onclick={() => onchange(item.id)}
      onkeydown={(e) => handleKeydown(e, item.id)}
      type="button"
    >
      {#if item.icon}
        <item.icon size={14} />
      {/if}
      <span>{item.label}</span>
      
      {#if value === item.id}
        <div class="active-indicator" transition:fade={{ duration: 150 }}></div>
      {/if}
    </button>
  {/each}
</div>

<style>
  .segmented-control {
    display: flex;
    background: rgba(255, 255, 255, 0.04);
    padding: 2px;
    border-radius: var(--radius-md);
    height: 32px;
    gap: 2px;
  }

  button {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 0 12px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: var(--text-meta);
    font-weight: 500;
    cursor: pointer;
    border-radius: calc(var(--radius-md) - 2px);
    flex: 1;
    outline: none;
    transition: color var(--motion-fast);
    z-index: 1;
    white-space: nowrap;
  }

  button:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.02);
  }

  button.active {
    color: var(--text-primary);
    background: transparent;
  }

  button:focus-visible {
    box-shadow: var(--focus-ring);
  }

  .active-indicator {
    position: absolute;
    inset: 0;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.04);
    border-radius: calc(var(--radius-md) - 2px);
    z-index: -1;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  }
</style>
