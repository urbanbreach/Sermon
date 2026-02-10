<script lang="ts" module>
  import { DropdownMenu } from 'bits-ui';
  
  export const Root = DropdownMenu.Root;
  export const Trigger = DropdownMenu.Trigger;
  export const Portal = DropdownMenu.Portal;
  export const Content = DropdownMenu.Content;
  export const Item = DropdownMenu.Item;
  export const Separator = DropdownMenu.Separator;
  export const Group = DropdownMenu.Group;
  export const GroupHeading = DropdownMenu.GroupHeading;
  export const CheckboxItem = DropdownMenu.CheckboxItem;
  export const RadioGroup = DropdownMenu.RadioGroup;
  export const RadioItem = DropdownMenu.RadioItem;
  export const Sub = DropdownMenu.Sub;
  export const SubTrigger = DropdownMenu.SubTrigger;
  export const SubContent = DropdownMenu.SubContent;
</script>


<style>
  /* Brutalist-Matte DropdownMenu Styles - to be applied via class prop */
  :global(.dropdown-content) {
    min-width: 160px;
    padding: var(--space-1, 4px) 0;
    background: var(--surface-floating, rgba(20, 20, 20, 0.96));
    border: 1px solid var(--divider-color, rgba(255, 255, 255, 0.14));
    border-radius: var(--radius-md, 8px);
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.45);
    outline: none;
    z-index: 1000;
  }

  :global(.dropdown-item) {
    display: flex;
    align-items: center;
    width: 100%;
    padding: var(--space-2, 8px) var(--space-3, 12px);
    font-family: inherit;
    font-size: var(--text-body, 14px);
    color: var(--text-primary, rgba(255, 255, 255, 0.92));
    background: transparent;
    border: none;
    cursor: pointer;
    transition: background-color var(--motion-fast, 0.15s) var(--ease-out, cubic-bezier(0.16, 1, 0.3, 1));
    outline: none;
  }

  :global(.dropdown-item:hover),
  :global(.dropdown-item[data-highlighted]) {
    background: var(--surface-hover, rgba(255, 255, 255, 0.10));
  }

  :global(.dropdown-item:focus-visible) {
    background: var(--surface-hover, rgba(255, 255, 255, 0.10));
  }

  :global(.dropdown-item[data-disabled]) {
    color: var(--text-disabled, rgba(255, 255, 255, 0.25));
    cursor: not-allowed;
    pointer-events: none;
  }

  :global(.dropdown-separator) {
    height: 1px;
    margin: var(--space-1, 4px) 0;
    background: var(--divider-color, rgba(255, 255, 255, 0.14));
  }

  :global(.dropdown-group-heading) {
    padding: var(--space-1, 4px) var(--space-3, 12px);
    font-size: var(--text-meta, 12px);
    color: var(--text-tertiary, rgba(255, 255, 255, 0.45));
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
</style>
