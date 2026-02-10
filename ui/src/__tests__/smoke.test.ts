import { describe, it, expect } from 'vitest';
import SkeletonCard from '../lib/components/SkeletonCard.svelte';
import App from '../App.svelte';
import TopBar from '../lib/components/TopBar.svelte';
import LeftNav from '../lib/components/LeftNav.svelte';
import BottomBar from '../lib/components/BottomBar.svelte';

describe('Smoke Test', () => {
  it('should pass', () => {
    expect(true).toBe(true);
  });

  it('should be able to import a Svelte component', () => {
    expect(SkeletonCard).toBeDefined();
  });

  it('should import app shell components', () => {
    expect(App).toBeDefined();
    expect(TopBar).toBeDefined();
    expect(LeftNav).toBeDefined();
    expect(BottomBar).toBeDefined();
  });
});
