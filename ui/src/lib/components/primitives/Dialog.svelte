<script lang="ts" module>
  import { Dialog } from 'bits-ui';
  
  export const Root = Dialog.Root;
  export const Trigger = Dialog.Trigger;
  export const Content = Dialog.Content;
  export const Overlay = Dialog.Overlay;
  export const Portal = Dialog.Portal;
  export const Title = Dialog.Title;
  export const Description = Dialog.Description;
  export const Close = Dialog.Close;
</script>

<style>
  :global(.dialog-overlay) {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(4px);
    animation: fadeOverlay 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  :global(.dialog-content) {
    position: fixed;
    left: 50%;
    top: 50%;
    z-index: 1001;
    transform: translate(-50%, -50%);
    width: 100%;
    max-width: 500px;
    max-height: 85vh;
    padding: var(--space-4, 24px);
    background: var(--surface-2, rgba(30, 30, 34, 0.95));
    border: 1px solid var(--divider-color, rgba(255, 255, 255, 0.07));
    border-radius: var(--radius-md, 12px);
    box-shadow: var(--shadow-3, 0 12px 32px rgba(0, 0, 0, 0.6));
    outline: none;
    animation: zoomContent 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  :global(.dialog-title) {
    margin: 0;
    font-size: var(--text-h3, 18px);
    font-weight: 600;
    color: var(--text-primary, rgba(255, 255, 255, 0.92));
  }

  :global(.dialog-description) {
    margin: var(--space-2, 8px) 0 var(--space-4, 20px);
    font-size: var(--text-body, 14px);
    color: var(--text-secondary, rgba(255, 255, 255, 0.7));
    line-height: 1.5;
  }

  :global(.dialog-close) {
    position: absolute;
    top: var(--space-3, 16px);
    right: var(--space-3, 16px);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--text-tertiary, rgba(255, 255, 255, 0.45));
    cursor: pointer;
    transition: all 0.15s ease;
  }

  :global(.dialog-close:hover) {
    background: var(--surface-hover, rgba(255, 255, 255, 0.1));
    color: var(--text-primary, rgba(255, 255, 255, 0.92));
  }

  @keyframes fadeOverlay {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes zoomContent {
    from { 
      opacity: 0; 
      transform: translate(-50%, -48%) scale(0.96); 
    }
    to { 
      opacity: 1; 
      transform: translate(-50%, -50%) scale(1); 
    }
  }
</style>
