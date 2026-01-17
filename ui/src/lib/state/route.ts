import { writable, derived, get } from 'svelte/store';

// Discriminated union for routes with parameters
export type Route =
  | { name: 'albums' }
  | { name: 'artists' }
  | { name: 'tracks' }
  | { name: 'settings' }
  | { name: 'diagnostics' }
  | { name: 'now-playing' }
  | { name: 'album-detail'; albumArtistSort: string; albumTitleSort: string }
  | { name: 'artist-detail'; artistSort: string }
  | { name: 'search-results'; query: string };

// Route stack for back navigation
const routeStack = writable<Route[]>([{ name: 'albums' }]);

// Current route is the top of the stack
export const currentRoute = derived(routeStack, ($stack) => 
  $stack[$stack.length - 1] || { name: 'albums' }
);

// Helper to get route name for simple comparisons
export const currentRouteName = derived(currentRoute, ($route) => $route.name);

// Navigate to a new route (push to stack)
export function navigate(route: Route): void {
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
      return stack.slice(0, -1);
    }
    return stack;
  });
}

// Check if we can go back
export const canGoBack = derived(routeStack, ($stack) => $stack.length > 1);

// Replace current route (doesn't add to history)
export function replaceRoute(route: Route): void {
  routeStack.update((stack) => {
    if (stack.length === 0) {
      return [route];
    }
    return [...stack.slice(0, -1), route];
  });
}

// Clear stack and set single route
export function resetTo(route: Route): void {
  routeStack.set([route]);
}

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

// Legacy compatibility - set route by name
export function setRoute(name: 'albums' | 'artists' | 'tracks' | 'settings' | 'diagnostics' | 'now-playing'): void {
  navigate({ name });
}
