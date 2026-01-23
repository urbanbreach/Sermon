import { writable, derived, get } from 'svelte/store';

// Discriminated union for routes with parameters
export type Route =
  | { name: 'albums' }
  | { name: 'artists' }
  | { name: 'tracks' }
  | { name: 'diagnostics' }
  | { name: 'now-playing' }
  | { name: 'preferences' }
  | { name: 'lyrics-fullscreen' }
  | { name: 'album-detail'; albumArtistSort: string; albumTitleSort: string }
  | { name: 'artist-detail'; artistSort: string }
  | { name: 'search-results'; query: string };

// Route stack for back navigation
const routeStack = writable<Route[]>([{ name: 'albums' }]);

// Forward stack for forward navigation
export const forwardStack = writable<Route[]>([]);

// Current route is the top of the stack
export const currentRoute = derived(routeStack, ($stack) => 
  $stack[$stack.length - 1] || { name: 'albums' }
);

// Helper to get route name for simple comparisons
export const currentRouteName = derived(currentRoute, ($route) => $route.name);

// Navigate to a new route (push to stack)
export function navigate(route: Route): void {
  forwardStack.set([]); // Clear forward history on new navigation
  routeStack.update((stack) => {
    // Don't push duplicate routes
    const current = stack[stack.length - 1];
    if (current && routesEqual(current, route)) {
      return stack;
    }
    // Limit stack size to prevent memory issues
    const newStack = [...stack, route];
    if (newStack.length > 20) {
      newStack.shift();
    }
    return newStack;
  });
}

// Go back to previous route
export function goBack(): void {
  routeStack.update((stack) => {
    if (stack.length > 1) {
      const current = stack[stack.length - 1];
      forwardStack.update(fs => [...fs, current]);
      return stack.slice(0, -1);
    }
    return stack;
  });
}

// Go forward to next route
export function goForward(): void {
  forwardStack.update(fStack => {
    if (fStack.length > 0) {
      const route = fStack[fStack.length - 1];
      routeStack.update(rStack => [...rStack, route]);
      return fStack.slice(0, -1);
    }
    return fStack;
  });
}

// Check if we can go back
export const canGoBack = derived(routeStack, ($stack) => $stack.length > 1);

// Check if we can go forward
export const canGoForward = derived(forwardStack, ($stack) => $stack.length > 0);

// Replace current route (doesn't add to history)
export function replaceRoute(route: Route): void {
  forwardStack.set([]);
  routeStack.update((stack) => {
    if (stack.length === 0) {
      return [route];
    }
    return [...stack.slice(0, -1), route];
  });
}

// Clear stack and set single route

// Helper to compare routes
function routesEqual(a: Route, b: Route): boolean {
  if (a.name !== b.name) return false;
  
  switch (a.name) {
    case 'album-detail':
      return (
        b.name === 'album-detail' &&
        a.albumArtistSort === b.albumArtistSort &&
        a.albumTitleSort === b.albumTitleSort
      );
    case 'artist-detail':
      return b.name === 'artist-detail' && a.artistSort === b.artistSort;
    case 'search-results':
      return b.name === 'search-results' && a.query === b.query;
    default:
      return true;
  }
}

