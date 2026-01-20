# Architectural Decisions

## Routing
- **Decision**: Use a simple Svelte writable store (`ui/src/lib/state/route.ts`) instead of an external router.
- **Rationale**: The app shell is simple with a flat hierarchy (Library sections vs Settings). A full router (like `svelte-routing` or `tinro`) is overkill for Milestone 00. Conditional rendering in `App.svelte` provides immediate switching without URL complexity, suitable for a desktop app feel.
