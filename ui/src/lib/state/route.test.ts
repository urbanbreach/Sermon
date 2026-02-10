import { get } from 'svelte/store';
import { beforeEach, describe, expect, test } from 'vitest';

import {
  canGoBack,
  canGoForward,
  currentRoute,
  currentRouteName,
  forwardStack,
  goBack,
  goForward,
  navigate,
  replaceRoute,
} from './route';

function resetRouteState(): void {
  while (get(canGoBack)) {
    goBack();
  }

  replaceRoute({ name: 'albums' });
  forwardStack.set([]);
}

describe('route navigation stack', () => {
  beforeEach(() => {
    resetRouteState();
  });

  test('navigate/goBack/goForward: tracks route history', () => {
    navigate({ name: 'artists' });
    navigate({ name: 'tracks' });

    expect(get(currentRoute)).toEqual({ name: 'tracks' });
    expect(get(currentRouteName)).toBe('tracks');
    expect(get(canGoBack)).toBe(true);
    expect(get(canGoForward)).toBe(false);

    goBack();
    expect(get(currentRoute)).toEqual({ name: 'artists' });
    expect(get(canGoForward)).toBe(true);

    goForward();
    expect(get(currentRoute)).toEqual({ name: 'tracks' });
    expect(get(canGoForward)).toBe(false);
  });

  test('replaceRoute: replaces current entry without adding history', () => {
    navigate({ name: 'artists' });

    replaceRoute({ name: 'search-results', query: 'shoegaze' });

    expect(get(currentRoute)).toEqual({ name: 'search-results', query: 'shoegaze' });

    goBack();
    expect(get(currentRoute)).toEqual({ name: 'albums' });
    expect(get(canGoBack)).toBe(false);
  });

  test('navigate: deduplicates consecutive identical routes', () => {
    navigate({ name: 'search-results', query: 'ambient' });
    navigate({ name: 'search-results', query: 'ambient' });

    goBack();
    expect(get(currentRoute)).toEqual({ name: 'albums' });
    expect(get(canGoBack)).toBe(false);

    navigate({ name: 'search-results', query: 'ambient' });
    navigate({ name: 'search-results', query: 'drone' });

    expect(get(currentRoute)).toEqual({ name: 'search-results', query: 'drone' });
    goBack();
    expect(get(currentRoute)).toEqual({ name: 'search-results', query: 'ambient' });
  });

  test('stack boundaries: back at start and forward at end are no-ops', () => {
    goBack();
    expect(get(currentRoute)).toEqual({ name: 'albums' });
    expect(get(canGoBack)).toBe(false);

    navigate({ name: 'artists' });
    goBack();
    expect(get(currentRoute)).toEqual({ name: 'albums' });

    goForward();
    expect(get(currentRoute)).toEqual({ name: 'artists' });

    goForward();
    expect(get(currentRoute)).toEqual({ name: 'artists' });
    expect(get(canGoForward)).toBe(false);
  });
});
