import { describe, it, expect } from 'vitest';
import * as ContextMenu from '../lib/components/primitives/ContextMenu.svelte';

describe('ContextMenu', () => {
  it('exports Root component', () => {
    expect(ContextMenu.Root).toBeDefined();
  });

  it('exports Trigger component', () => {
    expect(ContextMenu.Trigger).toBeDefined();
  });

  it('exports Portal component', () => {
    expect(ContextMenu.Portal).toBeDefined();
  });

  it('exports Content component', () => {
    expect(ContextMenu.Content).toBeDefined();
  });

  it('exports Item component', () => {
    expect(ContextMenu.Item).toBeDefined();
  });

  it('exports Separator component', () => {
    expect(ContextMenu.Separator).toBeDefined();
  });

  it('exports Group component', () => {
    expect(ContextMenu.Group).toBeDefined();
  });

  it('exports GroupHeading component', () => {
    expect(ContextMenu.GroupHeading).toBeDefined();
  });

  it('exports CheckboxItem component', () => {
    expect(ContextMenu.CheckboxItem).toBeDefined();
  });

  it('exports RadioGroup component', () => {
    expect(ContextMenu.RadioGroup).toBeDefined();
  });

  it('exports RadioItem component', () => {
    expect(ContextMenu.RadioItem).toBeDefined();
  });

  it('exports Sub component', () => {
    expect(ContextMenu.Sub).toBeDefined();
  });

  it('exports SubTrigger component', () => {
    expect(ContextMenu.SubTrigger).toBeDefined();
  });

  it('exports SubContent component', () => {
    expect(ContextMenu.SubContent).toBeDefined();
  });

  it('can be imported from primitives index', async () => {
    const primitives = await import('../lib/components/primitives');
    expect(primitives.ContextMenu).toBeDefined();
    expect(primitives.ContextMenu.Root).toBeDefined();
    expect(primitives.ContextMenu.Item).toBeDefined();
  });
});
